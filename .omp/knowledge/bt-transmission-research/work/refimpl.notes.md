# refimpl.notes.md — 四引擎参考实现机制证据（libtorrent / qBittorrent / Transmission / aria2）

slice: `ReferenceImpls`（我的范围：本地只读源码的「精确版本实现」证据轨）
产出：`work/refimpl.sources.csv`（87 条数据行，15 列）；本文件为机制说明。
所有证据日期 `retrieved_at=2026-10-17`，全部只读、未运行任何构建/下载/握手/抓包。

## 0. 版本锚与证据边界

| 工程 | commit | 分支/版本 | 我实际用到的入口 |
|---|---|---|---|
| libtorrent | `54bc4e924b34e34212c0af9584921039e438ea84` | branch `RC_2_1`；`include/libtorrent/version.hpp` 报 2.1.1.0 | `src/settings_pack.cpp`、`src/peer_connection.cpp`、`src/request_blocks.cpp`、`src/choker.cpp`、`src/piece_picker.cpp`、`src/session_impl.cpp`、`src/session.cpp`、`docs/gen_settings_doc.py`、`CMakeLists.txt` |
| qBittorrent | `607c9881e540510e0baaad28bc9ab5607ad20cd4` | master | `src/base/bittorrent/sessionimpl.cpp`、`torrentimpl.cpp`、`cmake/Modules/CommonConfig.cmake` |
| qBittorrent-EE（第三方） | `80f544d0b4efbe0a04edc57784c771ecf2b1b90e` | `v5_2_x`, `release-5.2.3.10-1` | 仅 1 行旁证（`S-CODE-QBT-015`） |
| Transmission | `c22a63d52f01d301f2131e4e2102ecc0ac3bb2df` | main, 4.2.0 | `libtransmission/{peer-mgr,peer-msgs,bandwidth,webseed}.cc`、`session-settings.h`、`constants.h` |
| aria2 | `9e7273583f83e881e3ec067b523ba88724088d2f` | master, 1.37.0 | `src/{OptionHandlerFactory,DefaultPeerStorage,BtSeederStateChoke,DefaultBtInteractive,DefaultBtRequestFactory,DefaultPieceStorage,BtPieceMessageValidator,Piece,SocketCore,DownloadCommand,BitfieldMan}.cc` |

行号全部为实际读到的行；`<slice>` 缺口见第 5 节。**本 slice 不含任何运行时测量**：凡称「运行时额度」均由代码路径推导，未在会话中观测。

引用约定：文中 `S-...-NNN` 指 `refimpl.sources.csv` 的 `source_id`。`contradicts_ids` 在本 slice 承载两类关系：(a) 来源间事实冲突；(b) **上限↔额度 / 库默认↔宿主生效值** 的成对关系（读其中一个不能预测另一个），凡 (b) 类都在本节明说。FluxDown 侧的数值来自本任务书冻结事实（未由我复核源码），标注为「context」。

---

## 1. 关键机制逐条

### M1 libtorrent：每 peer 请求窗口＝「配置硬上限 + 运行时公式」
- **主张**：每 peer 的在途请求上限由 `settings_pack::max_out_request_queue`（默认 500）固定；实际使用值是运行时算出的 `m_desired_queue_size`，不写回配置。
- **证据**：`S-CODE-LT-013`（`src/peer_connection.cpp:107,4785-4797`，构造时初始化、唯一 mutator 无人调用）、`S-CODE-LT-011`（`src/peer_connection.cpp:4812-4840`，`queue_time(3s) × download_rate / block_size`，clamp 到 `[2, cap]`，`m_slow_start` 时整段跳过、snubbed 时钉 1）。
- **适用条件**：`request_queue_time=3`、`max_out_request_queue=500`、block 16 KiB 全为默认时，公式退化为「约 3 秒下载量 / 16 KiB，夹在 2..500」；慢启动阶段窗口按收片增长，不受公式支配。
- **与 FluxDown 差异**：rqbit 侧有**两层**对应物——(1) 准入层：`PeerAdmissionConfig` 的 outbound/connecting 保留与逻辑 lease（context）；(2) 单 peer 请求窗口层：`TorrentStateLive::max_request_window()`（`crates/librqbit/src/torrent_state/live/mod.rs:526-532`，默认 `DEFAULT_PEER_REQUEST_WINDOW=128`，`:1389`）与 `PeerFlowControl.request_window`（初值即该上界，`:834,908,1397-1400`）。libtorrent 这里只是第 (2) 层的实现；把它与准入预算混为「每 peer 请求数」会得到不同单位与不同作用域。rqbit 侧的详细核见 §6/V5。
- **可检验预测**：固定 peer、固定限速把 `max_out_request_queue` 从 500 降到 50，在 RTT≥100 ms 链路上应出现「单 peer 吞吐被 50×16 KiB/BDP 卡住」的台阶；把 `request_queue_time` 从 3 提到 10 而 cap 不变，台阶应消失。若观测不到台阶，说明瓶颈在别处（拥塞窗口/磁盘/对端），而非请求窗口。

### M2 libtorrent：连接准入是双闸门（总量硬上限 + 每 tick 动态配额）
- **主张**：`connections_limit(+connections_slack)` 是总量闸门；`connection_speed` 是每 tick 的尝试配额，且 `smooth_connects=true` 时配额会被砍到「剩余槽位的一半」。
- **证据**：`S-CODE-LT-021`（`src/session_impl.cpp:3238-3259,4487`，入站 `>= limit+slack` 拒绝，出站 `>= limit` 停）、`S-CODE-LT-020`（`src/session_impl.cpp:4487-4530`，quota 扣除 `m_boost_connections` 后按 `(limit+1)/2` 截断）、`S-CODE-LT-009`（`connection_speed=30`、`torrent_connect_boost=30`）。
- **适用条件**：quota 是「每次 `try_connect_more_peers` 调用」的量，调用频率由 `tick_interval=500 ms` 驱动 → 名义上限 60 次/秒，实际被 `(limit+1)/2` 与剩余槽位进一步压低。
- **与 FluxDown 差异**：FluxDown 把连接数分三层（peer limit 256 默认 / connect limit 64 默认 / session peer 1024 默认，context），并有 `bt_session_connect_limit(session)=(session/4).clamp(1,128)` 的派生上限；libtorrent 的「session 级」只有 `connections_limit` 一个数字，没有「连接中」与「已连接」分离的预算。**不要把 libtorrent 的 200 与 FluxDown 的 256/64/1024 直接对比**：前者不含 connecting 与 live 的区分。
- **可检验预测**：把 `connections_limit` 设成远大于 `num_connections()` 时，新建连速率应仍受 `connection_speed` 限制（≈30/tick 上限的一半），而**不是**瞬间占满；反之若观测到一次性打满，说明 hits `torrent_connect_boost` 路径或测量窗口小于 tick。

