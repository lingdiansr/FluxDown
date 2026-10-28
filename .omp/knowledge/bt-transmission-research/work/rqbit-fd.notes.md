# pinned rqbit 实际能力与 FluxDown 适配（机制说明）

**证据边界**

- rqbit 侧全部行号来自本地 checkout `~/.cargo/git/checkouts/rqbit-341c9d2d87cbd6c5/6f9d64b/`，对应 rev `6f9d64b310ecb560990315af3c60df3111eec8e7`（根 `Cargo.toml:52`；`Cargo.lock:5751-5753` 记 `librqbit 9.0.1`，source 同为该 rev）。该 checkout 是 grafted shallow clone（无 tag 可达、submodule 未初始化），本文不依赖 tag 或上游网页版本号。
- FluxDown 侧对应 HEAD `602be2a67ddc98339e8e172e8786899be547da30`（2026-10-16）。
- 本文所有「rqbit 没有 X」的判断均以在该 checkout 全树检索为依据；「FD 没有 Y」同理。

---

## 1. 四个必答问题

### 1.1 piece 选择策略 = **顺序式**（每文件首尾优先 + 文件内近似顺序），不是 rarest-first，也不是混用

- **主张**：`PieceTracker::acquire_piece` 的三阶段是「偷（10×）→ 优先级片 / 队列片 → 偷（3×）」，而「队列片」的遍历顺序由文件优先级与文件内片顺序共同决定，全程不含任何稀缺度权重。
- **证据**：`crates/librqbit/src/piece_tracker.rs:112-164`（三阶段）；`torrent_state/live/mod.rs:552-564`（`file_priorities` 在 `TorrentStateLive::new` 中按文件名字母序 `sort_unstable_by_key(..relative_filename..)`，并带 `// TODO: make it configurable`）；`chunk_tracker.rs:236-249`（`iter_queued_pieces` 先按 `file_priorities` 顺序，再按 `FileInfo::iter_piece_priorities`）；`file_info.rs:14-36`（顺序为「首片、尾片、其余 `start→end` 顺序」）；`chunk_tracker.rs:157-185`（`queue_pieces` 初值 = 未持有且已选中的全部片，无序）。
- **适用条件**：多文件 torrent 按文件名字母序逐文件处理；`priority_pieces`（streaming 读位置）在队列片之前被检查，会短暂覆盖顺序偏好；`reserve` 时用 `peer_has_piece` 过滤，因此某个 peer 没有的片会被跳过、继续向后找——**实际到盘顺序会明显偏离严格顺序，但选择偏好本身不含稀缺度**。
- **与当前 FluxDown 的差异**：FD 未覆盖任何选择策略（`bt_downloader.rs` 未触碰 `PieceTracker` / `file_priorities`），因此 FD 的片顺序完全由上述 rqbit 逻辑决定。FD 侧的 `build_bt_segments` 注释（`bt_downloader.rs:3099-3101`）把 UI 段的散列外观解释为「BT piece 到达顺序近乎随机」，这与「选择是顺序式、但受 peer 可用性过滤」并不冲突：顺序是**偏好**，实际到达取决于哪个 peer 先响应。
  - 结论性差异：**FD 的 UI 散列外观不能反推 rqbit 采用了 rarest-first**；两者是不同层次的现象。
- **可检验预测**：
  1. 在单 seed + 单 leecher 的受控 swarm 中（seed 拥有全部片），按文件顺序观察 piece 完成序：首个完成片应为文件首片或尾片，而不是随机的稀有片。若观察到首尾优先、随后近似顺序推进，则「顺序式」成立。
  2. 在多文件 torrent（文件名与片序不一致，例如 `z.bin` 排在 `a.bin` 之前）中，`a.bin` 的片应整体先于 `z.bin` 被请求，即使 `z.bin` 的片在同一 peer 上更早可用。
  3. 若在若干 peer 上人为打乱可用片集合，观察是否出现「同一片被多 peer 同时请求」（rarest-first 的典型副产物）——按本实现不应出现，因为 `inflight` 保证一片只属于一个 peer（`piece_tracker.rs:167-177` + `STATE.md:58-69`）。

### 1.2 请求窗口：**可配置，默认 128 个未完成 chunk 请求**；对端 reqq 只能下调

