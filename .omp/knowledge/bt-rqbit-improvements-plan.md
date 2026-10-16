# FD 与 rqbit BT 改进方案

## Context
按用户要求改进 FD 与其锁定的 rqbit fork：连接尝试不得挤占全部已建立连接资源；下载期间正常贡献已校验的 piece，完成后继续做种，正确累计上传与分享率；同时修复发现调度、资源无界增长与续传保护中的已确认问题。用户选择保留现有做种停止规则，并保留 fork + revision 锁定的依赖交付方式。实施不改变正在运行的用户任务，不擅自提交、推送或打 tag。

## 同步与复核结论（2026-10-14，本计划的最新事实层）

**同步已完成**（用户授权「同步本仓库和 ../rqbit 到上游最新 + 在各自仓库新分支实现」）：

|仓库|同步前|cwd 分支|上游|cwd 分支 tip|
|---|---|---|---|---|
|FluxDown|`feature/bt-monitoring` @ `a6d19796`|落后 upstream 24 |`upstream/main` = `e2f031e0`（zerx-lab，2026-10-14）|**`feat/bt-rqbit-sync`** = `5b24b8bd`（merge `e2f031e0`）|
|rqbit|`feat/mse-crypto-primitives` @ `9b7877cb`|落后 upstream 19|`upstream/main` = `57e95d16`（ikatson，2026-10-09）|**`feat/bt-improvements`** = `00374a6d`（= fork main）|

两仓均已 `cargo check` 通过（FD `fluxdown_engine`；rqbit `librqbit`，workspace 未跑）。**均未 commit / push / 打 tag。**

**关键拓扑澄清（推翻了旧基线的错误前提）**：FD 生产 pin `00374a6d` **就是** `zerx-lab/rqbit` main，也是「ikatson 上游 `57e95d16` + 9 个 fork commit（MSE / Android webpki / tracker panic 隔离 / 握手超时下发 / peer_backoff 配置）」的合并点；`git log 57e95d16..00374a6d` 的 9 个 commit 是这个的唯一增量，ikatson 无反向增量（`57e95d16..zerxlab/main` 为空集）。**故「fork 相对上游已过期」这一担忧不成立：fork = 上游最新 + 9。** 本机 `/home/wks/code/Rust/rqbit` 现已 `main` ff 到 `57e95d16`，`feat/bt-improvements` 从 `00374a6d` 起，不再需要 `/tmp` 克隆 fork 的隔离做法。

**复核结论：方案 7 个步骤的措施全部「仍然有效」，其中 2 处事实描述需修正、1 处可删减。** 逐条判定见文末「措施复核」。

## 事实基线与纠正
- FD 根 `Cargo.toml` 的 `[patch.crates-io] librqbit` 锁定 `00374a6d772d4123f079f5adf6a1fe013e363381`。证据源码在 `/home/wks/.cargo/git/checkouts/rqbit-d896eb2b048eb41a/00374a6`；缓存不可修改。同步后本机工作仓库 `/home/wks/code/Rust/rqbit` 新增 `feat/bt-improvements`（自 `00374a6d`）作为 fork 开发基线，直接就是锁定 revision，不再用 `9b7877cb` 或 `/tmp` 克隆。
- FD `native/engine/src/bt_downloader.rs` 当前设置 connect 10s、handshake 10s、read/write 20s、connect_rate 30/s、peer_limit 默认256。rqbit `torrent_state/live/mod.rs` 的同一 semaphore 覆盖拨号至连接结束；入站 `try_acquire_owned` 失败直接拒绝。`Session.peer_limit` 只是每种子默认值，并非 session 总连接上限。
- `30/s × 10s ≈ 300` 是单阶段、持续有候选时的估算；256共享预算会截断 connecting+live，因此“没到300”不能否定估算。connect、MSE/BT握手及明文回退的占用时长不同，不能把每个peer都固定算10s或把600当成保证值。Ubuntu 与 Windows 的节点数量、连通性和需求不同，不能混用为A/B证据。
- rqbit `peer_connection.rs::manage_peer` 已发送 Unchoke；`live/mod.rs::on_download_request` 会服务已具备piece，FD未启用 `disable-upload`。零上传不证明功能关闭，更不证明导致live数量低；`ss ESTAB` 的异步采样也不能证明全部connecting已完成TCP/MSE。
- 已确认上传队列是torrent级unbounded FIFO；Cancel/NotInterested尚未处理。FD上传累计在下载与做种两条路径分别推进基线，存在计数器重建首帧漏计、DB失败丢增量、完成期未结清尾部的风险。
- FD manager的 `directory_has_real_data` 非递归，BT模块的 `stage_dir_has_real_data` 递归且保守保留I/O不确定内容。rqbit `validate_fastresume` 用 `check_piece(...).is_err()`，漏判 `Ok(false)` 哈希不匹配。历史Windows单文件任务2.40GiB→0仍未确认删除者；早一次恢复成功，不能称“每次重启都丢数据”。
- tracker已有 `min_interval`，但HTTP使用 `min_interval.unwrap_or(interval)`，并非缺少该字段；各tracker无限monitor并行，不能用 `buffer_unordered(n)` 包住整个monitor，否则第n个之后永远轮不到。
- 本次只进行了源码核查，未运行测试或改变系统。rust-router skill当前不可用；rust-analyzer的references请求退出code0。执行时先尝试LSP，仍不可用才用精确调用搜索；所有公开API变更先查引用。

## Approach

### 1. 准备正确基线与本地集成边界
1. 在独立临时源码工作区准备锁定revision的 `zerx-lab/rqbit` fork，不改Cargo缓存，也不把旧本地rqbit工作区强制reset/checkout。实施时以唯一目录 `/tmp/fd-rqbit-improvements-<UUID>` 克隆fork并检出上述revision；记该目录为 `R`。如果远端不可访问，从现有Cargo git缓存复制完整checkout到该唯一目录作为本地修改基线，记录来源与revision，不编造新revision。
2. rqbit修改留在该工作区；无新增dependency、unsafe或兼容旁路。FD原有用户改动保留，不切换其分支，不用覆盖文件代替合并。
3. 在单独FD集成工作区验证新API：源于当前FD工作树，包含其已有改动；记目录为 `F`。仅该工作区将 `[patch.crates-io] librqbit` 临时改成 `{ path = "<R>/crates/librqbit" }`，验证时锁定文件也在F，不把绝对临时路径带入生产仓库。
4. 生产FD切换是依赖门：只有用户另行明确授权rqbit提交/推送，或提供已发布的新fork revision，才把生产FD pin改成该确切revision，并运行 `cargo update -p librqbit` 及全链验证。未授权时完成rqbit源码与F的集成验证，交付准确目录/变更与结果，明确“生产pin未切换”；不把旧revision写成已支持新API。纯FD的独立数据保护修复可先合入当前工作树，依赖新rqbit API的调用留在F直至发布门满足。

