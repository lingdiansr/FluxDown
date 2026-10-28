# network-env slice — 网络生态数据入口、内核/运行环境、移动与 NAS 平台约束、测量方法

slice 前缀：`S-DATA-` / `S-DOC-KERNEL-` / `S-DOC-MOBILE-` / `S-DOC-OS-`
配套文件：`work/network-env.sources.csv`（36 条来源）
抓取日：2026-10-17（所有 `retrieved_at`）

## 0. 本 slice 的读取状态与边界

- 本 slice 为纯只读资料获取，未运行任何下载/announce/DHT/握手/抓包/整形，未修改 FluxDown 或参考仓库，未访问私有 tracker 或用户数据。
- 所有 HTTP 来源均为公开页面，抓取当次内容即 `locator` 所指内容；**未绑定任何 git commit 的网页**（内核文档树、man7、developer.android.com、developer.apple.com、CAIDA / RIPE / APNIC 页面）在 CSV 中一律不写 `doi_or_commit`，并在 `limitations` 标注「滚动内容 / 无版本戳」。
- 本地对照源码（libtorrent / qBittorrent / EE / transmission / aria2）与 rqbit pin 由其它 slice 负责，本 slice **不**引用其行号。
- 本 slice 残留的 FluxDown 事实只使用上游下发的冻结常量（见 §3 各条的「与当前 FluxDown 实现差异」），并标注「未在本次检索中复核」；**本 slice 未读取 FluxDown 源码**。

---

## A. 网络生态公开数据入口：方法、许可与已声明偏差

### A1. APNIC Labs IPv6 覆盖率不是 BT 可达率

- **主张**：APNIC Labs 的 IPv6 Capability 指标由「浏览器取回一个仅 IPv6 可达的 web 对象」的成败构成，任务通过在线广告系统分发（方法与隐私说明见 `S-DATA-APNIC-METHOD`）。
- **证据(locator)**：`S-DATA-APNIC-METHOD` §Why are we doing this? + §Data Collection and Privacy（https://labs.apnic.net/measureipv6/）。
- **适用条件**：仅覆盖「能看到该广告且执行脚本」的浏览器会话；不覆盖 App 内流量、纯非 HTTP 流量、无脚本客户端。
- **与当前 FluxDown 实现差异**：无关实现；关系到「用宏观覆盖率解释 FD 的 IPv6 peer 成功率」这类推断的合法性。FD 侧的地址族选择与 BT 监听/映射行为本 slice 未核对。
- **可检验预测**：若在同一接入点上把 FD 的 v4-only / v6-only / 双栈三态分别测量，其「IPv6 可用时的成功率」与 APNIC 该 AS/该国的覆盖率数字**无稳定相关**；而与该接入点自己的 `getaddrinfo`/接口地址状态强相关。反例：若观察到强相关，则说明覆盖率数字在该 AS 上实际反映了接入能力而非广告样本偏差。

### A2. 分国覆盖率的引用上限

- **主张**：2026-10-17 的分国表中 CN 首列为 49.47%（样本 39,531,459），HK 为 9.83%（样本 4,575,401），US 为 61.98%（样本 97,337,761）；两个百分列在 `/ipv6/help` 图表中的标签为 `IPv6 Capable` 与 `IPv6 Preference`。
- **证据(locator)**：`S-DATA-APNIC-STATS`，分国表行 CN / HK / US。
- **适用条件**：数字是**滚动快照**，页面未给窗口长度与置信区间；`IPv6 Preference` 的精确定义本次未在任何获取页核对（列为待核）。
- **与当前 FluxDown 实现差异**：无直接差异。
- **可检验预测**：把该数字当作「该地区 BT peer 中可走 IPv6 的比例」去预测 FD 的双栈建连成功率，会出现系统性高估（因为 BT peer 群体 ≠ 广告样本群体）。可用的证伪实验见 A1。

### A3. RIPE Atlas：许可与数据可见性约束（引用数字前必读）

- **主张**：RIPE Atlas Data 可作研究使用；**商业使用需事先取得 RIPE NCC 许可**；Probe Host 的探针网络参数（含 NAT 后的 RFC1918 与双重 NAT/CGN 下的 RFC6598 共享地址）在测量结果中可见。
- **证据(locator)**：`S-DATA-RIPE-ATLAS-TC-USE` Art. 3.3 / 4.5；`S-DATA-RIPE-ATLAS-TC-DATA` Art. 4.2 / 4.3。
- **适用条件**：版本 v3.5（2025-09-26 更新，2025-11-04 生效）。执行测量还需 Credits。
- **与当前 FluxDown 实现差异**：无；影响的是「能否把 Atlas 数据/测量嵌进产品与对外报告」。
- **可检验预测**：若 E0–E3 的延迟测量通过 RIPE Atlas 公共测量完成，则结果可公开（研究用途），但任何带商业目的的再发布必须走许可流程 → 实验协议里应把它标为「有附加许可条件的工具」而非默认工具。

### A4. RIPE RIS：只回答控制面路由问题