### M3 libtorrent：choker 有两套语义，`unchoke_slots_limit` 会被 rate-based 绕过
- **主张**：默认 `fixed_slots_choker` 下 `unchoke_slots_limit=8`（-1=不限）；一旦切到 `rate_based_choker`，配置槽位数被丢弃，槽位按上传统计动态推导，斜率硬编码 2048 B/s（源码自带 TODO），`rate_choker_initial_threshold=1024`。
- **证据**：`S-CODE-LT-004`、`S-CODE-LT-017`（`src/choker.cpp:196-221`）。seed 侧另有一套：`S-CODE-LT-018`（`src/choker.cpp:56-96`，round-robin 配额 = `piece_length × seeding_piece_quota(20)` 且已 unchoke > 1 分钟才让位）。
- **适用条件**：只有固定槽位算法才尊重配置值；qBittorrent 在宿主层把 `unchoke_slots_limit` 改成 20（`S-CODE-QBT-003`）而**不动** `choking_algorithm`，因此 20 在其默认构建里是真实生效的。
- **与 FluxDown 差异**：FluxDown 上传侧没有「槽位」抽象；对照价值在于「同一个配置键在不同算法下的语义完全不同」——若把 libtorrent 文档里的 `unchoke_slots_limit` 当成「上传并发上限」用于建模 FluxDown，会同时错在口径（session vs 每 torrent）与语义（固定 vs 动态）两处。
- **可检验预测**：在自身可控的 swarm 里对 libtorrent 主机切换 `choking_algorithm`（fixed↔rate_based）并固定其它参数：fixed 下 unchoked 数应恒为 `min(配置, 感兴趣 peer 数)`；rate_based 下 unchoked 数应随上传速率单调增且与配置无关。

### M4 libtorrent：end-game 是「最后手段」，不是默认并发复制
- **主张**：只有当 picker 无法凑满 `num_requests` 且 peer 未 choked 时才 `set_endgame(true)`；`strict_end_game_mode=true`（默认）下，只要仍有未请求过的片就不允许挑 busy block。进入 endgame 后 `desired_queue_size()` 恒返回 1。
- **证据**：`S-CODE-LT-016`（`src/request_blocks.cpp:185-188,245-263`）、`S-CODE-LT-012`（`include/libtorrent/aux_/peer_connection.hpp:518-523`）。
- **附带证据（block 超时）**：`S-CODE-LT-011` 同文件 5182/5243/5252 处显示超时重取时先把 `m_desired_queue_size` 临时设 2、取完再设 1；`request_a_block` 的「free_blocks>0 就不取消请求」保护见 5226-5230。
- **适用条件**：`strict_end_game_mode` 可关；private/低供给 swarm 才容易真正进入 endgame。
- **与 FluxDown 差异**：rqbit 侧是否存在等价 endgame 不在本 slice 证据内（属 `RqbitFluxdown` slice）。**不要用 libtorrent 的 endgame 行为推断 rqbit**；本文只能给出「有独立实现存在」这一对照点。
- **可检验预测**：在只保留 1 个 seed 且人为延迟若干 block 的受控 swarm 中，libtorrent 应在「凑不满请求」后才出现重复请求（`waste_piece_end_game` 计数>0），而不是从一开始就重复请求。

### M5 libtorrent：上传侧水位随速率浮动，下载侧由磁盘队列兜底
- **主张**：发送缓冲水位 `= uploaded_last_second × factor(50)/100` 夹在 `[10 KiB, 500 KiB]`；磁盘侧 `max_queued_disk_bytes=100 MiB`。
- **证据**：`S-CODE-LT-015`（`src/peer_connection.cpp:5277-5287`）、`S-CODE-LT-003`、`S-CODE-LT-005`。
- **适用条件**：水位是**每 peer 每次 fill 周期**重算；100 MiB 默认与 qBittorrent 的 `DiskQueueSize` 默认一致（`S-CODE-QBT-012`），不是巧合——qBittorrent 显式对齐了 2.x 默认。
- **与 FluxDown 差异**：FluxDown 用 batch/预分配写（context），本 slice 未取到其磁盘队列阈值；对照点是「水位是派生量而非配置量」，因此用 `send_buffer_watermark` 反推内存占用在高速上传时会低估。
- **可检验预测**：把 factor 从 50 降到 5，固定 1 MiB/s 上传时水位应贴在 10 KiB 下界（而非 500 KiB 上界），发送速率对磁盘抖动的敏感度应上升。

### M6 libtorrent：磁盘/校验线程与已废弃的队列开关
- **主张**：`aio_threads=10`（兼作 disk threads）、`hashing_threads=1`、`file_pool_size=40`、`checking_mem_usage=256`；`coalesce_reads/coalesce_writes/aio_max` 在 ABI-2 默认构建里被 `DEPRECATED_SET` 展开成空名零值（不可用），`aio_read/aio_write` 干脆不存在。
- **证据**：`S-CODE-LT-005`、`S-CODE-LT-010`（`src/settings_pack.cpp:73-87,146-147,335` + `CMakeLists.txt:739-742` + `include/libtorrent/settings_pack.hpp:465-489`）。
- **任务书前置修正**：任务书列举的 `min_request_queue`、`aio_read`、`aio_write` 三项在 libtorrent 2.x **都不是设置**（`S-CODE-LT-002`：`min_request_queue` 是 `src/peer_connection.cpp:80` 的 `constexpr int = 2`；后两者在源码与部署文档中均无匹配）。
- **与 FluxDown 差异**：无直接对应；价值在于提醒「文档列举的旧键名不可作为参数面清单」。
- **可检验预测**：对 ABI-2 构建调用 `settings_pack::get_bool(coalesce_reads)` 不应改变任何磁盘行为（无消费者，grep `src/*.cpp` 无命中），可作为构建口径是否真按 2.x ABI 编译的探针。

