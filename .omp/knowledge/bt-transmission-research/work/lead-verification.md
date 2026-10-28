# 独立复核台账（Lead 自验，非子任务产出）

本文件记录**由 lead 亲自读取原文/源码**得到的核对结论，用于 `README.md` 的「来源追溯」验收与机制对照表。
每条都给出：结论 / 证据 locator / 与前人文档或子任务产出的关系。

---

## V1 · 版本与 pin 冻结（已核对）

| 项 | 值 | 证据 |
|---|---|---|
| rqbit pin | `6f9d64b310ecb560990315af3c60df3111eec8e7`（version `9.0.1`） | `Cargo.toml:52` `rev = "6f9d64b…"`；`Cargo.lock` `librqbit 9.0.1` + 同 rev；checkout `.cargo/git/checkouts/rqbit-341c9d2d87cbd6c5/6f9d64b/` |
| 该 commit | 2026-10-16 `feat(bt): 收缩 peer 轮换的判定窗口到一分钟内` | `git -C <checkout> log -1` |
| libtorrent | `54bc4e924b34e34212c0af9584921039e438ea84`（branch `RC_2_1`，2026-08-24）；**grafted shallow clone，无 tag 可达** | `git -C /home/ldsr/code/C-Cpp/libtorrent log -1`；`tag --points-at HEAD` 空 |
| qBittorrent | `607c9881e540510e0baaad28bc9ab5607ad20cd4`（master，2026-08-24） | 同上 |
| transmission | `c22a63d52f01d301f2131e4e2102ecc0ac3bb2df`（main）；版本 `4.2.0` | `git log -1`；`transmission/CMakeLists.txt:112-114` `TR_VERSION_MAJOR/MINOR/PATCH = 4/2/0` |
| aria2 | `9e7273583f83e881e3ec067b523ba88724088d2f`（master）；版本 `1.37.0` | `git log -1`；`aria2/configure.ac` `AC_INIT([aria2],[1.37.0]…)` |
| libtorrent 子模块 | 未初始化（`-` 前缀） | `git submodule status`：`deps/asio-gnutls`、`deps/libdatachannel`、`deps/try_signal`、`simulation/libsimulator` |

**限制**：四个参考仓库都是 grafted shallow clone，`git describe --tags` 只能给出 commit 短号，**不能**用可变网页的版本号替代这些 commit。

---

## V2 · BEP 15 IPv6 紧凑格式（原文逐字核对 ✅）

- 结论：`BEP 15` 明确 **IPv6 announce 复用 v4 结构，但响应中 `<IP address, TCP port>` 对的 stride 为 18 字节而非 6**；请求里的 IP 字段仍是 32 位、IPv6 下应置 0；格式由底层 UDP 包的地址族决定。
- 证据：`https://www.bittorrent.org/beps/bep_0015.html`（Version `023256c7581a4bed356e47caf8632be2834211bd`）「## Announce / ### IPv6」节。
- 与 specs 子任务产出：`S-BEP-15` claim 与原文逐字一致（无夸大）。
- 附加事实（原文同页）：UDP tracker 重传曲线为 `15 * 2^n` 秒，n 从 0 增至 8（3840 s）；`connection_id` 客户端可用到收到后 1 分钟、tracker 应接受到发出后 2 分钟；`Post-History: 06-11-2016 … specified IPv6 format`。

## V3 · BEP 3 的两条规范语句（原文逐字核对 ✅，specs slice 原缺 → 已补）