- **主张**：`DEFAULT_PEER_REQUEST_WINDOW = 128`，`options.peer_max_request_window` 未设置时取 128；窗口单位是 chunk 请求数（每个 peer 的 `inflight_requests.len()`）；对端在扩展握手中声明的 `reqq` 会以 `min` 方式压低窗口，永不上调。
- **证据**：`torrent_state/live/mod.rs:1389`（常量）；`torrent_state/live/mod.rs:525-532`（`unwrap_or(DEFAULT).max(1)`）；`torrent_state/live/mod.rs:2609-2627`（`can_send_request = !i_am_choked && requested_inflight_count() < request_window`）；`torrent_state/live/peer/mod.rs:356-358`（`requested_inflight_count = inflight_requests.len()`）；`torrent_state/live/mod.rs:1606-1628`（`reqq.min(max_request_window)`，仅在变化时通知）；`peer_connection.rs:94-103`（字段与语义文档）；`session.rs:1261-1265` + `torrent_state/mod.rs:110-135`（下发链）。
- **适用条件**：窗口与连接上的 choke 状态耦合——被对端 choke 时窗口大小无意义（不取片）。同一 peer 的限速等待被插在窗口判定之间（`live/mod.rs:2250-2264` 两次 `wait_for_request_slot()` 夹着 `prepare_for_download`），所以受限速时窗口的实际驻留时间会被拉长。
- **与当前 FluxDown 的差异**：**FD 未设置 `max_request_window`**（`bt_downloader.rs:781-799` 的 `peer_opts` 只设了四个字段，其余 `..Default::default()`），所以 FD 的实际窗口就是 128，并被对端 reqq 单向下调。FD 侧的迁移文档（`bt-backend-migration.md:1032`）已记录「request window 固定 128」，与本核对一致。
- **可检验预测**：
  1. 在高 BDP 场景（RTT ≳ 150 ms × 带宽 ≳ 100 Mbps，BDP 明显超过 128 × 16 KiB = 2 MiB 在途量）下，单 peer goodput 应被窗口而非拥塞窗口限制；此时把 `max_request_window` 提到 256/512 应提升单 peer 吞吐，而在低 BDP 下应无差异——若在低 BDP 下也提升，说明限制不在窗口。
  2. 与一个 `reqq` 声明很小的客户端互联时，FD 单 peer 吞吐应降到与该 reqq 一致的水平；若未下降，说明 `reqq` 更新路径（`on_extended_handshake`）未命中。

### 1.3 choker：**不存在 choker**。连接建立即无条件 `Unchoke`，此后永不下发 `Choke`

- **主张**：pinned rqbit 没有 tit-for-tat、没有周期性 rechoke、也没有 optimistic unchoke。writer 任务发送 bitfield 之后无条件写出 `Message::Unchoke`；全树没有任何发送 `Message::Choke` 的代码路径。
- **证据**：`peer_connection.rs:458-486`（`Message::Unchoke.serialize(...)` 后 `trace!("sent unchoke")`，无条件）；对 `Message::Choke` 的引用只有接收/编解码侧：`torrent_state/live/mod.rs:1475`（`Message::Choke => self.on_i_am_choked()`）与 `crates/peer_binary_protocol/src/lib.rs:294,297,387,784`。上传侧的「需求」表达不是 choke 而是 `peer_interested`（`live/mod.rs:2309-2312`、`peers/mod.rs:234-245`）；`NotInterested` 会丢弃该 peer 的全部待发上传（`live/mod.rs:2037-2049`）。
- **适用条件**：所有已建立连接，含入站与出站；`disable-upload` 构建特性会改变是否发送 bitfield（`live/mod.rs:1663-1669`），但 Unchoke 的发送与该特性无关。**对端** choke 我们时会门控我们的取片（`live/mod.rs:1881-1885`、`2605-2607`），但不会取消已有在途请求。
- **与当前 FluxDown 的差异**：FD 未做任何上传侧公平性覆盖。因此 **FD 的上传调度是「全量 unchoke + 上传队列轮转」而不是「choke 集合选择」**——这解释了为什么观察上传行为时看不到 rechoke 事件：本次核对确认 pinned checkout 中**不存在任何发送 `Message::Choke` 的路径**，因此「unchoked/choked by rechoke」这类日志在当前 pin 下不可能由本实现产生。`[INFERENCE]` 该日志名在历史观测中很可能是上游 choker 实现的产物，本次未对上游 `ikatson/rqbit` 核对，故仅作线索。FD 的做种槽位控制（`bt_seeding.rs` 的 `seed_max_active`、FIFO `reconcile`）是 **torrent 级上下线**，与 per-peer choke 是两回事：达到上限的 seed 被暂停，而非「在线但不给它 unchoke」。
- **可检验预测**：
  1. 抓取任一 FD↔peer 连接的 BT 消息序列：应只有一次 `Unchoke`（建立时）、没有 `Choke`。若观察到 `Choke`，说明运行的不是 pinned fork 或另有实现。
  2. 上传侧异常检测：等待 60 秒后仍在线的 peer 若既不 `Interested` 也不取数据，在 rqbit 侧不会被 choke 掉，只会作为「无需求连接」占据 live 槽位。这直接指向 §3 的 turnover 才是唯一的清理机制。
  3. 由于没有 tit-for-tat，多任务下的上传带宽分配不应与「对方给我们的下载量」相关；若观察到相关，说明存在其他机制（如 torrent 级限速或上传队列轮转顺序）。

### 1.4 上传调度 = **per-torrent 单任务 + per-peer 轮转 + peer 内 FIFO + 两级硬上限**