### M7 libtorrent：peer 列表与 churn 参数
- **主张**：`max_peerlist_size=3000` / `max_paused_peerlist_size=1000`（每 torrent），超出按 95% 低水位裁剪；churn 侧 `peer_turnover=4%`、`peer_turnover_cutoff=90%`、`peer_turnover_interval=300 s`。
- **证据**：`S-CODE-LT-022`（`src/settings_pack.cpp:282-283` + `src/peer_list.cpp:354-367`）、`S-CODE-LT-007`。
- **适用条件**：仅在达到 peer limit 时按 turnover 比例断开（文档语义，`S-DOC-LT-001`）；未达限时不触发。
- **与 FluxDown 差异**：FluxDown 的 pinned rqbit 该 commit 标题是「收缩 peer 轮换的判定窗口到一分钟内」（任务书冻结事实），说明同一机制在 rqbit 里主动改过；**两侧的 turnover 周期不可互相套用**。
- **可检验预测**：libtorrent 主机把 `peer_turnover_interval` 从 300 s 改到 60 s 且保持限值触顶，被换出的 peer 数/分钟应按比例上升；若不变，说明该配置在该构建分支被宿主覆盖（qBittorrent 是把 4/90/300 原样透传的，`S-CODE-QBT-007`）。

### M8 Transmission：脉冲式 rechoke（10 s 周期 + 事件提前触发）
- **主张**：`RechokePeriod=10 s`、`BandwidthTimerPeriod=500 ms`、`PeerInfoPeriod=1 min`；新 peer 到来时 `rechokeSoon()` 把定时器压到 100 ms 提前脉冲。
- **证据**：`S-CODE-TR-001`（`libtransmission/peer-mgr.cc:1072-1074,1113`）、`S-CODE-TR-002`（`1132-1135,1157-1161`）、`S-CODE-TR-003`（`2348-2373`）。
- **适用条件**：脉冲只对 `peer_count>0` 且仍在运行的 torrent 生效。
- **与 FluxDown 差异**：这是**定时批处理**模型（周期性重排全 tolerated 集合），而 libtorrent 是 tick 驱动的 re-choke + rate 算法；FluxDown/rqbit 的 rechoke 语义需由 `RqbitFluxdown` slice 提供，本 slice 只能提供「存在第三种模型」的事实。
- **可检验预测**：把 rechoke 周期从 10 s 调到 1 s，应按比例缩短「新 peer 获得 unchoke 的时间」但不应改变稳定态的 unchoked 数（后者由 `uploadSlotsPerTorrent` 决定，见 M9）。

### M9 Transmission：peer 槽位是 session 级设置，请求额度是测速派生的
- **主张**：unchoke 数量上限取自 `session->uploadSlotsPerTorrent()`（默认 8），不是每 peer 常量；choke 决策还读写带宽调度器的 `is_maxed_out(Up)`——上传饱和时 peer 保持原 choked 状态且不再新开 optimistic；optimistic unchoke 存活 4 个脉冲（`OptimisticUnchokeMultiplier=4`）。
- **证据**：`S-CODE-TR-004`、`S-CODE-TR-006`、`S-CODE-TR-005`、`S-CODE-TR-007`。
- **与 FluxDown 差异**：Transmission 把「是否已吃饱」这一**运行时状态**接进 choke 决策（跨模块耦合），libtorrent 的 fixed 算法不这么做；这类耦合是「上传饱和时换谁来上传」的另一种答案，值得作为 FluxDown 的上传侧候选设计参考，但在缺乏 FluxDown 侧对应证据前不得当结论。
- **可检验预测**：在把上传限速压到很低时，Transmission 的 optimistic unchoke 轮换应停滞（因每次脉冲都 `is_maxed_out`），表现为单个 peer 长期占据唯一槽位；限速放宽后轮换应恢复。

### M10 Transmission：请求流水线深度完全由实测速度推导
- **主张**：`desired_request_count_ = min(ceil, max(blocks_in_period, 32))`，其中 `blocks_in_period = 实测片速率 × 10 s / 16384`，`ceil = 对端 reqq 或 500`；调度时按差额一次性补齐。
- **证据**：`S-CODE-TR-013`（`libtransmission/peer-msgs.cc:186,191,2308-2317`）、`S-CODE-TR-014`（`604-607,2064-2074`）、`S-CODE-TR-012`（block 16 KiB）。
- **适用条件**：需要 metainfo、未完成、且本端 interested+unchoked；`peer_reqq_` 仅在对端广播 `reqq>0` 时设置。
- **与 FluxDown 差异**：与 libtorrent 同构（都是「窗口=时延×速率」），但 Transmission 的下界是 32 blocks（512 KiB 在途/peer），libtorrent 默认下界只有 2 blocks；**这是同一机制上相差 16 倍的下界**，是 FluxDown 若引入下界时的直接参考点。
- **可检验预测**：固定 RTT 与限速，把对端 reqq 从 500 降到 50：Transmission 在途请求应在慢速链路上被压到 50，而 libtorrent（`max_out_request_queue` 不动、`request_queue_time=3`）仍按公式给值——两侧对同一网络条件的响应点不同。