- **主张**：RIS 由部署在 IXP 的 RRC 收集自愿 peering 网络的 BGP 数据，并以 MRT 格式（RFC 6396）发布，dumps 每 8 小时、updates 每 5 分钟；RRC00/24/25 为 multi-hop 收集器。
- **证据(locator)**：`S-DATA-RIPE-RIS` §How does RIS work? / §1. RRCs；`S-DATA-RIPE-RIS-MRT` §Name and location。
- **适用条件**：覆盖取决于志愿者 peering；单个收集器的 peer 数据混在同一文件。
- **与当前 FluxDown 实现差异**：无直接差异；RIS 只能为「某 AS 之间的路由是否存在」提供控制面证据，不能替代 FD 侧对端可达性实测。
- **可检验预测**：某对 AS 之间存在 BGP 路径（RIS 可见）而两 AS 内主机间的实际 TCP/uTP 建连失败率仍可很高 → 控制面可达性与数据面可达性是两个量。该预测**否证**「用 RIS 路径存在性替代连通性测试」的做法。
- **许可缺口**：RIS 概览页与 MRT 文档页**均未声明数据许可条款**（本 slice 在 RIS 文档树中检索 `licen|CC BY|open data|copyright|attribut` 无命中）→ CSV 中 `许可: missing`。**不得**把「文件可下载」推断为「可任意再分发」。

### A5. CAIDA：AS 关系与 pfx2as 的访问条件与已知局限

- **主张 1（访问条件）**：pfx2as 数据集受 **CAIDA Acceptable Use Agreement (AUA) for Publicly Accessible Datasets** 约束，要求按指定格式引用，并要求用户**向 CAIDA 报告**使用该数据集的出版物；数据自 2005-05-09（v4）/2007-01-01（v6）起每日生成。
- **证据(locator)**：`S-DATA-CAIDA-PFX2AS` §Description / §Note on Multi-origin ASes / §Acceptable Use Agreement / §Data Access。
- **主张 2（方法局限）**：在 ToR 公式化下 **p2p 链路无法被可靠推断**（页面引 Di Battista 等与 Erlebach 等的复杂性结果，并总结「迄今没有技术能可靠识别 p2p 链路」）；customer cone 被定义为「在**我们观测到的 BGP 路径**上」沿 p2c 可达的 AS 集合。
- **证据(locator)**：`S-DATA-CAIDA-ASREL` §History of inference algorithms / §Customer Cone。
- **适用条件**：推断输入是公开 BGP 表的观察点集合；AS Relationships 的许可文本在本次未能取得（catalog 条目页为 JS 渲染，正文未取回；概览页不含 AUA）→ CSV 中 `许可: missing`，**只可引用方法与局限，不可据以宣称「已获授权使用该数据集」**。
- **与当前 FluxDown 实现差异**：无直接差异。可用于把 E3 的公网结果按 AS 归类，但归类本身受 A5 主张 2 限制。
- **可检验预测**：若按 AS 关系把 peer 分成 c2p / p2p 两类并预测吞吐差异，则**必须**预期 p2p 类存在显著误分类（含被误判的 c2p），因此任何「p2p peer 更差/更好」的结论都需先给出误分类率的上下界，否则不可报告。

---

## B. 内核与运行环境（Linux 桌面 + NAS）

### B1. socket 缓冲与 TCP 旋钮的真实语义

- **主张**：`tcp_moderate_rcvbuf` 启用时 TCP 自动调整接收缓冲但不超过 `tcp_rmem[2]`；`tcp_notsent_lowat` 控制写队列未发送字节并决定 POLLOUT 何时报告；`net.core.rmem_max` / `wmem_max` 是上限、`rmem_default` / `wmem_default` 是默认；`optmem_max` 默认 128 KB 且 TCP tx zerocopy 也受它限制。
- **证据(locator)**：`S-DOC-KERNEL-IPSYSCTL-TCP`（tcp_moderate_rcvbuf 451-455；tcp_notsent_lowat 769-…；tcp_rmem 555-…；tcp_wmem 752-…）；`S-DOC-KERNEL-SYSCTLNET`（rmem_default 221-224；rmem_max 226-229；wmem_default 247-250；wmem_max 252-255；optmem_max 368-375）。
- **适用条件**：文档树为滚动内容，无版本戳；内核默认值可被发行版覆盖（本机为 CachyOS 7.2.9-2，实际取值须读 `/proc/sys`）。
- **与当前 FluxDown 实现差异**：FD 的 session 级参数上限为 `BT_SESSION_PEER_LIMIT_MAX=4096`、`BT_PEER_LIMIT_MAX=512`、`BT_PEER_CONNECT_LIMIT_MAX=256`（冻结事实，本次未复核）。这些是**单进程 fd/socket 需求量的量级来源**：每个 peer 连接占 1 个 socket fd（TCP 或 uTP/UDP），而 `RLIMIT_NOFILE` 决定进程能开多少 fd（见 B2）。文档给出的 `rmem_max` 是**单 socket 上限**，与会话 peer 数相乘才是潜在内核内存量级，二者不可混为一谈。
- **可检验预测**：在 rmem_max 默认（通常 ~208 KiB）下把并发 live peer 从 64 提到 512，若接收方向出现 `netstat -su` 的 receive buffer errors 或吞吐不升反降，则说明该链路的瓶颈是**每 socket 缓冲总和/内存压力**而非 peer 数上限；若指标不变则否证。