- 结论 A（片序）：`Downloaders generally download pieces in random order, which does a reasonably good job of keeping them from having a strict subset or superset of the pieces of any of their peers.`
- 结论 B（流水线义务）：`downloaders should keep several piece requests queued up at once in order to get good TCP performance (this is called 'pipelining')`；且「无法立即写入 TCP buffer 的请求应排队在内存中，而不是应用级网络缓冲，以便 choke 时整批丢弃」。
- 证据：`https://www.bittorrent.org/beps/bep_0003.html`（Version `0e08ddf84d8d3bf101cdf897fc312f2774588c9e`）「# peer messages」`'piece'` 段之后（结论 A）；「# peer protocol / Connections start out choked and not interested」段之后（结论 B）。
- 关系：`specs.sources.csv` 未收录这两条；已由 lead 写入 `work/specs.addendum.csv`（locator 指向原文段）。
- 意义：**规范本身并不要求 rarest-first**；它要求的是「随机顺序（描述性）」+「一次排多个请求（SHOULD）」。任何「规范要求 rarest-first」的说法都是错的。

## V4 · BEP 52 的「测试向量」到底在哪（原文核对 ✅）

- 结论 1：`BEP 52` 正文**没有**测试向量引用。其「# Resources」只列 BitTorrent Economics PDF 与 Wireshark dissector。
  - 证据：`https://www.bittorrent.org/beps/bep_0052.html`（Version `7392d83f374b071ca9601728c27be2d05e199e0c`，Status **Draft**）第 541–547 行。
- 结论 2：BEP 52 正文指向的**可执行参考实现**是 `http://bittorrent.org/beps/bep_0052_torrent_creator.py`（已实际读取全文）。
  - 关键语义（可当 byte-level 口径依据）：`BLOCK_SIZE = 2**14`（16 KiB）；`meta version = 2`；`file tree` 以零长度键 `b''` 存放 properties；`pieces root` = 该文件 merkle 树根；`piece layers` 只对 `length > piece_length` 的文件生成条目；hybrid 额外写 `pieces` 与 `files`/`length`；`.pad` 文件以 `attr='p'` 标记；**merkle padding 用 32 字节全零 `bytes(32)` 而不是 `hash(zeros)`**；小文件在少于一个 piece 时按「下一个 2 的幂」补零而不是按 piece 边界。
- 结论 3：作者（libtorrent/arvid）发布的官方**测试种子**是真实可取的集成 fixture：
  - `https://libtorrent.org/bittorrent-v2-test.torrent`（v2-only，`urn:btmh:1220caf1e1c3…302e`）
  - `https://libtorrent.org/bittorrent-v2-hybrid-test.torrent`（hybrid，`urn:btih:631a31dd…42ac` + `urn:btmh:1220d8dd32…efabb`）
  - 证据：`https://blog.libtorrent.org/2020/09/bittorrent-v2/`「# testing」节。
- 结论 4：候选仓库 `github.com/bittorrent/bittorrent-v2-test-vectors` 与 `github.com/the8472/bittorrent-v2-test-vectors` 均 **HTTP 404**（lead 各实测一次）。**不得**在交付物中声称存在该仓库。
- 同源机制事实（blog 已读）：v2 用 SHA-256，info-hash 为 SHA-256，对 DHT/tracker 公告时**截断为 20 字节**；merkle 树叶固定 16 KiB（= block size），piece size 仍存在且仍用于 HAVE/REQUEST；片的 verifier 是「piece 子树的根」；每文件独立 merkle 树 ⇒ 文件天然按 piece 对齐（等价于文件后隐含 pad 文件）；hybrid 有**两个 info-hash ⇒ 两个 swarm**；v2 支持位经**新增 PEX flag** 传递（不是 BEP 11 的标准位）；libtorrent 2.0 起对 v2 torrent **强制拒绝**前导零（如 `i0004e`）、`i-0e`、未排序字典键，理由是 round-trip 会改变 info-hash。

## V5 · pinned rqbit 的 v2 支持是**部分**的（源码核对 ✅）