- **主张**：每个 torrent 仅有 `MAX_PENDING_PER_PEER=128` / `MAX_PENDING_PER_TORRENT=4096` 两个背压常量；`UploadQueue::pop_next` 在 `rotation` 上轮转 peer、peer 内部 FIFO；`task_upload_scheduler` 是单任务，负责排队、限速与代际校验，真正的读盘发生在该 peer 自己的 writer 任务中。
- **证据**：`torrent_state/live/upload_queue.rs:25-31`（两个上限）；`upload_queue.rs:85-142`（`enqueue` 去重 + `pop_next` 轮转）；`upload_queue.rs:144-192`（`cancel` / `drop_peer` / `drop_stale_generations`）；`torrent_state/live/mod.rs:750-820`（调度循环）；`torrent_state/live/mod.rs:1964-2035`（`on_download_request`：校验片是否已拥有、入队、`upload_requests` 计数、超限报错断连）；`torrent_state/live/mod.rs:2037-2049`、`:2050-2076`（NotInterested / Cancel）。
- **适用条件**：超 per-peer 上限被当作**请求洪水**处理（直接断开该 peer），超 per-torrent 上限也报错——注意错误信息只有 `torrent upload backlog is full`，**不指明是哪个 peer 造成的**。
- **与当前 FluxDown 的差异**：FD 透传 `upload_queue_depth`、`upload_requests`、`uploading_peers`、`interested_peers`（`bt_downloader.rs:3169-3173`），因此配额是否顶满**可观测**；但**「因为超 per-peer 上限被断连的 peer 数」不可观测**——该路径只返回 `anyhow` 错误，不增加任何 counter。FD 的做种限速在 torrent 与 session 两级叠加（`live/mod.rs:781-787`），两级限速的等待时间不计入任何计数器。
- **可检验预测**：
  1. 若把 `MAX_PENDING_PER_PEER` 视为常量（确实不可配），则在 FD 上无法通过配置改变它；因此「上传背压调优」在不改 pin 的前提下不可行。
  2. 单 torrent 上传队列深度在健康 swarm 下应远低于 4096；若 `upload_queue_depth` 长期贴近上限，则吞吐瓶颈在磁盘读或限速而非网络。
  3. 轮转公平性的可检验后果：向 N 个 peer 同时上传时，各 peer 的完成速率应大致均等，而不随请求到达顺序倾斜；一个 peer 的请求积压（最多 128）不会阻塞其他 peer。

---

## 2. 会话准入预算：pinned fork 相对上游的主要结构改动

- **主张**：pinned fork 把上游「一个信号量覆盖 peer 全生命周期」拆成 **session 级 4 个信号量 + per-torrent 3 个信号量**，并为入站保留 headroom；magnet metadata 探测另走一条只能取 connecting 出站份额的独立租约路径。
- **证据**：`peer_admission.rs:1-23`（模块文档明确对比上游语义：「an outgoing connection takes a permit before dialling and only releases it when the connection ends, and an incoming connection that cannot take a permit is dropped on the floor」）；`peer_admission.rs:149-178`（4 个信号量初值 1024 / 960 / 128 / 112）；`peer_admission.rs:230-280`（获取顺序与全量回滚）；`peer_admission.rs:297-353`（per-torrent 三层）；`peer_admission.rs:211-228`、`440-448`（`MetadataLease` 类型隔离）；`peer_admission.rs:740-882`（`DialScheduler` 轮转公平 + 会话级定速）；`session.rs:1157-1196`（入站握手前拒绝）；`torrent_state/live/mod.rs:1540-1578`（握手回调内同步提升，失败即拆链）。
- **数值落点（默认配置）**：
  | 量 | 值 | 证据 |
  |---|---|---|
  | session 逻辑 peer | 1024 | `peer_admission.rs:53` |
  | session 逻辑 peer 的入站预留 | 64 | `peer_admission.rs:81-83` + `:118-121` |
  | session 逻辑 peer 的出站份额 | 960 | `peer_admission.rs:167-170` |
  | session connecting | 128 | `peer_admission.rs:55` |
  | session connecting 的入站预留 | 16 | `peer_admission.rs:86-88` |
  | session connecting 的出站份额 | 112 | `peer_admission.rs:172-175` |
  | per-torrent connecting | 64 | `peer_admission.rs:54` |
  | per-torrent live | 256（FD 默认） | `bt_downloader.rs:410` |
  | per-torrent live 的出站份额 | 240 | `peer_admission.rs:313-318` |
  | metadata 探测并发 | min(64, 8) = 8 | `peer_admission.rs:97-106` |
  | 拨号速率 | 30/s | `peer_admission.rs:56` + `bt_downloader.rs:797` |