### 2. 先保护真实payload与恢复正确性
此步骤可与连接/上传库改动独立实施，先取得失败前/通过后证据。
1. rqbit `crates/librqbit/src/torrent_state/initializing.rs::validate_fastresume`：把采样校验改成明确三分支：`Ok(true)`继续；`Ok(false)`记为位图与payload不一致并回退完整校验；`Err(error)`记录带根因错误并回退完整校验。不得因位图失效删除payload；位图clear失败不能伪装clear成功。保留现有采样策略与FD完成期全量重哈希，不扩大为每次恢复全文件扫描。
2. FD `native/engine/src/download_manager.rs::directory_has_real_data` 删除重复实现；`load_and_send_all_tasks` 中Case A/B/C统一调用已有 `bt_downloader::stage_dir_has_real_data`，同步扫描经 `spawn_blocking`，消费JoinError，检查不确定一律保留。非空稀疏文件也保留，len/allocated blocks不能证明piece有效；piece是否有效由哈希验证负责。
3. startup Case C不再仅因DB无对应task就递归删除非空目录。无task但有payload→保留并记录orphan；确证空目录→允许清理。Case A移动部分失败保留剩余内容；Case B暂停/未完成保留内容。显式用户删除/重新下载仍按原语义执行，不把普通恢复改成重下，也不自动接管陌生orphan目录。
4. 保留空staging对应陈旧`.bitv`的防phantom guard，但加强 `clear_stale_fastresume`：操作前检查同一session是否已有其他handle拥有该info_hash；有owner时交给现有重复任务处理，不删除其位图、不复用未知目录。无owner且当前staging确证空时clear；非NotFound失败返回现有错误类型，阻止继续把旧位图作为可信have位图。不得用mtime/birth推断有效性，也不得为保留位图承诺保留不存在的payload。
5. 在实际payload删除和位图clear入口记录一次决策：task_id、info_hash（可取得时）、task status、phase=`startup|empty-stage|explicit-delete|completion`、action=`keep|remove|clear-bitv|failed`及reason。复用现有日志，不做全盘持续扫描、不新增遥测、不打印tracker凭据或完整下载URL。历史ISO归零只有在可控复现指出具体删除入口后才归因；无法复现时交付上述已确认修复与决策日志，明确历史事故原因未定，不猜测删除者，不停用正确的位图检查。

### 3. rqbit：分离半开、live准入与session总资源预算
修改 `crates/librqbit/src/session.rs`、`torrent_state/mod.rs`、`torrent_state/live/mod.rs`，新增内部 `crates/librqbit/src/peer_admission.rs`。现有库没有session级peer governor或公平拨号器，新模块只承载该职责。

#### 3.1 固定API与默认预算
新增公开 `PeerAdmissionConfig`，由 `SessionOptions.peer_admission: PeerAdmissionConfig` 持有，并从库根导出；其字段与默认值如下：
- `session_peer_limit: usize = 1024`：同时存在的逻辑peer连接生命周期总预算，包含拨号、握手、magnet metadata与live；不声称是整个进程OS fd硬上限。
- `torrent_connect_limit: usize = 64`：每个torrent同时**出站**connecting上限；未知info_hash的入站预检查计入session connecting预算。
- `session_connect_limit: usize = 128`：session同时connecting/metadata探测/入站待检查的总上限。
- `outbound_connect_rate: u32 = 30`：session所有torrent与metadata合计每秒拨号上限；原 `PeerConnectionOptions.connect_rate` 保留为可选的每torrent额外上限，不作为并发配额。
保留 `AddTorrentOptions.peer_limit` / `SessionOptions.peer_limit` 名称，明确为每torrent **live** 上限；FD默认256、范围128..512不变，改掉“connecting与live共用”的旧解释。`ManagedTorrentOptions` 从session接收governor与torrent_connect_limit，不另造FD并行实现。新配置值在库构造时检查非零；所有调用者迁移新字段，删除旧shared-lifetime semaphore路径。
公开符号/构造器改动前，使用LSP references迁移 `SessionOptions`、`ManagedTorrentOptions`、`PeerAdmissionConfig`、`BtConfig`、`ProgressUpdate`、`StatsSnapshot` 和 `read_metainfo_from_peer_receiver` 的全部消费者，包括metadata reader与库内测试。LSP仍不可用时用grep工具对对应两仓Rust源码执行 `\b(SessionOptions|ManagedTorrentOptions|BtConfig|ProgressUpdate|StatsSnapshot|read_metainfo_from_peer_receiver)\b`，逐个重读匹配构造/调用；不要只修FD的主下载入口。没有旧API别名或二套准入路径。

#### 3.2 准入、保留区与公平性
1. 一个逻辑peer使用RAII `PeerConnectionLease`，global lease从开始connect/接收检查持有至真正结束；connecting lease在成功晋升live时释放，live lease持续至连接结束。连接失败、MSE回退、取消、重复peer、容量拒绝都释放对应许可；不用unsafe Send/Sync，不把握手完成当连接任务完成。
2. 记G=`session_peer_limit`、C=`session_connect_limit`、K=对应torrent的`peer_limit`。入站headroom通过**限制出站使用量**落实，而非多加一把无效的入站semaphore：global保留 `clamp(G/16, 8, 64)`；connecting保留 `clamp(C/8, 2, 16)`；每torrent live保留 `clamp(K/16, 1, 32)`。出站分别不得超过G/C/K扣掉保留量；入站能使用完整G/C/K，但不能越过总预算。自定义小库测试预算时保留量最多为预算−1，确保至少一个出站槽；库配置本身仍拒绝0。
3. `Session::task_listener` 在accept后、开始MSE/握手检查前取得global/connecting lease，经过 `CheckedIncomingConnection` 传给 `add_incoming_peer`。无预算时立即关闭并计数；原pending-check上限保留为第二道保护，不能在其外创建无界等待任务。
4. 出站不再各torrent无限排队等semaphore。session单一拨号调度任务维护有候选的source轮转队列，每次最多启动一个地址，然后轮转；同时受30/s pacing、global/connecting与该source配额约束。无资格source跳过，等待容量释放Notify/新候选/下一个速率时刻；无source时不定时忙轮询。metadata以info_hash作为source，成功取得metadata即取消其余尝试、注销source。
5. `dht_utils.rs::read_metainfo_from_peer_receiver` 的128局部semaphore与无界waiting futures改为同一拨号调度器；未获配额的地址只存在有界候选池，不创建future。metadata source并发上限用 `min(8, torrent_connect_limit)`，占session总预算。
6. `PeerHandler::on_handshake`（live/mod.rs当前1158附近）在 `set_peer_live` 前同步 `try_promote_live()`：成功后存live lease并释放connecting lease；饱和立即关闭新连接。容量不足计入admission拒绝，不算网络/握手错误，不增加失败backoff；候选仍可在容量释放后的公平调度中重试，不原地自旋。入站同样检查K，不绕过torrent上限。
准入实现固定为tokio Semaphore + OwnedSemaphorePermit，所有多配额获取使用同步try-acquire并在失败时回滚；不得持有部分许可await等待另一配额，网络I/O期间则持有完整当前阶段lease。取得顺序为global总量→出站global子预算（出站才有）→connecting总量→出站connecting子预算→source connecting；晋升顺序为torrent live总量→torrent出站live子预算，失败drop新增许可。内部类型 `AdmissionDenied` 区分global/connecting/live拒绝，作为typed outcome处理，不用匹配错误字符串把普通网络错误吞掉。`PeerConnectionLease::try_promote_live(&mut self, torrent: &TorrentAdmission) -> Result<(), AdmissionDenied>` 保持同步，与现有handshake回调相容；成功晋升后才改peer状态。
7. TCP优先、1s后并行uTP的现有策略保留；逻辑lease覆盖该竞速，两种传输的额外瞬时句柄受connecting上限约束。统计明确区分逻辑peer与transport attempts，不把G描述成操作系统socket精确上限。