| 能力 | 状态 | 证据 |
|---|---|---|
| `Id32`（32 字节 hash）与「截断 20 字节供 DHT/tracker」 | **有** | `crates/librqbit_core/src/hash_id.rs:182-185`（注释明写 BEP 52） |
| magnet `urn:btmh:1220` 解析 / `as_id32()` | **有** | `crates/librqbit_core/src/magnet.rs:81-97`、`:27`；`:18` 要求 btih 或 btmh 至少一个 |
| 以官方 v2 测试种子为**测试向量** | **有** | `crates/librqbit_core/src/magnet.rs:200-212` `test_parse_magnet_v2`，用的正是 `btmh:1220caf1e1c3…302e`（与 libtorrent blog 同一 infohash） |
| SHA-256 摘要 trait 骨架 | **有** | `crates/sha1w/src/lib.rs:14`（注释明写 BEP 52） |
| `file_tree` / `piece_layers` / `meta version` 解析 | **无** | 全仓 grep 只命中 `error.rs`，无解析实现 |
| merkle 校验模块 | **无** | `find crates -name "*merkle*"` 为空 |
| `hash request`/`hashes`/`hash reject`（消息号 21/22/23） | **无** | peer 协议无对应消息 |
| 14 个 `V2*` 错误变体 | **已定义但从未被抛出** | `crates/librqbit_core/src/error.rs:24-54`；对这些变体名的全仓 grep 在 `error.rs` 之外**零命中** |

**结论**：pinned rqbit 能**识别** v2 与 hybrid 的 infohash 与 magnet，但**不做** v2 元数据/merkle 校验。任何「已支持 BEP 52」的说法必须限定为「v2 infohash/magnet 层面」。
**旁证**：`ikatson/rqbit` issue #546（2026-02-02 开）为 v2 支持的**设计讨论**，状态 Open，先做 v2-only、16 个 stacked PR；作者指出 hash 消息是标准 wire 消息（固定 ID 21/22/23）而**不是** BEP 10 扩展消息；hybrid 要求 v1 与 v2 校验**都通过**而非「优先 v2」。
**注意**：该 issue 是上游仓库的设计记录，**不能**当作 pinned fork 的实现状态。

## V6 · pinned rqbit 的 per-peer 请求窗口（源码核对 ✅）

- 配置项：`PeerConnectionOptions::max_request_window: Option<usize>`，doc comment 明写「per peer 最多保留的 chunk 请求数（**默认 128**，且**绝不超过对端 reqq**）」——`crates/librqbit/src/peer_connection.rs:98-102`。
- 默认值常量：`DEFAULT_PEER_REQUEST_WINDOW = 128`——`crates/librqbit/src/torrent_state/live/mod.rs:1389`。
- 生效值：`max_request_window()` = `peer_max_request_window.unwrap_or(DEFAULT).max(1)`——`live/mod.rs:526-532`；扩展握手后 `request_window = reqq.min(state.max_request_window())`——`live/mod.rs:1618-1626`。即**有效窗口 = min(对端 reqq, 本端配置)**。
- 发送门控：`can_send_request()` = 未被 choke 且 `requested_inflight_count() < request_window`——`live/mod.rs:2615-2627`。
- **FluxDown 未配置该项**：`grep 'max_request_window|peer_max_request_window' native/engine/src/ native/protocol/src/` 零命中 ⇒ 恒定走默认 128（被对端 reqq 夹住）。
- 相关历史：fork 内有 `79ff792e feat(peer): make the per-peer request window configurable`（上游能力进入 fork）。
- **口径澄清**：这是「per-peer 在途请求数」上限，**不是**带宽时延积推导出的最优窗口，也**不是** session 级总量。

## V7 · pinned rqbit 的 piece 选择**不是** rarest-first（源码核对 ✅）

- 入口：`crates/librqbit/src/piece_tracker.rs:123-164` `acquire_piece()`，doc comment 明写三步策略：
  1. `try_steal` 阈值 **10×**（从慢 10 倍的对端手里抢）；
  2. 先遍历 **`priority_pieces`**（streaming 读位置），再遍历 `chunks.iter_queued_pieces(file_priorities, file_infos)` 的「自然排队顺序」；
  3. `try_steal` 阈值 **3×**。
