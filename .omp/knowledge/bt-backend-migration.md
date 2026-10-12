# BT 后端迁移与成熟度分析

> 本文记录 FluxDown 当前 BT 后端（librqbit）与 libtorrent 类成熟 BT 引擎的差距、迁移工程量和风险。UI 不在本分析范围内；但 API、协议、daemon、移动端构建和发布仍属于范围。
>
> 外部仓库状态按本次检索结果记录；外部仓库会继续变化，版本与提交以链接为准。

## 一、上一轮 libtorrent 接入结论

FluxDown 当前没有通用 BT backend trait。BitTorrent 深度绑定到 librqbit：`native/engine/src/bt_downloader.rs` 直接使用 `Session`、`ManagedTorrent`、`AddTorrentOptions`、`SessionPersistenceConfig` 等类型；`bt_partfile.rs` 与 `bt_sparse.rs` 直接实现 librqbit storage 适配；`bt_seeding.rs` 使用 librqbit torrent handle 语义；`download_manager.rs` 直接依赖 `SharedBtSession`、`BtHandle` 和 librqbit 生命周期。

核心规模：

- `native/engine/src/bt_downloader.rs`：约 8,310 行。
- `native/engine/src/bt_partfile.rs`：约 1,083 行。
- `native/engine/src/bt_sparse.rs`：约 252 行。
- `native/engine/src/bt_seeding.rs`：约 1,042 行。
- 关键 BT 集成测试：`bt_file_selection.rs`、`bt_parts_reseed.rs`、`bt_sparse_add.rs`，另有 pause/resume、corruption、completion、unattended、删除保护等回归覆盖。

因此，libtorrent 不是替换一个 Cargo 依赖，而是迁移以下语义：

1. magnet metadata 获取与超时。
2. 文件选择与部分文件下载。
3. staging、重命名、完成搬移。
4. 跨文件边界 piece 校验与 parts sidecar 做种。
5. pause/resume/delete 与 metadata 获取并发竞态。
6. fast-resume、DHT、tracker、监听端口与 session 生命周期。
7. 上传统计、做种槽位、分享率/时长/不活跃限制。
8. Windows/macOS/Linux/NAS/Android/iOS 构建与发布。

粗估（1 名熟悉 Rust、C++、BT 和跨平台构建的工程师）：

|范围|粗估|难度|
|---|---:|---|
|Linux x64 libtorrent POC|5–10 人周|中高|
|先抽 FluxDown-owned backend 边界并保留 librqbit|2–4 人周|中|
|桌面/NAS 生产可用并保持当前任务语义|10–16 人周|高|
|全平台、迁移、发布与移动端|10–20+ 人周|很高|
|继续扩展现有 librqbit fork|1–8 人周，取决于功能|低到中|

推荐路线：先定义只含 FluxDown 自有 DTO 的 `BtBackend`/session/handle/alert contract，再把现有 librqbit 包装为默认实现；将 BT 集成测试转成 backend contract tests；仅在 Linux x64 上做 libtorrent POC；先解决 storage、resume、部分文件和做种语义，再决定跨平台发布。

如果没有明确的 libtorrent 必需能力，继续维护当前 librqbit fork 的风险和成本更低。

## 二、lingdiansr/rqbit 与上游 rqbit 的当前关系

### 2.1 事实状态

本次读取到的主分支状态：

|仓库|主分支提交|版本/状态|
|---|---|---|
|`lingdiansr/rqbit`|`002a34848162358e79c094f7284c5bcb210a54fa`|Cargo workspace version `9.0.1`，约 2,091 commits|
|`ikatson/rqbit`|`57e95d16c830863a1fde32d8fa98153b53a3d0b4`|Cargo workspace version `9.0.1`，约 2,110 commits|

GitHub compare 页面显示 `lingdiansr:main...ikatson:main` 有 **19 个提交、22 个文件变更**。fork 的主分支停在 2026-09-08 左右；上游主分支已经推进到 2026-10-09 左右。

当前可见信息表明：

- `lingdiansr/rqbit` 是 `ikatson/rqbit` 的公开 fork。
- fork 的 workspace `Cargo.toml` 与上游基本一致，版本仍为 `9.0.1`。
- fork 当前主要是上游的滞后镜像，而不是一个包含大规模独立 BT 能力的分支。
- 上游近期提交主要由上游作者合并，fork 没有显示出独立的成熟度路线。

### 2.2 fork 与上游之间已出现的上游修复

比较页和上游提交列表显示，上游在 fork 停止同步后继续合入了多项网络、磁盘和稳定性修复，包括：

- incoming connection listener 在错误分支下可能 `select!` 崩溃的问题。
- peer handshake 需要持续读取直到完整握手，而不是假定一次 read 完整返回。
- Windows DHT 单个 UDP 接收错误不应杀死 DHT worker。
- storage wrapper 需要正确转发 `pwrite_all_vectored` 和类型身份。
- 2 GiB 以上文件在 32-bit `off_t` 路径下的写入问题。
- streaming 在初始检查完成前不应错误开始。
- peer request window 可配置。
- info-hash 字符串识别需要严格限制为 40 位十六进制。
- 统计连接到完整 torrent 的 peer 数量。
- peer handshake 能力探测和 incoming connection 处理。
- `socks5h://` 代理支持。

因此，若在两者中选择基础版本：

- **优先使用上游当前主线**，而不是 `lingdiansr` 当前主线。
- 如果必须使用 fork，先把上游提交同步进 fork，再进行 FluxDown 定制。
- 不应把 fork 当前状态判断为“比上游更接近 libtorrent 的成熟实现”。当前证据不支持这个结论。

### 2.3 与 FluxDown 实际依赖的关系

当前 FluxDown 根 `Cargo.toml` 使用的是 `zerx-lab/rqbit` 的固定 revision，而不是直接使用 `lingdiansr/rqbit` 主分支。

因此：

- `lingdiansr/rqbit` 与 `ikatson/rqbit` 的比较，主要说明上游 rqbit 的公开发展状态。
- FluxDown 实际 fork 的独立改动仍需以根 `Cargo.toml` pin 的 revision 和该 revision 的 diff 为准。
- 不能直接把 `lingdiansr` 的提交状态当成 FluxDown 当前后端的完整状态。

## 三、rqbit 已经具备的能力

上游 README 声称已支持以下核心能力：

- BEP-3 基础 BitTorrent 协议。
- BEP-5 DHT。
- BEP-7 IPv6 tracker extension。
- BEP-9 magnet metadata。
- BEP-10 extension protocol。
- BEP-11 PEX。
- BEP-12 multitracker metadata。
- BEP-14 local service discovery。
- BEP-15 UDP tracker。
- BEP-20 peer ID conventions。
- BEP-23 compact peer lists。
- BEP-27 private torrents。
- BEP-29 uTP。
- BEP-32 IPv6 DHT。
- BEP-47 padding files / extended file attributes。
- BEP-53 magnet file selection。

非 UI 功能还包括：

- sequential downloading；README 表述为默认且唯一模式。
- 已有文件续传。
- 基于正则的选择性下载。
- pause / unpause / delete。
- server persistence。
- streaming 和 seeking。
- UPnP 端口映射。
- UPnP MediaServer。
- IPv6。
- SOCKS5 / SOCKS5H 代理。
- fastresume。
- 下载/上传限速。
- Prometheus metrics。
- per-peer stats endpoint。
- 创建 torrent 并开始做种。

这说明 rqbit 已经是可用的 BT 引擎，不是从零开始的实验实现。差距主要集中在**协议覆盖广度、长期稳定性、存储抽象、恢复完整性、可观测性、跨平台验证和边缘行为**，而不是基础下载链路。

## 四、距离 libtorrent 类成熟引擎还需要的工作

以下按不考虑 UI 的前提拆分。API、daemon、测试、部署、移动端构建仍计入。

### 4.1 P0：生产可靠性和数据安全

这是比增加新协议更优先的工作。

#### A. 持久化写入必须具备故障安全

上游 `TODO.md` 仍列出：

- 磁盘满时 `.rqbit-session.json` 可能变成 0 字节。
- 总上传字节数尚未持久化。

需要实现：

- 临时文件写入 + `fsync` + 原子 rename。
- 磁盘满、权限错误、rename 失败时保留上一份有效状态。
- session/resume data 版本号、校验和、损坏回退。
- 多份 generation 文件或 journal，防止断电时只剩坏文件。
- 上传总量和做种基线的持久化。
- 恢复时区分“状态文件坏了”和“torrent 数据坏了”。

FluxDown 已经自行依赖 librqbit fastresume，并在 DB 中持久化更多做种字段；rqbit 后端若继续承担这些职责，必须与 FluxDown 的 DB 事务和状态事件保持一致。

#### B. 删除、暂停、恢复和关闭必须做长期 soak

需要系统性覆盖：

- metadata 未完成时 delete。
- add 与 pause 同时发生。
- add 与 shutdown 同时发生。
- session 重建时仍有后台 peer task。
- 删除后旧 peer / DHT / tracker task 不再持有资源。
- pause 后所有 socket、文件句柄、后台 task 是否真的释放。
- 恢复时 piece 状态与磁盘内容不一致。
- 同一 info-hash 多次 add 的幂等性。