### M11 Transmission：带宽调度是固定周期派发 + 增量轮转
- **主张**：`tr_bandwidth::allocate(period_msec)` 每 500 ms 被调用；只有 `is_limited_` 的方向才把 `bytes_left_ = desired_speed × period/1000`（即限量方向每脉冲得到半秒配额）；`phase_one` 对高/普通/低三档 peer 数组各跑一次 Up/Down，每次给一个 peer 3000 B 的 flush 增量，用不完就换下一个。
- **证据**：`S-CODE-TR-017`（`libtransmission/bandwidth.cc:146-161` + `peer-mgr.cc:1072,2697-2699`）、`S-CODE-TR-018`（`bandwidth.cc:178-215,217-275`）、`S-CODE-TR-019`（`279-293` 父子 clamp）、`S-CODE-TR-020`（默认限速未启用，`speed_limit_*_enabled=false`）。
- **与 FluxDown 差异**：这是**用户态令牌式节流**（周期派发 + 轮转），libtorrent 用率限制器+水位，aria2 用反应式 deferral（M15）——三者是三种不同实现族。FluxDown 若要做「多任务公平」，Transmission 的优先级三档 + 轮转增量是最小的可复用模型。
- **可检验预测**：设 1 MiB/s 上传限速，一个高优先级 peer 与十个普通 peer 竞争：Transmission 的高档数组应先被填满再轮到普通档；把 `Increment` 视作常量时，单 peer 的最小可服务粒度应约为 3000 B/轮。

### M12 aria2：choke 在 10 s 轮上做，seeder 算法另有时帧
- **主张**：`CHOKE_ROUND_INTERVAL = 10 s`（`PeerChokeCommand` 每次 execute 检查，到点才执行）；seeder 侧 `TIME_FRAME = 20 s`，每次 unchoke `count = (round_==2) ? 4 : 3`，轮次 0..2 循环，早期轮次还会随机挑一个 optimistic 并清掉其它。
- **证据**：`S-CODE-ARIA2-009`、`S-CODE-ARIA2-010`。
- **与 FluxDown 差异**：aria2 的 unchoke 槽位数是**硬编码 3/4**（无配置键），与 libtorrent 的 `unchoke_slots_limit`、Transmission 的 `uploadSlotsPerTorrent` 都不同：aria2 这条路根本没有暴露该参数。
- **可检验预测**：aria2 做种时观测到的 unchoked 数应只在 {3,4} 上取值，且在 20 s 时帧内轮换；若配置项无法改变该值，说明该实现未把 choke 参数配置化。

### M13 aria2：请求流水线小下界 + 倍增，block 64 KiB
- **主张**：`maxOutstandingRequest_` 初值 `DEFAULT_MAX_OUTSTANDING_REQUEST=6`，当「上一轮未完成请求 - 当前未完成请求」按 4 倍阈值触发时翻倍，上限 `UB_MAX_OUTSTANDING_REQUEST=256`；block 尺寸 `MAX_BLOCK_LENGTH=64 KiB`。
- **证据**：`S-CODE-ARIA2-011`、`S-CODE-ARIA2-012`、`S-CODE-ARIA2-013`。
- **与 FluxDown 差异**：aria2 的 block 是 libtorrent/Transmission 的 4 倍（64 KiB vs 16 KiB），且窗口从 6 起步——**「每 peer 请求数」在三个实现间不可直接比较，必须同时说明 block 大小**。
- **可检验预测**：同一链路同一限速下，aria2 的单 peer 在途字节起点应约为 6×64 KiB=384 KiB；libtorrent 默认下界 2×16 KiB=32 KiB、Transmission 32×16 KiB=512 KiB。三者字节口径可直接对比而请求数不可。

### M14 aria2：piece 选择 rarest-first + 块内低位优先 + 顺序累积哈希
- **主张**：`DefaultPieceStorage` 默认 `RarestPieceSelector`，流式选择器由 `stream-piece-selector` 决定（默认/inorder/random/geom(alpha=1.5)）；未命中流式时按 `getFirstMissingUnusedIndex` 取**最低索引**块；哈希只在块按顺序到达时累积（`begin == nextBegin_`），乱序到达不推进。
- **证据**：`S-CODE-ARIA2-014`、`S-CODE-ARIA2-015`、`S-CODE-ARIA2-017`；校验器本身只做边界检查不做哈希（`S-CODE-ARIA2-016`）。
- **与 FluxDown 差异**：aria2 的「块内顺序」+「顺序累积哈希」意味着乱序 block 必须缓存并在齐序后重放才能算 hash，这对磁盘/内存压力模型的影响与 libtorrent 的「任意块到达即可入队校验」不同。**可检验预测**：故意乱序投递 block 时，aria2 的 `isHashCalculated` 不应提前为真，而 libtorrent 的 piece 完成判定不受到达顺序影响。

### M15 aria2：限速是反应式（无令牌桶）
- **主张**：`SocketCore` 内没有任何 rate/quota/bucket 逻辑；限速在命令层实现——测速超限就 `addCommandSelf()` + `disableReadCheckSocket()/disableWriteCheckSocket()` 把命令推迟，上传侧把待发消息挪进 `tempQueue` 延后发送，接收循环在超限时 break。
- **证据**：`S-CODE-ARIA2-018`、`S-CODE-ARIA2-019`、`S-CODE-ARIA2-020`。
- **与 FluxDown 差异**：这是**事后纠偏**而非预算派发，瞬时突发比令牌桶更明显；做多任务公平/交互延迟护栏实验时，aria2 不能当「有令牌桶」的实现来对照。
- **可检验预测**：对 aria2 施加 1 MiB/s 限速并测 100 ms 窗的速率分布，其突发度（p99/p50）应显著高于同条件 libtorrent；若两类实现 burst 接近，说明瓶颈不在限速器。

### M16 aria2 是独立实现（不是 libtorrent 宿主）
- **主张**：全仓 grep `libtorrent` 只命中 `README.rst` 的协议文档引用；源码不 include/link libtorrent 或 boost::asio，走自研 Command/DownloadEngine 循环与 `Bt*` 消息类。
- **证据**：`S-CODE-ARIA2-021`。
- **与 FluxDown 差异**：FluxDown 走 librqbit，因此把 aria2 当「另一个客户端行为样本」是成立的，但**不能把 aria2 的参数语义当作 librqbit 的对照物**；只有机制（如上面几条）可迁移。