- 排队顺序：`crates/librqbit/src/chunk_tracker.rs:236-249` `iter_queued_pieces()` = 按 `file_priorities` 逐文件 → 每文件内 `file_infos.iter_piece_priorities()` → 过滤 `queue_pieces`。
- 文件内片序：`crates/librqbit/src/file_info.rs:15-26,33-35` `iter_piece_priorities()` = **该文件的第一个 piece、最后一个 piece，然后其余按索引升序**。
- 调用方：`live/mod.rs:1881-1940` `acquire_next_piece()`（注释「try steal (10x) → reserve → steal (3x)」），传 `peer_avg_time` 用于 steal 判定。
- **结论**：pinned rqbit 无 rarest-first、无随机片序；是「文件优先 + 文件首尾优先 + 升序」的排队序，外加「抢慢对端」的加速机制。与 rqbit 自述的 sequential 为主相符。
- 另：`InflightPiece` 保证一个 piece 同时只归一个对端（除被抢），`into_chunks()` 在暂停时把在途 piece 重新排队（`piece_tracker.rs:104-110`）——这是 pause/resume 正确性的相关机制。

## V8 · pinned rqbit 的上传请求队列（源码核对 ✅）

- `crates/librqbit/src/torrent_state/live/upload_queue.rs`：
  - `MAX_PENDING_PER_PEER = 128`（`:28`）、`MAX_PENDING_PER_TORRENT = 4096`（`:31`）。
  - 入队结果 `EnqueueOutcome`：`Duplicate`（同 chunk 去重）/ `PeerBacklogExceeded`（**视为请求洪水，调用方应断开该 peer**）/ `TorrentBacklogExceeded`（**真实错误，不入队、不静默丢入无限等待**）/ `Queued`（`:51-64`、`:100-120`）。
  - 调度：`rotation: VecDeque` **轮转服务**各 peer（`:80`、`:123-142`），保证不被单 peer 饿死。
  - `generation` 守卫：连接身份变化后，旧连接的排队请求**永不被写入新连接**（`:43-49`、`:148-167`）。
- 这是「上传侧背压 + 公平轮转 + 连接代际安全」的完整机制，可直接对照 E1-5（上传限速与互惠）与 E0（epoch 对齐）。

## V9 · pinned rqbit 的准入层次与保留量（源码核对 ✅）

- 四个信号量：`global`(session_peer_limit)、`global_outbound`、`connecting`(session_connect_limit)、`connecting_outbound`——`crates/librqbit/src/peer_admission.rs:150-178`。
- 保留量：`global_reserve = reserve(session_peer_limit,16,8,64)`、`connecting_reserve = reserve(session_connect_limit,8,2,16)`——`:80-88`；`reserve()` 定义 `:116-121`（`budget/divisor` 夹到 `[min,max]`，再夹到 `budget-1`，**永不留满**）。
- 出站额度：`outbound_budget(budget,reserve) = budget.saturating_sub(reserve).max(1).min(budget)`——`:108-114`（**非零预算至少放行 1 个出站**）。
- metadata 探测并发上限：`max_concurrent_metadata_reads = torrent_connect_limit.min(8)`——`:90-106`。
- 拒绝原因：`AdmissionDenied = {SessionPeers, SessionConnecting, TorrentConnecting, TorrentLive}`，各带稳定标签（`session-peer-limit` 等）——`:123-146`。
- **缺口**：`PeerAdmission` 只有单一 `rejected: AtomicU64`——`:156`。要按 `M-CONN-06` 分原因计数**必须**改 rqbit（归入缺口 G4）。
- **口径澄清**：`global_outbound`/`connecting_outbound` 是**保留后的有效出站额度**，与会话总预算（`global`）不同名也不同意——这正是 `M-BUDGET-01` vs `M-BUDGET-02` 要分开的原因。

## V10 · 文档 pin 漂移（必须登记）