当前 FluxDown 的 `pending_deletes`、`inflight_adds`、`pause_epochs`、句柄缓存和 seeding 生命周期，正是为这些边界情况增加的补偿层。迁移或升级 rqbit 时不能只验证正常下载。

#### C. 恢复数据要从“能恢复”提升到“可证明恢复

需要增加：

- 断电恢复测试。
- 强制 kill -9 后恢复测试。
- 进程在写 resume data 中途被杀测试。
- 数据文件被外部截断、移动、改名、只读、替换后的行为测试。
- 全量校验与 fastresume 采样校验的明确边界。
- 大文件、稀疏文件、部分文件和跨文件 piece 的恢复测试。
- 旧版本 resume data 迁移和降级策略。

### 4.2 P1：文件选择、piece 调度和 storage 能力

这是 rqbit 与 libtorrent 类成熟引擎的主要功能差距之一。

#### A. 完整文件优先级模型

上游 `TODO.md` 仍列有：

- file priority。
- 按选中文件起止位置计算 piece priority。

当前 README 只承诺：

- sequential downloading，且是默认和唯一模式。
- 正则选择文件。
- BEP-53 指定文件索引。

成熟引擎通常需要：

- 每文件 priority：dont download / low / normal / high。
- 每 piece priority。
- 选定文件的首尾 piece 处理。
- streaming / sequential priority 与普通下载 priority 并存。
- 动态改变 priority 不破坏当前 peer request。
- 仅下载部分文件时，跨文件边界 piece 保证正确。
- padding file 不被错误暴露或错误计入用户进度。

对 FluxDown 来说，这部分还要与 `bt_partfile`、最终重命名和做种 storage 结合，不能只在 chunk tracker 增加一个整数数组。

#### B. Storage 抽象需要从“内部实现”提升为稳定契约

libtorrent 官方文档明确提供 Custom Storage、disk interface、disk observer、buffer allocator 和 storage holder 等扩展点。

rqbit 当前已有 storage 代码和 wrapper，但要达到成熟后端级别，还需要：

- 明确的文件布局 / piece offset / padding 模型。
- 自定义 storage 的完整 trait contract。
- 读、写、truncate、flush、move、rename、reopen 的一致语义。
- sparse file / preallocation 策略。
- 文件句柄池与最大打开文件数。
- 大文件 offset 的跨平台无损处理。
- 磁盘错误分类，而不是统一转成 torrent error。
- 同一 piece 的并发读写、校验与缓存失效规则。
- 外部重命名后的恢复和做种。
- 能供 FluxDown parts sidecar 使用的稳定 storage 接口。

#### C. 磁盘 I/O 调度和缓存

libtorrent 类成熟引擎的优势不只是网络协议，也包括磁盘数据面。需要评估：

- read/write cache。
- piece cache。
- write coalescing。
- 磁盘队列公平性。
- 多 torrent 间 I/O 竞争。
- streaming 任务对普通任务的优先级。
- `fsync` 策略。
- 低速机械盘和网络盘行为。
- 取消时的写入排空。
- 句柄关闭和重新打开的成本。

上游 TODO 仍提到 reopening files、读写切换优化和文件遍历抽象，说明这些方面还没有完全收敛。

### 4.3 P1：网络和协议覆盖

#### A. DHT 仍有长期维护项

上游 `TODO.md` 中未完成或标记为部分完成的项目包括：

- announced peer cache 清理。
- 只向足够接近 info-hash 的节点发送 token。
- 15 分钟 bucket refresh 仍存在重复查询缺陷。
- announced peer 持久化只有 partial 状态。
- 完成搜索后 DHT 节点维护和缓存收敛。

需要做：

- RFC/BEP 驱动的 routing table refresh。
- token 生命周期、作用域和滥用防护。
- announced peer TTL / LRU / 持久化策略。
- query 并发上限和 per-node backoff。
- bad node 评分、封禁和恢复。
- UDP socket 错误隔离。
- IPv4/IPv6 双栈的独立健康状态。
- DHT 状态文件原子持久化。
- DHT 大规模 torrent / 多任务压力测试。

#### B. Tracker 和 peer discovery 的完整性

rqbit 已支持 UDP tracker、multitracker、PEX 和 LSD，但成熟化仍需要验证：

- announce 周期和失败退避是否符合不同 tracker 行为。
- tracker response 异常、超大 peer list、重复 peer 的处理。
- HTTP/HTTPS tracker 的代理和 TLS 失败分类。
- private torrent 是否严格禁止 DHT / PEX / LSD。
- tracker tier 顺序和备用 tracker 迁移。
- IPv6 peer 地址和 compact response 的边界。
- tracker、DHT、PEX、LSD 来源的去重和优先级。

#### C. Web seed / HTTP seed 能力

上游 README 的已支持 BEP 列表没有列出 BEP-17/BEP-19 web seed。不能仅因为 HTTP API 能接收 `.torrent` URL，就认为已经具备 web seed 下载能力。

如果目标是对齐成熟引擎，需要明确是否实现：

- HTTP seed。
- GetRight-style web seed。
- web seed 与 P2P peer 的调度关系。
- HTTP Range、校验和、重试、代理和限速。
- web seed 对私有 torrent 的约束。
- web seed 与 piece picker 的公平性。

这是一个独立的功能包，不应混入普通 HTTP 下载器后简单复用。

#### D. Peer protocol 和连接管理

上游近期仍在修复握手读取、incoming listener、peer capability 和 request window，说明该区域仍是长期风险面。

需要继续补齐：

- 所有握手和 extension message 的渐进式读取。
- malformed length / oversized message / slowloris 防护。
- peer request window 的动态调节。
- peer 连接超时、读写超时、keepalive 策略。
- half-open connection 限流。
- incoming handshake 并发上限。
- connection slot 与每 torrent / 全局公平性。
- peer choking / optimistic unchoke 的行为验证。
- MSE/加密握手策略和明文兼容策略。
- NAT、IPv4、IPv6、uTP、TCP 的独立回归矩阵。

上游 TODO 仍列有 keepalive 策略问题；即使不是新功能，也应作为连接稳定性工作处理。

#### E. Proxy 和网络策略

上游 README 明确列出 SOCKS5 / SOCKS5H，但没有将“完整 BT 网络代理策略”作为已完成能力来描述。

若要达到 FluxDown 当前的网络策略目标，需要明确支持矩阵：

|流量|直连|HTTP proxy|SOCKS4/5|SOCKS5H|代理认证|
|---|---|---|---|---|---|
|tracker HTTP/HTTPS|需定义|需定义|需定义|需定义|需定义|
|DHT UDP|通常不经 HTTP/SOCKS|通常不支持|需定义|需定义|需定义|
|peer TCP|需定义|需定义|需定义|需定义|需定义|
|peer uTP|需定义|需定义|需定义|需定义|需定义|
|web seed|需定义|需定义|需定义|需定义|需定义|

不能把不支持的路径静默回退直连，否则会违反用户的代理配置语义。

### 4.4 P1：做种、上传统计和分享策略

rqbit README 支持创建 torrent 并开始做种，也有上传限速；但上游 TODO 仍列出总上传字节持久化。

如果要达到 FluxDown 当前的做种管理能力，后端还要支持：

- cumulative uploaded bytes 持久化。
- session 重启后的上传基线恢复。
- per-torrent upload rate limit。
- 全局、队列、任务三级限速叠加。
- ratio limit。
- post-completion ratio limit。
- seeding time limit。
- inactive seeding time limit。
- max active seeder FIFO 队列。
- pause / promote / demote 的状态一致性。
- stop、delete、delete-files 三种达到限制后的动作。
- 重新挂载已完成但被重命名的文件。
- 部分文件选择后的正确做种。
- 做种停止原因的稳定错误码。

其中很多可以由 FluxDown 自己在 `bt_seeding.rs` 实现，但后端必须提供可靠的：

- uploaded snapshot。
- current upload speed。
- torrent finished state。
- pause/resume。
- local data verification。
- file priorities / storage read。

### 4.5 P1：统计和可观测性

上游 README 已有 global metrics、torrent stats 和 per-peer stats endpoint，但 TODO 仍列有 per-file stats、per-peer stats partial 和上传持久化。

需要补齐：

- per-file downloaded / uploaded / wanted / remaining。
- per-piece state和校验耗时。
- per-peer：IP、client、来源、速度、请求窗口、choke 状态、失败原因、已拥有 piece 比例。
- tracker/DHT/PEX/LSD 来源归因。
- disk I/O latency、cache hit、flush latency。
- session 级连接槽、half-open、handshake、timeout 计数。
- metadata resolution latency。
- resume/recheck 耗时。
- error category 和可重试性。

FluxDown 当前进度事件已经包括上传速度、上传累计值和做种状态；如果后端统计粒度不足，daemon 只能用错误的近似值补齐。

### 4.6 P1：安全和不可信输入防护

rqbit 已经在持续修复 bencode recursion depth、info-hash 识别、握手读取等问题。要达到成熟引擎级别，还需要把安全工作变成持续工程：

- bencode 深度、字典键数、字符串长度、总内存预算。
- torrent 文件总大小、文件数、路径深度和路径长度限制。
- 路径穿越、绝对路径、Windows 保留名和 Unicode 归一化。
- malformed peer message fuzzing。
- malformed tracker response fuzzing。
- DHT packet fuzzing。
- magnet query 解析 fuzzing。
- metadata extension 的内存和超时预算。
- peer 发送超大 bitfield / piece message 的拒绝策略。
- 文件系统错误不能误判成“piece 已完成”。
- 密盘、只读目录、权限变化和网络盘断开后的 fail-closed 行为。