### M17 qBittorrent 是 libtorrent 宿主：覆盖清单与构建口径耦合
- **主张**：引擎构造为 `lt::session_params` + `lt::session`；每个 libtorrent 设置都由宿主显式赋值（`sessionimpl.cpp:1950-2230`），其中**覆盖库默认值**的有：`connections_limit 200→500`、`unchoke_slots_limit 8→20`、`rate_limit_ip_overhead true→false`、`mixed_mode_algorithm peer_proportional→prefer_tcp`、`file_pool_size 40→100`、`checking_mem_usage 256→32`、`alert_queue_size 2000→INT_MAX/2`；**原样透传**的有：`peer_turnover 4/90/300`、`max_out_request_queue 500`、`max_allowed_in_request_queue 2000`、`diskQueueSize 100 MiB`；**从未触碰**的有 `smooth_connects`。
- **证据**：`S-CODE-QBT-001`、`S-CODE-QBT-003`、`S-CODE-QBT-004`、`S-CODE-QBT-013`、`S-CODE-QBT-011`、`S-CODE-QBT-005`、`S-CODE-QBT-007`、`S-CODE-QBT-009`、`S-CODE-QBT-012`、`S-CODE-QBT-008`、`S-CODE-QBT-010`；per-torrent 上限走 handle（`S-CODE-QBT-002`）。
- **构建口径耦合**：`coalesce_reads/coalesce_writes` 的赋值被 `#ifndef QBT_USES_LIBTORRENT2` 包住，而该宏由「libtorrent ≥ 2.0.10」决定（`S-CODE-QBT-014`），与 libtorrent 自身把这些键在 ABI-2 下编译掉的事实（`S-CODE-LT-010`）一致。
- **与 FluxDown 差异**：**这是本 slice 最重要的口径提醒**——「客户端默认值」既不是库默认值也不是用户配置值。做四客户端对比时，必须固定「三层中的哪一层」：库默认（libtorrent `default_settings()`）／宿主覆盖（qBittorrent 的 `sessionimpl.cpp`）／用户配置（GUI/WebUI）。
- **可检验预测**：读 qBittorrent 会话的 `settings_pack` 快照（或对照 UI 默认值）应得到 `unchoke_slots_limit=20`、`rate_limit_ip_overhead=false`；若得到 8/true，说明该构建走了不同的代码分支。

---

## 2. 四引擎对照表（机制 → 实现落点 → 默认值 → 与 FluxDown 差异）

FluxDown 列一律标注「context」＝由任务书冻结事实给出，本 slice 未复核其源码。

| 机制 | libtorrent（54bc4e9） | qBittorrent（607c988）＝宿主 | Transmission（c22a63d） | aria2（9e72735） | 与 FluxDown（context）差异 |
|---|---|---|---|---|---|
| 每 peer 在途请求 | `m_desired_queue_size = queue_time(3)×rate/16KiB`，clamp `[2, max_out_request_queue=500]`；`src/peer_connection.cpp:4812-4840` | 透传上限 500；`sessionimpl.cpp:2064` | `min(ceil, max(rate×10s/16KiB, 32))`，ceil=对端 reqq 或 500；`peer-msgs.cc:2308-2317` | 6 起、倍增到 256，block 64 KiB；`DefaultBtInteractive.cc:105-106,344-348` | 无直接对应量；rqbit 侧额度在准入层（`PeerAdmissionConfig` 保留量） |
| 连接总量 | `connections_limit 200` + `slack 10`（入站超 slack 才拒），出站 `>=limit` 停；`session_impl.cpp:3238-3259,4487` | 覆盖为 500；`sessionimpl.cpp:2176` | `peer_limit_global 200`（超限关最不活跃者）；`peer-mgr.cc:2514-2541` | `bt-max-peers 55`（option 层，`OptionHandlerFactory.cc:1563-1567`） | FluxDown 分 peer 256 / connect 64 / session 1024 三层（context），libtorrent 无 connecting/live 之分 |
| 连接尝试速率 | `connection_speed 30`/tick，`smooth_connects` 砍半；`session_impl.cpp:4487-4530` | 透传 30；从不触碰 `smooth_connects` | 由 peer limit 与 peer-info 目标（×3）间接限制；`peer-mgr.cc:2590-2593` | `bt-request-peer-speed-limit 50K` 作阈值触发补连；`ActivePeerConnectionCommand.cc:89-90` | FluxDown `bt_session_connect_limit=(session/4).clamp(1,128)`（context）；libtorrent 无派生上限 |
| 上传槽位 | `unchoke_slots_limit 8`（rate_based 时被绕过，斜率硬编码 2048 B/s） | 覆盖为 20；`sessionimpl.cpp:2178` | session 级 `uploadSlotsPerTorrent 8` + optimistic×4 脉冲；`peer-mgr.cc:2301-2316,2228-2250` | 硬编码 3（轮 2 时 4），无配置键；`BtSeederStateChoke.cc:123` | FluxDown 无槽位抽象；对照仅用于「同键不同语义」的陷阱说明 |
| piece 选择 | 优先级 0..7（默认 4），rarest_first/sequential 互斥，partial 上限 `num_peers×3/2` 或 2048 blocks；`piece_picker.cpp:1986-1998` | 透传（`piece_extent_affinity` 默认 false） | 见 `peer-mgr` 排序与 `ChokeData`；本 slice 未取 picker 细节 | `RarestPieceSelector` 默认 + 流式选择器（geom 1.5）；块低位优先；`DefaultPieceStorage.cc:93-113` | FluxDown 侧 piece 排序策略需 `RqbitFluxdown` slice 证据 |
| endgame | 仅当凑不满请求才进入；`strict_end_game_mode true`；`request_blocks.cpp:185-188,245-263` | 透传 | 本 slice 未取 | `createRequestMessagesOnEndGame` 收集全部缺失块并随机打乱；`DefaultBtRequestFactory.cc:166-236` | 三实现均有 endgame，但触发条件与重复度不同；不可互相套用 |
| 磁盘/IO | `aio_threads 10`、`file_pool_size 40`、`max_queued_disk_bytes 100 MiB` | `file_pool_size 100`、`checking_mem_usage 32`、队列 100 MiB | 未取（`unused_cache_size_mbytes 4` 等为遗留）；`session-settings.h:110` | 未取 cached 实现 | FluxDown 使用 batch+预分配（context），阈值不可与三者直接比 |
| 限速实现 | 率限制器（session 级 `upload/download_rate_limit` 默认 0=不限） | 覆盖同上两键（`sessionimpl.cpp:1402-1403,1969-1970`） | 周期派发 `bytes_left_=speed×500ms/1000` + 轮转 3000 B 增量；`bandwidth.cc:146-161,178-275` | 反应式 deferral（无令牌桶）；`DownloadCommand.cc:135-142` | 三种族；FluxDown 的限速语义需另一 slice |
| 默认不限速 | `upload_rate_limit 0`、`download_rate_limit 0` | 沿用 | `speed_limit_up/down_enabled=false`，数值 100 为惰性占位；`session-settings.h:104-105,117-118` | `max-overall-*-limit 0`、`max-*-limit 0`；`OptionHandlerFactory.cc:451-452,480-482,1820-1829` | 四者一致：默认不限速 |
| peer 列表/轮换 | 3000/1000 每 torrent（95% 低水位）；`peer_turnover 4%/90%/300s` | turnover 原样透传 4/90/300 | peer-info 目标 = limit（完成后）或 ×3；`peer-mgr.cc:2590-2593` | `bt-max-peers 55` 为上限 | rqbit pin 专门改过轮换判定窗口（任务书冻结事实）→ 周期不可互套 |
| 定位 | 库；客户端需自行覆盖 | **libtorrent 宿主**（会话=库实例） | 独立实现（libtransmission） | 独立实现（非 libtorrent） | FluxDown 用 librqbit：属「库 + 自研宿主」路线，与 qBittorrent 同构 |