- `.omp/knowledge/bt-observability-plan.md:14` 写 pinned checkout 为 `d995b0c9583faf08b42bd55ddfa0ee529e5ec9c1`（2026-10-16）。
- `.omp/knowledge/bt-backend-migration.md:999` 写 `d995b0c9583faf08b42bd55ddfa0ee529e5ec9c1`（2026-10-12；此前 `d9e79c939f223679612cd0b06bd00ed49bd2eff8`）。
- **实际** pin 是 `6f9d64b310ecb560990315af3c60df3111eec8e7`；`git log` 显示 `d995b0c9` 是 `6f9d64b3` 的**祖先**（其后至少还有 `c17ed3ae`/`70a73c2e`/`076d8a06`/`e2b7e62a`/`6f9d64b3` 等）。
- **含义**：两份文档的「已确认事实」描述的是**更早的 rev**。引用它们时必须标注为「文档所述 rev，早于当前 pin」，不得当作当前 pin 的观测结果。
- 同批可验证的上游修复确实已在 fork 中（`git log` 命中）：`57e95d16`（握手读完整）、`2b123074`（Windows DHT 单包错误不杀 worker）、`176484e2`（storage wrapper 转发 `pwrite_all_vectored`）、`8ebbd6c8`（streaming 等初始检查）、`2869d7ae`（只把 40 位 hex 当 info-hash）、`3599380b`（统计拥有完整 torrent 的已连接 peer）、`79ff792e`（请求窗口可配）、`d9e79c93`（Android HTTPS 不 panic / tracker panic 隔离）——这些支持 `bt-backend-migration.md` §2.2 的结论。

## V11 · `bt_peer_limit` 既有实测证据的性质（动机，不是受控基线）

- 数字出处（**仅**注释/文档，无原始数据）：
  - `native/engine/src/bt_downloader.rs:400-404`：128 上限被约 120 个连接中的尝试占满、只剩 5–10 个可用 peer；512 使信号量不再成为瓶颈，「稳态 1.1 → 12.6 MB/s（交错重复多轮的均值）」，RSS +约 15 MB。
  - `native/engine/src/bt_downloader.rs:458`：uTP 约 2× live peers。
  - `.omp/knowledge/engine.md:76-77`、`.omp/knowledge/bt-backend-migration.md:865,1122,1128` 复述同一组数字。
- **性质判定（按计划要求）**：单一 6.9 GB 公开种子的**公网**观测、无发布方差/置信区间、无对照声明、**未报告完成时间**（只报稳态速率）、原始数据不在仓库中。
- ⇒ 只可作为**动机与待核查案例**，**不得**当受控基线，也**不得**表述为「确定的 N 倍因果提升」。注意 1.1→12.6 是稳态速率的约 11.5×，而非 5×；且 `M-GOODPUT-01`（wall completion）与 `M-COMPLETE-01` 都尚未测量。
- 相关可核查线索（供 E0/E1 复现）：fork 内存在 `076d8a06 test(bt): bound the candidate pool at scale and measure the real announce peak`、`e2b7e62a test(bt): close the two real-topology verification gaps`、`70a73c2e test(bt): cover upload-while-downloading and the peer aggregates` 三个测试提交，命名指向「真实拓扑 / announce 峰值 / 候选池规模」的测量，可在 E0 中先读这些测试确认它们测的是什么。
- **工具线索**：libtorrent 仓库有 `simulation/libsimulator` 子模块（**未初始化，未读取**），命名指向 libtorrent 的网络模拟器；E1 的「受控 swarm」可评估复用其思想，但本轮不做任何安装与初始化。

---

## 由本轮复核直接产生的缺口增补