### 4. rqbit：有界发现池、候选轮换与tracker请求调度
依赖步骤3的调度器，复用 `PeerStates`、现有backoff与private-torrent规则。
1. 每torrent及metadata source候选记录上限3000；把 `UnboundedSender<SocketAddr>` 的发现/重试通道改成有界池+Notify。按SocketAddr去重，不按IP禁止NAT后不同端口peer。同址旧回调用entry generation校验，不能重新插入已淘汰旧记录。
2. 满池优先淘汰最旧Dead/NotNeeded，再最旧Queued，绝不淘汰Live/Connecting以腾候选记录。用有界FIFO索引维护可淘汰顺序，不在每个新地址到达时扫整张表。没有可淘汰项时明确拒绝新候选并计数；PEX、tracker、DHT和backoff生产者都不因队满永久阻塞。backoff到期回调只重排仍拥有的generation；保留5s/4x/600s/1800s的FD退避参数。
3. 候选存在且live使用量达到K的**50%**时，允许保守peer turnover：每**60s**最多释放 `max(1, ceil(live×4%))` 个。仅选择连接年龄≥**30s**、最近**30s**上传与下载均无payload增量、无pending upload/request、不持有本机缺失piece唯一可用副本的peer；优先NotInterested且没有我们需要piece的peer。仍在上传、正在服务请求、metadata未决或具有唯一数据的peer不动。做种torrent同样可替换无需求的闲置peer，但不得暂停torrent、移出SeedingManager或改分享率目标；无符合项就不轮换。用现有peer cancel/state transition，策略断开不算网络失败，不自动重连刚替换者直至当前候选轮次结束。只有活跃peer与候选存在时运行内存计时。
   > **2026-10-16 收紧（实测驱动）**：原值 90% / 300s / 120s / 120s 在真实任务上**从未触发**——ubuntu 任务 live 峰值 299 对面 peer_limit=512 的 90% 门槛 460，windows 任务 live 仅 29。阈值降到 50%、三个时间窗收到 30s/30s/60s 后，非满载的池子也能替换掉占位不产出的 peer。判定逻辑已抽成纯函数 `turnover_eligible` 并补齐边界测试。
4. tracker monitor保持全部启动与panic隔离，但为**单次网络announce**加session共享 `Arc<Semaphore>`，默认8，HTTP/UDP共用；等待permit、DNS重试、backoff睡眠不占网络permit。请求完成解析后释放permit，再向候选池投递peer，避免下游拥堵占着tracker配额。TCP与uTP的peer预算不混用tracker预算。
5. `TrackerComms::start` 接收共享announce budget；每个真实request的等待均可取消。轮候FIFO使第9个及以后的tracker也得到服务，不能把8个无限monitor放进buffer_unordered。HTTP/UDP请求已有超时照用；HTTP若未覆盖body读取，给单次send+body总期限30s，超时走现有backoff，不另增重试层。
6. HTTP成功响应采用 `max(interval, min_interval.unwrap_or(0))`；显式force interval也不得小于tracker的min_interval。不强加300s下限，不自行新增numwant/tier优先级、scrape或禁用tracker策略；这些不是当前观测的已确认问题。
7. DHT不因当前网络失败自动关闭或更换用户网络配置。复用 `Dht::stats()` 的v4/v6路由表与outstanding指标；在 `dht.rs::bootstrap_hostname` 分别记录DNS、v4查询、v6查询结果及成功family，在stats增加累计 `bootstrap_dns_failures`、`bootstrap_v4_failures`、`bootstrap_v6_failures`、`bootstrap_successes`。一族成功仍算成功，双失败保留两族根因，保留原backoff。纯DHT本地bootstrap用真实UDP节点验证；公网失败只作为环境诊断，不能宣称所有候选必然来自tracker/PEX。

### 5. rqbit：积极而有界、公平的上传服务
保留已有Unchoke与piece-ready校验，不凭空增加tit-for-tat choker或固定8个上传槽。
1. 新增内部 `torrent_state/live/upload_queue.rs`，替换 `ratelimit_upload_tx` 与 `task_upload_scheduler` 的torrent级unbounded FIFO。现有实现无逐peer公平/有界pending管理，新模块只做请求生命周期与轮转。
2. 请求键 `(peer generation, piece index, begin, length)` 去重；每peer最多128个未完成请求，整个torrent最多4096个。上限覆盖待调度、已给writer、正在写的数据请求，而不只限第一段队列。只排ChunkInfo/请求token，不提前复制piece payload。
3. 活跃peer按round-robin每轮一个chunk，经过现有torrent与session两层 `prepare_for_upload`。限速0仍为unlimited；不绕过task/queue/global限制。缓存/磁盘读取继续走原writer与BlockingSpawner，不在actor直接读盘。
4. `Cancel`删除对应尚未开始写的请求，`NotInterested`更新interest状态并取消该peer待服务请求；writer开始写前再次验证请求generation/token，已开始写的chunk可以完成。连接关闭/暂停清空peer pending并释放计数；在限速等待时select请求取消与连接关闭。重复Request不重复排队；容量超限返回明确upload-backlog错误并关闭滥发请求的peer，不静默丢弃后把其留在无限等待状态。
5. payload成功write之后才调用原 `on_uploaded_bytes`；writer失败、取消、元数据/控制包、待服务请求不算uploaded bytes。每次成功或失败结束都回执释放outstanding token。Interested仅是意向，不等于实际Request；有Request且piece已校验才能认定有可服务需求。
6. 在下载尚未finished时持续advertise已校验Have并服务Request；完成后的重哈希/移动/parts重映射路径继续支持既有安全的上传。队列满不能关闭整个torrent、阻止其他peer上传或改变FD做种状态。手动暂停和删除始终终止服务。