### B2. fd 上限是并发的硬闸门之一

- **主张**：`RLIMIT_NOFILE` 是「可打开的最大 fd 号加一」，超出即失败；非特权进程抬不了硬限，且硬限不能超过 `/proc/sys/fs/nr_open`（否则 EPERM）。
- **证据(locator)**：`S-DOC-KERNEL-RLIMIT` RLIMIT_NOFILE (201-204)、EPERM/nr_open (328-330)。
- **适用条件**：软限/硬限语义不同；`limits.conf` 与 systemd 单元可各自设定。
- **与当前 FluxDown 实现差异**：FD 的 `BT_SESSION_PEER_LIMIT_MAX=4096` 意味着最坏情形下（多任务 × 全量 peer × 磁盘/日志/DB fd）会话需要数千个 fd；若进程 `RLIMIT_NOFILE` 软限仍为常见默认 1024，则 fd 会先于 peer 上限成为约束，且**失败形态是 open/socket 报错而不是「peer 数被限制」**（冻结事实，FD 侧实际软限未复核）。
- **可检验预测**：在 E1（受控基线）把会话 peer 顶格时，若进程 fd 使用量接近软限而准入拒绝计数（`AdmissionDenied` 系列）未增长，则瓶颈是 fd 而非准入逻辑；把软限提高后吞吐提升即支持该假设。

### B3. UDP/uTP 方向的内存记账

- **主张**：`udp_mem` 是全体 UDP socket 排队页数上限（min/pressure/max 三档），超过压力时单个 socket 仍可占用 `udp_rmem_min` / `udp_wmem_min`；`somaxconn` 默认 4096（5.4 前 128）。
- **证据(locator)**：`S-DOC-KERNEL-IPSYSCTL-UDP`（udp_mem 835-846；udp_rmem_min 848-852；udp_wmem_min 854-858；somaxconn 位于同文件 TCP variables）。
- **适用条件**：文档为内核默认，发行版可覆盖。
- **与当前 FluxDown 实现差异**：FD 采用 rqbit 的 uTP（UDP）与 TCP 混合传输（记忆与冻结事实），因此 uTP peer 的收发内存走 `udp_mem` 记账而与 TCP 的 `tcp_mem` 分离；`somaxconn` 影响的是 FD 的入站 TCP 监听 backlog，而不是 peer 数上限。
- **可检验预测**：在 E2 的「uTP-only」臂中，若把 uTP peer 数拉高到 UDP 页数压力区，表现应为**整体 uTP 吞吐下降而 TCP 臂不降**；若是混合臂同样下降，则瓶颈在内核内存之外（如磁盘或 CPU）。

### B4. AQM/整形：fq_codel 与 CAKE 的适用条件

- **主张**：`fq_codel` 以随机流哈希 + 每流 CoDel 提供公平与低延迟，默认 target 5ms / interval 100ms / limit 10240 pkt / memory_limit 32MB，且文档要求 interval 取瓶颈最坏 RTT 量级；CAKE = COBALT + deficit-mode shaper + DRR++，rtt 预设（默认 internet=100ms）决定 AQM 时标，`lan`(=1ms) 档位会因时标接近内核 jitter 而可能过早发拥塞信号导致吞吐下降。
- **证据(locator)**：`S-DOC-KERNEL-FQCODEL` §PARAMETERS；`S-DOC-KERNEL-CAKE` §DESCRIPTION / §ROUND TRIP TIME PARAMETERS / §FLOW ISOLATION PARAMETERS。
- **补充主张**：CAKE 的 `memlimit` **不应按 BDP 估算**而应按可接受的最坏内存占用设定；GSO super-packet 默认被拆分以降低竞争流延迟（>10 Gbps 才考虑 `no-split-gso`）；单队列 `cake` 受全局 qdisc 锁限制，可整形带宽受单 CPU 限制，`cake_mq` 用多 TX 队列规避。
- **证据(locator)**：`S-DOC-KERNEL-CAKE-MEM`（memlimit ~460-472；split-gso 488-500；cake_mq 对比 548-563）。
- **适用条件**：仅当该 qdisc 附加在实际瓶颈接口时生效；`bandwidth` 未设置时 CAKE 默认 `unlimited`（不整形）。
- **与当前 FluxDown 实现差异**：这是**与 FD 进程之外**的层次（qdisc 在接口上），FD 侧无对应旋钮；但它是「高 goodput 但网页/游戏不可用不算优秀」这条护栏指标（计划 §5 交互与资源护栏）在主机侧的干预点。
- **可检验预测**：在 E2 的「竞争交互流」臂中，同一 FD 配置下：① 无 qdisc → 空载/带载 RTT 差显著上升；② 挂 `fq_codel` → 该差值下降且 goodput 基本不变；③ 若该差值不降，则瓶颈不在本机出口 qdisc（可能在接入侧上行或 Wi-Fi 层），此时 CAKE `rtt` 调参也不会改善。

### B5. TUN/代理路径会「吃掉」qdisc（对 FD 影响最大的内核条目）