### 4.7 P2：测试和互操作性

这是从“功能可用”到“成熟引擎”的最大长期成本。

需要建立固定的 backend contract 和外部互操作矩阵：

#### 外部客户端

- libtorrent 2.x。
- qBittorrent。
- Transmission。
- aria2。
- 其他主流客户端生成的 torrent。

#### 网络场景

- 仅 tracker。
- 仅 DHT。
- 仅 PEX。
- 无 IPv6。
- IPv6-only peer。
- NAT 无端口映射。
- UPnP 成功 / 失败 / 超时。
- UDP 被阻断。
- tracker TLS 错误。
- MSE 强制 / 优先 / 禁用。
- SOCKS5 / SOCKS5H。
- 高延迟和丢包。
- 多 tracker 返回重复 peer。

#### 磁盘场景

- 2 GiB 以上文件。
- 多 TiB torrent。
- 稀疏文件。
- 磁盘满。
- 目录只读。
- 文件外部删除 / 改名 / 替换。
- 断电和 kill -9。
- 多文件跨 piece 边界。
- 部分选择完成后做种。
- 大量小文件。
- 网络文件系统。

#### 压力和长期运行

- 数百个 torrent。
- 数万 peer。
- 高速下载和高速上传。
- 长时间 DHT 运行。
- 大量 metadata 同时解析。
- 频繁 pause/resume/delete。
- session 重建。
- 内存泄漏、FD 泄漏、task 泄漏、peer 泄漏。

测试形式应包括：

- 单元测试。
- property tests。
- fuzz tests。
- 多客户端互操作测试。
- 网络仿真测试。
- 磁盘故障注入。
- 长时间 soak。
- 性能基准。
- crash/restart regression。

### 4.8 P2：跨平台和发布工程

上游 rqbit 的公开发行重点是桌面和服务器：README 列出 Windows/macOS release、Linux 手工构建、Homebrew、Docker 和 systemd。

FluxDown 的目标更广，还包括 Android/iOS。若要将 rqbit 核心提升到 FluxDown 全平台生产后端，需要：

- Windows MSVC / socket / file I/O 回归。
- macOS 文件句柄、sandbox、IPv6 和 notarization。
- Linux glibc / musl / NAS。
- Android NDK、ABI、静态库、TLS 和 socket 行为。
- iOS 静态库、arm64 device、simulator、Xcode 构建。
- 统一数据目录和 resume data 路径。
- feature flags，避免移动端带入不需要的服务器能力。
- CI matrix、交叉编译、缓存和产物审计。
- 崩溃日志、核心转储和版本回滚策略。

这部分不属于 UI，但会显著影响总工程量。

## 五、fork/upstream 的具体工作顺序

### 5.1 先同步上游，不要基于滞后的 fork 开新功能

建议先做：

1. 固定上游 commit 和版本。
2. 对 `lingdiansr` 与上游的 19 个提交逐个审计。
3. 对 FluxDown 实际使用的 `zerx-lab/rqbit` revision 做三方 diff：
   - 当前 FluxDown pin。
   - `lingdiansr` 对应祖先。
   - `ikatson` 当前主线。
4. 保留 FluxDown 必需的 fork patch：
   - MSE。
   - Android/iOS socket。
   - fastresume 行为。
   - storage wrapper。
   - 大文件和 Windows 修复。
5. 为每个上游升级提交增加对应的 FluxDown 回归测试。

### 5.2 先补 contract，不先加第二套后端

推荐内部边界：

```text
FluxDown-owned DTO
  ├── TorrentMetadata
  ├── BtFile
  ├── BtStats
  ├── BtAlert
  ├── BtResumeState
  └── BtStorageLayout

BtBackend / BtSession / BtHandle
  ├── rqbit backend
  └── future libtorrent backend
```

业务层不应暴露：

- `librqbit::Session`
- `librqbit::ManagedTorrent`
- libtorrent C++ handle
- 后端私有 resume data 类型
- 后端私有 storage 类型

### 5.3 按优先级实现

|优先级|工作|原因|
|---|---|---|
|P0|resume/session 原子持久化|防止断电和磁盘满损坏任务|
|P0|pause/delete/shutdown soak|当前后端最容易产生资源泄漏和僵尸下载|
|P0|大文件、部分文件、跨 piece 校验|直接影响用户数据正确性|
|P0|DHT、peer、tracker fuzzing|不可信网络输入是常态|
|P1|file/piece priority|从顺序下载提升到成熟调度模型|
|P1|web seed|补齐成熟客户端常用的数据源|
|P1|完整 proxy 矩阵|与 FluxDown 网络策略一致|
|P1|做种统计和 share limits|满足长期做种管理|
|P1|per-file/per-peer/disk metrics|支持诊断和性能调优|
|P2|跨客户端互操作|验证真实生态兼容性|
|P2|长时间 soak 和故障注入|发现常规测试看不到的问题|
|P2|Android/iOS 全平台发布|最后处理，避免过早锁定 C++/FFI 架构|

## 六、工程量结论（不含 UI）

### 6.1 只使用上游 rqbit，补 FluxDown 所需能力

粗估：**8–16 人周**。

适用范围：

- 同步上游主线。
- 保留 FluxDown 自有 DB、daemon、做种策略。
- 补齐 resume、storage、partial files、统计、proxy 和故障测试。
- 不新增 libtorrent。
- 不追求 libtorrent 的所有可选协议特性。

其中：

- 上游同步和适配：约 1–2 人周。
- P0 稳定性与持久化：约 3–5 人周。
- storage / priority / seeding：约 2–5 人周。
- 互操作、fuzz、soak：约 2–4 人周。

### 6.2 目标是达到 libtorrent 类生产成熟度

粗估：**20–40+ 人周**，不含 UI。

这里不是说要重写 libtorrent 的所有能力，而是需要覆盖：

- 网络协议和 DHT 长期稳定性。
- storage 和 resume 数据可靠性。
- file/piece priority。
- web seed / proxy 等实际能力。
- 做种和上传统计。
- 互操作性、fuzz、故障注入和 soak。
- Windows/macOS/Linux/NAS；若含 Android/iOS，取上界或超出上界。

### 6.3 直接引入 libtorrent

仍需在上述业务工作之外增加：

- Rust/C++ FFI 或 helper RPC。
- C++ 构建、依赖、许可证和打包。
- libtorrent storage/resume 语义适配。
- 后端 contract 和双实现测试。
- 旧 rqbit 任务迁移或完整 recheck。

因此直接引入 libtorrent的总量通常不会低于“把 rqbit 补到成熟”的路线，区别是：

- libtorrent 可以复用成熟的数据面能力。
- FluxDown 必须承担 FFI、ABI、平台和迁移成本。
- storage、事件、错误和恢复语义仍需要自行适配。

## 七、最终判断

### lingdiansr/rqbit

当前更像上游 rqbit 的滞后 fork，不是成熟度明显高于上游的独立实现。适合作为某个固定版本的依赖来源，不适合作为“比上游更成熟”的默认判断。

### ikatson/rqbit 上游

已经具备可用 BT 客户端和库的核心能力，适合继续演进；但距离 libtorrent 类成熟引擎，主要还缺长期可靠性、storage/resume 完整性、priority 模型、网络边界、做种统计、互操作测试和跨平台验证。

### FluxDown

短期最合理路线：

```text
上游同步/三方审计
  → backend contract
  → rqbit contract tests 全绿
  → P0 数据安全和生命周期
  → P1 storage / priority / seeding / proxy
  → P2 互操作、fuzz、soak
  → 最后评估 libtorrent
```

如果目标只是“下载稳定、支持磁力、部分文件、做种和恢复”，继续投资 rqbit 的边际成本低于接入 libtorrent。

如果目标是“复用 libtorrent 的完整生态和长期数据面能力”，应先完成 backend contract 和 contract tests，再把 libtorrent作为第二实现或替换实现接入，而不是直接在 `bt_downloader.rs` 中替换具体类型。

## 外部事实来源

- lingdiansr/rqbit：<https://github.com/lingdiansr/rqbit>
- ikatson/rqbit：<https://github.com/ikatson/rqbit>
- fork/upstream compare：<https://github.com/lingdiansr/rqbit/compare/main...ikatson:main>
- 上游 README：<https://raw.githubusercontent.com/ikatson/rqbit/main/README.md>
- 上游 TODO：<https://raw.githubusercontent.com/ikatson/rqbit/main/TODO.md>
- libtorrent reference：<https://www.libtorrent.org/reference.html>

# 八、BT 速度问题与详细监测实施建议

## 8.1 诊断结论

BT 速度不稳定/不高很可能与 peer 发现和连接有关，但当前证据不足以认定这是唯一主因。现有 `native/engine/src/bt_downloader.rs` 已具备共享 `librqbit::Session`、DHT、公共/订阅 tracker、IPv4/IPv6 监听、UPnP、MSE、fast-resume 和较积极的连接参数。因此第一优先级不是盲目增加 tracker 或连接速率，而是补齐网络阶段观测。

当前连接参数包括：

- connect timeout：10 秒。
- handshake timeout：10 秒。
- read/write timeout：20 秒。
- 出站连接尝试上限：30 次/秒。
- peer 失败退避：5 秒起步，最长 600 秒。