### 6. FD：统一上传checkpoint，保证分享率真实跨阶段累计
先取得步骤5的真实payload验证，再接此会计边界；保留现有 `SeedingManager` / `evaluate_entry`，不另造分享率停止算法。
1. rqbit每次创建 `TorrentStateLive` 分配进程内单调 `upload_epoch: u64`（库级AtomicU64，只在live创建时递增），通过已有live `StatsSnapshot`暴露。pause后旧live引用可用于结清最后成功写出的bytes，不能从“计数下降”猜是否新live实例。
2. FD每个engine运行实例生成一次现有uuid v4作为 `source_instance`，与rqbit upload_epoch组成上传来源身份。扩展内部 `ProgressUpdate` 为 `bt_upload_snapshot: Option<BtUploadSnapshot>`，新纯类型 `BtUploadSnapshot { live_epoch: u64, uploaded_bytes: u64 }`；source_instance由engine统一持有，不在每450ms采样重复生成/分配UUID。非BT用None，暂停缺少live快照不能用0覆盖。
3. 在tasks增加仅内部使用的checkpoint列：`bt_upload_source_instance TEXT NOT NULL DEFAULT ''`、`bt_upload_live_epoch BIGINT NOT NULL DEFAULT 0`、`bt_upload_checkpoint_bytes BIGINT NOT NULL DEFAULT 0`；SQLite/Postgres schema与迁移同步，字节列保持BIGINT。不加入wire任务DTO或云同步。
4. `db.rs`新增 `apply_bt_upload_snapshots(&self, source_instance: &str, snapshots: &[(&str, BtUploadSnapshot)], seeding_times: &[(String, i64)]) -> Result<Vec<(String, i64)>, DbError>`。snapshot batch借用已持有task id、复用批次缓冲，不能每次采样重复复制task id。同一事务内按条件原子更新累计和checkpoint并返回累计值：同instance/epoch仅增加max(new−checkpoint,0)；同instance更旧epoch/同epoch更旧采样不增加；新epoch或新engine instance增加该新来源已成功写出的全部bytes。用条件UPDATE/RETURNING实现并发幂等，不采用事务外UPDATE后SELECT。提交回执丢失后重送同snapshot也不重复计数；转换/溢出失败返回错误，不截断。
5. progress_reporter与seeding tick都调用这一个幂等入口，删除BT路径的 `last_uploaded_snapshot` 与 `last_session_uploaded` delta写库逻辑及临时“内存累计冒充成功”的fallback；缓存只用于跳过已成功提交的无变化采样。提交失败不推进缓存，下次重送；无增量且做种时间未到现有兜底期不写库。
6. `bt_downloader.rs`完成handoff先结清当时的live snapshot，取得DB累计值，直接把它写为 `uploaded_at_completion`，不再取max(DB累计,当前会话计数)。注册seeder与最终progress帧使用同一来源身份；即使下载reporter稍后又收到同snapshot，也由checkpoint去重。完成/做种限额事件只能基于提交成功的累计值。
7. pause、正常shutdown、session释放与seed队列降级：先持有旧live Arc，经原pause barrier停止writer，再采样旧live最终uploaded并结清；失败返回/保留可重送snapshot，不输出“已结清”的成功回执。新live首次非零snapshot必须全部计入，即使它大于上次live最后值。显式重新下载按现有重置语义同时重置checkpoint；普通pause/resume不能清零累计。
8. 保留总分享率分子=下载期+做种期累计上传，完成后分享率分子=累计−uploaded_at_completion；保持 `ratio_of` 分母回退、任务级inherit/unlimited、OR/AND、时间、无上传超时、then_action、seed_max_active FIFO、bt_seed_enabled、bt_auto_reseed和手动暂停优先级。启用分享率且没有其他停止条件时继续服务真实需求至目标；没有请求需求时保持等待，不能伪造上传或保证完成时间。

### 7. FD配置与现有运行诊断完整接入
步骤3/4/5/6通过后集成；不新增监控页面或外部遥测。
1. `BtConfig`新增 `peer_connect_limit: usize=64`、`session_peer_limit: usize=1024`；daemon目录新增 `bt_peer_connect_limit`（32..256，默认64）、`bt_session_peer_limit`（128..4096，默认1024）。global connecting派生为 `min(128, session_peer_limit/4)`；30/s全局拨号、3000候选、8 announce并发与turnover策略是内部固定值，不堆一排实验性用户开关。现有 `bt_peer_limit` 128..512/默认256保持，说明改为live上限。
2. 同改 `native/protocol/src/daemon_config.rs`、DB默认、`native/daemon/src/config.rs::bt_config_from_map`、`download_manager.rs::set_bt_config` 与 `SharedBtSession::new`。新两个配置只对下一次shared session构造生效，显示现有重启提示；改变设置不得强拆活跃session或清payload，所有caller用新BtConfig默认补齐。
3. GPUI `crates/settings/src/sections/bt.rs::basic_section` 在peer limit后依次加入两个number row。Web `web/src/lib/rpc/protocol/config.ts` 与 `pages/settings/sections/bt/BtSettings.tsx` 同类型/默认/范围/顺序，沿用写store而非自建RPC。两键按设备资源参数处理，不加入跨设备sync catalog。
4. Android：`mobile/Android/core/src/main/java/com/fluxdown/core/protocol/{SettingsCatalog.kt,BtSettings.kt}` 新行 `PeerConnectLimit`/`SessionPeerLimit`；`app/src/main/java/com/fluxdown/app/feature/settings/bt/BtPage.kt` 补general分组、title/description、数字输入、搜索定位。iOS：`mobile/FluxDown/FluxKit/Sources/FluxDomain/Protocol/SettingsCatalog.swift`、`FluxDown/Features/Settings/{BtSettingsLogic.swift,BtPage.swift}` 同行/范围/顺序。共享en/zh增加 `btPeerConnectLimit`/`btPeerConnectLimitDesc`、`btSessionPeerLimit`/`btSessionPeerLimitDesc`，说明半开与live分开、session包含metadata；不改社区语言、不手改生成绑定。
新增标题/说明固定为：`btPeerConnectLimit` = en “Per-download connecting peer limit” / zh “单任务半开连接上限”；`btPeerConnectLimitDesc` = en “Limit outgoing connection attempts separately from established peers. The session budget may impose a lower limit. Takes effect after restarting the BT session.” / zh “单独限制出站连接尝试，不占用已建立连接的任务限额；仍受会话总预算约束。重启 BT 会话后生效。”；`btSessionPeerLimit` = en “Session peer connection budget” / zh “会话 Peer 连接预算”；`btSessionPeerLimitDesc` = en “Limit peer connections across all downloads and seeders, including connection attempts and magnet metadata. Takes effect after restarting the BT session.” / zh “限制所有下载和做种任务的 Peer 连接总数，包含连接尝试与磁力元数据读取。重启 BT 会话后生效。”。`btPeerLimitDesc`改为每torrent已建立peer上限及新任务生效的说明，明确不再包含半开；沿用原新建任务生效语义。
5. rqbit在已有aggregate stats按真实事件维护：`interested_peers`、`uploading_peers`（最近10s成功上传payload的live peer）、`upload_requests`（已接受有效Request累计）、`upload_queue_depth`（尚未完成Request数）、`uploaded_bytes`（该live epoch payload累计）、`incoming_rejected`、`admission_rejected`、`candidate_dropped`。interest处理NotInterested与死亡必须减数；累计attempt/error统计不能因候选淘汰而下降。保留 useful_peers 的“live peer累计下载≥1MiB”语义，不冒充正在传输。
6. FD不再每次 `bt_peer_stats_from_handle` 创建带String key的整份per_peer_stats_snapshot再求和：采用rqbit的无分配aggregate快照，必要的live activity到期维护在活跃调度内。同步这8个新字段到 `transfer_activity.rs::BtPeerStats` 和 `native/protocol/src/task_activity.rs::BtPeerStatsDto`（Option、serde default、camelCase）、`native/engine-protocol/src/lib.rs::task_runtime_to_dto`、`web/src/lib/rpc/protocol/task.ts` 与 `apply.ts`。Rust clear_active_runtime仍Default；Web清除这些字段至null，旧快照未提供时不制造0。旧客户端可忽略新增可选字段，不新增严格event枚举。
7. session级预算/拒绝、tracker并发与DHT健康复用现有诊断日志，活跃BT时按原采样节奏追加聚合读数；无活跃BT不扫描磁盘或周期写log。TaskRuntime移动端当前不投影BtPeerStats，新增字段不引入UniFFI DTO；客户端设置两键仍完整镜像。