- **主张**：`IFF_BACKPRESSURE` 开启时，TUN/TAP 内部 ring buffer 满会使内核**停止队列**而不是丢包，从而让已附加 qdisc 的 AQM/整形/公平性真正生效；**无该标志时 TX 丢包发生、qdisc 事实上被绕过**，应用只能从丢包感知拥塞。该标志是设备属性，适用于全部队列，且设备超过 1 个队列后无法再修改。文档另建议可同时减小 `txqueuelen` 把缓冲转移到 qdisc（减少 bufferbloat），但可能带来性能代价，且在单个 TUN/TAP 队列上跑多流时背压可能因开销降低性能。
- **证据(locator)**：`S-DOC-KERNEL-TUNTAP` §3.4 qdisc backpressure（文件头 SPDX GPL-2.0）。
- **适用条件**：须已附加 qdisc，`noqueue` 下无效。
- **与当前 FluxDown 实现差异**：当 FD 的流量经 TUN（VPN/代理/虚拟网卡）出网时，主机侧 qdisc 干预的有效性取决于该 TUN 设备是 `IFF_BACKPRESSURE` 还是默认丢包行为；本 slice 未核对 FD/用户环境是否走 TUN。
- **可检验预测**：同一 FD 配置下，经 TUN 与直连物理网卡两条路径分别测「空载 vs 带载 RTT 差」：若 TUN 路径的 RTT 差不可被 qdisc 调参改善而物理路径可以，则强烈提示 TUN ring buffer 先于 qdisc 丢包/排队。

### B6. 文件分配与稀疏：预分配不等于持久化，洞探测不保证

- **主张**：默认 `fallocate()` 分配并把新增子区域置零，成功后对该区间的写入**保证不会因空间不足失败**；`FALLOC_FL_KEEP_SIZE` 不改文件大小；`FALLOC_FL_PUNCH_HOLE` 须与 `FALLOC_FL_KEEP_SIZE` 或运算使用；分配以块为单位可能超出请求范围。`SEEK_DATA`/`SEEK_HOLE`（Linux 3.1+）用于映射稀疏文件空洞，但**文件系统没有义务报告空洞**，因此不是「实际已分配存储」的保证机制。
- **证据(locator)**：`S-DOC-KERNEL-FALLOCATE` §Allocating/Deallocating/Zeroing + §HISTORY；`S-DOC-KERNEL-LSEEK` §Seeking file data and holes (62-105)。
- **适用条件**：取决于文件系统（文档已列各操作的起始版本与支持集）。
- **与当前 FluxDown 实现差异**：FD/引擎的下载文件预分配与续传策略本 slice 未核对；本条给出的是**判据**：把「文件大小正确」当作「数据已落盘」是错的（`SEEK_HOLE` 也不能证明），因此计划 §5「恢复与正确性」里 `t_done` 要求最终校验与文件可用，与本条一致而非冲突。
- **可检验预测**：在 E1 中强制断电/杀进程后重启，若 FD 报「已完成」而全文件 hash 不匹配，则 FD 的完成判定没有覆盖「已分配但未写入/未 fsync」的区间 → 该 run 必须标为正确性失败而非性能失败。

### B7. 块设备队列与机械盘可观测项

- **主张**：内核 stable ABI 定义了 `/sys/block/<disk>/inflight`（驱动中 pending/in-flight 请求数，读/写各一）与 `/sys/block/<disk>/queue/{iostats, max_sectors_kb, nomerges, nr_requests, rotational, rq_affinity, scheduler, wbt_lat_usec}`；`inflight` 与 `queue/nr_requests`（及 SCSI 的 `queue_depth`）相关。
- **证据(locator)**：`S-DOC-KERNEL-SYSFSBLOCK`（inflight 94-97；iostats 424；max_sectors_kb 489；nomerges 540；nr_requests 553；rotational 649；rq_affinity 656；scheduler 670；wbt_lat_usec 709）。
- **适用条件**：可读写性以条目 `[RO]/[RW]` 标注；本次为 `partial`（879 行文件仅读到前 300 行 + 检索命中片段），**各属性完整语义未逐条核对**。
- **与当前 FluxDown 实现差异**：FD 的磁盘背压/写队列策略本 slice 未核对；本条只提供「在 NAS 与桌面两端都能取到、且不依赖 FD 内部埋点」的 I/O 观测面。
- **可检验预测**：E1 中若 goodput 停滞而网络侧在途请求未满，同时 `inflight`/`nr_requests` 已达上限或 `wbt_lat_usec` 反映高写延迟，则背压在块层而非网络层；反之若 `inflight` 很低而 goodput 停滞，则瓶颈在 FD 侧调度或对端。

---

## C. 移动端与 NAS 平台约束

### C1. Android：后台执行与 FGS 时限

