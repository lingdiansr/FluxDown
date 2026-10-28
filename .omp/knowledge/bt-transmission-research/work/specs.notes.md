# specs slice — 机制说明（BEP wire/metadata、发现与地址族、可达性与传输、调度与恢复、MSE/PE、外部 RFC）

范围：只读规范正文（BEP 3/5/6/7/9/10/11/12/14/15/19/21/23/27/29/32/40/47/52/55、RFC 6817/4787/5382/6598、Vuze MSE）。全部 `retrieved_at=2026-10-17`。代码/实现的“现状差异”只引用本次任务冻结事实（FluxDown 常量与 pinned rqbit `peer_admission.rs`），未在本 slice 直接读取的 FluxDown 源码一律标 `[INFERENCE]`，需与 `RqbitFluxdown` slice 交叉核对。

每个条目格式：**主张 → 证据(locator) → 适用条件 → 与当前 FluxDown 实现差异 → 可检验预测**。locator 见 `specs.sources.csv` 对应 source_id 行。

---

## A. Wire 与元数据

### A1. v1 握手与 info-hash 绑定
- 主张：v1 握手固定为 `19 + "BitTorrent protocol" + 8 reserved + 20B info-hash + 20B peer_id`；任一侧不一致即断连。
- 证据：S-BEP-3 `peer protocol`。
- 条件：v1 连接；reserved 全零是基线，各扩展位按 BEP 占用。
- 差异：FluxDown/rqbit 走 librqbit `9.0.1`（pin `6f9d64b3…`）；reserved 位是否置（DHT/Fast/BEP10/BEP52-upgrade）需在 rqbit 侧核实 `[INFERENCE]`。
- 可检验预测：抓一段 v1 握手，第 2 字节应为 0x13，第 6–7 处 reserved 非零位应与协商成功的扩展集一致。

### A2. 块大小 16 KiB 与流水线
- 主张：request 的 length 事实上 16 KiB（2^14），请求更大的连接会被关闭；协议要求一次排队多个 request（pipelining）。
- 证据：S-BEP-3 `peer messages`；S-BEP-52 `peer messages`；BEP 10 `reqq`。
- 条件：v1/v2 均以 16 KiB 块为事实标准；v2 另有 hash request 层。
- 差异：rqbit 单任务 outbound 预算 `outbound_budget(budget,reserve)=budget.saturating_sub(reserve).max(1).min(budget)`，单任务上限 `BT_PEER_LIMIT_DEFAULT=256`；实际在途 request 数是否受 `reqq`/窗口约束需核实 `[INFERENCE]`。
- 可检验预测：在途 request 字节 ≈ 窗口内 peer 数 × 每 peer 并发请求数 ×16 KiB；若显著低于 BDP 则需要更大请求窗口。

### A3. v2/hybrid piece 层与 file tree
- 主张：v2 `piece length` 为 2 的幂且 ≥16 KiB，每个文件按 piece 边界对齐（末 piece 可短，形成 alignment gap）；`file tree` 用零长键存属性，`length` 表示文件；`pieces root` 为 16 KiB 块、分支因子 2 的 merkle 根（SHA2-256）；`piece layers` 在 info 之外且必须与 pieces root 一致。
- 证据：S-BEP-52 `info dictionary`、`file tree`、`pieces root`、`piece layers`。
- 条件：meta version=2；实现必须先检查 `meta version` 再校验其它字段。
- 差异：FluxDown 的 BT 任务是否支持 v2/hybrid 解析与双 hash 校验，本 slice 未核实，列缺口。
- 可检验预测：对同一 hybrid 种子，v1 与 v2 infohash 均应可用；多文件 hybrid 必含 BEP47 padding 文件以对齐 v1 piece。

### A4. hybrid 升级与兼容义务
- 主张：hybrid 种子的 `pieces` 与 `files`/`length` 必须描述同序同数据；多文件 hybrid 必须用 BEP47 padding 达成对齐；发起连接发送 sha1 infohash 时可在 reserved 最后一字节第 4 高位（0x10）标记支持新格式，对端可用新 infohash 升级；下载中须同时校验两种 piece 格式，不一致时 abort 或回退单一格式。
- 证据：S-BEP-52 `Upgrade Path`。
- 条件：同时部署 v1/v2 的 swarm。
- 差异：FluxDown/rqbit 是否做双 hash 校验与 0x10 位协商，需核实 `[INFERENCE]`；若不支持则只能停在 v1 子 swarm。
- 可检验预测：对 hybrid 种子分别以 v1/v2 演进，若内容一致，两条路径最终字节一致；对端若置 0x10，本端应能响应升级。