底层已有 `peer_stats.live`、`connecting`、`queued`、`seen`、`dead` 统计，但当前 `TaskRuntime` 主要只向上层暴露 `connected_peers`，无法区分：

```text
discovered → queued → connecting → handshake → live
        → choked/requested → data peer → failed/dead
```

因此目前不能仅凭速度判断是发现失败、连接失败、peer 不上传、piece 调度问题还是磁盘瓶颈。

另一个已确认缺口：BT Session 的 `ConnectionOptions.proxy_url` 当前为 `None`。通用 HTTP 代理模型不等于 BT peer、tracker、DHT 流量已经支持代理。

## 8.2 需要记录的监测指标

第一阶段应在引擎内部建立 BT 专用快照，不必立即扩展所有 UI：

```text
peers_seen
peers_queued
peers_connecting
peers_live
peers_dead
handshakes_succeeded
handshakes_failed
active_data_peers
metadata_peers
download_speed
upload_speed
```

来源应尽量拆分为：

```text
tracker_http
tracker_udp
dht
pex
lsd
incoming
outgoing
```

连接失败至少区分：

```text
dns
connect_timeout
connection_refused
tls_or_proxy
handshake_timeout
protocol_error
peer_choked
disk_or_storage
```

指标应按固定时间窗口记录增量，而不是只记录累计值。例如：

```text
过去 10 秒:
seen +42
connecting +18
handshake success +3
handshake failed +15
live = 2
active data peers = 1
download = 120 KiB/s
```

同时记录：

- piece request/in-flight 数量。
- 实际产生数据的 peer 数量。
- piece 写入延迟。
- hash 校验 CPU 时间。
- staging/storage 写入吞吐。
- Session、任务和队列级限速。
- 入站与出站连接数量。

## 8.3 现象到根因的初步映射

|现象|优先怀疑|关键指标|
|---|---|---|
|`seen` 低，`connecting` 也低|tracker/DHT/PEX 发现失败|announce 成功率、DHT 节点数、来源 peer 数|
|`seen` 高，`connecting` 高，`live` 低|NAT、防火墙、端口或握手失败|connect/handshake 成功率、失败原因|
|`live` 高但速度接近 0|peer choke、无可用 piece 或调度问题|unchoked 数、active data peers、request 数|
|`live` 稳定但速度周期性归零|request window、piece 调度或磁盘阻塞|in-flight piece、写入延迟、校验耗时|
|只有 magnet 慢|metadata peer 发现/连接失败|metadata peer 数、metadata 请求响应时间|
|无 tracker magnet 特别慢|DHT bootstrap、UDP、DHT refresh|DHT 查询成功率、IPv4/IPv6 状态|
|多任务时速度骤降|Session 限速、连接槽或磁盘竞争|全局连接数、任务数、磁盘队列、限速值|

## 8.4 实施优先级

### P0：观测面

1. 暴露 `seen/queued/connecting/live/dead`。
2. 记录 tracker、DHT、PEX、LSD、incoming、outgoing 来源。
3. 记录 connect、handshake、timeout、拒绝和协议错误分类。
4. 区分 `live peer` 与实际提供数据的 `active data peer`。
5. 记录 request window、in-flight piece 和磁盘写入/校验耗时。
6. 形成 1 秒或 5 秒窗口的速度与状态快照。

### P1：网络路径

1. 验证监听端口的外部可达性，而不是只验证本地 bind。
2. 记录 UPnP 映射结果、IPv4/IPv6 监听状态和入站连接数。
3. 接入 BT 专用代理配置，并区分 tracker、peer、DHT 的代理语义。
4. 建立 tracker 健康度、延迟、返回 peer 数和失败退避。
5. 检查 DHT 路由表持久化、refresh、IPv4/IPv6 与 UDP 可达性。
6. 基于握手成功率做自适应连接速率和 peer backoff。

不要在没有指标前直接提高当前 30 次/秒的连接速率。过高的连接尝试可能耗尽 NAT/路由器连接表、增加无效 peer 竞争和移动端耗电。

### P2：调度和本地 I/O

1. 完善 file priority、piece priority 和 request window。
2. 区分 sequential/streaming 调度与普通下载调度。
3. 增加稀缺 piece 和跨文件 piece 的回归测试。
4. 测量随机写、hash 校验、staging 和磁盘队列，避免把本地瓶颈误判为网络瓶颈。
5. 增加磁盘满、网络切换、强杀恢复和长期多任务 soak 测试。

## 8.5 推荐验证矩阵

至少准备以下三类可重复样本：

1. tracker 和 DHT 均可用的公开 torrent。
2. 主要依赖 DHT 的 magnet。
3. tracker 可用、DHT 禁用的 torrent。

每个样本记录：

- 首个 peer 发现时间。
- 首个握手成功时间。
- 达到 1、5、10 个 active data peer 的时间。
- 30 秒和 5 分钟平均速度。
- 速度 P50/P95/P99。
- 连接失败率和重连次数。
- 磁盘写入吞吐与延迟。

推荐实施顺序：

```text
监测快照
  → 可重复基准
  → 代理与入站可达性
  → tracker/DHT 健康度
  → 自适应连接与退避
  → piece/request 调度
  → 磁盘、恢复和长期 soak
```

在监测数据证明当前数据面不足以前，不建议直接切换 libtorrent。应先审计当前固定的 `zerx-lab/rqbit` revision，并优先吸收与 listener、handshake、DHT、request window、连接统计相关的上游修复。

## 8.6 当前实现切片

当前 `feature/bt-monitoring` 分支已完成第一切片：

- `fluxdown_engine::transfer_activity::TaskRuntime` 增加可选 `BtPeerStats`。
- BT 进度采样将 librqbit 的 `seen/queued/connecting/live/dead` 快照传入运行时事件。
- `fluxdown_protocol::TaskRuntimeDto` 增加可选 `btPeerStats`，旧 wire 缺字段时回退为 `None`。
- daemon 协议清理非活跃任务时同步清零 BT peer 快照。
- Web TypeScript DTO 与 reducer 已同步，非 BT runtime 保持 `null/undefined`。
- 暂未添加 `active_data_peers`、unchoke、request window、tracker/DHT 来源等指标；当前固定的 librqbit 统计接口未提供这些字段，不能用 `live` 冒充有效数据 peer。

验证覆盖：

- Rust engine/protocol/daemon/agent/GPUI 桌面目标编译检查。
- engine-protocol wire 映射、camelCase 序列化、旧 wire 兼容测试。
- Web runtime reducer 清理测试。
## 九、当前运行性能诊断与代理影响（2026-10-10）

### 9.1 当前任务证据

诊断对象：`ea9d9c4a-32ab-48e9-ab07-733e6935c750`，Ubuntu 26.04.1 Desktop ISO，大小约 6.48 GiB。

运行日志位置：`~/.local/share/fluxdown/logs/fluxdown_2026-10-10.log`。

观察到的实时状态：

| 指标 | 观察值 | 判断 |
|---|---:|---|
| `live` peer | 126–128 | 长时间固定在 128 |
| `connecting` | 0–2 | 连接槽位基本没有空闲 |
| `queued` | 1,341–1,364 | 大量候选 peer 排队 |
| `seen` | 1,488–1,493 | peer 发现并未完全失败 |
| `dead` | 1–2 | 当前存活池不是大量死连接 |
| 实时下载速度 | 3.4–4.4 MB/s | 有吞吐，但波动明显 |
| 配置下载限速 | 0 | 未设置 FluxDown 全局下载限速 |
| 配置代理主机 | 空 | 未配置 FluxDown 手工代理 |

日志时间序列显示：

```text
11:57:08  live=49  connecting=79  queued=192   seen=381
12:02:40  live=127 connecting=1   queued=977   seen=1161
12:08:12  live=128 connecting=0   queued=1418  seen=1546
```

这说明主要现象不是“完全找不到 peer”，而是候选 peer 持续增加，活动连接达到固定上限后，大量 peer 进入排队。

### 9.2 当前最可能的根因

#### P0（历史分析，已修复）：rqbit 每个 torrent 曾受 128 个 peer 连接上限约束

`native/engine/src/bt_downloader.rs` 的 `AddTorrentOptions` 没有设置 `peer_limit`。当前使用的 rqbit revision 在 `crates/librqbit/src/torrent_state/live/mod.rs` 中对缺省值使用：

```rust
peer_limit.unwrap_or(128)
```

因此当前任务的 `live=128` 与实现默认值完全吻合。`queued` 不断上升且 `connecting` 接近零，进一步支持“连接槽位已满”的判断。

> **后续修正（2026-10-12）**：本 P0 已在代码中解决——`AddTorrentOptions.peer_limit` 现由
> `bt_peer_limit`（128..512，默认 256）显式下发，`SessionOptions.peer_limit` 也取同一配置值；
> 上游 #825 的实测（128 → 512：稳态 1.1 → 12.6 MB/s）与本节判断一致。下文 §10.2 记录的是
> 当时第一阶段的范围。

这不等于提高连接数一定会提高速度；128 个连接可能包含慢 peer 或被 choke 的 peer。但这是当前最直接、最可验证的限制，应先做 `128/256/512` 的同种子 A/B。

#### P0：DHT bootstrap 持续失败

日志反复出现：