- **主张 1**：Doze 期间系统挂起应用网络访问、忽略 wake lock、推迟标准 AlarmManager 闹钟、不做 Wi-Fi 扫描、不运行 sync adapter 与 JobScheduler，仅在维护窗口放行，且维护窗口频率随时间下降；App Standby 下 idle 应用约**每天一次**获得网络访问。
- **证据(locator)**：`S-DOC-MOBILE-ANDROID-DOZE` §Doze restrictions；`S-DOC-MOBILE-ANDROID-STANDBY` §Understand App Standby。
- **主张 2**：Android 14（API 34）起前台服务必须声明 service type；`dataSync` 用于数据上传/下载且无运行时前置权限；`mediaProcessing` 有时限（每 24 小时 6 小时，由应用所有该类服务共享）。
- **证据(locator)**：`S-DOC-MOBILE-ANDROID-FGS-TYPES` §Data sync / §Media processing。
- **主张 3**：WorkManager 适用于 <10 分钟的可延迟任务；**长于 10 分钟的 worker 会被系统重调度**，需前台服务承载并因此受相同限制。
- **证据(locator)**：`S-DOC-MOBILE-ANDROID-DATATRANSFER` §Use WorkManager for data transfer / 对照表。
- **适用条件**：Doze/Standby 对 API 23+ 全应用生效（与 targetSdk 无关）；用户可把应用加入 battery optimization 豁免名单改变行为（豁免后仍受其他限制）。
- **与当前 FluxDown 实现差异**：FD 移动端为 `mobile/Android` 经 `native/mobile` UniFFI 共享核心（记忆/冻结）。以这些平台规则推演：移动端 BT 会话在 Doze 期间**不可能**保持持续传输（网络被挂起），在 App Standby 下最坏约每天一次放行；用户主动发起的下载应走 user-initiated data transfer job 路径而非后台 worker。**FD 移动端当前实际使用哪种 API 本 slice 未核对**（不得据本条断言 FD 已有缺陷）。
- **可检验预测**：移动端后台成功率实验（计划 §5 的 NAS/移动端补充 P1）中，若「屏幕关闭 >30 分钟后 resumed 的字节增量 ≈ 0 而进程未被杀」，则命中 Doze/Standby 而非崩溃；把应用加入 battery optimization 豁免后该差值显著减小即支持该归因。若豁免与否无差别，则瓶颈在别处（后台被杀、网络切换、温度降频）。

### C2. Android：网络能力 API 的语义陷阱

- **主张 1**：`NET_CAPABILITY_INTERNET` 只表示网络「已被配置为可访问互联网」，**不等于实际可达**；系统能给出的最近似「实际可达」指标是 `NET_CAPABILITY_VALIDATED`，但文档明确它仍可能遭遇基于 IP 的过滤或信号劣化导致的突然失联。
- **证据(locator)**：`S-DOC-MOBILE-ANDROID-NETSTATE` §NetworkCapabilities and LinkProperties。
- **主张 2**：当新网络成为默认网络时，应用新开连接走新网络，而**此前默认网络上的所有剩余连接会被强制终止**；系统偏好非计费与更快的网络。
- **证据(locator)**：`S-DOC-MOBILE-ANDROID-NETSWITCH` §Listen to network events。
- **适用条件**：读取状态无需权限；绑定后台网络等操作需 `CHANGE_NETWORK_STATE`。
- **与当前 FluxDown 实现差异**：FD 在移动端若以「有效网络存在」作为可下载前提，则 `INTERNET` capability 是**过强**的前置条件判断（会出现「判定为可下载但实际打不通」）；反之若只在切换后被动重试，则可能因「剩余连接被强制终止」而丢失在建 peer 连接。FD 移动端实际判断逻辑本 slice 未核对。
- **可检验预测**：在 Wi-Fi↔蜂窝切换实验臂中，若 FD 的 peer 连接全部重建且首个有效字节时延 ≈ 建连+握手成本（而非 0），则符合「连接被强制终止」的预期；若观察到部分连接存活，则说明这些连接绑定在未被切换的网络对象上（Android 允许多网络并存），需要显式记录原因为「网络对象未变」。

### C3. iOS：后台传输由系统代跑，且被限速

- **主张 1**：后台 `URLSession` 的传输由**独立于应用的进程**执行，限制包括：必须提供 delegate、仅支持 HTTP/HTTPS、重定向总被跟随且无法拦截、仅支持从文件发起的上传任务。
- **主张 2**：系统对在后台启动新下载施加 rate limiter，**每次 resume/relaunch 后延迟增加**，应用回到前台时延迟重置为 0；因此文档建议用少量 session 承载多任务，而非反复单任务启动。
- **主张 3**：`BackgroundTasks`（iOS 13+）用于数分钟级后台任务，较长任务在支持设备上可利用外部电源/网络/GPU，但调度时机由系统决定，文档未给硬性时限。
- **证据(locator)**：`S-DOC-MOBILE-IOS-URLSESSION-BG` §Comply with background transfer limitations / §Use background sessions efficiently；`S-DOC-MOBILE-IOS-BGTASKS` Overview。
- **适用条件**：`isDiscretionary` 会等待插电/Wi-Fi 等条件；`earliestBeginDate` 只保证不早于该时刻。
- **与当前 FluxDown 实现差异**：iOS 端 FD 为 `mobile/FluxDown`（记忆/冻结）。这些限制说明 iOS 端的 BT 这类**长连接、多对多**协议不适合直接用 URLSession 后台模式（其模型是 HTTP 传输），后台能力需要 BackgroundTasks/前台时完成。FD iOS 端是否走 BT 会话或仅做轻量任务本 slice 未核对。
- **可检验预测**：iOS 端若在后台用「反复新建单一后台下载任务」的模式，应观察到启动延迟逐次累积（rate limiter）；改为单一 session + 多任务后累积效应消失。反例（延迟不累积）则说明该设备/系统未施加该限速。