- **重要组合事实**：FD 用 `bt_session_connect_limit(session)=(session/4).clamp(1,128)` 派生 session connecting（`bt_downloader.rs:440-448`），默认 1024/4 = 128，恰好等于 rqbit 默认。于是**生效的出站 connecting 额度是 112（session 出站份额），而 per-torrent 上限是 64**——单任务时 64 先绑定，多任务时 112 是硬顶。
- **与当前 FluxDown 的差异**：
  1. FD 未使用 `outbound_connect_rate` 以外的手段控制拨号；`DialScheduler` 的轮转公平对 FD 有益（多任务不再互相饿死），但它的效果**不可观测**：FD 只透传 `connecting`（当前值），没有「因预算被拒的拨号次数」。
  2. `rejected` 计数（`peer_admission.rs:184-187`）**没有进入任何快照**（不在 `AggregatePeerStats` 也不在 `StatsSnapshot`），因此 session 级「被准入拒绝的总次数」在 FD 上不可读；只有入站方向通过 `stats.peers.admission_rejected` 暴露（`session.rs:1185-1187`）。
  3. **FD 的注释与现状不一致**：`bt_downloader.rs:395-404` 把 128→512 的收益归因于「librqbit 用同一个信号量同时计连接中与已连接」，并称 512 让信号量不再成为瓶颈。但 FD 的 `build_opts` 已注入 `peer_admission`（`bt_downloader.rs:832-837`），该机制描述对应的是**上游语义**。因此那次 1.1→12.6 MB/s 的测量是在旧的单信号量语义下取得的，其归因对当前 pin 不再成立。
- **可检验预测**：
  1. 单任务 64 个出站连接中时（`connecting ≈ 64` 且 `live` 远低于 256），提升 `bt_peer_limit` 到 512 **不应**改变 `connecting` 上限——因为绑定的是 `torrent_connect_limit=64`。若观察到 `connecting` 随 `peer_limit` 上升，说明有其他路径。
  2. 多任务合计 `connecting` 应停在 112 附近（session 出站 connecting 份额），而不是 1024/512 的任意组合。若长期远低于 112 且候选充足，瓶颈在候选供给或拨号速率（30/s），可用 `connection_attempts` 的增速与 `queued` 存量判别。
  3. 入站拥塞时可检验：当 `live` 接近 256 且大部分是出站时，出站应停在 240（而非 256），同时入站仍可继续建立——若入站也被拒，说明预留逻辑未生效。

---

## 3. peer 轮换（turnover）：pinned commit 的主题，也是唯一的上传侧「清理」机制

- **主张**：pinned 版本用四个常量约束「过度淘汰」：候选须存活 ≥30s、空闲（双向无 payload）≥30s、无在途请求、且未表达 `Interested`；仅在 live 数达到 per-torrent 上限的 50%、有等待候选、且距上次运行 ≥60s 时运行；单次最多淘汰 `max(1, ceil(live/25))`（≈4%），淘汰方式是把 peer 置为 `NotNeeded`（策略断开，不计网络错误、不安排 backoff）。
- **证据**：`torrent_state/live/mod.rs:249-272`（四个常量及注释「cycled out within the first minute of its life」，与 commit 标题「收缩 peer 轮换的判定窗口到一分钟内」一致）；`:274-302`（`turnover_eligible`）；`:304-396`（`maybe_turnover_peers`、`budget = max(1, live_now.div_ceil(25))`、`set_state(PeerState::NotNeeded, ..)`）；触发点 `:489-496`（`DialSource::release_idle_peers` 中调用，即**只在准入被拒时**才有机会运行）。
- **文档与代码的局部不一致**：`maybe_turnover_peers` 的函数文档写「at or above 90% of the per-torrent cap」（`live/mod.rs:307-308`），而代码用 `TURNOVER_LIVE_THRESHOLD_PERCENT = 50`（`:272`、`:337`）。以代码为准；引用该策略时必须注明这一点。
- **与当前 FluxDown 的差异**：
  1. FD 不覆盖 turnover 任何常量，因此**FD 的「僵尸 peer 清理」完全依赖这个 30s/30s/60s 机制**，而它只有在准入失败时才会被触发（`release_idle_peers` 的调用点是 `DialScheduler::step` 的 `Err(_)` 分支，`peer_admission.rs:850-857`）。含意：**当准入不紧张（`connecting` 未满载、候选不排队）时，即使存在大量空闲 peer，turnover 一次也不会跑**。
  2. `farmer` 式判据「无在途请求 且 未 Interested」意味着：一个只做种不下载、但对方对我们数据有需求（已 `Interested`）的连接**永远不会被轮换**——它是有效上传贡献者，这符合设计意图。
- **可检验预测**：
  1. 若在低压力场景（`live` 远低于上限、无候选积压）观察到 `dead` 数长期不增长而 `live` 保持高位不变，则印证「turnover 未被触发」；此时 `not_needed` 应接近 0。
  2. 在人为制造候选积压（提供大量不可达候选地址）后，应在 ≤90s 内观察到至多约 4% 的 live peer 转入 `not_needed`，且 `connection_errors` **不因这些断开而增加**（因为策略断开不计错误）。若 `connection_errors` 同步跳升，说明这些连接是自然失败而非被本轮换。
  3. 一个持续下载但不给任何数据的 peer（有在途请求）不会被轮换；把该 peer 的响应完全静默（连 choke 都不发）后，它应因读写超时（FD 配 20s）而进入 `dead`，早于 30s 的 turnover 判据生效——因此**在 FD 的 20s 读写超时配置下，turnover 对「完全不响应」的 peer 几乎无用，真正被它清掉的是「响应但不给数据」的 peer**。