```text
error in bootstrap: no successful lookups
router.utorrent.com:6881
router.bittorrent.com:6881
dht.libtorrent.org:25401
```

同时 tracker subscription 成功获取了 70 和 20 行 tracker，合并后有 83 个 tracker，且任务仍能发现大量 peer。因此当前任务大概率依赖 tracker/PEX 获取 peer，DHT 失效会降低来源多样性和后续补充能力，但不是“完全没有速度”的唯一原因。

启动时还观察到：

```text
BT TCP listen: [::]:6881
DHT listen: [::]:56994
UPnP forwarder: port=6881
```

DHT 使用独立随机 UDP 端口，而 UPnP 日志只显示 6881，这是需要重点验证的端口映射问题。也可能是 UDP、IPv6、NAT 或外部防火墙导致；现有日志能确认 DHT 失败，但不能单独证明具体网络根因。

#### P1：已连接 peer 的有效数据质量未知

`live=128` 只表示连接存活，不表示 peer 正在提供数据。当前还没有：

- per-peer 下载速率；
- unchoked/interested 数量；
- 实际产生数据的 active peer 数；
- piece request/in-flight 数；
- peer 来源和握手成功率。

因此不能仅凭 `live` 判断 piece 调度是否有问题。当前速度在 0.9–9.0 MB/s 之间波动，最近采样约 3.4–4.4 MB/s，说明连接池中 peer 质量或可用 piece 供给不稳定。

#### 当前证据不支持的原因

- `speed_limit_bytes=0`，session 日志为 `speed_limit=0 B/s`：不是 FluxDown 全局限速。
- 当前只有一个 active task：不是多任务抢占。
- `fetched` 与 `checked` 的差距约几十 MiB，未见 checksum/storage 错误：暂不像磁盘或校验成为主瓶颈。
- 文件选择和初始检查只造成约两秒启动延迟：不是启动阶段拖慢持续速度。

### 9.3 libtorrent 是否有同类 `peer_limit`

有，但命名和层级不同。以 libtorrent 官方 2.1.0 文档为准：

| libtorrent 配置 | 作用 | 默认值 |
|---|---|---:|
| `settings_pack::connections_limit` | session 全局连接上限 | 200 |
| `torrent_handle::set_max_connections()` | 单个 torrent 连接上限 | unlimited（`-1` 表示 unlimited） |
| `settings_pack::connections_slack` | 全局上限之外允许的入站连接余量 | 10 |
| `settings_pack::unchoke_slots_limit` | 全局上传 unchoke 槽位 | 8 |

官方文档：

- <https://libtorrent.org/reference-Settings.html>
- <https://libtorrent.org/reference-Torrent_Handle.html>

因此不能把 libtorrent 的默认 `connections_limit=200` 直接等同于“每个 torrent 默认 200”。libtorrent 的有效连接数通常受：

```text
min(session connections_limit,
    torrent_handle max_connections,
    网络/系统资源限制)
```

影响；单 torrent 的 `max_connections` 默认是 unlimited，但 session 全局默认仍是 200。`unchoke_slots_limit` 主要影响上传，不应直接当成下载连接上限。

如果 FluxDown 迁移到 libtorrent，合理的初始映射不是盲目设置 unlimited，而是：

```text
session connections_limit = 400 或 600
torrent max_connections   = 256
```

再通过同种子基准测试调整。当前 FluxDown 只有一个活动 torrent 时，libtorrent 的全局 200 仍可能成为比单 torrent limit 更早触发的限制。

### 9.4 开启 Clash 的影响

当前 FluxDown 的 BT 路径明确使用：

```rust
ConnectionOptions {
    proxy_url: None,
    ...
}
```

并且 `download_manager.rs` 的自动代理决策明确排除了 BT URL。因此：

| Clash 工作方式 | 对当前 BT 的影响 |
|---|---|
| 只启动 Clash，未启用 TUN/Redir/TProxy，FluxDown 未配置代理 | 通常为 0；BT 仍然直连 |
| Clash 仅提供 HTTP/SOCKS 端口，但 FluxDown 未填入该端口 | 通常为 0 |
| Clash TUN 模式 | 会影响 BT 的 TCP、UDP、DHT、tracker 和 peer 连接 |
| Clash Redir/TProxy 模式 | 会影响被重定向的 TCP/UDP；具体取决于规则 |
| Clash 只代理 TCP、UDP 未正确转发 | DHT 和 UDP tracker 可能失败，peer 来源下降 |
| Clash 代理所有 BT peer | 可能增加 RTT、握手失败率和连接建立时间；入站 peer 通常也不可达 |

所以“开 Clash 会影响多少”没有固定百分比。对当前实现，影响范围是：

- **普通系统代理模式：接近 0**，因为 BT 没有使用 `ProxyConfig` 的 HTTP/SOCKS 代理；
- **TUN/透明代理模式：可能很大**，尤其是 UDP/DHT 和大量 peer TCP 连接；
- **当前已观察到的 DHT bootstrap 失败可能与 Clash 的 TUN/UDP/IPv6 路径有关，但也可能是 NAT/防火墙或当前 DHT 端口配置问题**。

最可靠的判定方法是使用同一个 info-hash 做两次 5 分钟 A/B：

```text
Clash 完全退出 / TUN 关闭
Clash TUN 或透明代理开启
```

保持 torrent、peer_limit、保存目录和其他配置不变，比较：

- `live / connecting / queued`；
- DHT bootstrap 成功率和 DHT 节点数；
- tracker announce 成功率；
- 首个 peer 和首个有效 piece 的时间；
- 30 秒和 5 分钟平均下载速度。

### 9.5 推荐下一步

1. 先显式设置 rqbit `peer_limit`，做 `128/256/512` A/B。
2. 记录 DHT 实际 UDP 监听端口、UPnP 映射结果和 IPv4/IPv6 bootstrap 结果。
3. 暂时关闭 Clash TUN/透明代理，仅保留 Clash 进程，重复同一个 torrent 测试。
4. 增加 per-peer 速率、unchoke、request window 和 tracker/DHT/PEX 来源统计。
5. 只有在提高连接上限并恢复 DHT 后仍明显低于基准，才继续定位 piece 调度或 storage。
# 十、Pinned `librqbit` 参数总表与第一阶段调优决策

本章以 FluxDown 根 `Cargo.toml` 的 `[patch.crates-io]` 为版本边界：`zerx-lab/rqbit` revision `00374a6d772d4123f079f5adf6a1fe013e363381`（2026-10-12 随上游 #825 更新，此前为 `d9e79c939f223679612cd0b06bd00ed49bd2eff8`）。上游 `ikatson/rqbit` 只用于 API 语义对照；上游默认值不覆盖 pinned fork 的运行结论。`peer_limit` 是每 torrent 上限，也是 `SessionOptions` 的会话缺省，但不是 session 全局连接上限。

## 10.1 版本、参数作用域与生效规则