### A5. padding 文件语义
- 主张：`attr` 含 `p` 即 padding 文件，内容全零参与 piece hash；识别的客户端可不落盘，但必须仍能服务覆盖 padding 的请求；推荐路径 `[".pad","N"]`。
- 证据：S-BEP-47 `Padding files`。
- 条件：多文件对齐/去重场景。
- 差异：FluxDown 是否跳过 padding 落盘并保留请求服务能力，需核实 `[INFERENCE]`。
- 可检验预测：收到的 padding 区间请求应返回零字节且不产生磁盘写；本地磁盘不应出现 `.pad` 实体文件。

### A6. 元数据交换 ut_metadata
- 主张：magnet 下 metadata 以 16 KiB 块交换（`ut_metadata`，`metadata_size`），无全文者必须回 reject；未识别 msg id 必须忽略；可能对超量请求做 flood 保护。
- 证据：S-BEP-9 `metadata`、`extension header`、`request`/`data`/`reject`。
- 条件：BEP10 扩展协商成功。
- 差异：rqbit 有 `max_concurrent_metadata_reads = torrent_connect_limit.min(8)`（本 slice 冻结事实），与“拒绝超量”方向一致但具体阈值不同。
- 可检验预测：magnet 任务阶段应出现 request→data 的 16 KiB 块流；`metadata_size` 与实际 info 长度一致。
- 补充：`data` 消息在 bencoded 字典中带 `total_size`（语义同 `metadata_size`），消息号固定 `request=0 / data=1 / reject=2`，配合 `piece` 定位分片——这三者合起来才是 `ut_metadata` 的可核对字段集。

### A7. 扩展协商 reqq / upload_only
- 主张：`reqq` 为对端可承受的未决 request 数（libtorrent 默认 250）；`upload_only=1`（定义于 BEP21，不在 BEP10 正文）表示该 peer 不再下载。
- 证据：S-BEP-10 `handshake message`；S-BEP-21 `Extension Header`。
- 条件：BEP10 握手；两侧 ID 为本地值，0 表示禁用，后续握手增量生效。
- 差异：FluxDown/rqbit 是否发送/尊重 `upload_only` 影响做种任务被判为 partial seed，需核实 `[INFERENCE]`。
- 可检验预测：partial seed 会话的扩展握手应含 `upload_only=1` 且 announce 带 `event=paused`。

### A8. v2 测试向量与实现兼容记录的可得性（补漏）
- 主张：**BEP 52 正文本身未提供测试向量引用**。其页面全部外链仅为 BEP 0/6/7/15/23/29、`bep_0052_torrent_creator.py`、BitTorrent Economics PDF 与 Wireshark 页面；正文中的示例是 file tree/bencoded 文本示例（如 `fileA.txt` 的 bencode 串），不含带期望 hash 的 test vector。
- 证据：S-BEP-52（正文链接清单行）、S-BEP-52（`bep_0052_torrent_creator.py` 行）；实际检索：BEP52 HTML 的 `<a href>` 全集、`bep_0052_torrent_creator.py`。
- 可核对的“入口”分两类（均非 BEP 正文链接）：
  - 官方参考实现（BEP52 正文链接）：`https://www.bittorrent.org/beps/bep_0052_torrent_creator.py`（S-BEP-52，`type=code`）——给出生成逻辑，但无期望输出，**不是** test vector。
  - 官方测试向量（BEP52 作者在 libtorrent 博客发布）：v2-only `https://libtorrent.org/bittorrent-v2-test.torrent` 与 hybrid `https://libtorrent.org/bittorrent-v2-hybrid-test.torrent`（S-SPEC-1/S-SPEC-2，`type=dataset`；range 探针均返回 HTTP 206 / application/x-bittorrent），记载于 S-SPEC-3（libtorrent 博客）。