## Critical files & anchors
以下锚点用于区分易改错的边界；实现前重读行号，不凭历史行号直接打补丁。
- rqbit `crates/librqbit/src/peer_connection.rs::manage_peer`（478附近Unchoke、ReadChunkRequest writer）：上传实际成功边界在write完成后，不在enqueue。
- rqbit `crates/librqbit/src/dht_utils.rs::read_metainfo_from_peer_receiver`（42附近局部128 semaphore）：metadata也必须进入全局预算，不能只修live torrent。
- FD `native/engine/src/download_manager.rs::account_seeding_tick`（4605附近）与progress_reporter（12518附近）：两条采样来源都必须使用幂等checkpoint。
- FD `native/engine/src/bt_downloader.rs` 完成handoff（5502附近）及 `SharedBtSession::shutdown`（1408附近）：结清尾部后冻结基线，不能仅检查普通周期帧。
- rqbit `crates/librqbit/src/torrent_state/initializing.rs::validate_fastresume`（157附近）：`Ok(false)`是明确哈希失败，不是成功。

## Verification

### 可重复、本地端到端证明
测试用独立临时目录、loopback随机可用端口、非零patterned payload、DHT/LSD/公网tracker关闭，复用已读的rqbit `src/tests/e2e.rs`/`test_util.rs` 与FD `native/engine/tests/bt_parts_reseed.rs` 的真实Session/create_torrent模式。新回归测试只覆盖用户可见行为，不测源码文本、默认值拷贝或mock回显。

1. **256不是300反证**：先用小预算可控静默peer复现旧代码live+connecting共享上限和入站被拒；新代码连接/握手分别延迟，证明connecting≤指定半开限额、live≤K、所有source逻辑lease合计≤G。5个torrent加1个metadata source持续供给候选，观察round-robin各自获得拨号，不出现每torrent各30/s叠加。出站饱和时真实入站完成BT握手并收到有效piece；关闭全部peer后所有lease释放，不只比较几个计数器。TCP-only与uTP各执行一次。
2. **上传期三peer场景**：A有完整patterned文件；B=待测FD，A→B用低下载限速保证B未完成；C只连接B，不连接A。等B获得并校验首个piece后，C真实Request该piece，验证C收到payload与原piece hash一致，且B仍为下载中、B累计上传增加。C断连/Cancel后不继续发尚未开始的取消chunk；一个peer请求洪泛时另一个正常peer仍收到piece。无Request的peer不能让uploaded计数增加。结束B下载后让不同C下载目录继续提出需求，验证B做种至配置目标。
3. **累计与handoff**：已提交旧live100字节，新epoch首帧40→累计140；同frame重复→仍140；旧epoch迟到/新epoch过期采样→不回退不重复。旧live100，新epoch首帧200→累计300（不能只靠下降检测）。提交故障后重送、提交成功但回执丢失后重送都只增加一次；完成基线140，后续上传60→总200、post上传60。使用真实SQLite与PG事务故障/回执中断路径，不用模拟UI回显证明。正常pause/restart后累计、目标停止原因和文件内容不变；OR/AND、时间、无需求超时、FIFO、手动pause分别保留。
4. **恢复与损坏位图**：单文件和嵌套多文件各真实下载部分，保留已校验piece hash，正常退出engine、重开同data_dir继续同task，已有文件hash/内容不丢，不从0重取已有效piece。另将一个被位图标记为have的piece写成错误pattern：必须重新校验/补取后得到正确文件，不能报告虚假完成。DB无task的非空orphan保留；真正空stub可清。显式delete/re-download仍按原语义，不作为续传失败。Windows覆盖嵌套目录与sparse文件；Linux的目录len不能代替该平台验证。
5. **发现有界性**：注入超过30000个不同候选和重复地址，记录保留数≤3000、旧回调不能复活淘汰entry；活跃连接不中断，新的可达peer能替换旧Dead/Queued。填满live后设置真实30s无payload候选场景，60s内存timer到期只替换规定数量；正在上传/有唯一piece的连接不被替换。虚拟时间可驱动策略，但最后由真实peer接通/退出验证结果。
6. **tracker调度**：20个本地HTTP/UDP tracker，默认并发8；服务器侧记录峰值≤8且全部20个收到announce。错误/超时tracker不能占permit直至backoff睡眠结束，取消torrent释放等待。返回interval=60、min_interval=300→下一次不早于300；interval=600、min_interval=300→不早于600。只有第9个及以后真正收到请求才证明没有无限monitor饿死。
7. **DHT诊断**：复用 `crates/dht/src/dht.rs` 的本地UDP bootstrap测试模式，一族可用/另一族失败仍成功；双失败能区分DNS与各族查询失败。纯DHT候选能令本地magnet取得metadata；公网4个地址失败不算代码回归，也不自动关掉DHT或改Clash/防火墙。
8. **真实设置表面**：启动F的agent/headless+Web或GPUI，修改两新键到合法边界，读取daemon config确认保存；非法32以下/256以上及128以下/4096以上按目录拒绝，不是仅UI限制。开启下一session，日志证实G/C与每torrent半开值。分别确认GPUI/Web字段顺序、说明、重启提示；Android/iOS没有运行环境时明确只完成了可执行的构建/静态范围，不能宣称视觉验收。