### C4. NAS：机械盘空闲唤醒与网络文件系统

- **主张 1**：`hdparm -B` 设置 APM 级别（1–127，低值更省电/高值更好性能），`-S` 设置 standby(spindown) 超时（1–240 为 5 秒倍数＝5 秒–20 分钟；241–251 为 30 分钟单位；0＝禁用），`-C` 查询当前电源模式（active/idle/standby/sleeping）。
- **证据(locator)**：`S-DOC-OS-HDPARM`（-B 63-67；-C 88-92；-S 544-556；-y 675-678）。
- **主张 2**：NFS 单请求最大载荷 1,048,576 字节；未指定 rsize/wsize 时按双方能力协商取最大值；`nconnect` 最多 16 条连接；默认（未指定或 `async`）客户端延迟发送应用写入。
- **主张 3**：`soft`/`softerr` 在 retrans 次后以 EIO/ETIMEDOUT 失败，且文档明确「所谓 soft 超时在某些情况下会导致**静默数据损坏**」，仅当客户端响应性比数据完整性更重要时使用；默认为 `hard`（无限重试）。
- **证据(locator)**：`S-DOC-OS-NFS`（rsize 168-192；wsize 194-219；nconnect 309-322；async 1185-1190）；`S-DOC-OS-NFS-SOFT`（93-109）。
- **适用条件**：hdparm 需固件支持；NFS 参数随内核版本演进；文档建议在易丢包/UDP 环境裁剪 rsize/wsize。
- **与当前 FluxDown 实现差异**：NAS 场景下 FD 的下载目录若在 NFS 挂载点上，则「BitTorrent 的随机 piece 写入」与 NFS 的**顺序性差 + 默认延迟写 + 超时重传**相互放大；同时机械盘 spindown 与 BT 的持续小随机写天然冲突（每次落盘都可能唤醒盘）。FD 的 NAS 部署形态（本地盘 vs NFS/SMB 共享）本 slice 未核对。
- **可检验预测**：NAS 上若下载目录为 NFS 挂载，应观察到「写延迟长尾 + goodput 抖动」明显高于本地盘，且在把 `rsize/wsize` 裁剪到几帧后长尾下降；若磁盘长期 spindown 而 BT 仍在低速运行，应观察到周期性唤醒（`-C` 状态在 standby 与 active/idle 之间摆动）与首字节时延尖峰。

---

## D. 测量方法与可重复性

### D1. 时间同步与单向时延的可信度边界

- **主张**：单向时延的时钟相关不确定度为 `Esynch(t) + Rsource + Rdest`（两端同步误差上界 + 两端时钟分辨率），必须计入任何实现的误差分析；文档给出量级：GPS 可达数十微秒，普通 NTP 可能只到数毫秒，且后者取决于所用 NTP 代理之间延迟性质的**稳定与对称**（而对称性正是要测的对象）。主机时间与线上时间之差（host time vs wire time）需另计。
- **证据(locator)**：`S-DOC-OS-IPPM-OWD`（§3.5 321-333；§3.7.1 442-495；§3.7.2 497-528）。
- **补充主张**：NTP 的 offset 估计为 `theta = 1/2[(T2-T1)+(T3-T4)]`、delay 为 `delta = (T4-T1)-(T3-T2)`；路径不对称时 delta 可为负（文档举例 -6.4 ms），须 clamp 到不小于系统精度；根同步距离 `LAMBDA = EPSILON + DELTA/2` 定义为「所有原因造成的最大误差」。
- **证据(locator)**：`S-DOC-OS-NTP`（theta/delta 1296-1300 与 1326-1329；LAMBDA 2107-2110）。
- **适用条件**：跨主机比较只在有同步误差界时成立；计划 §5 已要求「计时统一用单调时钟、跨主机只在校时且有误差边界时比较」——本条给出该要求的具体误差项。
- **与当前 FluxDown 实现差异**：FD 侧的同机计时（`pause` 生效时延、首个有效字节）用单调时钟即可，不受本条约束；**只有当实验跨主机（如 NAS↔桌面、E3 远端）时**才需要 Esynch 项。FD 现有指标是否区分这两类计时，本 slice 未核对。
- **可检验预测**：若把「跨主机单向时延」用两台仅 NTP 同步（无 GPS）的机器测量，应出现毫秒级、随同步误差漂移的固定偏移；在同一被测路径上重复测量，该偏移的**日内漂移**应与 NTP 重同步事件相关。若观察不到漂移，则说明误差项被其它噪声掩盖，此时不应宣称毫秒级结论。

### D2. 校准与随机化：让 run 级结论可信