---

## 3. 「硬上限 vs 运行时工作额度」在本 slice 的一致结论

读源码得到的一致模式：**四家都把「配置键」当上限/开关，把「运行时可用的额度」另算，两者常有不同单位或不同作用域**。成对证据（`contradicts_ids` 互指）：

| 硬上限（配置键） | 运行时额度（派生量） | 证据 |
|---|---|---|
| `max_out_request_queue = 500` / peer | `m_desired_queue_size = 3s×rate/16KiB`，clamp 到该上限 | `S-CODE-LT-013` ↔ `S-CODE-LT-011` |
| `connections_limit 200`（+`slack 10`） | 每 tick 尝试配额 `min(connection_speed, (剩余槽+1)/2)` | `S-CODE-LT-021` ↔ `S-CODE-LT-020` |
| `unchoke_slots_limit = 8`（默认算法下生效） | rate_based 算法下完全由上传速率重算（+2048 B/s/槽） | `S-CODE-LT-004` ↔ `S-CODE-LT-017` |
| `peer_limit_global 200` / `peer_limit_per_torrent 50`（配置硬闸门） | `desired_request_count_` 由实测速率派生，clamp `[32, reqq|500]` | `S-CODE-TR-021` ↔ `S-CODE-TR-022` |

另有两类「不是矛盾但必须成对读」的关系，也在 `contradicts_ids` 里互指：
- **库默认 vs 宿主生效值**：`S-CODE-LT-004/005/006/007/008/009/010/011/021` 与 `S-CODE-QBT-003/004/005/006/008/009/011/012/014` 等（qBittorrent 覆盖了 8 项库默认值）。
- **文档默认 vs 源码默认**：`S-DOC-LT-002` ↔ `S-CODE-LT-022`（唯一真冲突）以及 `S-CODE-LT-001` ↔ `S-CODE-LT-015`（后者记录的运行时水位不在配置面上）。

工程含义：任何「把 X 调到 N 就会更快」的假设，若把上限当成额度（或反之），实验会测错对象。E0 阶段应先确认所测参数属于哪一类（`bt-observability-plan.md` 的口径需与此对齐）。

## 4. 文档默认值 vs 源码默认值

- `S-DOC-LT-001`：部署版 reference-Settings.html 自报 **Version 2.1.0**，本地源码 `LIBTORRENT_VERSION` 为 **2.1.1.0**（`include/libtorrent/version.hpp:17-25`）——两者不是同一构建产物。
- 逐字段对照（我实际读到的）：`request_timeout 60`、`request_queue_time 3`、`max_out_request_queue 500`、`max_allowed_in_request_queue 2000`、`unchoke_slots_limit 8`、`connections_limit 200`、`connections_slack 10`、`file_pool_size 40`、`max_queued_disk_bytes 100*1024*1024`、`peer_turnover 4/90/300`、`aio_threads 10`、`alert_queue_size 2000`、`max_peerlist_size 3000`、`max_paused_peerlist_size 1000` —— **全部一致**。
- **但这个「一致」没有独立信息量**：`docs/gen_settings_doc.py:62-79` 直接解析 `../src/settings_pack.cpp` 的 `SET()/DEPRECATED_SET()` 行生成表格，属同源生成。因此它不能用来交叉验证源码默认值，只能用来确认「部署页对应的是哪一版表」。
- **唯一真冲突**：`tuning-ref.html` 的 “peer list size” 节写 **4000**，源码与 reference-Settings 都是 **3000/1000**（`S-DOC-LT-002` ↔ `S-CODE-LT-022`）。该页其余可核数值（连接默认 200）与源码一致，因此是单点笔误/陈旧，而不是整页失效。
- `tuning-ref.html` 的 “high performance seeding” 节只给方向不给数值（`S-DOC-LT-003`），具体数值需读 `S-CODE-LT-023`（`src/session.cpp:117-207` 的 `high_performance_seed()`：`connections_limit 8000`、`max_out_request_queue 1500`、`unchoke_slots_limit -1`、`request_timeout 10`、`send_buffer_watermark 3 MiB`、`factor 150`、`low_watermark 1 MiB`、`aio_threads 8`、`max_queued_disk_bytes 7 MiB`、`active_limit 20000`）。**这组值表明「全部上限都可被策略替换，而运行时算法一字不改」**。

## 5. refimpl 缺口清单