---

## 4. 计数器的真实作用域（引用时必须区分）

| 计数器 | 作用域 | 归零条件 | 证据 |
|---|---|---|---|
| `session_stats.counters.fetched_bytes` / `uploaded_bytes` | session 生命周期 | session 销毁 | `session_stats/mod.rs:13-20` |
| `session_stats.peers.*`（含 `seen`/`live`/`admission_rejected`） | session 级聚合 | session 销毁 | `session_stats/mod.rs:22-26`；`peers/stats/mod.rs:12-41` |
| `StatsSnapshot.uploaded_bytes` / `connection_*` / `total_connecting_ms` / `upload_requests` / `peer_stats.steals` | **单个 live 实例** | torrent 暂停/恢复产生新 live 实例即归零 | `live/stats/snapshot.rs:7-36`（`live_epoch` 文档） |
| `seen` | torrent 级累计去重地址数 | 不归零；peer 死亡不减少 | `peers/mod.rs:139-158`、`live/mod.rs:716-727` |
| `live` / `connecting` / `queued` / `dead` / `not_needed` | point-in-time | — | `peers/stats/mod.rs:43-52` |
| `useful_peers` | torrent 级，**上传侧** ≥1 MiB 阈值 | 跨阈值 +1、断开 −1 | `peer/stats/atomic.rs:11-17`、`peers/stats/mod.rs:29-33` |
| `interested_peers` | torrent 级，「对端向我们表达需求」 | 同上 | `peers/stats/mod.rs:25-28` |
| `admission_rejected` / `incoming_rejected` | **session 级** | session 销毁 | `peers/stats/mod.rs:34-40` 注释 |
| `peer_admission.rejected` | session 级，**许可获取失败次数**（非 peer 数） | session 销毁 | `peer_admission.rs:184-187`、`:282-294` |

**两条必须记住的推论**：

1. 跨 pause/resume 累加上传，必须用 `live_epoch`（`live/stats/snapshot.rs:14-19`）作为基线键；否则「计数器归零」与「没有新上传」不可区分。FD 的 `bt_seeding.rs` 已严格遵循这条：`QueuedSeed` 不保留上传检查点（`bt_seeding.rs:320-327`），`reconcile` 提升排队项时把 `last_committed_upload` 置 `None` 并注释「其首个样本带新 epoch，全额计入」（`bt_seeding.rs:478-490`），`observe_upload` 则按 `prev.live_epoch == snapshot.live_epoch` 区分「同一 live 实例只认严格更大的计数」与「epoch 变了即全额计入」（`bt_seeding.rs:549-570`）。
2. rqbit 的 `useful_peers` 是**上传侧** 1 MiB 阈值（peer 从我们这里取走 ≥1 MiB）。FD 的 `BtPeerStats.useful_peers` 字段注释写「Live peers that have fetched at least 1 MiB for this torrent」（`transfer_activity.rs:38-39`），方向相反。**引用 FD 的 `usefulPeers` 时必须先澄清方向**，否则会把「有多少人在下载我们」误读成「有多少 peer 给我们数据」。

---

## 5. 既有文档的 pin 与「设计 vs 已落地」

### 5.1 pin 不一致（已登记为 S-DOC-FD-PIN-01/02）

| 位置 | 文档写的 revision | 与现状 |
|---|---|---|
| `bt-observability-plan.md:14` | `d995b0c9583faf08b42bd55ddfa0ee529e5ec9c1`（2026-10-16） | **不一致**：`Cargo.toml:52` 与 `Cargo.lock:5751-5753` 均为 `6f9d64b310…` |
| `bt-backend-migration.md:999` | `d995b0c…`（称 2026-10-12 随上游 #825 更新） | **不一致** |
| `bt-backend-migration.md:1071` | 断言「`Cargo.toml` patch 与 checkout 都是 `d995b0c…`」 | **事实错误** |
| `bt-backend-migration.md:52` | `002a34848162358e79c094f7284c5bcb210a54fa` | **不一致**（第三个值） |

处置：引用这两份文档的任何「pinned rqbit 已具备/不具备 X」结论之前，必须按 `6f9d64b` 重新核对。本 slice 的所有 rqbit 结论都已用 `6f9d64b` 的源码行号给出，未依赖文档转述。

### 5.2 逐条「设计 / 已落地 / 本机可观测」判定