- **主张 1（校准）**：`measured value = true value + systematic error + random error`；若能确定系统误差应从报告值中扣除；报告应给出校准误差 e，使真值落在报告值 ±e 内的概率至少 95%。校准需至少数百次重复；**随机误差与测量负载相关**（并行测多条路径会增加中断/调度/磁盘 I/O 误差），因此除空载基线外还应在**现场负载**下校准。采样用伪随机 Poisson 过程以避免自同步效应。
- **证据(locator)**：`S-DOC-OS-IPPM-CAL`（§3.7.3 530-615；§3.8 636-670；Poisson 703-706、782-786）。
- **主张 2（随机化）**：随机化实验运行顺序用于防御无关外部因素（文档举例：若按固定「四低四高」顺序运行而昼夜环境温度又影响响应，会把环境效应错归因于因子）；重复（replication）用于估计同设置下的离散度并检验方差齐性。
- **证据(locator)**：`S-DOC-OS-NIST-RANDOM` §Randomization / §Replication。
- **适用条件**：校准需要已知或近似零时延的隔离网络段；NIST 该页是 2^k 全因子示例，**不是 AB/BA 交叉设计专章**，且未覆盖顺序效应（period effect）与 carryover。
- **与当前 FluxDown 实现差异**：计划 §6 要求「按随机 AB/BA 分块配对而非永远先 A 后 B」「1s 快照不是独立实验重复」。本条为该要求的依据：① 随机化对应顺序效应与时间漂移；② 重复的**单位是 run 而非采样点**；③ 报告需给校准误差而非只给均值。
- **可检验预测**：在 E1 的 A/B 两实现比较中，若按固定顺序跑 6 对，两实现的差值会随时间块系统性漂移（例如始终后跑者更快）；改为 AB/BA 交替后该漂移应基本消失。若交替后仍存在同向漂移，则说明存在未建模的单调时间效应（如磁盘预热、swarm 状态演化），此时结论须降级为「与时间不可分离」。

---

## 分层事实表（网络生态）

**本表所有分层均未在本次研究中取得样本。** 下表仅登记分层定义与「需要什么证据」，`覆盖状态` 一律为**未覆盖**；APNIC 分国数字只是广告样本的宏观覆盖率，**不是**任一层的 BT 可达性样本，也不作为该层的证据。

| 分层 | 覆盖状态 | 本次实际取得的样本 | 允许引用的宏观数据（仅作背景，不作该层证据） | 需要什么证据才算覆盖 |
|---|---|---|---|---|
| 国内·同运营商（电信/联通/移动/广电各自内部） | 未覆盖 | 无 | APNIC 分国表 CN 行（`S-DATA-APNIC-STATS`，广告样本） | 该接入运营商网络内的实测路径：接入侧 AS、上下行标称与基准能力、RTT/丢包窗口、BT 建连与首个有效字节时延 |
| 国内·跨运营商（电信↔联通↔移动等） | 未覆盖 | 无 | 同上 | 两端各自接入 AS 与实测互联路径、时段、RTT/丢包、peer 来源构成 |
| 教育网（CERNET/CERNET2 及校园网） | 未覆盖 | 无 | 无（本 slice 未取得任何教育网专门数据入口） | 校内出口策略、v4/v6 双栈状态、端口开放/映射情况、可测的跨境与国内路径 |
| 海外·东亚 | 未覆盖 | 无 | APNIC 分国表 HK/JP/KR/TW 行（广告样本） | 许可范围内自有/远端样本的实测路径与 peer 构成 |
| 海外·欧洲 | 未覆盖 | 无 | APNIC 分国表 FR/DE/GB/NL 行（广告样本） | 同上 |
| 海外·北美 | 未覆盖 | 无 | APNIC 分国表 US/CA 行（广告样本） | 同上 |
| 跨境（如 国内↔北美、国内↔欧洲、国内↔东亚） | 未覆盖 | 无 | 无 | 有许可的跨境实测：两端 AS、经过的 IX/中转、RTT 与丢包、UDP 与 TCP 分别的端到端结果、时段 |
| IPv4 公网 / NAT / CGN / IPv6 双栈（作为属性分层） | 未覆盖（无实测映射结果） | 无 | RIPE Atlas T&C 4.3 关于 CGN（RFC6598）地址可见于结果的条款（`S-DATA-RIPE-ATLAS-TC-DATA`，仅为规则而非样本） | 本机与对端的 NAT 映射/过滤实测、IPv6 地址与可达性实测、UDP 分端点结果 |
| 家庭路由器 / Wi-Fi / 有线 / TUN 代理（作为路径分层） | 未覆盖 | 无 | 内核 TUN/TAP 背压文档（`S-DOC-KERNEL-TUNTAP`，机制而非样本） | 接口与真实路径记录、链路质量、NAT/conntrack 能力、MTU/PMTU 线索 |
| public / private、热门 / 稀缺 / 高 churn 种子 | 未覆盖 | 无 | 无 | 有授权的合法内容与站点规则下的 swarm 观测（本 slice 未接触任何 tracker/swarm） |

**硬性禁止（本 slice 复述，后续执行同样适用）**：不得用 VPN/代理出口冒充某地的本地接入样本；不得因「某地区 IPv6 覆盖率高」或「某 AS 有 BGP 路径」就断言该层可达；不得索取密钥、付费数据或私有站点数据。缺海外/某运营商样本时必须写「未覆盖」，不得以宏观数字代替。

---

## network-env 缺口清单