|参数|来源 API / FluxDown 位置|作用域|单位；当前 pinned 值 / 上游默认 / FluxDown 覆盖|持久化与 wire/UI|热更新|收益、风险、观测|决策与原因|
|---|---|---|---|---|---|---|---|
|`bt_enabled`|`BtConfig.enabled`; `daemon_config.rs`|session / policy|bool；true / true / true|DB `config`; daemon/GPUI/Web/mobile catalog|session 重建|关闭 BT 释放监听与 DHT；误关会拒绝 BT 任务|设置；用户协议偏好|
|`bt_enable_dht`|`BtConfig.enable_dht`; `SessionOptions.dht`|DHT|bool；true / true / true|DB + daemon config|session 重建|增加无 tracker 发现；UDP/NAT/代理失败需单独诊断|设置；不能自动覆盖网络策略|
|`bt_enable_upnp`|`BtConfig.enable_upnp`; `ListenerOptions.enable_upnp_port_forwarding`|listener/NAT|bool；true / false(库默认关闭) / true|DB + daemon config|session 重建|改善入站可达性；路由器映射失败不能等同 DHT 失败|设置；环境约束|
|`bt_port_start` / `bt_port_end`|`SharedBtSession::new`|listener|port；6881..6891 / library listener default / 6881..6891|DB + daemon config|session 重建|端口范围内探测可用端口；无限扩展会违反用户约束|设置；当前实现只在范围内尝试|
|`bt_mse_mode`|`SessionOptions.mse_mode`; `BtMseMode`|peer protocol|enum；enabled / library default / enabled|DB + daemon config|session 重建|兼容性与加密策略；forced 可能减少 peer|设置；协议安全策略不自动改|
|`bt_custom_trackers`|`SharedBtSession::new` tracker merge|tracker|newline URLs；用户值 / library none / 用户值+内置|DB + daemon config|session 重建|扩大发现；坏 tracker 增加 announce 噪声|设置；规范化去重但不静默删除|
|`bt_tracker_sub_enabled` / `_urls`|daemon config + subscription merge|tracker|bool/text；true/用户 URL / library none / 用户值|DB + daemon config|下次 session；缓存后台刷新|社区列表增加来源；失败不应删用户 URL|设置；缓存是后台维护状态|
|`bt_tracker_sub_cache` / `_updated_at`|daemon read-only projection|tracker|text/timestamp；缓存 / library none / cache|DB read-only + snapshot|刷新完成后|可诊断订阅健康；缓存不可伪装用户输入|固定内部投影；只读|
|`bt_seed_enabled` / `bt_auto_reseed`|`download_manager` DB reads|seeding|bool；true / library absent / true|DB config + daemon catalog|立即（生命周期读取）|做种/重挂载行为；错误会占用磁盘和上传|设置；不由吞吐控制器改变|
|`bt_seed_max_active`|`SeedingManager::set_cap`|seeding|count；0 unlimited / library absent / 0|DB + daemon config|立即|限制上传槽与资源；过低降低做种服务|设置；已有安全 live path|
|`bt_seed_ratio_limit`, `_post_ratio_limit`, `_time_limit_minutes`, `_inactive_time_limit_minutes`, `_limit_operator`|`SeedingLimitConfig`; `bt_seeding.rs`|seeding|ratio/minutes/enum；0/0/0/0/or / library absent / user values|DB + daemon config|立即/下一 tick|可预测的做种停止；不能自动改变用户策略|设置|
|`only_files` / `only_files_regex` / `list_only`|pinned `AddTorrentOptions`; `build_add_torrent_options`|torrent/storage|file IDs/regex/bool；task value / library API / task value|tasks DB / task RPC|add-time|降低磁盘与带宽；selection 与 metadata 初始化竞态|设置/任务输入；add 时固化|
|`output_folder` / staging / `overwrite`|`AddTorrentOptions`; `download_manager`|torrent/storage|path/path/bool；task value / library API / task value|tasks DB + filesystem|add-time|决定文件布局与恢复；路径错误可损坏任务|设置/任务输入；不自动改|
|`paused` / start policy|`AddTorrentOptions.paused`; manager lifecycle|torrent|bool；task value / false / task value|tasks DB / task RPC|add/resume|避免未确认任务启动；暂停必须释放运行资源|设置/任务输入|
|`initial_peers`|pinned `AddTorrentOptions`|torrent discovery|peer endpoints；task value / optional / task value|task request only|add-time|加速 metadata；错误地址增加连接失败|设置/任务输入；不由控制器猜测|
|task upload limit|`task_ratelimits`; `AddTorrentOptions.ratelimits`|torrent|B/s；task value / optional / task value|tasks DB|add-time|保证单任务上限；live handle 没有安全统一 setter|设置；已有任务不重加|
|global download/upload limit|`SessionOptions.ratelimits`; manager setters|session|B/s；DB value / library 0 / DB value|DB + daemon config|立即|全局带宽上限；自动策略不得突破|设置；已有热更新路径|
|`peer_limit` / `bt_peer_limit`|`AddTorrentOptions.peer_limit` + `SessionOptions.peer_limit`; `BtConfig.peer_limit`|torrent + session default|peers；128..512，默认 256 / pinned fork default 128 / FluxDown 256|DB + daemon/GPUI/Web/mobile catalog|下次 add；已有 torrent 不删除重加|增大候选并发；CPU/FD/连接/NAT压力；观测 `seen/queued/connecting/live/dead`|设置；无可靠 live setter，保守范围|
|session global connection limit|`Session.peer_limit`（= `SessionOptions.peer_limit`，本 fork 与 per-torrent 同源）|session|connections；FluxDown 用 `bt_peer_limit`，上游 API 语义/session 缺省|同一 `bt_peer_limit`|session 重建|session 内新建 torrent 的缺省上限；已存在 handle 不变|设置；与 per-torrent 值一致|
|`concurrent_init_limit`|`SessionOptions.concurrent_init_limit`|session init|count；3 / library default / 3|无|session 重建|抑制 metadata/tracker storm；过低延迟批量启动|固定内部；初始化期字段|
|runtime worker threads|`available_parallelism().clamp(2, 8)` in `SharedBtSession::new`|runtime|threads；2..8 / library host-dependent / FluxDown clamp|无|session重建|按 CPU 保守扩展；过大增加栈/调度成本|安全自动；已有边界|
|blocking pool|`BT_MAX_BLOCKING_THREADS`|runtime|threads；64 / Tokio default differs / 64|无|session重建|保护校验/storage；运行时扩张会放大 IO 峰值|固定内部；缺少压力控制指标|
|connect timeout|`PeerConnectionOptions.connect_timeout`|peer|seconds；10 / upstream default differs / 10|无|session重建|淘汰不响应 peer；过短误杀慢链路|固定内部；无观测和 live setter|
|read/write timeout|`PeerConnectionOptions.read_write_timeout`|peer|seconds；20 / upstream default differs / 20|无|session重建|保留慢但健康 peer；过长占连接槽|固定内部|
|handshake/MSE timeout|`PeerConnectionOptions.handshake_timeout`|peer|seconds；10 / upstream default differs / 10|无|session重建|防 slowloris；过短影响高延迟 peer|固定内部|
|connect rate|`PeerConnectionOptions.connect_rate`|peer|attempts/s；30 / upstream default differs / 30|无|session重建|平滑连接建立；过高触发 NAT/路由器/FD压力|固定内部；先观测再调|
|peer backoff|`PeerBackoffConfig`|peer|seconds/factor；5/4/600/1800 / pinned override / 5/4/600/1800|无|session重建|避免坏 peer 重试风暴；过长降低恢复速度|固定内部|
|request window|pinned torrent live piece requester|piece/peer|requests；128 / fork default 128 / 128|无|session重建/内部|影响流水线吞吐与内存；未暴露完整指标|固定内部|
|incoming handshake limit|pinned listener/session constant|listener|connections；256 / fork constant / 256|无|session重建|防入站握手耗尽资源；过低损失入站|固定内部|
|listen mode/address/IPv4-only|`ListenerOptions`, session fallback|listener|TCP, `[::]` then IPv4；pinned implementation / library API / FluxDown fallback|无|session重建|双栈与端口探测；IPv6/UDP环境需分别诊断|自动 fallback；不扩大用户端口范围|
|DHT bootstrap / retry / requery / response timeout / route table|`DhtSessionConfig`, DHT internals|DHT|addresses/timers/table；FluxDown bootstrap + fork defaults / upstream semantics / explicit bootstrap|DHT persistence file|session重建|改善纯磁力发现；错误 bootstrap 会制造假象|固定内部；失败先做 UDP/IPv4/IPv6 可达性诊断|
|DHT persistence|`DhtPersistenceConfig`, `dht.json`|DHT/storage|path/interval；app data + pinned interval / library default / explicit path|filesystem|session init|减少冷启动发现成本；损坏必须清理重试并保留 error chain|自动降级；已有语义|
|tracker normalize/dedupe|`tracker_subscription::merge_dedup`|tracker|URL set；normalized set / library set semantics / merged set|DB cache|立即在构造期|减少重复 announce；坏 tracker 仍可观测|自动；不永久剔除|
|built-in `PUBLIC_TRACKERS`|`bt_downloader.rs`|tracker|static URL list；FluxDown list / n/a / list|source only|binary update|提供基础发现；一次失败不能证明永久失效|固定内部|
|announce/reannounce/retry/backoff|pinned tracker/DHT internals|tracker|timers；fork defaults / upstream semantics / unchanged|internal|session runtime|发现稳定性；缺少统一 success/failure metrics|固定内部；先补观测|
|piece size / locks / scheduler|pinned metadata/live state|piece|bytes/locks；torrent metadata + fork internals / API semantics / unchanged|torrent metadata/files|torrent lifetime|决定随机 IO 与公平性；改动影响恢复兼容|固定内部|
|fast-resume bitfield / checksum|session persistence + `.bitv`|storage|bool/bitfield；enabled / library API / enabled|`bt_session` files + tasks DB|session init|减少恢复校验；损坏需全量校验/清理|自动校验；不改结构|
|sparse / staging / completed move|`bt_sparse`, `bt_partfile`, manager|storage|filesystem behavior；platform-specific / library storage API / FluxDown wrappers|filesystem + tasks DB|task lifecycle|支持部分文件与完成搬移；磁盘错误/竞态风险|固定内部|
|storage flush/re-read|pinned storage + completion verification|storage|I/O policy；fork behavior / API semantics / unchanged|files|task lifecycle|保证校验正确；缺少 latency metrics|固定内部|
|LSD / PEX / TCP/UDP/uTP|pinned session feature paths|discovery/peer|feature flags；fork behavior / README support / unchanged|none|session init|增加来源；代理/TUN可能改变 UDP 结果|固定内部；分来源观测待补|
|UPnP/NAT-PMP and port probe|listener + `probe_listener_port`|listener/NAT|bool/ports；UPnP + configured range / library API / user value|DB for policy|session init|可达性；映射失败要诊断而非扩范围|自动范围内 probe + 设置 policy|
|seeding reconcile/FIFO|`SeedingManager`|seeding|slots/period；configured cap + manager tick / library absent / user cap|DB + memory|tick|控制做种资源；reconcile 不能改用户 limits|自动执行用户策略|
|peer/CPU/FD/IO pressure|engine/host OS observation|resource|counts/latencies；当前 peer snapshot partial / n/a / none|not unified|n/a|决定第二阶段是否安全；当前缺少统一指标|固定；禁止自动调|

### 10.2 第一阶段落地的 `bt_peer_limit`