| 文档条目 | 判定 | 依据 |
|---|---|---|
| `per_peer_stats_snapshot()` 可返回逐 peer 记录 | **已落地** | `torrent_state/live/mod.rs:1123-1132`、`api.rs:275-290` |
| 逐 peer 计数器含 `fetched_bytes`/`uploaded_bytes`/`connection_attempts`/`total_time_connecting_ms`/`downloaded_and_checked_pieces`/`total_piece_download_ms` | **已落地，但字段名不同** | `peer/stats/atomic.rs:19-34` 实际为 `incoming_connections` / `outgoing_connection_attempts` / `outgoing_connections` / `errors`；文档写的 `connections` 在 pinned 中不存在（`bt-observability-plan.md:16-24` 需修正） |
| 逐 peer 记录有 `source`（tracker/DHT/PEX/LSD）字段 | **未落地（设计）** | pinned 中 peer record 无来源字段；检索无命中 |
| 握手阶段 / 失败原因枚举计数 | **未落地（设计）** | `AggregatePeerStats` 与 `StatsSnapshot` 均无相关字段（`peers/stats/mod.rs:12-41`、`live/stats/snapshot.rs:7-36`） |
| `daemon.task.peerStats` RPC、`BtPeerObservation` DTO | **未落地（设计）** | `native/protocol/src` 与 `native/daemon/src` 无命中 |
| `usefulPeers`（累计）/ `contributingPeers`（窗口）分离 | **未落地（设计）** | 只有 `useful_peers`，且方向是上传侧 |
| `primarySourceContribution` / `sourceReachability` 两套口径 | **未落地（设计）**，文档自己推荐第一版只做后者 | `bt-observability-plan.md:338-345` |
| 不引入自动 peer 调优控制器（第一阶段不提供 `auto`） | **明确不做（设计边界）** | `bt-backend-migration.md:1056`、`bt-observability-plan.md:403,457` |
| request window 固定 128 | **现状判断，正确** | 与 §1.2 核对一致 |
| `peer_limit` 无可靠 live setter | **现状判断，正确** | 与 §6 `S-CODE-FD-HOT-03` 一致 |
| 「已有 torrent 不删除重加才生效」 | **现状判断，正确** | `bt_downloader.rs:1496-1505`、`:4139-4144` |

### 5.3 本机可观测的既有面（不依赖文档）

FD 已经通过 `BtPeerStats` 把 rqbit 的聚合暴露到 runtime 事件：`bt_downloader.rs:3128-3177` 逐字段映射，包含 `seen/queued/connecting/live/dead/live_tcp/live_utp/live_socks/useful_peers/connection_attempts/connection_successes/connection_errors/total_connecting_ms/fetched_bytes/downloaded_pieces/total_piece_download_ms/peer_steals/uploaded_bytes/uploading_peers/interested_peers/upload_requests/upload_queue_depth/incoming_rejected/admission_rejected`（外加 `BtConfig` 未覆盖的 `live_socks`）。三个 u64 连接计数用 `u32::try_from(..).unwrap_or(u32::MAX)` 钳制（`:3142-3144`），因此极高计数会饱和而不是回绕——引用 `connection_attempts` 达到 `u32::MAX` 的样本时要注意这是饱和值。

---

## 6. rqbit 实际能力 → FluxDown 适配 → 可观测缺口