1. **RIS 数据许可条款未取得**（`S-DATA-RIPE-RIS`、`S-DATA-RIPE-RIS-MRT` 均为 `许可: missing`）：RIPE RIS 概览页与 RIS 文档树内检索许可关键字无命中。→ 需从 RIPE NCC 政策/法务页或直接向 RIPE NCC 确认后方可引用 RIS 数据，**当前只能引用方法与格式**。
2. **CAIDA AS Relationships 的 AUA 文本未取得**（`S-DATA-CAIDA-ASREL` 为 `许可: missing`）：catalog 条目页为 JS 渲染未取回正文。→ 可参照同一 catalog 家族页面（pfx2as）指向的 `CAIDA Acceptable Use Agreement for Publicly Accessible Datasets`，但**在确认该数据集适用同一 AUA 之前不得据此声称已获授权**。
3. **RIPE Atlas 文档子页不可达**（`S-DATA-RIPE-ATLAS-DOCS` 为 `partial`）：`/docs/02-what-is-atlas/` 返回 404，未能取得测量类型清单、Credits 明细、探针覆盖统计的官方页。→ 需重新定位当前文档路径（VitePress 站点结构已变）后再引用平台能力。
4. **`IPv6 Preference` 列定义未核对**：`S-DATA-APNIC-STATS` 的第二个百分比列只从 `/ipv6/help` 图表标签确认名称为 `IPv6 Preference`，**语义未在任何获取页核对**。→ 引用该列前必须找到定义页。
5. **内核文档全部为滚动内容、无版本戳**：`S-DOC-KERNEL-*` 与 `S-DOC-OS-*`（man7 页面 colophon 除外）均未绑定具体内核/发行版版本；本机为 CachyOS 7.2.9-2（`uname -r`），但**未读取本机 `/proc/sys`、`/sys/block` 实际取值**，因此所有「默认值」主张都是文档默认而非本机实测值。
6. **`S-DOC-KERNEL-SYSFSBLOCK` 为 `partial`**：879 行的 stable ABI 文件仅读到前 300 行并用检索命中行，`rotational` / `scheduler` / `wbt_lat_usec` 的完整语义未逐条核对。→ 需要这些属性的确切语义（如 scheduler 可选值、wbt 与写延迟的换算）时应重新定位到 `Documentation/block/` 下的专页。
7. **机械盘 vs SSD 的顺序/随机性能权威依据缺失**：本 slice **未取得任何**厂商中立、可引用的权威文档来支撑「HDD 随机写远慢于顺序写、SSD 无寻道惩罚」这一常被当作前提的事实；`S-DOC-KERNEL-SYSFSBLOCK` 只给出可观测/可调属性，不给性能结论。→ 缺口登记；在任何实验里只能把盘型当作**协变量并实测**，不得引用博客推测或用文档推断性能差异。
8. **AB/BA 交叉设计的专章依据缺失**：`S-DOC-OS-NIST-RANDOM` 提供随机化与重复的通用依据，但**不是交叉设计专章**，未覆盖 period effect 与 carryover。→ 若正式协议需要引用 crossover 设计方法（含 washout、顺序效应的检验），需另找统计方法学来源；当前计划 §6 的「随机 AB/BA 分块配对」在文献依据上仍是**部分支撑**。
9. **移动端实测条件未确认**：`S-DOC-MOBILE-*` 全为平台规则文档，本 slice 未持有 Android/iOS 测试设备的任何读数；FD 移动端实际使用的后台 API、电池优化豁免状态、FGS 类型**均未核对**。→ 计划 §5 的「移动端后台成功率/温度/能耗」只能先写采集协议，不得生成数据。
10. **NAS 形态未确认**：FD 在 NAS 上的部署（本地盘 / NFS / SMB、机械盘型号、spindown 设置）本 slice 未核对；`S-DOC-OS-HDPARM`/`S-DOC-OS-NFS` 只提供机制与判据。
11. **国内网络状态与生态数据入口全部缺失**：本 slice 未取得任何国内运营商、教育网或国内测量平台的可引用数据入口（除 APNIC 广告样本的分国数字）。→ 计划 §4 的国内分层背景资料在该 slice 仍为空白；不得用 APNIC 覆盖率充数。
12. **RIPE Atlas / RIS 的时间范围与覆盖偏差未能定量化**：两处的官方页都未给出本 slice 可引用的「覆盖 × 时间 × 偏差」量化说明（Atlas 文档子页 404，RIS 页面无许可与覆盖统计）。→ 在 E3 里若使用这些数据作协变量，须先补齐。

---

## 与其他 slice 的接口约定（供合并时参考）

- 本 slice **不引用**任何 rqbit / libtorrent / transmission / aria2 / qBittorrent 源码行号，避免与 `rqbit-fd` / `refimpl` slice 的 `locator` 语义冲突。
- 与本 slice 直接相关的计划 §5 指标层：`交互与资源护栏`（B4/B5 提供主机侧干预点与判据）、`活跃 peer 与生态`（A1–A5 提供外部数据的**引用上限**）、`恢复与正确性`（B6 提供「预分配≠落盘」判据）、`NAS/移动端补充`（C1–C4 提供平台约束与判据）。
- 计划 §6 的 E2「NAT/IPv6/UDP 与竞争交互流」与 E0「观测语义审计」应消费本 slice 的 B1–B5、D1–D2；E3 的分层协变量应先消费本文件的「分层事实表」与缺口 11、12。