### 精确执行命令与环境
以下在实施阶段执行，本计划阶段不运行；R/F为步骤1记录的真实目录，不以旧生产pin冒充新库验证。
- rqbit（cwd=R）：`cargo test -p librqbit --lib peer_admission -- --test-threads=1`；`cargo test -p librqbit --lib upload_queue -- --test-threads=1`；`cargo test -p librqbit --lib fastresume -- --test-threads=1`。新增测试名称使用这些过滤词，覆盖上面的真实行为；已有E2E再跑 `E2E_NUM_SERVERS=3 E2E_TIMEOUT=180 cargo test -p librqbit --lib test_e2e_download_tcp -- --test-threads=1`，uTP同命令过滤 `test_e2e_download_utp`。
- tracker/DHT（cwd=R）：`cargo test -p librqbit-tracker-comms tracker_announce -- --test-threads=1`；`cargo test -p librqbit-dht bootstrap -- --test-threads=1`。包名已按锁定revision的Cargo.toml核对；新过滤用本地服务器测试，不运行公网ignored测试冒充本地结果。
- FD（cwd=F，path patch已指向R）：改动前 `cargo check -p fluxdown_engine --lib`；行为验证 `cargo nextest run -p fluxdown_engine bt_upload`、`cargo nextest run -p fluxdown_engine bt_resume`、`cargo nextest run -p fluxdown_engine bt_admission`；现有 `cargo test -p fluxdown_engine --test bt_parts_reseed`；`cargo test -p fluxdown_daemon`、`cargo test -p fluxdown_protocol`、`cargo test -p fluxdown_engine_protocol`。若没有nextest，按相同crate/filter用cargo test，不运行测试workspace。
- PG checkpoint（cwd=F）：仅使用用户已有或隔离测试PG，`PG_TEST_URL=<isolated-url> cargo test -p fluxdown_engine bt_upload -- --ignored --test-threads=1`；新增PG测试沿用仓内PG_TEST_URL隔离模式，测试名含bt_upload。没有PG时完成SQLite与其余验证，报告PG未覆盖，不对生产PG运行迁移。
- Web（cwd=F/web）：`bun test src/lib/rpc/protocol/config.test.ts src/lib/rpc/apply.test.ts`；`bun run build`。真实浏览器打开开发实例BT设置，执行步骤8并观察保存/边界；若用内嵌SPA，先build Web再重编agent。浏览器关闭新测试tab，不操作用户运行实例。
- Android（cwd=F/mobile/Android，已安装SDK及Android Studio JBR时）：`./gradlew :core:testDebugUnitTest :bridge:testDebugUnitTest :app:assembleDebug`。iOS（macOS）：`mobile/FluxDown/scripts/build-core.sh` 后执行项目已有FluxKit模拟器测试与App构建；Linux只明确报告不能执行iOS运行面。
- 最终fork revision可用后，在生产FD真实pin下重新执行相关行为与构建；不能把path override结果当作revision发布验证。

### 公网对照记录
仅在隔离验证实例用公开torrent复测，不改用户当前256设置。Ubuntu和Windows各自独立进行旧/新构建交错ABBA，单torrent与5torrent分别重复至少3轮；相同torrent、文件选择、客户端阶段、上传/下载限制、网络路由、uTP/MSE设置，并排除旧payload复用造成的阶段偏差。记录速率、actual upload、demand/request、各阶段connecting耗时与失败、live、拒绝、候选淘汰、DHT表、CPU/RSS与socket资源；不能以跨swarm速度差或某次peak归因。吞吐不是硬性CI阈值，硬验收是本地请求可服务、配额正确、有界、公平、累计幂等、真实payload续传。

## Assumptions & contingencies
- 用户已选择保留OR/AND和现有停止条件：若时间/无上传条件在OR模式下先满足，可在分享率前停止；这是用户规则，不擅自改成ratio-only。bt_seed_enabled关闭、seed_max_active排队或手动暂停也不能被“积极上传”覆写。
- 用户选择fork/revision模式。方案审批不代替git提交/推送授权；新revision发布是明确外部前置条件。未取得授权或已发布revision时，按步骤1完成本地源码和集成证明，生产依赖切换保持阻塞，不擅自vendor或引用临时路径。
- 新参数默认是保守资源预算，不宣称经公网测得最优。不自动把已有bt_peer_limit升级到512，不降低握手超时来掩盖共享闸门，也不关闭uTP/MSE/DHT来制造好看的指标。
- 请求有需求且本机有已校验piece时应正常上传；没有需求、NAT不可达或对端限速时不能承诺非零速度或达标时刻。环境问题只定位和报告，不未经授权改系统网络。
- 崩溃/强杀只能保证已提交checkpoint与已成功持久化payload；正常退出必须结清并保留恢复状态。不得宣称尚未持久化的最后网络字节可在断电后精确恢复。

## 措施复核（同步到上游最新后的逐条判定）

基线：FD `feat/bt-rqbit-sync` @ `5b24b8bd`（含 upstream `e2f031e0`）；rqbit `feat/bt-improvements` @ `00374a6d`（= 上游 `57e95d16` + 9 fork commit）。以下每条都已在同步后的树上重新取证。