| 能力/机制 | pinned rqbit 实际实现（locator） | FluxDown 适配（locator） | 可观测缺口 |
|---|---|---|---|
| piece 选择 | 顺序式：文件名字母序 + 每文件首尾优先（`piece_tracker.rs:112-164`、`chunk_tracker.rs:236-249`、`file_info.rs:14-36`） | 未覆盖，全部走 rqbit | 无「当前在请求哪些片 / 某片被请求了几次」指标；`inflight` 大小只有 per-peer 私有字段 |
| 请求窗口 | 默认 128 chunk，可配，reqq 单向下调（`live/mod.rs:1389,525-532,1606-1628`） | **未设置**→取默认 128（`bt_downloader.rs:781-799`） | 无「窗口占用率」「因窗口满导致的等待次数」；`wait_for_request_slot` 不计数 |
| choker | **无 choker**，无条件 Unchoke（`peer_connection.rs:458-486`） | 无相关配置 | 无 choke 事件流；「上传公平性」只能从 `upload_queue_depth` 间接看 |
| 上传调度 | per-torrent 单任务 + peer 轮转 + peer 内 FIFO + 128/4096 上限（`upload_queue.rs:25-31,122-142`、`live/mod.rs:750-820`） | 透传 `upload_queue_depth`/`upload_requests`（`bt_downloader.rs:3169-3173`） | 无 per-peer 上传队列深度；无「超 per-peer 上限被断连」计数 |
| 会话准入 | 4 + 3 个信号量、入站预留、metadata 独立租约、轮转拨号器（`peer_admission.rs:149-178,297-353,740-882`） | 注入 `PeerAdmissionConfig`（`bt_downloader.rs:832-837`） | `peer_admission.rejected` 未进入任何快照；无「按 `AdmissionDenied` 原因分解」的计数 |
| 入站拒绝 | 握手前拒绝 + `admission_rejected`（`session.rs:1157-1196`） | 透传 `admission_rejected`（`bt_downloader.rs:3175`） | 与 `incoming_rejected` 的区分度在 FD 侧已有；但 session 级数值落到逐 task 事件里易被误读为 per-task |
| peer 轮换 | 30s/30s/60s/50%、≈4%、`NotNeeded` 策略断开（`live/mod.rs:249-396`） | 未覆盖 | 无「本轮换淘汰了多少 peer」；`not_needed` 是当前值，不是累计淘汰数；策略断开不计入 `connection_errors`，所以无法从错误率反推 |
| TCP/uTP 竞速 | TCP 优先，1s 后并起 uTP（`stream_connect.rs:247-368`） | `enable_utp` 决定 `ListenerMode`（`bt_downloader.rs:765-779`） | per-family transport 统计在 `ConnectStatsAtomic` 中，但 **session 快照只在 `connections` 字段暴露**（`session_stats/snapshot.rs:14-20` 的 `ConnectStatsSnapshot` 未进入 `BtPeerStats`）；`live_tcp/live_utp` 只反映当前存量 |
| MSE | Disabled/Enabled/Forced，Enabled 下明文回退 2s 嗅探（`mse/mod.rs:26-42,185-195`） | `BtMseMode` 映射 + `mse_mode` 下发（`bt_downloader.rs:349-356,818`） | 无「MSE 成功/回退/强制失败」计数；Forced 下的丢弃只表现为连接失败 |
| 超时 | rw 10s / keepalive 120s 默认；connect/handshake 无默认（`peer_connection.rs:71-103,431-434,458-462`） | 覆盖为 connect 10s / rw 20s / handshake 10s（`bt_downloader.rs:788-794`） | 无「超时按阶段分解」的计数；`connection_errors` 只有总数 |
| 存储 | 同步 + positioned vectored pwritev（`storage/mod.rs:1-23`）；sparse 仅 Windows（`filesystem/sparse.rs`、`fs.rs:69-99`） | `bt_sparse` 补 Windows init 期标记（`bt_sparse.rs:88-101`）；`bt_partfile` 边车（`bt_partfile.rs:1-20`） | 无磁盘队列深度 / 写盘延迟 / 校验耗时计数器 |
| 统计 | `AggregatePeerStats` 17 字段 + `StatsSnapshot` 25 字段（`peers/stats/mod.rs:12-41`、`live/stats/snapshot.rs:7-36`） | 逐字段映射到 `BtPeerStats`（`bt_downloader.rs:3128-3177`） | `session_stats` 的 `ConnectStatsSnapshot` 未透传；Prometheus 导出有 `rqbit_peers_queued` 重名缺陷（`session_stats/snapshot.rs:96-97`） |
| 逐 peer 明细 | `per_peer_stats_snapshot` 可用（`live/mod.rs:1123-1132`） | **未调用**（`native/` 无命中） | 整个 per-peer 观测面缺失：无法回答「哪个 peer 贡献了多少字节」「peer 的 client 名/transport/state」 |

---

## 7. `rqbit-fd` 缺口清单（要说清这四个问题还缺哪些计数器）

**A. 回答 piece 策略与流水线所缺**

1. **per-peer inflight chunk 请求数**（`live.peer/mod.rs:356-358` 已有，但只到 per-peer 内部）：缺它就无法验证「窗口是否绑定」，只能从 `download_speed` 反推。
2. **窗口满导致的等待次数 / 等待时长**：`wait_for_request_slot`（`live/mod.rs:2155-2166`）是唯一的窗口压力点，目前完全静默。缺它无法区分「peer 不给数据」与「我们没在请求」。
3. **每次 `acquire_piece` 的结果分布**（`Reserved` / `Stolen` / `NoneAvailable`）：只有 `peers.steals` 累计数，缺 `NoneAvailable` 次数与抢夺的触发频率，无法判断「无片可取」还是「窗口限制」。
4. **当前请求中的片索引集合（或分布摘要）**：用于验证「顺序式 + 首尾优先」的实际落点，并区分「选择顺序」与「peer 可用性过滤」的贡献。

**B. 回答上传调度与 choker 所缺**

5. **per-peer 上传队列深度**：只有 torrent 级 `upload_queue_depth`，无法判断公平性是否被单个 peer 拖累。
6. **超 per-peer 上限（`PeerBacklogExceeded`）的断连计数**：当前只返回 `anyhow` 错误（`live/mod.rs:2022-2027`），该异常在 FD 的 `connection_errors` 里与网络失败混在一起。
7. **`Cancel` / `NotInterested` 丢弃的待发请求数**：`drop_peer` / `cancel` 的返回值只进 `trace!`（`live/mod.rs:2041-2046`、`:2061-2074`），无法量化「对端取消造成的无效读盘」。
8. **无 choker 的显式确认指标**：需要一个「本连接是否曾下发 Choke」的常量/断言式观测（或日志校验），否则「没有 choker」这一机制结论只能靠读代码。

**C. 回答准入门槛所缺**