| 缺口 | 内容 | 依据 |
|---|---|---|
| G16 | pinned rqbit **无** v2 元数据/merkle 校验；14 个 `V2*` 错误变体从未被抛出 | V5 |
| G17 | `PeerAdmission.rejected` 无分原因计数 | V9 |
| G18 | per-peer 请求窗口在本端**不可由 FluxDown 配置**（固定默认 128，被对端 reqq 夹住） | V6 |
| G19 | 文档 pin（`d995b0c9`）早于实际 pin（`6f9d64b3`） | V10 |
| G20 | `bt_peer_limit` 证据链无原始数据与方差 | V11 |
| G21 | pinned rqbit **没有 choker**：连接建立即无条件 `Unchoke`，此后永不下发 `Choke` | V13 |
| G22 | `peer_admission.rejected` 计的是**许可获取失败次数**（单次出站准入最多 +4），不是被拒 peer 数 | V14 |
| G23 | `useful_peers` 的文档措辞与代码方向不一致（注释写上传语义、代码在下行路径累加） | V12 |

---

## V12 · `useful_peers` 的口径（源码 + 单测双重核对 ✅，并**反驳**一条子任务结论）

被核查的主张（`RqbitFluxdown` 冲突 ④）：*「FD `useful_peers` 注释为下载侧，rqbit 实为**上传侧** 1 MiB」*。

**判定：该主张不成立；`useful_peers` 是下载侧。** 依据（按证据强度排序）：

1. **调用点（决定性）**：`note_peer_fetched_bytes()` 全仓只有一处生产调用 —— `crates/librqbit/src/torrent_state/live/mod.rs:2340`，位于 `async fn on_received_piece(...)`（`:2320` 起）之内。该函数上下文明确是**接收**路径：`"peer sent an invalid piece"`、`"peer sent us an invalid piece"`、以及紧随其后的 `mark_download_activity`/`last_download_at`（注释 `this peer just sent us payload`）。同函数 `:2341` 同时累加 `fetched_chunks`。
2. **反方向计数另有其人**：`uploaded_bytes` 经 `on_uploaded_bytes` 钩子累加（`peer_connection.rs:49` 声明、`:605` 调用 → `live/mod.rs:1581-1599` 实现），与 `fetched_bytes` 互不重叠。
3. **单测锁定语义**：`crates/librqbit/src/torrent_state/live/peers/mod.rs:378-405` `useful_peers_counts_each_peer_once_and_decrements_on_disconnect` 断言：阈值下不计 → 跨过 1 MiB 恰好计 1 → 再多流量不重复计 → **peer 断开即减回 0**。

**同时修正的口径**（已回写 `metrics.csv` 的 `M-PEER-02`）：
- `useful_peers` 是 **point-in-time**「当前仍连接、且本连接内累计 `fetched_bytes` 已跨过 1 MiB 的 peer 数」，**不是**单调生命周期累计。`.omp/knowledge/bt-observability-plan.md:334` 的措辞「生命周期累计超过 1 MiB 的 peer 数」应改读为前者。
- `USEFUL_PEER_BYTES` 的 doc comment（`crates/librqbit/src/torrent_state/live/peer/stats/atomic.rs:11`「Payload a peer must have taken **from us**」）与 `note_peer_fetched_bytes` 的 doc comment（`peers/mod.rs:217`「payload this peer fetched **from us**」）**方向写反**，属文档缺陷，不得据其推断上传侧。

## V13 · pinned rqbit **没有 choker**（源码核对 ✅）

- 结论：连接建立后，writer 任务在（可选的）bitfield 之后**无条件**发送 `Message::Unchoke`，日志 `sent unchoke`，随后进入主循环；生产代码中**不存在**下发 `Message::Choke` 的路径。因此 BEP 3 所述「每 10 s 重掐、按速率互惠、30 s 轮换 optimistic unchoke」的算法在 pinned rqbit 中**没有实现**；对端的 choke 只被当作「我不该取片」的输入。
- 证据：`crates/librqbit/src/peer_connection.rs:458-485`（writer 起始序列，`Message::Unchoke` 在 `:478`）；接收侧 choke 状态仅 `live/mod.rs:1475-1476`（`on_i_am_choked`/`on_i_am_unchoked`）与 `:2304-2318`。
- 意义：FluxDown 的上传公平/防滥用**不靠 choker**，而靠上传请求队列（V8 的 per-peer 128 / per-torrent 4096 + 轮转 + `PeerBacklogExceeded` 断连）。这直接决定 E1-5（上传限速与互惠）能操纵什么、以及「互惠」在此实现对中其实不存在——**这是必须先固定的前提，否则会把队列效应误读成互惠效应**。
- 与子任务产出：`S-CODE-RQBIT-CHOKE-01/02/03` 与本节一致（已交叉确认）。