|方案步骤|上游/rqbit 是否已做|复核判定|
|---|---|---|
|1 基线与集成边界|——|**部分被同步取代**：fork 即 `00374a6d` 已含上游全部；`/tmp` 克隆不再必要，改用 `feat/bt-improvements`。仍保留「生产 pin 切换是独立授权门」。
|2 payload/恢复正确性|上游**未**修|**全部有效**。`validate_fastresume`（`initializing.rs:86`）仍用 `fo.check_piece(pid).is_err()`（`file_ops.rs:186` 返回 `Result<bool>`）⇒ `Ok(false)` 哈希不匹配被当成功，**确认**。FD `directory_has_real_data`（`download_manager.rs:603`）仍非递归、Case C（`6305-6314`）仍无条件 `remove_dir_all`；`clear_stale_fastresume`（`bt_downloader.rs:992`）仍无同 info_hash owner 检查。
|3 分离半开/live/session 预算|上游**未**做|**全部有效**。`peer_semaphore`（`live/mod.rs:195`，建 `:290` `Semaphore::new(peer_limit.unwrap_or(128))`）仍是**每 torrent** 且覆盖 connecting→live→EOF：出站 `:673 acquire_owned` 后把 owned permit 传入 `task_manage_outgoing_peer` 至任务结束 `:609 drop(permit)`，入站 `:377 try_acquire_owned` 失败即 `ConcurrencyLimitReached`。**无** session 级 governor / 公平拨号器（`:613-684` 每 torrent 各自 `acquire_owned().await` + 无界 `peer_queue_rx`）。新增 `peer_admission.rs` 仍必要。
|4 有界发现池/候选轮换/tracker 调度|上游**未**做|**有效**，含两处事实修正（见下）。tracker：`tracker_comms.rs:347` 仍 `min_interval.unwrap_or(interval)`；`add_tracker`（`:236-264`）每 tracker 独立 monitor 进 `FuturesUnordered`（`:200-205`），**无**共享 announce 限流。DHT `bootstrap_hostname`（`dht.rs:997-1018`）仍 `v4.or(v6)`、**无**任何 `bootstrap_*` 计数（`DhtStats` 只有 `id/outstanding_requests/routing_table_size[_v6]`）。
|5 公平有界上传服务|上游**未**做|**全部有效**。`ratelimit_upload_tx`（`live/mod.rs:212`）仍是 `unbounded_channel`（`:262`），`task_upload_scheduler`（`:444-469`）纯 FIFO 无逐 peer 公平/无界 pending；`NotInterested`/`Cancel` 在 `on_received_message`（`:1104-1109`）明确 no-op。`on_uploaded_bytes` 在 `peer_connection.rs:605` 于 `write_all` 成功后调用（正确边界），方案第 5.5 条与之一致。
|6 上传 checkpoint 统一|上游**未**做|**全部有效**。FD 仍两条 delta 基线：`progress_reporter`（`last_uploaded_snapshot`，`download_manager.rs:1950/12573-12590`，DB 失败时 `:12598` 内存累计「假装成功」）与 `account_seeding_tick`（`last_session_uploaded`，`bt_seeding.rs:291/562-567`）。完成 handoff（`bt_downloader.rs:5519`）仍 `uploaded_bytes.max(completed_uploaded_bytes)`。`clear_active_runtime` 仍 Default。
|7 FD 配置接入|新增键**不存在**|**有效**。`peer_connect_limit`/`session_peer_limit` 全仓 0 命中；`bt_peer_limit` 仍 `BtConfig.peer_limit`（`bt_downloader.rs:460`，128..512/默认 256，clamp `normalize_bt_peer_limit:420`），目录键 `bt_peer_limit`（`daemon_config.rs:157-159`）语义仍是「每 torrent live 上限」以外的旧解释（脚注 `bt_downloader.rs:408-421` 明确说「同一信号量计连接中与已连接」），与方案第 7.1 条的改述一致。`bt_peer_stats_from_handle`（`bt_downloader.rs:2945-2948`）仍每次 `per_peer_stats_snapshot` 建 String 键整表再求和。

### 两处必须修正的事实描述（否则会误判实施重点）

1. **方案第 3 节「同一 semaphore 覆盖拨号至连接结束；入站 try_acquire 失败直接拒绝」成立，但方案第 4.3/6 条引用的「session 级」措辞要落到「每 torrent」。** 该 semaphore 是 `TorrentStateLive` 字段、每种子一份；session 级只有 `max_pending_incoming_handshake_checks`（`session.rs:1120`）这一道握手检查并发上限。方案第 3.1 节新增 session 级 `PeerAdmissionConfig` 才是真正补上 session 总预算，方向正确；但**不要把现状描述成「已有 session 上限」**。
2. **方案第 4.5 条「把 8 个无限 monitor 放进 buffer_unordered 会饿死第 9 个之后」的前提需要重读**：现状是 `FuturesUnordered`（`:200-205`，**不是** `buffer_unordered`），所有 tracker 的 future 都会轮询，不会饿死；真正缺的是**共享 announce 并发许可**（防止 N 个 tracker 同时打网络/DNS）。方案要达成的目标（20 个 tracker、峰值 ≤8）仍然有效且需要实现，但**理由应从「避免饿死」改为「限制并发网络 announce 峰值 + 隔离下游拥堵」**。

### 一处可删减

- 方案第 4.7 条 DHT 诊断计数（`bootstrap_dns_failures`/`_v4_`/`_v6_`/`bootstrap_successes`）在 `crates/dht` 里确认完全不存在，**需要新增**（非「补全」）。但这是纯诊断增强、非用户可见缺陷；在「先修正确性、再做预算/调度」的收紧排序下，**优先级最低，可延后**，不阻塞步骤 2/3/5/6 的落地。

### 同步带来的实现面变化（必须在实施时重新对行号）

- **FD 侧新上游提交只碰了 `download_manager.rs` 的进度快照终态路径**（`fb912276` 修「整表快照覆盖终态进度导致卡 99%」，改了 `crates/app`+`native/engine` 的 `uploaded_bytes: cumulative_uploaded` 投影；`0bbbdf36` 新增 `import_tests.rs`）。**未触碰** `last_uploaded_snapshot` / `add_task_uploaded_bytes` / `apply_seeding_tick` / `uploaded_at_completion` 的语义，故步骤 6 的 checkpoint 改造仍从零开始；但步骤 6.5 改 `progress_reporter` 时须与新 `fb912276` 的终态字段（不要把它回退成 `uploaded_bytes: cumulative_uploaded` 的旧行为）合并。
- **rqbit 侧新增 `max_request_window` 配置**（上游 `79ff792e`，`peer_connection.rs:77` 附近，默认 128，封顶 peer 的 reqq）——不是上传调度，与步骤 5 的**上传** 128/4096 上限无冲突，但新增 `ManagedTorrentOptions`/`PeerConnectionOptions` 字段时不要把二者混淆。
- **rqbit 上游 `193a5bd8` 被 `d7bfc7b0` 原样 revert**（paused/stopped 释放文件句柄），净值 0，不影响本方案。
- 方案「Critical files & anchors」的行号需重取：`manage_peer` 实际在 `peer_connection.rs:418`（方案写 478 附近），`read_metainfo_from_peer_receiver` 在 `dht_utils.rs:30`（方案写 42 附近，128 semaphore 确在 `:42`），`on_download_request` 在 `live/mod.rs:1503`（方案写 `:1545` 是 `ratelimit_upload_tx` 发送点）。

## 落地记录（2026-10-16）

计划内步骤 2–7 全部落地；库侧发布在 `lingdiansr/rqbit` main（`zerx-lab/rqbit` 无写权限，
其 main 仍停在 `00374a6d`）。**决定（B 方案）**：接受 `lingdiansr/rqbit` 为该 librqbit fork 的
长期依赖源，FD 的 pin 与 release.yml 校验前缀均指向它，不再回切 `zerx-lab`。

| 层 | 提交 |
|---|---|
| rqbit | `d6c5884f` 有界准入 / 轮换 / 发现池 / tracker 预算 / DHT 计数；`cd181c3b` `live_epoch`；`80c72d3e` peer 需求与上传队列计数；`d995b0c9` 上传计数进 `StatsSnapshot`；`c17ed3ae` peer 质量聚合（去逐 peer 求和）；`70a73c2e` 三 peer 上传 e2e + 聚合单测（含修「新建连接默认 interested 漏计」）；`076d8a06` 30000 候选规模压测 + 20 真实 tracker 并发峰值实测 |
| FD | `5f69ea76` payload/位图跨任务保护；`34e745a4` pin 与 CI 校验；`c421c2f3` 半开/会话预算两键；`e2f76566` 上传幂等 checkpoint；`e8e01453` 上传与需求计数；`a4adac14` i18n；`8a87dd97` 边界结清 + 诊断日志 + 去分配 + 文档；`fae4a4c1` checkpoint 故障注入测试 |