1. **grafted shallow clone，无 tag 可达**（全部 5 个参考仓）：`git describe` 失败，只能按 commit 引用；`tag ↔ commit` 关系、CHANGES/Release 说明、submodule 版本均无法校验。上文所有版本号（libtorrent 2.1.1.0、Transmission 4.2.0、aria2 1.37.0、EE `release-5.2.3.10-1`）取自源码/仓库内文本，非 tag 校验结果。**不得把网页上的版本号当作这些 commit 的版本**。
2. **未做任何运行时观测**：全部「运行时额度」「动态」「自适应」结论均为代码路径推导，无会话快照、无 counter 读数、无抓包。任何「实际生效值」仍需 E0/E1 观测（例如 `m_desired_queue_size` 只有 `peer_log_alert` 能看，属 trace 级）。
3. **文档页与源码非同版**：见第 4 节；reference-Settings 默认值可视为同源生成，不能当独立证据。
4. **ABI 分支未编译验证**：`coalesce_reads/coalesce_writes/aio_max` 的「不可用」结论基于 `#define DEPRECATED_SET` 与 CMake 默认 `TORRENT_ABI_VERSION=2` 的源码推断，未实际编译两种 ABI 观察差异（任务禁止构建）。
5. **任务书三项前置假设被证伪（已登记）**：`min_request_queue` 非设置（硬编码 2）；`aio_read`/`aio_write` 在 2.x 源码与部署文档中均不存在。若后续 slice 仍按原清单引用这三项，会引用到不存在的参数面。
6. **未覆盖的实现细节**（本 slice 未读，需要时另开）：
   - libtorrent：uTP 拥塞控制实现体（`src/utp_*`/`src/udp_socket*` 未读，仅取了 `utp_*` 配置默认值与 `utp_cwnd_reduce_timer`）、DHT/tracker announce 调度、BEP 52 v2/hybrid 处理、MSE 握手、磁盘 hash 校验线程实现、`peer_turnover` 的实际执行点（只取了默认值与文档语义）。
   - qBittorrent：GUI/WebUI 暴露面清单（只看代码引用，未看 UI 定义）、`SessionImpl::configure()` 之外的第二条设置路径（若存在）、EE 与上游的差异范围（仅 1 行旁证）。
   - Transmission：piece picker 细节（`peer-mgr` 的请求选片与 `ChokeData` 排序之外的部分）、`bep55/introducer`、announce 调度、`session-settings` 全部默认值（只取与 peer/带宽/请求相关的子集）。
   - aria2：LPD 实现细节（只登记了 gate 行 `BtSetup.cc:217`）、DHT、tracker 重试与 tier 语义、`PeerInteractionCommand` 的完整状态机。
7. **`contradicts_ids` 语义**：本 slice 用它表达 (a) 来源冲突 与 (b) 上限/额度、库默认/宿主生效值的成对关系（见第 3、4 节）。合并进全局 `sources.csv` 时需保留这一约定说明，否则会被误读为事实冲突。
8. **字段数不一致（需 main 裁决）**：任务书写「严格 16 列」但枚举出的字段名只有 **15** 个（`source_id … contradicts_ids`），与计划里全局 `sources.csv` 的字段表一致。本 slice 按**枚举名**输出 15 列；若全局要求补齐第 16 列，需明确列名后统一回填（不要按「16」硬加空列）。
9. **FluxDown 侧对照受限**：本 slice 不读 FluxDown 源码，所有「与 FluxDown 差异」列均基于任务书冻结事实（`BT_PEER_LIMIT_*`、`BT_PEER_CONNECT_LIMIT_*`、`BT_SESSION_PEER_LIMIT_*`、`bt_session_connect_limit`、`PeerAdmissionConfig` 保留量、rqbit pin 标题）。这些数值未由我复核，若与仓库现状不符，以仓库为准并回改本文件。
10. **`url` 字段的取值方式（已核实）**：84 条 `code` 行的 `url` 填 `<上游仓库>/commit/<sha>`，`doi_or_commit` 填同一 sha。上游归属经 HTTP 状态码验证（`curl -o /dev/null -w %{http_code}`，全部 200；对照：伪造 sha 返回 404，说明 200 有区分度）：`arvidn/libtorrent`、`qbittorrent/qBittorrent`、`c0re100/qBittorrent-Enhanced-Edition`（第三方分支）、`transmission/transmission`、`aria2/aria2`。**这只证明「该 commit 页面存在于该仓库」，不证明远端树与我读取的本地 grafted clone 逐字节一致**；行号一律以本地 clone 为准。3 条 `doc` 行的 `url` 为实际抓取的文档页。
11. **`topic` 已统一为小写连字符 slug**（如 `hard-cap`、`request-pipeline`、`doc-vs-source`），并对 4 个同义值做了合并（`bt-*` 前缀去掉、`rate limiting`→`rate-limit`、`bt-choke`→`choking`）。如需与其它 slice 的 `topic` 词表合并，以本文件与 `refimpl.sources.csv` 的现值为准。

---

## 6. 交叉核对：他人 slice 的 pinned rqbit 主张（lead-verification V5–V9）

复核对象：`~/.cargo/git/checkouts/rqbit-341c9d2d87cbd6c5/6f9d64b`，pin `6f9d64b310ecb560990315af3c60df3111eec8e7`（checkout 路径与 commit 与任务书冻结事实一致）。

**本节的 rqbit 行不进我的 `refimpl.sources.csv`**：我的 `source_id` 前缀被限定为 libtorrent / qBittorrent / Transmission / aria2 四族，rqbit 归 `RqbitFluxdown` slice，应由其前缀承载以免重复计数。本节只给「独立复核结论 + `路径:行号`」。