## V14 · `peer_admission.rejected` 的真实语义（源码核对 ✅）

- 结论：`rejected` 在 `try_take()` 内对**每一次**失败的 `try_acquire_owned()` 自增（`crates/librqbit/src/peer_admission.rs:282-294`，自增在 `:290`）。而 `acquire()`（`:230-280`）对一次**出站**准入依次尝试 `global` → `connecting` → `global_outbound` → `connecting_outbound` 四个信号量（失败即 `release_all()` 返回），故**一次被拒的出站准入最多使 `rejected` 增加 4**。
- ⇒ `rejected` 的口径是「**许可获取失败次数**」，**不是**「被拒 peer 数」，也**不是**可加和的来源分解。`M-CONN-06` 若直接使用它会把单一原因放大数倍。
- `AdmissionDenied` 四值与标签已核对：`SessionPeers`/`SessionConnecting`/`TorrentConnecting`/`TorrentLive` → `session-peer-limit`/`session-connecting-limit`/`torrent-connecting-limit`/`torrent-live-limit`（`:123-146`）。要按原因分解**必须**改 rqbit 分列计数（缺口 G4 不变）。
- 入站侧：`try_acquire_incoming()` 只取 `global` 与 `connecting`，**永不占用出站份额**（`:203-209`），与「为入站保留 headroom」的设计一致。

## V15 · FluxDown 侧两处文档缺陷（源码核对 ✅）

1. **`bt_peer_limit` 的「热生效」说法错误**：`native/engine/src/download_manager.rs:4762-4771` 的注释称「`seed_max_active` **and the live `bt_peer_limit`** take effect on the running session immediately」。实际 `set_peer_limit()`（`native/engine/src/bt_downloader.rs:1017-1023`）只把值存进 `AtomicUsize`，其自身 doc comment 明确写「librqbit has no safe live setter for an existing torrent's peer semaphore, **so existing handles remain unchanged**」。⇒ 只有 `seed_max_active` 是热生效；`bt_peer_limit` 只影响**后续** add 的 torrent。这条直接影响 `M-BUDGET-01` 的「配置变更需重建 session 才生效」判定与 E1-1 的操纵方式（**不能**在同一 session 内改预算后立即测量）。
2. **`bt_peer_limit` 收益归因在现有 pin 上已不成立**：FD 注释（`bt_downloader.rs:400-404`）把 128→512 的收益归因于「librqbit 用同一个信号量同时计连接中与已连接」，但 `build_opts`（`bt_downloader.rs:828-837`）**已经注入** `peer_admission`（把 connecting 与 live 分开计数）。⇒ 该归因描述的是**未接准入层之前**的状态；在当前 pin 上必须重新测量，不能沿用。

## V16 · 额外可观测性缺陷（源码核对 ✅）

- `crates/librqbit/src/session_stats/snapshot.rs:35-97` 在 `as_prometheus` 中**连续两次**写出名为 `rqbit_peers_queued` 的 gauge（先 `peers.queued`、后 `peers.seen`）⇒ `seen` 在 Prometheus 输出里被**同名覆盖**，无法单独读取。`useful_peers`、`interested_peers`、`incoming_rejected` 在该输出中也未被暴露。
- 影响：任何「用 Prometheus 端点做 E0/E1 采集」的方案必须先绕开或修掉这个重名；否则 `M-CONN-01`/`M-CONN-05` 的读数会静默错误。