- 检索记录（未证实的入口一律不写入来源）：候选仓库 `https://github.com/bittorrent/bittorrent-v2-test-vectors` 探测返回 **HTTP 404**（不存在），故不出现在 CSV 中；确口的官方 test-vector 只有上述作者方测试种子。
- 与当前 FluxDown 实现差异：rqbit/FluxDown 是否用这些测试种子做过验证，未核实。
- 可检验预测：对 S-SPEC-1 的 btmh magnet 与本端算出的 v2 infohash 应一致；对 S-SPEC-2 的 v1/v2 两个 infohash 应分别落入两个 swarm 且共享同一份数据。

---

## B. 发现与地址族

### B1. tracker tiers
- 主张：`announce-list`（tiers）出现时忽略单个 `announce`；tier 顺序处理、tier 内 URL 洗牌一次后按序、成功 tracker 移到 tier 前部。
- 证据：S-BEP-12 `Metadata Extension`、`Order of Processing`。
- 条件：多 tracker 种子。
- 差异：FluxDown/rqbit 的 announce 调度是否实现 tier 前移，需核实 `[INFERENCE]`。
- 可检验预测：单 tier 内首选成功 tracker 在后续 announce 中保持首位。

### B2. 紧凑地址解析（IPv4 6 字节）
- 主张：紧凑 peer 为 **6 字节**：4 字节 IPv4 + 2 字节端口，均网络字节序；DHT 的 peer contact 同为 6 字节。
- 证据：S-BEP-23 `Overview`；S-BEP-5 `Contact Encoding`；S-BEP-15 `Announce`（v4 stride 6）。
- 条件：`compact=1` 或 UDP tracker；DHT values。
- 差异：解析长度错配（把 18 当 6）是常见互操作 bug；实现需按来源区分。rqbit 解析路径需核实 `[INFERENCE]`。
- 可检验预测：注入一段 6 字节紧凑串应恰好解析出 ⌈len/6⌉ 个 peer，且 IPv4 与端口与构造值一致。

### B3. 紧凑地址解析（IPv6 18 字节）
- 主张：IPv6 紧凑 peer 为 **18 字节**：16 字节 IPv6 + 2 字节端口；BEP7 `peers6`、BEP15 v6 响应 stride=18、BEP32 DHT `values` 均为 18 字节；BEP32 要求能解析 6/18 混排的 hybrid `values` 但不主动发。
- 证据：S-BEP-7 `Announce Response`；S-BEP-15 `IPv6`；S-BEP-32 `values`。
- 条件：v6 地址族连接/DHT v6。
- 差异：`nodes6`/`peers6`/`values` 是否在 FluxDown 侧被正确按 18 解析，需核实 `[INFERENCE]`；解析器按“除以 6”会错。
- 可检验预测：18 字节紧凑串应解析出 ⌈len/18⌉ 个 IPv6 peer；BEP32 混排输入必须不 panic。