`bt_peer_limit` 是 daemon 配置键，整数范围 **128..512**，默认 **256**。它只写入新建 torrent 和应用重挂载做种的 `AddTorrentOptions.peer_limit`；现有 torrent 不通过 pause/delete/re-add 伪造 live 调整。session 重建也不宣称会改变已存在 handle 的 peer admission。配置 patch 成功只表示配置已保存；“下次新建任务生效”是对外契约。

> 2026-10-12 合并上游 #825 后，**session 级 `SessionOptions.peer_limit` 也取同一配置值**（不再固定 512），
> 使新建 torrent 未显式给 `peer_limit` 时的缺省与设置一致。已存在 handle 仍不重建。

第一阶段不提供 `auto` 值。原因：当前已能观察 `seen/queued/connecting/live/dead`，但没有稳定的握手成功率、握手延迟、有效数据 peer、FD/CPU/磁盘压力统一链路，也没有 pinned fork 的安全 live semaphore setter。未来自动策略必须先补齐这些指标，并使用 128..512、步长 64、5 分钟窗口、冷却与 hysteresis；不能用 pause/delete/re-add 调参。

### 10.3 第一阶段允许自动处理与固定项

允许自动处理：CPU worker 在 2..8 内选择；端口只在用户范围内探测；tracker URL 规范化/去重和内置+订阅合并；DHT persistence 损坏时清理并保留完整错误链后重试，再在无法启动时关闭 DHT继续下载；shared session 复用/空闲释放/失效重建；全局 limiter 热更新；做种 slot reconcile；fast-resume 校验。

保持固定：DHT bootstrap 与内部 timer、TCP listener 与 IPv4 fallback policy、connect/read-write/handshake timeout、connect rate、peer backoff、request window 128、`concurrent_init_limit=3`、入站 handshake 上限 256、blocking pool 64、piece scheduler/locks、storage layout、静态 tracker 成员、MSE 默认策略。共同原因是初始化期字段、无可靠 live setter、缺少观测，或调整会造成连接风暴、tracker 限流、协议兼容性下降、CPU/FD/磁盘失控。

> uTP listener 已不再是固定项：2026-10-12 合并上游后 `bt_enable_utp`（默认开）在
> `TcpAndUtp` / `TcpOnly` 间切换，见 §10.6.1 与本文件相关说明。

### 10.4 版本边界与验证基线

|检查|结论|
|---|---|
|FluxDown pin|`Cargo.toml` patch 与 checkout 都是 `00374a6d772d4123f079f5adf6a1fe013e363381`（2026-10-12 起）|
|pinned 事实源|`crates/librqbit/src/session.rs`、`torrent_state/live/mod.rs`、`peer_connection`、DHT/limits/storage 模块|
|上游对照|`ikatson/rqbit` README、公开 `AddTorrentOptions`/`ManagedTorrent`/`TorrentHandle` 语义；不把上游默认值写成 FluxDown 当前值|
|代码验证|`cargo check -p fluxdown_engine --lib`、`cargo check -p fluxdown_protocol --lib`、daemon/protocol contract tests、Web settings tests|
|网络验证|同一 torrent/目录/限速分别测 128/256/384/512，单任务和多任务各 5 分钟；记录速度、peer 生命周期、announce/DHT、CPU/RSS/FD/磁盘 IO|
|环境矩阵|DHT UDP 阻断、IPv6 不通、UPnP 失败、Clash/TUN 开关、代理开关分别记录；不能把发现失败归因成 peer limit|

本回合没有把未实际运行的网络 A/B 结果写入报告。没有成功率、延迟、资源压力和可复现环境数据前，不进入自动 peer 调优。

## 10.5 上游公开 add-torrent / managed-torrent API 对照

下表把 pinned fork 公开结构中容易遗漏的字段和句柄动作单独列出。`upstream default` 只有在公开源码/API 明确给出时才填写；否则写“未作为稳定契约公开”，避免把一个 fork 的隐式默认误报成上游保证。

|上游参数或动作|pinned / FluxDown 语义|上游默认或公开状态|生效与决策|
|---|---|---|---|
|`paused`|`AddTorrentOptions.paused`；manager 先暂停添加，再按任务状态恢复|false（公开字段默认）|add-time；任务输入/设置|
|`only_files`|预选文件直接写入 add options；对 Initializing handle 不依赖 post-add 更新|`None`|add-time；任务输入|
|`only_files_regex`|FluxDown 在自身任务规划阶段解析为文件索引，再走 `only_files`|公开字段；默认 `None`|add-time；任务输入|
|`list_only`|FluxDown 不用作生产下载；元数据探测走独立 probe 路径|公开字段；默认 `false`|固定内部|
|`output_folder`|每任务 staging 目录；完成期由 manager 搬到最终目录|公开字段；默认 session output folder|add-time；设置/任务输入|
|`sub_folder`|FluxDown 显式设置 output folder 并自行完成命名搬移，不依赖后端子目录策略|公开字段；默认后端行为|固定内部，避免完成命名分叉|
|`overwrite`|生产 add 与做种 re-add 均为 true，路径安全由 `PathGuardFactory` 和完成期保护|公开字段；安全默认由调用者决定|固定内部/任务文件策略|
|`initial_peers`|当前任务协议没有把任意 peer 地址作为常规设置透传|公开字段；默认空|设置/任务输入；暂不自动填充|
|`peer_limit`|`bt_peer_limit` 128..512；写入新 torrent/reseed add options|pinned fork 缺省 128；FluxDown 256|设置；已有 handle 不重建|
|per-torrent `ratelimits`|任务上传限速 add-time 烘焙；下载由 session limiter 控制|公开字段；默认无限制|设置；无统一 live setter|
|preferred peer id / client name|FluxDown 使用 session client identity；没有任务级用户契约|公开 API/内部字段，默认后端生成或 session 值|固定内部；协议身份不自动改|
|MSE / `disable_trackers`|MSE 是 `SessionOptions.mse_mode`；tracker disable 只在测试/特殊 probe 使用|公开字段；各自默认由调用者/SessionOptions 决定|MSE 设置；tracker policy 固定|
|force tracker interval|FluxDown 不覆盖 tracker 返回的 announce/reannounce 计划|公开/内部 tracker API；默认跟随响应|固定内部；缺少统一健康指标|
|storage factory|FluxDown 注入 `PathGuardFactory`、Windows sparse factory、parts seed factory|公开扩展点；默认 filesystem storage|固定内部；磁盘语义需保持|
|pause / resume|`ManagedTorrent`/`Session` pause/unpause；manager 缓存 handle|公开句柄动作|运行期自动执行任务生命周期|
|delete / delete files|manager 区分删除任务与删除文件，并处理 pending add|公开句柄动作|运行期自动执行；不用于调 peer limit|
|move storage / rename|FluxDown 在完成期自有 staging→最终目录搬移和去重命名|公开动作不等同 FluxDown 成品命名|固定内部；避免破坏 task DB 语义|
|update selected files|pinned fork 在 Initializing 阶段拒绝；FluxDown 只在初始化完成后重试|公开动作；阶段限制来自 fork|运行期自动执行，失败保留 error|
|set upload/download limit|global/session limiter 有 manager 热更新；task upload 只在 add-time|公开 API 分层不同|用户设置；不得突破全局 cap|
|status/progress/peer stats|FluxDown 保留 `BtPeerStats` 的 `Option`；生命周期、transport、useful-peer、连接累计计数和 piece 质量快照均可映射；当前 live peer 集合变化会影响质量分母|公开 status/stats API；粒度随版本变化|观测；None 不伪造为 0|
|fast-resume / persistence|session.json 主动清理；`.bitv`、torrent metadata 和 FluxDown DB 共同恢复|公开 persistence/fastresume；原子性约束未完全保证|自动恢复/校验；固定结构|

这一区分了“公开 API 存在”和“FluxDown 已接入并承诺语义”。未接入的上游字段不进入 daemon/protocol/UI，也不通过自动控制器猜测。

## 10.6 当前速度排查：除 `peer_limit` 外的有效参数

本次本机 Ubuntu 种子短测使用同一 `.torrent`、独立临时目录、重编译后的 daemon 和默认 `bt_peer_limit=256`。任务从 `preparing` 正常进入 `downloading`；RPC runtime 快照连续采集 114.8 秒、230 个样本，`fetchedBytes` 从 1,358,346,240 增至 2,345,728,000，增量 987,381,760 字节，折算平均 8.20 MiB/s（约 68.8 Mbit/s）。同一窗口 `live` 为 232..248、`usefulPeers` 为 186..223；末样本为 `live=246`、`usefulPeers=223`、`queued=1186`、`connecting=10`、`seen=1477`、`dead=35`。REST 任务状态在主动停止前为 `downloadedBytes=2,440,460,288` / `totalBytes=6,482,409,472`（约 2.27 GiB / 6.04 GiB）；未采集完整下载耗时、稳定 5 分钟窗口或 128/384/512 对照，因此以下仍是参数影响排序，不是完整 A/B 结论。

该窗口的独立日志显示 session 以 `DHT=true`、`UPnP=true`、`peer_limit=256`、83 个 tracker、`speed_limit=0`、`upload_limit=0` 启动；多个 tracker announce 成功并返回 peer，但四个 DHT bootstrap 地址反复记录 `no successful lookups`。这证明本环境存在 DHT bootstrap 异常和 tracker 可用性，但不能把其吞吐影响单独归因于 `bt_peer_limit`。REST 任务详情不携带 `btPeerStats`，本次质量数据来自 RPC `taskRuntimeChanged`；该接口仍不能提供来源级 tracker/DHT/PEX/LSD 归因。