### V5 请求窗口 128 —— 成立，需两点补充
- `const DEFAULT_PEER_REQUEST_WINDOW: usize = 128;` → `crates/librqbit/src/torrent_state/live/mod.rs:1389` ✓
- 取值：`max_request_window()` = `peer_max_request_window.unwrap_or(128).max(1)` → `live/mod.rs:526-532` ✓
- 生效：仅当对端 `reqq>0` 时 `request_window = reqq.min(max_request_window())` → `live/mod.rs:1618-1626` ✓
- **补充 1（重要）**：窗口初值在**连接建立时**就已等于 `max_request_window()`：`PeerFlowControl::new(self.max_request_window())` 见 `live/mod.rs:834`、`:908`，而 `PeerFlowControl::new` 在 `:1397-1400` 把 `request_window` 直接设为该值。所以默认 128 不是「收到 reqq 之后才生效」，而是**未收到 reqq 前就是 128**。
- **补充 2**：FluxDown 未配置该 option（`grep -rn max_request_window native/engine/src native/protocol/src` 零命中）✓ → 128 属「库默认」层，不是宿主覆盖，也不是用户配置。
- 与 §2 对照：libtorrent 是 `[2, 500]` 区间 + 按速率推导（`S-CODE-LT-011`），Transmission 是 `[32, reqq|500]` + 按速率推导（`S-CODE-TR-013`），rqbit 是**单一上界 128 且无推导公式**——三者同一机制、三种结构。

### V6 piece 选择不是 rarest-first —— 成立
- `piece_tracker.rs:123-164` `acquire_piece` 的实际顺序：① `try_steal(&req, 10.0)`（抢很慢 peer）② 在 `priority_pieces` 中找「未 have、非在途、对端有」者 ③ `chunks.iter_queued_pieces(...)` 顺序取 ④ `try_steal(&req, 3.0)`。文件头注释（`:118-121`）自述该顺序 ✓
- 排队序 `chunk_tracker.rs:236-249`（`iter_queued_pieces`）：按 `file_priorities` 顺序 → 该文件未下完 → `f.iter_piece_priorities()` → 该片在 `queue_pieces` 中 ✓
- 文件内片序 `file_info.rs:15-26`（`iter_piece_priorities`）= 首片、末片、其余升序（注释原文 "first, last, everything else from start to end"）✓
- 结论：排序中**没有 rarity/availability 输入**；与 libtorrent（`rarest_first` 优先级档，`S-CODE-LT-019`）和 aria2（`RarestPieceSelector`，`S-CODE-ARIA2-014`）都不同。因此**不得**用 rarest-first 的直觉去解释 rqbit 早期上传为 0（那要先排除 V7 的积压上限与 §1/M2 的拨号预算）。

### V7 上传队列上限与轮转 —— 成立
- `MAX_PENDING_PER_PEER: usize = 128`、`MAX_PENDING_PER_TORRENT: usize = 4096` → `torrent_state/live/upload_queue.rs:26-31` ✓
- `enqueue` 守卫顺序：重复 → 每 peer 上限（`PeerBacklogExceeded`）→ 每 torrent 上限（`TorrentBacklogExceeded`，明确注释「不会被静默丢弃」）→ `upload_queue.rs:100-119` ✓
- 轮转公平：入队时把 peer 追加到 `rotation`（`:116-119`）；`pop_next` 从 `rotation` 头取，取完仍非空则 `push_back` 到队尾（`:121-143`）✓
- `generation` 守卫：字段 `:47`；`cancel(peer, key, generation)` 要求代际相等（`:148-156`）；`drop_stale_generations`（`:177-183`）；单测 `stale_generation_requests_are_never_served`（`:320-330`）✓
- 维度提醒：这 128/4096 限制的是「**待发请求积压**」，与 libtorrent 的 `unchoke_slots_limit`（限制「谁能传」，`S-CODE-LT-004`）不是同一维度；把 128 当「上传并发上限」会得到错误结论。

### V8 v2 支持是部分的 —— 成立，可再加三条佐证
- `Id20`/`Id32` 类型别名与 `impl Id32`（含 `truncate_for_dht`）→ `crates/librqbit_core/src/hash_id.rs:180-184` ✓
- magnet `urn:btmh:1220` 分支 → `crates/librqbit_core/src/magnet.rs:89-92`（处于 `:81-97` 的 `xt` 处理内）；`as_id32` → `magnet.rs:27`；官方 v2 测试向量单测 `test_parse_magnet_v2` → `magnet.rs:200-212` ✓
- **加强 1**：`file_tree` / `piece_layers` / `merkle` 在整个 `crates/` 内**没有任何解析实现**。唯一命中是 `librqbit_core/src/error.rs` 的错误**消息文本**，以及 `crates/librqbit/src/upnp_server_adapter.rs:434,451,482` 三个名为 `test_torrent_file_tree_*` 的 **UPnP 目录树**测试（同名不同事，不构成 BEP 52 证据）。
- **加强 2**：`error.rs:24-54` 的 14 个 `V2*` 变体在**全仓没有构造点**（grep 除 `error.rs` 外零命中）→ 不可达错误分类，说明产生这些错误的 v2 解析路径不存在。
- **加强 3**：`Id32` / `btmh` / `as_id32` 在 `crates/librqbit/src/`（客户端 crate）**零引用**，只出现在 `librqbit_core` 的解析/类型层 → v2 支持止步于「magnet 文本解析 + 哈希类型」，未进入 torrent 解析与下载路径。
- 对照边界：本 slice 只取证了 **aria2/Transmission 未涉及 v2**（属 §5.6 缺口），libtorrent 的 BEP 52 实现也未逐条取证；所以这句话只能写成「rqbit 相对 libtorrent 是部分支持」，**不能**扩写为「四引擎 v2 支持情况」。

### V9 与我的产出是否冲突
- **无直接冲突**：我的 87 行不含任何 rqbit 主张，四引擎结论不依赖 rqbit 行为。
- **1 处间接冲突、已就地修正**：§1/M1 我写「rqbit 侧的对应量是 `PeerAdmissionConfig` 的保留与 lease」，若被读成「rqbit 没有每 peer 请求窗口」即为**错误**——它**有**独立机制（`max_request_window`，默认 128，见 V5），与准入层并存。引用 M1 时以本节 V5 为准。
- **1 处需警惕的叠加效应**：V7 的「每 peer 128 待发请求」与 V5 的「每 peer 请求窗口 128」数值相同但**语义不同**（积压容量 vs 在途请求窗口），二者在 rqbit 内是两套结构，不得互相代用。