### B4. DHT 请求/响应与 token
- 主张：KRPC 单包请求、单包响应、无重试；token=SHA1(IP‖secret)，secret 每 5 分钟轮换，接受 10 分钟内的旧 token；`get_peers` 返回 values 或 nodes + token；`announce_peer` 必须回同 IP 的 token，`implied_port=1` 时用 UDP 源端口。
- 证据：S-BEP-5 `KRPC Protocol`、`Overview`、`get_peers`、`announce_peer`。
- 条件：UDP；节点须先 `find_node` 自举或从 `nodes`/PORT 消息获得种子节点。
- 差异：BEP32 的 `want`/`nodes6` 是否被实现，决定 v6 DHT 能力。rqbit DHT 实现细节属 `RqbitFluxdown` slice。
- 可检验预测：无 `` `want`` 的 IPv4 `find_node` 响应不得含 18 字节项；`announce_peer` 用错误 token 应被拒。

### B5. PEX 消息结构与合规
- 主张：`ut_pex` 载荷含 `added`/`added.f`/`added6`/`added6.f`/`dropped`/`dropped6`；仅成功建连后入 `added`，断连后必须发 `dropped`，只增不减属非合规；≤1 条/分钟；非首条消息 added 与 dropped 各 ≤50；默认地址族活连接 <25 时放宽 liveness（保留最近 25 条）。
- 证据：S-BEP-11 `Protocol Extension`（含 flags 表）、`Filling underpopulated lists`、`Security Considerations`。
- 条件：BEP10 协商成功；PEX 数据视为不可信。
- 差异：rqbit 的 PEX 实现与 50/25 阈值是否遵守需核实 `[INFERENCE]`。
- 可检验预测：一分钟内收到的 `ut_pex` 应 ≤1 条；连续捕捉不应出现只 added 不 dropped 的单向膨胀。

### B6. LSD
- 主张：LSD 用 SSDP 式 HTTP-over-UDP 组播 `239.192.152.143:6771` / `[ff15::efc0:988f]:6771`；报文 `BT-SEARCH * HTTP/1.1`，含 Host/Port/Infohash/cookie；每 5 分钟每接口一次、每分钟 ≤1；对端 IP 取 UDP 源地址。
- 证据：S-BEP-14 `Protocol`、`Usage`。
- 条件：同 LAN；私有种子上必须禁用（BEP27）。
- 差异：FluxDown 是否启用 LSD、是否在私有时关闭，需核实 `[INFERENCE]`。
- 可检验预测：LAN 内同 info-hash 会话应互相发现；私有任务下组播报文数应为 0。

### B7. UDP tracker 超时/重试
- 主张：无响应则按 15·2^n 秒重传，n=0…8（3840 s 上限）；connection_id 客户端可用 1 分钟、tracker 接受 2 分钟；scrape 单次约 ≤74 个 info-hash，无全量 scrape。
- 证据：S-BEP-15 `Time outs`、`UDP connections / spoofing`、`Scrape`。
- 条件：UDP tracker。
- 差异：FluxDown/rqbit 的超时与重试是否与 15·2^n 对齐，影响“发现慢”归因，需核实 `[INFERENCE]`。
- 可检验预测：连续失败时应观察到指数退避的时间戳间隔（15/45/105/…）。

---

## C. 可达性与传输

### C1. uTP 头/类型/选择性 ACK
- 主张：v1 头含 type/ver/extension/connection_id + 3 个 32 位时间/窗口字段 + seq_nr/ack_nr；类型 ST_DATA=0/ST_FIN=1/ST_STATE=2/ST_RESET=3/ST_SYN=4；扩展 1 为选择性 ACK（≥32 位、32 的倍数，首 bit=ack_nr+2）。
- 证据：S-BEP-29 `header format`、`type`、`Selective ACK`。
- 条件：UDP 上的 uTP；SYN 返回连接 id+1。
- 差异：rqbit 是否启用 SACK 与 uTP（MSE+uTP 已在 FluxDown 调优中启用，见 memory）——具体开关归 `RqbitFluxdown`。
- 可检验预测：丢包场景应出现 SACK 扩展与 3 重复 ACK 触发的重传。

### C2. uTP 拥塞控制与 LOSS_TARGET 等价物
- 主张：`CCONTROL_TARGET`=100 ms，`base_delay`=最近 2 分钟滑动最小，`off_target=CCONTROL_TARGET-our_delay` 驱动窗口增长；丢包时 `max_window *= 0.5`；超时 `max(rtt+rtt_var*4,500)` ms、初始 1000 ms、连续超时加倍，并将包大小降为 150 字节重启。
- 证据：S-BEP-29 `congestion control`、`packet loss`、`timeouts`、`packet sizes`。
- 条件：延迟型控制；目标为“0 缓冲占用”，让位于交互流量。
- 差异：LEDBAT(RFC6817) 是独立实验规范，其 TARGET 也是 100 ms，但**不能**用 RFC 文本替代 BEP29 的实现细节，反之亦然。
- 可检验预测：带载 vs 空载的单向时延差应≈100 ms 上限；丢包事件后速率应骤降为一半。

### C3. uTP 的分组类型与 ACK 语义（补充）
- 主张：uTP 的 ACK 是包级（非字节级），无法 repackage 重传；`wnd_size` 为接收缓冲区剩余字节，约束在途。
- 证据：S-BEP-29 `overview`、`wnd_size`。
- 条件：uTP 端点。
- 差异：与 TCP 字节级重传不同，uTP 丢包即整包重发，开销更大。
- 可检验预测：uTP 丢包重传的字节数应为整包整数倍，而非任意区间。

### C4. BEP55 holepunch 流程与“何时不适用”
- 主张：`ut_holepunch` 载荷 `msg_type/addr_type/addr/port/err_code`；type 0x00 rendezvous /0x01 connect /0x02 error；错误码 0x01 NoSuchPeer/0x02 NotConnected/0x03 NoSupport/0x04 NoSelf；收到 connect 双方各自发起 uTP 连接；发起方未在握手中声明支持时中继必须静默忽略；已连接或目标自持时必须静默忽略。
- 证据：S-BEP-55 `Protocol Extension`、`Implementation notes`。
- 条件：需 BEP10+BEP29；中继须与目标已连接。
- 差异：FluxDown/rqbit 是否实现 ut_holepunch（PEX flag 0x08），本 slice 未核实，列缺口。
- 可检验预测：两侧均在 NAT 后的会话，若中继可用应观察到 rendezvous→connect→uTP SYN 序列；无支持方应无报文往返。

### C5. NAT UDP/TCP 行为要求（穿透前提）
- 主张：UDP：EIM/ADM/APDM 三映射、EIF/ADF/APDF 三过滤、mapping timer ≥2 min、outbound refresh 必须为真、hairpinning 必须支持、行为必须确定；TCP：REQ-1 EIM、REQ-2 支持 simultaneous-open、REQ-4 未请求 SYN 至少 6 s 内不得响应、REQ-5 established idle ≥2h4m、REQ-7 禁止 port overloading。
- 证据：S-RFC-4787 `4.1/4.3/5/6/8/9`；S-RFC-5382 `4.1/4.2/4.3/5/6/7.1`。
- 条件：家用/运营商 NAT；这些是“应当满足”的 BCP，不代表真实设备都满足。
- 差异：真实 NAT 常偏离（对称 NAT、短计时器）；BEP55/BEP29 的可达性收益取决于对端 NAT 配合。
- 可检验预测：对同一 UDP 端点向不同目标发包，若外部端口不变则判定 EIM；TCP simultaneous-open 在合规 NAT 后应可建立。

### C6. CGN 地址空间与可达性硬约束
- 主张：`100.64.0.0/10` 为共享地址空间；跨运营商边界不得转发且须 ingress 过滤；同范围跨接口须能转换；CGN 下部分 P2P 无法开放入站端口而无法做种。
- 证据：S-RFC-6598 `7 IANA Considerations`、`4`、`5.2`。
- 条件：运营商 CGN；影响入站可达。
- 差异：FluxDown 若识别到 CGN 地址应正确判定“不可公开可达”，避免误报端口映射成功（与“开启 UPnP≠端口可达”一致）。
- 可检验预测：CPE 外部地址落在 100.64/10 时应判定为不可全球可达，入站连尝试应失败/需 relay。

---

## D. 调度与恢复

### D1. 启动 ramp-up 与 Allowed Fast
- 主张：新 peer 需要数分钟 ramp-up；Allowed Fast（`<len=0x0005><op=0x11><index>`，k=10）在 choked 时也服务指定 piece；集合由 `ip & 0xFFFFFF00` + infohash 迭代 SHA1、`index=y%sz` 生成，且**无 IPv6 规定**。
- 证据：S-BEP-6 `Allowed Fast`、`Allowed Fast Set Generation`。
- 条件：Fast Extension 双方启用。
- 差异：算法仅 IPv4，IPv6 端需另行定义（实现常退化）。
- 可检验预测：给定 infohash/IP/sz，本端生成的 allowed-fast 集应与规范算例一致（IPv4 向量）。

### D2. Reject 语义与请求守恒
- 主张：Fast/v2 下每个 request 恰好一个响应（piece 或 reject）；choke 时须拒绝所有请求；收到未请求的 piece 必须断连。
- 证据：S-BEP-6 `Modifications…`/`Reject Request`；S-BEP-52 `peer messages`。
- 条件：Fast Extension 或 v2。
- 差异：请求计数的“已发/已答/已取消”口径必须守恒，否则会出现“请求泄漏”误判；具体计数器归 `RqbitFluxdown`。
- 可检验预测：任一连接的 request 发出数应等于 piece+reject 收到数（忽略在途）。

### D3. endgame 与重复字节
- 主张：endgame 向所有 peer 请求所有缺失 piece，到达即向其他 peer 发 cancel；重复传输是设计内的代价。
- 证据：S-BEP-3 `'cancel' messages`；S-BEP-52 `cancel messages … must be responded to`.
- 条件：仅下载末期。
- 差异：`useful/wire` 效率指标会被 endgame 拉低，属预期而非故障。
- 可检验预测：尾部阶段 cancel 数显著上升，重复 piece 字节 >0。

### D4. 私有种子的发现通道关闭
- 主张：`private=1` 时只能向私有 tracker announce、只连其返回的 peer；多 tracker 时一次一个，切换时断开所有当前 peer 且不得 DHT/PEX/LSD 发现。
- 证据：S-BEP-27 `intro`。
- 条件：私有种子的硬约束。
- 差异：FluxDown 若对私有种仍启用 DHT/PEX 即违规，属需核实的红线 `[INFERENCE]`。
- 可检验预测：私有任务运行期 DHT get_peers/PEX/LSD 报文数应为 0。

### D5. Web seed 行为
- 主张：`url-list` 在主区（非 info）；URL 以 `/` 结尾时追加 `name`；多文件时为根目录 + `name` + `path/file`；HTTP 可用 byte-range、FTP 只能指定起点；piece SHA 不匹配必须关闭并丢弃该 URL，收到 “busy” 不必丢弃。
- 证据：S-BEP-19 `Metadata Extension`、`Multi-File Torrents`、`Client Implementation Overview`、`HTTP and FTP Optimizations`。
- 条件：存在 web seed。
- 差异：FluxDown 是否有 web seed 支持与校验失败丢 URL 逻辑，本 slice 未核实，列缺口。
- 可检验预测：给坏的 web seed 应触发断连+丢弃；对同一文件 web seed 与 BT 片混下时最终校验一致。

### D6. Canonical Peer Priority
- 主张：`priority = crc32-c(sort(masked_client_ip, masked_peer_ip))`；IPv4 默认掩码 `FF.FF.55.55`、同 /16 `FF.FF.FF.55`、同 /24 `FF.FF.FF.FF`；IPv6 默认 `FFFF:FFFF:FFFF:5555:…`、/48、/56 逐级收窄；IP 相同则用端口；示例 `crc32-c(624C14007BD50000)=ec2d7224`；仅在任务内使用。
- 证据：S-BEP-40 `Overview`、`Examples`、`Notes`。
- 条件：用于分散连接建立、缓解 DDoS。
- 差异：rqbit 是否按 BEP40 排序拨号，影响“公平拨号”能力，归 `RqbitFluxdown`。
- 可检验预测：对给定 IP 对，本端算出的优先级应等于 `ec2d7224` 型测试向量。

### D7. v2 hash 请求与 choke 解耦
- 主张：`hash request`(21)/`hashes`(22)/`hash reject`(23) 可在 choke 状态下发送；对 choked peer 的 hash 请求可单独限速；紧跟某 chunk 请求且本端服务该 chunk 时，不得拒绝 base layer=0 的 hash 请求。
- 证据：S-BEP-52 `peer messages`。
- 条件：v2 连接。
- 差异：FluxDown 若不支持 v2 则此机制不适用。
- 可检验预测：choke 状态下 v2 hash 交换仍应发生。

---

## E. MSE / PE

### E1. MSE 非 BEP
- 主张：MSE（aka PHE）是 Vuze wiki 上的规范（version 1.0 frozen），**不是编号 BEP**；BEP 索引未列任何 MSE/PE。
- 证据：S-MSE-1 `title`；S-BEP-0 索引。
- 条件：协议混淆/加密。
- 差异：文档标注时必须写“非 BEP 编号规范”。canonical `wiki.vuze.com` 已 404，需用归档快照。
- 可检验预测：任何引用若把 MSE 标成 BEP 编号即为错误。

### E2. MSE 密钥交换与协商位
- 主张：DH 768-bit safe prime、G=2，私钥 ≥128 bit（推荐 160）；VC=8 字节 0x00；`crypto_provide`/`crypto_select` 为 32 位位域，0x01=plaintext、0x02=RC4；SKEY 对 BT 应取 torrent info hash；RC4 前 1024 字节丢弃。
- 证据：S-MSE-1 `DH Parameters`、`Constants/Variables`、`Functions`。
- 条件：发起方需提供全部支持方法，响应方置单一选择位；未知位忽略。
- 差异：FluxDown 调优已启用 MSE（memory），实际 crypto 模式与 SKEY 选取需核实 `[INFERENCE]`。
- 可检验预测：抓到的第 3 步应含 `HASH('req2',SKEY) xor HASH('req3',S)`；错 SKEY 应在 628 字节内被终止。

### E3. MSE 协商失败的可见现象
- 主张：握手在多种条件下被判无效并可早断：A 在 30 s 内 <96 字节、>608 字节、628 字节内未见正确 S hash、无支持的 SKEY hash、VC 解码失败、无受支持 crypto_provide 或位域为 0。
- 证据：S-MSE-1 `Optional early termination conditions`。
- 条件：fail-fast 策略。
- 差异：表现为“加密连接永不建立/被立即断开”，容易误判为防火墙；需在诊断中区分 MSE 协商失败 vs TCP 失败（观察计划中“握手阶段”口径）。
- 可检验预测：明文-only 客户端连接 require-crypto 对端应在第 4/5 步前被断。

---

## F. 「规范化义务 → FluxDown/rqbit 现状差异 → 可检验预测」表

| # | 规范化义务（source_id） | FluxDown/rqbit 现状（本 slice 已知/推断） | 可检验预测 |
|---|---|---|---|
| 1 | v1 握手 info-hash 必须一致（S-BEP-3） | rqbit 9.0.1 走标准握手，未核实 reserved 位组合 | 抓包 reserved 非零位 == 已协商扩展集 |
| 2 | 16 KiB 块 + pipelining（S-BEP-3/52, BEP10 reqq） | 单任务预算 `outbound_budget=budget.saturating_sub(reserve).max(1).min(budget)`，上限 256 | 在途 request 字节 ≈ peers×并发×16 KiB，且受 BEP10 协商值约束 |
| 3 | v2/hybrid 双格式校验（S-BEP-52） | 是否支持 v2/hybrid 未核实（缺口） | hybrid 种子 v1/v2 均可达且最终字节一致 |
| 4 | padding 不落盘但须服务请求（S-BEP-47） | 未核实（缺口） | 磁盘无 `.pad` 实体；padding 请求返回零字节 |
| 5 | `ut_metadata` 无全文回 reject（S-BEP-9） | rqbit `max_concurrent_metadata_reads=torrent_connect_limit.min(8)` | 无全文 peer 对 request 回 reject |
| 6 | 紧凑地址 6/18 字节按来源解析（S-BEP-5/7/15/23/32） | 解析正确性未核实（缺口） | 6 字节→⌈n/6⌉，18 字节→⌈n/18⌉，混排不 panic |
| 7 | DHT token 5/10 分钟作用域（S-BEP-5） | 归 `RqbitFluxdown` | 旧 token 在 10 min 内应被接受 |
| 8 | PEX added↔dropped 配对、≤1/min、≤50（S-BEP-11） | 未核实 | 1 min 内 ≤1 条；无单向 added 膨胀 |
| 9 | 私有种关 DHT/PEX/LSD（S-BEP-27） | 红线，未核实 | 私有任务 DHT/PEX/LSD 报文为 0 |
| 10 | uTP 100 ms 目标、丢包减半、150 B 下限重启（S-BEP-29） | MSE+uTP 已启用（memory） | 带载-空载时延差≈100 ms；丢包后速率减半 |
| 11 | holepunch rendezvous/connect/error 语义（S-BEP-55） | 是否实现未核实（缺口） | 双侧 NAT 后可观察 rendezvous→connect→SYN |
| 12 | NAT 要求 EIM/ADF/同时开/6s 静默（S-RFC-4787/5382） | 依赖对端 NAT | 同端点不同目标外部端口不变= EIM |
| 13 | CGN 100.64/10 不可公开可达（S-RFC-6598） | 可达性判定未核实 | CPE 地址落在 CGN 应判不可全球可达 |
| 14 | MSE 非 BEP、DH+RC4、协商失败可早断（S-MSE-1） | MSE 已启用（memory） | 第 3 步含 XOR 校验值；错 SKEY 早断 |
| 15 | BEP40 优先级测试向量（S-BEP-40） | 归 `RqbitFluxdown` | 给定 IP 对算出 `ec2d7224` 型值 |
| 16 | BEP21 `upload_only`+`event=paused`（S-BEP-21） | 是否发送未核实 | partial seed 会话含 upload_only=1 且 announce paused |
| 17 | v2 测试向量可得性（S-BEP-52/S-SPEC-1..3） | BEP52 正文无 test vector；官方入口为 libtorrent 测试种子 | 解析 S-SPEC-1/S-SPEC-2 得到的 infohash 应等于其 magnet 值 |

---

## G. specs 缺口清单

1. **v2/hybrid 支持未知**：FluxDown/rqbit 是否解析 `meta version=2`、校验 `piece layers`、处理 hash request 21/22/23 与 0x10 升级位，本 slice 未读实现源码 → 需 `RqbitFluxdown` 确认。BEP52 为 Draft，部署面待估。
2. **padding 落盘行为未知**：是否跳过 `.pad` 写盘、是否仍服务覆盖请求（BEP47），未核实。
3. **Web seed 支持未知**：`url-list` 解析、byte-range、SHA 失败丢 URL（BEP19）未核实。
4. **holepunch 支持未知**：`ut_holepunch` 与 PEX flag 0x08 是否实现（BEP55）未核实。
5. **LSD 启用与私有关闭未知**：BEP14 组播报文在 FluxDown 是否发生、私有时是否禁用，未核实。
6. **UDP tracker 重试曲线未知**：是否严格 15·2^n、connection_id 1/2 分钟作用域（BEP15）未核实。
7. **DHT/节点解析长度鲁棒性未知**：6/18 字节混排、`nodes6`/`want`（BEP32）处理正确性未核实。
8. **allowed-fast 集合实现未知**：是否按 BEP6 规范算法（IPv4-only）生成，未核实。
9. **`upload_only`/partial-seed 语义未知**：BEP21 握手与 announce 行为未核实。
10. **BEP 索引状态非部署率**：BEP0 索引 Last-Modified 2018，Status 仅编辑状态；不得据此推断实际部署度。多条 BEP 为 Draft/Accepted，真实客户端支持率需另行测量。
11. **MSE canonical 入口缺失**：`wiki.vuze.com` 返回 HTTP 404（见 S-MSE-2），只能依赖 Internet Archive；MSE 无编号规范，版本号“1.0 frozen”为 wiki 自述。
12. **规范字段格式歧义（已知一处）**：BEP11 正文把 `dropped` 描述为 “IPv6 compact format”，与 `dropped`(IPv4)/`dropped6`(IPv6) 的字段命名冲突；已按字面记录并标 limitations。
13. **交付格式列数（已定案）**：任务书面曾写“严格 16 列”，但枚举字段名为 15 个；Main 已确认“16 列”为笔误。本文件按枚举的 **15 列** 输出（与计划及本仓库 schema 一致）。
14. **BEP 52 正文未提供测试向量引用（补漏确认）**：BEP52 页面外链全集只有 BEP 0/6/7/15/23/29、`bep_0052_torrent_creator.py`、BitTorrent Economics PDF、Wireshark，无 test-vector 仓库或兼容性记录引用。实际检索过的入口：(a) BEP52 正文（含其 `bep_0052_torrent_creator.py`，无期望输出，非向量）；(b) 候选仓库 `https://github.com/bittorrent/bittorrent-v2-test-vectors` → HTTP 404（不存在，故不列为来源）；(c) BEP52 作者 libtorrent 博客给出的官方测试种子 v2-only / hybrid（已登记为 S-SPEC-1/S-SPEC-2，经 S-SPEC-3 佐证）。未编造任何仓库名。