### 10.6.1 已确认会影响吞吐的层

|优先级|参数/路径|当前语义|对速度的具体影响|结论|
|---|---|---|---|---|
|1|`speed_limit_bytes`|`SessionOptions.ratelimits.download_bps`；`0` = 不限|会直接给整个 BT session 限速；HTTP 的分段数不影响 BT 下载|先确认实际 config 值；任何非零值都比调 peer 更优先排除|
|1|`upload_limit_bytes` + task/queue upload limit|全局上传 limiter 与 torrent 上传 limiter 叠加|上传过低可能减少对端愿意提供的服务；过高会挤占 100M 上行、ACK/NAT/路由器资源|保持用户上限；不要用自动策略突破|
|2|`bt_peer_limit`|每 torrent semaphore + session 缺省，当前 256；范围 128..512|只限制同时占用连接资源的 peer，不保证存在更多可用 peer；#825 实测 128→512 在公网 swarm 里提升显著（1.1→12.6 MB/s）|先看 `live/queued` 证据再决定提高；提高后留意 CPU/FD/RSS|
|2|DHT / tracker 发现|DHT 开启；显式 4 个 bootstrap；内置、用户、订阅 tracker 合并|决定候选 peer 数和首个有效 peer 时间；发现失败时提高 peer 上限没有收益|当前比 peer limit 更值得补观测|
|3|`connect_rate=30`|每秒最多 30 个出站连接尝试|影响冷启动和坏 peer 替换速度，不直接限制已稳定传输吞吐|固定；缺少连接失败率/握手延迟前不调|
|3|connect/handshake/read-write timeout|10s/10s/20s|坏 peer 占用连接槽的时长与慢 peer 保留时间之间的折中|固定；过短可能误杀高延迟 peer|
|3|MSE `enabled`|先尝试 MSE；对明文 peer 失败后重新建立明文连接|部分 peer 会产生额外一次握手，影响建连阶段；不应影响稳定传输|只作为诊断对照，不默认关闭协议保护|
|3|listener TCP / uTP（`bt_enable_utp`，默认开）/ IPv4 fallback|监听为 `TcpAndUtp`（关则 `TcpOnly`），优先双栈再 IPv4|开 uTP 能连上更多 NAT 后 peer（#825 实测稳态再翻倍），代价是 UDP 收发的系统态 CPU 更高|用户设置；改动经重建共享会话生效|
|4|worker threads / blocking pool|BT runtime `2..8` worker，blocking pool `64`|影响 metadata、校验和 storage 调度；对单个 100M 顺序下载通常不是首要上限|已有安全边界，保持固定|
|4|staging / filesystem storage|每任务 staging；默认 filesystem storage；完成期再搬移|磁盘写入、校验和搬移可能成为上限；下载中不应反复搬移最终文件|需记录磁盘写入等待和空间，不先改存储结构|

### 10.6.2 本回合发现并修正的 tracker 行为

`BtConfig` 注释和调优契约要求“内置 tracker + 用户 tracker + 订阅缓存”合并去重，但原实现只在用户自定义列表为空时使用 `PUBLIC_TRACKERS`；一旦 `bt_custom_trackers` 非空，内置列表会被完全替换。这样会让一个过期或过短的用户列表降低发现能力，且不容易从速度表现中区分出来。

现已改为始终合并：

```text
PUBLIC_TRACKERS
  + bt_custom_trackers
  + bt_tracker_sub_cache（订阅启用时）
  -> merge_dedup -> HashSet<Url>
```

这不是提高连接上限，而是修复候选来源被意外削弱的问题；坏 tracker 仍只影响 announce 噪声，不会静默删除用户配置。

### 10.6.3 当前不能从现有短测推出的结论

- 不能证明 256 已是最优值：短测没有 128/384/512 对照，也没有 5 分钟稳定窗口。
- 不能证明 peer 数是瓶颈：短测没有 128/384/512 对照，也没有 5 分钟稳定窗口；现在可看到生命周期、传输、连接质量快照，但仍没有来源级成功率。
- 本窗口 `live=232..248` 已接近 256，`queued=866..1186` 持续积压，满足“值得安排 384 对照”的网络侧前置迹象；但未采集 CPU/FD/IO 压力，也没有 384 对照，不能据此把默认值改为 384 或 512。
- 不能证明磁盘不是瓶颈：尚未同步采集写入吞吐、IO wait、RSS、FD 和错误日志。
- 不能把 DHT、tracker、IPv6、UPnP、Clash/TUN 或代理影响归因给 `peer_limit`；新增 peer 质量快照能缩小范围，但不能替代来源级 announce/DHT 指标。

### 10.6.4 安全排查顺序

1. 读取并记录实际 `speed_limit_bytes`、`upload_limit_bytes`、`bt_enable_dht`、`bt_enable_upnp`、`bt_mse_mode`、tracker 合并数量和实际监听地址。
2. 在不改变总带宽上限的前提下，对同一 torrent 做短窗口 `peer_limit=128/256/384/512` 对照；每组至少保存 `seen/queued/connecting/live/dead`。
3. 单独做 DHT/IPv6/UPnP/TUN 开关对照；优先区分“没有候选 peer”和“候选足够但连接/传输失败”。
4. 只有当 `live` 接近 256、`queued` 长期积压且 CPU/FD/IO 无压力时，才有理由测试 384 或 512。
5. 若 `live` 很低或 `dead`/连接失败高，先修复发现、UDP、IPv6、NAT 或代理路径，不提高 peer 上限。

## 10.7 已落地的 BT peer 质量快照与自动调优边界

`TaskRuntime.btPeerStats` 现在在 BT 进度事件中保留两层信息：生命周期计数来自 pinned `AggregatePeerStats`，连接质量和传输计数来自当前 live peer snapshot。除五个生命周期计数外，其余字段均为 `Option`；后端未提供观测时必须保持 `None`，不能把未知转换成零。

|字段|口径|用途与限制|
|---|---|---|
|`seen` / `queued` / `connecting` / `live` / `dead`|当前 torrent 的点时刻生命周期计数|判断发现、排队、建连和失效阶段；不是累计事件总数|
|`usefulPeers`|当前 live peer 中累计 fetched bytes ≥ 1 MiB 的 peer 数|比 `live` 更接近“有数据贡献”的 peer；阈值是诊断口径，不是协议语义|
|`liveTcp` / `liveUtp` / `liveSocks`|当前 live peer 的传输类型计数|区分 TCP/uTP/SOCKS 路径；不能推导 DHT 或 tracker 来源|
|`connectionAttempts` / `connectionSuccesses` / `connectionErrors`|当前 live peer 记录上的累计连接计数|用于粗略判断握手/连接质量；不包含已经从 live peer map 移除的 peer，不能当全局成功率|
|`totalConnectingMs`|当前 live peer 记录累计连接耗时|可计算粗略平均建连时长；样本集合变化会改变分母|
|`fetchedBytes` / `downloadedPieces` / `totalPieceDownloadMs`|torrent live snapshot 的累计网络字节、已校验 piece、piece 下载耗时|区分“收到数据但未完成校验”和 piece 调度/校验吞吐；不是磁盘写入耗时|
|`peerSteals`|pinned piece 调度器的累计 peer steal 次数|反映 piece 请求被其他 peer 接管的压力；不能单独判定异常|

实现位置：

- engine：`native/engine/src/bt_downloader.rs::bt_peer_stats_from_handle`；
- engine-owned model：`native/engine/src/transfer_activity.rs::BtPeerStats`；
- wire conversion：`native/engine-protocol/src/lib.rs` 与 `native/protocol/src/task_activity.rs`；
- Web mirror/reducer：`web/src/lib/rpc/protocol/task.ts`、`web/src/lib/rpc/apply.ts`。

当前策略仍是**观测，不是自动调参**。安全的后续控制器必须满足：

1. 以 `128/256/384/512` 为离散候选，不得动态生成任意值；
2. 以至少 5 分钟窗口计算 `usefulPeers/live`、连接错误率、平均建连耗时、吞吐和资源压力；
3. 只有 `queued` 长期积压、`live` 接近上限、`usefulPeers` 随上限增加且 CPU/FD/IO 无压力时，才允许提升一档；
4. `live` 低、`connectionErrors` 高、平均建连耗时上升或 `usefulPeers/live` 下降时，先诊断 DHT/tracker/IPv4/IPv6/NAT/代理，不得继续提高上限；
5. 使用冷却时间和 hysteresis，禁止在相邻窗口来回切换；不得通过 pause/delete/re-add 改变现有 torrent；
6. wire 缺失或 `None` 的指标必须让控制器保持观测态，而不是按零触发扩容或降容。

这些条件仍缺少来源级 announce/DHT 统计、主机 CPU/FD/磁盘 IO 和稳定长窗口数据，因此本回合不启用自动 peer-limit 控制器。

Pinned 公开 API 仍未提供可信的 `handshakes_succeeded`、`handshakes_failed`、`active_data_peers`、metadata peer 数、tracker/DHT/PEX/LSD 来源归因或 storage 写入延迟。`connection_successes` / `connection_errors` 保持连接计数命名，**不得**作为握手成功率展示。后端没有匹配语义前，这些缺失字段保持 `None` 或不进入 wire 契约。