9. **`PeerAdmission::rejected` 与 `AdmissionDenied` 原因分解**：`rejected` 是许可获取失败次数（`peer_admission.rs:184-187`），既未进 `AggregatePeerStats` 也未进快照；`as_str()` 的四个标签只在 `debug!` 日志里出现一次（`session.rs:1188-1193`）。缺它就无法回答「瓶颈是 session peer / session connecting / torrent connecting / torrent live 中的哪一个」。
10. **按时刻的四个信号量可用额度**（`global` / `global_outbound` / `connecting` / `connecting_outbound` 的 `available_permits()`）：这是判断「预算是否真的顶格」的唯一直接证据。当前只能从 `connecting` 与 `live` 的存量间接推断，而 `connecting=112` 顶格与「候选不足」在现有指标下**不可区分**（两者的 `connecting` 都低于上限）。
11. **候选队列深度与拨号速率实际值**：`DialScheduler` 每 tick 启动的 dial 数、被 `dial_is_due` 跳过的次数、候选队列长度。缺它无法判断 `queued` 存量高是因为拨号速率（30/s）还是因为预算。
12. **`metadata` 探测的占用与失败**：`MetadataLease` 是独立预算（`peer_admission.rs:211-228`），但 magnet 阶段的探测次数/成功率完全没有计数器——这直接影响「首个 payload 时延」的归因。

**D. 回答 peer 轮换与长尾所缺**

13. **turnover 的累计淘汰数与触发次数**：`maybe_turnover_peers` 返回 `rotated`，调用方 `release_idle_peers` 丢弃该返回值（`live/mod.rs:489-496`）。缺它就无法知道轮换是否在运行。
14. **被策略断开（`NotNeeded`）与自然失败（`dead`）的区分计数**：`not_needed` 是 point-in-time 存量，`connection_errors` 不含策略断开，因此「为什么这个 peer 消失了」currently 无解。
15. **per-peer 存活时长与空闲时长的分布**：`connected_at` / `last_download_at` / `last_upload_at` 已在 `LivePeerState`（`peer/mod.rs:315-321`），但没有聚合导出；缺它无法判断「30s 判据是否与 FD 的 20s 读写超时互相重叠」。

**E. 回答统计口径所缺**

16. **`ConnectStatsSnapshot` 进入 `BtPeerStats`**：`{tcp,utp,socks} × {v4,v6}` 的 attempts/successes/errors 已经存在于 rqbit（`stream_connect.rs:115-127`），但 FD 只透传 `live_tcp/live_utp/live_socks` 三个存量值。缺它无法回答「uTP 到底贡献了多少连接成功、IPv6 是否在起作用」。
17. **握手阶段与失败的分子/分母**：pinned 中不存在（`AggregatePeerStats` 无对应字段），FD 也不存在。在补齐之前，**任何「握手成功率」都必须写成「transport 连接成功率」**，即 `connection_successes / connection_attempts`，不能声称是 BT 握手成功。
18. **`fetched_bytes` 的窗口增量**：FD 透传的是累计 `fetched_bytes`。要区分「哪些 peer 在当前窗口仍在贡献」（`contributingPeers`）必须做窗口差分，而 per-peer 都不导出，因此窗口贡献 peer 数**当前不可得**。

---

## 8. 本 slice 内发现的冲突（供交叉核对）

| 冲突 | 位置 | 性质 |
|---|---|---|
| turnover 阈值：文档注 90% vs 代码常量 50% | `live/mod.rs:307-308` vs `:272,337` | **代码内注释与实现不一致**，以代码为准 |
| `set_bt_config` 称 `bt_peer_limit` 立即对运行中的 session 生效 vs 实际只影响后续 add | `download_manager.rs:4767-4768` vs `bt_downloader.rs:1020-1023,1496-1505,4139-4144` | **注释与实现不一致**；迁移文档 `:1022` 的「无可靠 live setter」判断是对的 |
| FD 把 128→512 的收益归因于「单信号量被连接中占满」 vs 当前已注入 `peer_admission` | `bt_downloader.rs:395-404` vs `:832-837` | **旧归因对当前 pin 已不成立**，该测量需在新语义下重做 |
| FD `useful_peers` 注释为下载侧 vs rqbit 实现为上传侧 | `transfer_activity.rs:38-39` vs `peer/stats/atomic.rs:11-17` | **语义方向冲突**，引用前必须澄清 |
| `bt-backend-migration.md:1071` 断言 patch 与 checkout 均为 `d995b0c…` vs `Cargo.toml:52` 为 `6f9d64b…` | 文档 vs 仓库 | **文档事实性错误** |
| `bt-observability-plan.md:16-24` 列 `connections` 字段 vs pinned 实为 `outgoing_connections`/`incoming_connections` | 文档 vs `peer/stats/atomic.rs:19-34` | **文档字段名错误** |
| `rqbit_peers_queued` 被写两次（queued 与 seen） | `session_stats/snapshot.rs:96-97` | **上游可观测性缺陷**，Prometheus 读数不可用 |