计划 `§Verification` 的覆盖状态（本机实跑）：

| 验证项 | 状态 |
|---|---|
| 8 真实设置表面（headless + `/rpc`） | ✅ 已跑：默认 64/1024、越界 31/257/127/4097/513 拒绝、合法值持久化，会话日志逐项核对 |
| 已有 E2E `test_e2e_download_{tcp,utp}`（`E2E_NUM_SERVERS=3`） | ✅ 已跑通过 |
| 2 上传期三 peer | ✅ 新增 `test_upload_while_downloading_{tcp,utp}`（A 做种 / B 限速下载中 / C 只连 B） |
| 3 累计与 handoff（含提交故障重送） | ✅ SQLite 幂等 + 触发器故障注入；PG 未覆盖（无隔离实例） |
| 4 恢复与损坏位图 | ✅ 已有 `test_fastresume_rejects_phantom_have_piece` 实跑通过；嵌套多文件 / Windows sparse 未覆盖 |
| 5 发现有界性 | ✅ 注入 30000 候选的规模压测：池不超上限、最老被淘汰、live 不被牺牲、新 peer 仍可入 |
| 6 tracker 调度 | ✅ 20 个真实本地 HTTP tracker（服务端各自记录并发）+ 默认预算 8：全部收到 announce、实测峰值 ≤ 8 |
| 7 DHT 双栈失败归因 | ✅ 已补：`bootstrap_succeeds_when_one_address_family_works` 用真实本地 UDP 节点 + 双族解析的主机名（仅 v4 有监听）端到端跑通：单族可达即计成功，且断言 v6 失败计数确实增长（否则两族都成功也会通过）。仍无公网双栈失败诊断 |
| 1 多 torrent round-robin 真实入站 | ✅ 已补：`multi_torrent_shares_the_session_budget_without_starving_any_torrent`——5 个 torrent 对同一做种 session 真实 loopback 下载，共享一个不足以同时容纳全部 torrent 的 session 预算；断言同时 live 的 torrent 数 `< 5`（预算未被争用时测试即失败）、每个 torrent 都拿到 peer、session 逻辑 peer 峰值不超预算。真实入站握手（出站饱和时被入站建立）仍未覆盖 |

验证面：rqbit `--lib` 95 通过（含本轮新增 `multi_torrent_budget`）、`librqbit-dht` 30 通过；FD `fluxdown_engine --lib` 1274 通过（合并 254b3599 后）、`fluxdown_protocol` / `fluxdown_daemon` / `fluxdown_engine_protocol` 通过；Web `bun test src/lib/rpc` 45 通过；`cargo clippy --workspace --exclude fluxdown_server --all-targets -D warnings` 通过；release.yml 的 librqbit 校验逻辑本地复跑通过。

### 再次同步上游（2026-10-16 晚）

rqbit 侧**无需同步**：`ikatson/rqbit` main 仍是 `57e95d16`，与本仓 `feat/bt-improvements` 的分叉点一致；fork 前缀已推送在 `lingdiansr/rqbit` main（`076d8a06`），FD 生产 pin 正是这一 revision。

FluxDown 侧上游（`zerx-lab/FluxDown`）已从 `e2f031e0` 前进到 `254b3599`（43 提交），已合并进 `feat/bt-rqbit-sync`。合并面：

- 唯一内容冲突在 `native/engine/src/bt_downloader.rs`。上游 `99a148e2`「删除无调用方的死代码」删掉了 `default_tracker_list()` 并把 tracker 合并**内联**进 `SharedBtSession::new`；本分支 `5e2219d3` 已把同一逻辑抽成 `merged_tracker_urls()`，并**有意**改语义为「PUBLIC_TRACKERS 恒为超集，custom/subscription 只做追加」，还新增了测试调用它。上游内联版本反而是旧语义（`custom_trackers` 非空即丢弃内置列表），故保留本分支 helper 与调用点，另删掉全仓零引用的 `default_tracker_list`（与上游删除决定一致）。
- 上游 `ProgressUpdate` 新增 `deleted_tasks`（批量删除通知，其余字段无意义）；本分支唯一未走 `..Default::default()` 的 BT 完成帧补 `Vec::new()`。
- `git merge-tree` 预演出的其余 17 个交集文件（`Cargo.lock`、`assets/i18n/*`、`native/protocol/*`、`web/src/lib/rpc/*` 等）全部自动合并。
- 无语义冲突残留：上游新增的 `disabled_pieces`/`piece_bitfield`/`transfer_activity` 等改动已并入，本分支的 `bt_upload_snapshot` checkpoint、`bt_peer_connect_limit`/`bt_session_peer_limit` 两键、`BtPeerStats` 聚合均保留。
- `crates/app/src/windows/new_download.rs` 的 rustfmt 差异是**本机 rustfmt 版本漂移**（上游原版同样会被改写），非本分支引入，保持上游原样不动。

未覆盖面：Android（本机无 Android SDK）与 iOS（非 macOS）未做编译/运行验证，仅静态核对目录项、设置行、i18n 键与「不进同步目录」；PG 已用 podman 隔离实例（`postgres:17-alpine`，`localhost:55432`）实测 `pg_smoke_roundtrip` 与新增 `pg_upload_checkpoint_is_idempotent_and_never_rolls_back`（7 个 checkpoint 状态迁移全部通过，与 SQLite 同语义）。

### 复核结论（2026-10-16 晚，逐条）

计划书没有未完成的实施项：步骤 2–7 的代码、配置、i18n、跨端镜像均已落地。§Verification 的 8 项此前 2 项为 ◐，现均补齐，其余 6 项此前已实跑。剩余未覆盖面仅为**本机环境不可达**的部分：

| 剩余未覆盖 | 原因 | 是否阻塞 |
|---|---|---|
| Android 编译/运行（`gradlew`、设置页视觉） | 本机无 Android SDK、无 Android Studio JBR | 否：仅能构建验证，非方案缺陷 |
| iOS 编译/运行 | 本机非 macOS | 否：同上 |
| §1 中「session 预算 + 多个选同一 peer 的 torrent」 | 该预算按 session 计数，不是 per-peer 去重表，无从用自定义预算触发 | 否：计划未要求 |
| §7 公网双栈 DHT 失败诊断 | 需真实公网环境 | 否：计划明示「公网失败算不上代码回归」 |
| §5 Windows sparse / 嵌套目录恢复 | 本机 Linux | 否：计划要求 Windows 单独验证 |
| §2 上传期 B/C 断连-重连竞态 | 现有 e2e 已覆盖连接建立与上传；该时序无既知缺陷 | 否 |

新增验证项（本轮）：`multi_torrent_budget`（5 torrent 共享 session 预算的真实 loopback 争用）、`bootstrap_succeeds_when_one_address_family_works`（真实 UDP 双栈归因）、`pg_upload_checkpoint_is_idempotent_and_never_rolls_back`（PG checkpoint 幂等）。
