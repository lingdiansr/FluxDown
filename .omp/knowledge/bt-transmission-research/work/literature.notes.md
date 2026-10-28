# literature slice — 机制说明与提取笔记

范围：为「BT 传输体验」研究包提供**学术与规范文献证据**。source_id 前缀统一 `S-PAPER-`。
配套数据：`work/literature.sources.csv`（78 数据行 / 30 个独立来源；其中 14 个来源 `access_status=full_text`，共 62 条带 locator 的 claim 行）。
所有 `retrieved_at` = 2026-10-17。**未读全文的条目一律标 `metadata_only`，其 claim 只描述题录层信息，不写实验结论。**

本 slice 只做案头检索与本机只读抓取；未运行任何下载、announce、DHT 爬取、握手、抓包或网络整形。

---

## 1. 检索记录（完整查询式 + 入口 + 日期）

入口说明：`web_search` = 本会话检索引擎（Bing/DDG 混合后端，返回题录+摘要+原始 URL）；
`read` = 直接抓取原始出版页/作者页/机构仓储 PDF（markit 转文本）；`arXiv`/`USENIX`/`ACM DL`/`IEEE`/`HAL`/`Duke/KTH/UMass/MPI` 镜像 = 原始出版方或作者/机构仓储。
HAL 源（`inria.hal.science`）在本日多次 **ECONNRESET**，改用 arXiv / 作者镜像替代（见 S-PAPER-Legout2006 的 limitations）。

| # | 查询式（原样） | 入口 | 日期 | 命中用于 |
|---|---|---|---|---|
| Q1 | `Incentives Build Robustness in BitTorrent Cohen 2003 workshop economics peer-to-peer` | web_search | 2026-10-17 | 种子 3 |
| Q2 | `Rarest First and Choke Algorithms Are Enough Legout Urvoy-Keller Michiardi` | web_search | 2026-10-17 | 种子 1 |
| Q3 | `Analyzing and Improving a BitTorrent Network's Performance Mechanisms INFOCOM 2006` | web_search | 2026-10-17 | 种子 2 |
| Q4 | `Dissecting BitTorrent Five Months in a Torrent's Lifetime Izal IPTPS` | web_search | 2026-10-17 | 种子 4 |
| Q5 | `BitTorrent request pipeline disk bottleneck performance model` | web_search | 2026-10-17 | 主题「流水线与资源」（无直接命中，见缺口） |
| Q6 | `LEDBAT uTP congestion control evaluation low priority latency Carofiglio` | web_search | 2026-10-17 | 主题「TCP/uTP 与混合流」 |
| Q7 | `Understanding Churn in Peer-to-Peer Networks Stutzbach Rejaie IMC 2006` | web_search | 2026-10-17 | 主题「swarm 动态与完成时间」 |
| Q8 | `Multi-perspective Analysis of Carrier-Grade NAT Richter IMC 2016` | web_search | 2026-10-17 | 主题「连接性与生态」 |
| Q9 | `Qiu Srikant Modeling and performance analysis of BitTorrent-like peer-to-peer networks SIGCOMM 2004` | web_search | 2026-10-17 | 主题「流水线与资源」 |
| Q10 | `bufferbloat measurement controlled delay CoDel latency paper` | web_search | 2026-10-17 | 主题「TCP/uTP 与混合流」（AQM） |
| Q11 | `Taming the Torrent system for reducing cross-ISP P2P traffic Choffnes Bustamante` | web_search | 2026-10-17 | 主题「连接性与生态」 |
| Q12 | `network measurement reproducibility empirical study IMC ACM artifacts` | web_search | 2026-10-17 | 主题「自适应与验证方法」 |
| Q13 | `Copa practical delay-based congestion control NSDI 2018 Arun Balakrishnan` | web_search | 2026-10-17 | 主题「TCP/uTP 与混合流」 |
| Q14 | `reproducibility of empirical networking research P2P measurement repeatability bias paper` | web_search | 2026-10-17 | 主题「自适应与验证方法」 |
| Q15 | `Ford Srisuresh Kegel NAT traversal state of the art peer-to-peer TCP hole punching measurement` | web_search | 2026-10-17 | 主题「连接性与生态」 |
| Q16 | `Muscariello Rossi Testa Valenti LEDBAT the new BitTorrent congestion control protocol 2010 conference proceedings` | web_search | 2026-10-17 | 核实 LEDBAT 会议出处 |
| Q17 | `P4P Provider Portal for Applications SIGCOMM 2008 Xie` | web_search | 2026-10-17 | 主题「连接性与生态」（题录） |
| Q18 | `Pouwelse Garbacki Epema Sips BitTorrent P2P file-sharing system measurements and analysis IPTPS 2005` | web_search | 2026-10-17 | 题录核对 |
| Q19 | `Ford Srisuresh Kegel Peer-to-Peer Communication Across Network Address Translators USENIX 2005` | web_search | 2026-10-17 | 题录核对 |
| Q20 | `Gkantsidis Rodriguez Network Coding for Large Scale Content Distribution INFOCOM 2005 Avalanche` | web_search | 2026-10-17 | 反例（网络编码） |
| Q21 | `Bindal Cao Chan Medved Improving Traffic Locality in BitTorrent via Biased Neighbor Selection ICDCS 2006` | web_search | 2026-10-17 | 题录核对 |
| Q22 | `Clustering and Sharing Incentives in BitTorrent Systems Legout Liogkas Kohler Zhang SIGMETRICS 2007` | web_search | 2026-10-17 | 题录核对 |
| Q23 | `delicate tradeoffs in BitTorrent-like file sharing protocol design ICNP 2006 authors` | web_search | 2026-10-17 | 题录核对 |
| Q24 | `A Performance Study of BitTorrent-like Peer-to-Peer Systems Guo Duan Chen Fu JSAC 2007` | web_search | 2026-10-17 | 题录核对（纠正作者） |
| Q25 | `Leonard Rai Loguinov On lifetime-based node failure and stochastic resilience of decentralized peer-to-peer networks IEEE/ACM ToN` | web_search | 2026-10-17 | 题录核对 |
| Q26 | `BitTorrent disk I/O bottleneck piece verification upload throughput measurement study` | web_search | 2026-10-17 | 主题「流水线与资源」（**无直接命中**，见缺口） |
| Q27 | `Aggarwal Feldmann Scheideler Can ISPs and P2P users cooperate for improved performance CCR 2007` | web_search | 2026-10-17 | 题录核对 |

直接 `read`（原始出版页/仓储 PDF）：
- `https://www.cs.princeton.edu/courses/archive/fall17/cos561/papers/BitTorrent03.pdf`（Cohen 2003）
- `https://arxiv.org/abs/cs/0609026` + `https://arxiv.org/pdf/cs/0609026`（Legout 2006）
- `https://www.usenix.org/conference/nsdi-07/...` + `https://www.usenix.org/legacy/event/nsdi07/tech/piatek/piatek.pdf`（Piatek 2007）
- `https://inria.hal.science/inria-00000156v3/document`（Understanding BitTorrent, HAL）→ **ECONNRESET，未采用**
- `https://inria.hal.science/inria-00091678/document`（种子 1 HAL 镜像）→ **ECONNRESET，未采用**
- `https://courses.cs.duke.edu/fall25/compsci512/internal/readings/bittorrent.pdf`（Izal 2004）
- `https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/infocom06.pdf`（Bharambe/Herley/Padmanabhan 2006）
- `http://conferences.sigcomm.org/sigcomm/2004/papers/p444-qiu1.pdf`（Qiu & Srikant 2004）
- `http://www.barsoom.org/papers/imc-2006-churn.pdf`（Stutzbach & Rejaie 2006）
- `https://nickduffield.net/download/papers/ton-2007-sampling.pdf`（Stutzbach et al. 2007）
- `https://arxiv.org/pdf/0907.3874`（Cuevas et al. 2011）
- `https://people.cs.umass.edu/~phillipa/CSE390/choffnes08.pdf`（Choffnes & Bustamante 2008）
- `https://www.kth.se/social/files/5527dbf2f276543890e0c409/LEDBAT.pdf`（Rossi et al. 2010）
- `https://www.usenix.org/system/files/conference/nsdi18/nsdi18-arun.pdf`（Arun & Balakrishnan 2018）
- `https://www.prichter.com/imc176-richterA.pdf`（Richter et al. 2016）
- `https://arxiv.org/pdf/2307.09958`（Sermpezis et al. 2023）
- `https://cacm.acm.org/practice/controlling-queue-delay`（CoDel CACM）→ **HTTP 403，未采用**

种子核对结论（现场核对作者/年份/原始出处）：
1. **Rarest First and Choke Algorithms Are Enough** = Legout / Urvoy-Keller / Michiardi，**IMC 2006**（ACM DOI `10.1145/1177080.1177106`，arXiv `cs/0609026`）。HAL 镜像失败，改用 arXiv PDF 取得全文。✓
2. **Analyzing and Improving a BitTorrent Network's Performance Mechanisms** = Bharambe / Herley / Padmanabhan，**IEEE INFOCOM 2006**（Barcelona）。作者页（cormac.herley.org）核对题名与年份，全文取自 Microsoft Research 镜像。✓
3. **Incentives Build Robustness in BitTorrent** = Bram Cohen，**2003-05-22**，First Workshop on Economics of Peer-to-Peer Systems；无 DOI。✓
4. **Dissecting BitTorrent: Five Months in a Torrent's Lifetime** = Izal / Urvoy-Keller / Biersack / Felber / Al Hamra / Garcés-Erice，**PAM 2004**（Antibes Juan-les-Pins，2004-04）。✓

---

## 2. 精读提取表（14 篇 full_text）

每篇固定提取：研究问题 | 模型/实验方法 | 代码与数据可得性 | 样本与观测年代 | 客户端/协议/网络/工作负载 | 对照与重复 | 效应与不确定性 | 作者局限 | **反证** | 迁移到当前 FluxDown 所需前提。

### 2.1 S-PAPER-Cohen2003 — Incentives Build Robustness in BitTorrent（workshop 2003）
- 研究问题：如何用 tit-for-tat 激励在无中心调度的前提下达到 Pareto 效率与稳健性。
- 方法：设计文档，无实验/测量。
- 数据可得性：无数据；无实现代码随文。
- 年代/样本：2003 年 mainline 默认行为。
- 客户端/协议：mainline BitTorrent；TCP；固定 4 unchoke + 单 optimistic unchoke；piece/block 256 kB/16 kB。
- 对照/重复：无。
- 效应/不确定性：给出设计默认（5 个 pipelined 请求、10 s rechoke、30 s optimistic、>60 s snub、元数据开销 <0.1%），但无量化验证。
- 作者局限：自述“tracker 带宽开销”为主要扩展瓶颈，未做实测。
- **反证**：Piatek 2007 与 Bharambe 2006 均显示该激励可被策略客户端或不公平带宽分布削弱（见 §4）。
- 迁移 FD 前提：这些默认值是“协议层习惯”，不是 FD 当前配置；FD 的 peer/连接上限由自身常量与 rqbit 准入决定。

### 2.2 S-PAPER-Legout2006 — Rarest First and Choke Algorithms Are Enough（IMC 2006）
- 研究问题：rarest-first 与 choke 算法是否值得被 source/network coding 或 bit-level tit-for-tat 取代。
- 方法：插桩 mainline v4.0.2，在 26 个真实 torrent 上各跑 8 h（1–3 次/种子）。
- 数据可得性：无公开数据集；镜像（HAL）不可达，改用 arXiv 全文。
- 样本/年代：26 torrent，2005-06 至 2006-05；单个插桩客户端；上行限速 20 kB/s。
- 客户端/协议：mainline 4.0.2（含新版 seed-state choke）；TCP；真实公网 swarm。
- 对照/重复：每 torrent 单次结果，多运行时仅用于校准；作者自述无法做统计重复。
- 效应/不确定性：rarest-first 达到接近理想熵；v4.0 seed-state choke 修复了旧版公平性问题；endgame 影响小。
- 作者局限：单客户端插桩、torrent 集有限、无法重复实验。
- **反证**：Bharambe 2006 的仿真得出 rate-based TFT 无法防不公平、需 block-level TFT；Qiu 2004 证明 optimistic unchoke 给 free rider ~20% 带宽。三者构成直接冲突（§4）。
- 迁移 FD 前提：FD/rqbit 的 unchoke 实现与 choke 轮次（rqbit 的 rechoke 窗口被 pinned commit 收缩到一分钟内）需确认与 mainline 语义差异；不能把 2006 mainline 结论直接当 FD 结论。

### 2.3 S-PAPER-Izal2004 — Dissecting BitTorrent: Five Months in a Torrent's Lifetime（PAM 2004）
- 研究问题：用长时间 tracker 日志 + 插桩客户端量化 BitTorrent 的宏观效率与会话行为。
- 方法：1.77 GB Red Hat 9 tracker 日志（5 个月）+ 1 个校园网插桩客户端。
- 数据可得性：tracker 日志由第三方提供；无公开数据集。
- 样本/年代：181,847 sessions；2003-04 至 2003-08。
- 客户端/协议：tracker 视角 + 单客户端；TCP；ADSL/校园网。
- 对照/重复：单 torrent；客户端仅一次运行，不可重复。
- 效应/不确定性：仅 19% session 完成；完成者均速约 1.3 Mb/s；>85% 数据由 20 个 peer 提供；上下行吞吐不相关。
- 作者局限：单 torrent、tracker 只记录优雅离开、NAT/多客户端去重不完美。
- **反证**：Qiu 2004 的“上下行成正比/全局知识”模型与实际“吞吐不相关”不符（§4；Qiu 先行假设与实际冲突，体现在 S-PAPER-Qiu2004 的 limitations）。
- 迁移 FD 前提：现代 swarm/内容规模与 2003 差异大；本结果是“冷门时刻/老 trace 外推边界”的典型限制，不可直接用于 FD 容量基线。

### 2.4 S-PAPER-Bharambe2006 — Analyzing and Improving a BitTorrent Network's Performance Mechanisms（INFOCOM 2006）
- 研究问题：拆解 BitTorrent 各机制（LRF、TFT、seed 行为、并发上传数）对利用率/完成时间/公平性的独立贡献。
- 方法：离散事件仿真（开源 simulator），可开关机制。
- 数据可得性：模拟器公开；workload 取自 Red Hat tracker + Gnutella 带宽分布。
- 样本/年代：默认 1000 leecher、200 MB 文件、400 块、1 seed；仿真规模上限 8000 节点。
- 客户端/协议：仿真数据面（无 tracker 控制面）；无 packet-level TCP；仅 edge bottleneck。
- 对照/重复：作者称多次运行差异统计不显著，故多图单次运行。
- 效应/不确定性：91% 上行利用率；部分节点上传 6.26× 下载（不公平）；rate-based TFT 不防不公平；smartseed 明显改善低带宽 seed 场景。
- 作者局限：忽略 endgame、传输层细节、内部瓶颈；无法外推到更大规模。
- **反证**：Legout 2006 的真实 torrent 实验认为 choke 公平且抗 free rider → 与本篇“rate-based TFT 不公平”直接冲突（§4）。
- 迁移 FD 前提：FD 的公平性应以“多任务/多 peer 归一化 goodput 分布”实测，而不是沿用任一仿真结论；块级 TFT 若引入需先评估对 FD 上传效率的代价。

### 2.5 S-PAPER-Qiu2004 — Modeling and Performance Analysis of BitTorrent-Like P2P Networks（SIGCOMM 2004）
- 研究问题：用流体模型解释可扩展性、效率与激励对吞吐的影响。
- 方法：确定性流体模型 + 随机流体（Ornstein–Uhlenbeck）+ 非合作博弈；数值仿真 + 一条真实 trace 校验。
- 数据可得性：无公开数据；模型可复现。
- 样本/年代：2004；真实 trace 仅一条（<100 次完成下载）。
- 客户端/协议：抽象 BitTorrent-like；假设全局知识（激励分析）。
- 对照/重复：仿真与模型对比；trace 一次。
- 效应/不确定性：T=1/(θ+β) 与 λ 无关；η 接近 1；free rider 得 ≈1/(n_u+1)；局部稳定。
- 作者局限：忽略节点度、最大并发上传等参数；激励分析用全局知识。
- **反证**：Izal 2004 实测上下行吞吐不相关、完成比例仅 19%，与模型理想化不符（§4）。
- 迁移 FD 前提：FD 的“有效吞吐”需按验证字节口径实测；模型的 T 只能作理论下界/诊断参考。

### 2.6 S-PAPER-Piatek2007 — Do Incentives Build Robustness in BitTorrent?（NSDI 2007）
- 研究问题：tit-for-tat 激励是否对策略客户端稳健。
- 方法：48 h 大规模测量（301,595 IP、3,591 AS）+ trace 驱动模型 + BitTyrant 在 114 个真实 swarm 部署 + PlanetLab 合成 swarm。
- 数据可得性：BitTyrant 与匿名 trace 公开。
- 样本/年代：2006-04 测量；真实 swarm 300–2000 peer。
- 客户端/协议：Azureus/BitTyrant（Azureus 为 2006 最流行）；TCP；公网。
- 对照/重复：每 swarm 同时加入 BitTyrant 与未改 Azureus，配对比较；PlanetLab 6 次试验给标准差。
- 效应/不确定性：BitTyrant 中位 +70%（1 Mbit 客户端）；完成时间比中位 1.72；协议开销 0.9%→1.9%。
- 作者局限：合成 swarm 用带宽上限近似；BitTyrant 自身也可被更策略方利用；未覆盖 uTP/加密。
- **反证**：直接反驳 Cohen 2003“incentives build robustness”的前提（§4）。
- 迁移 FD 前提：FD 的互惠/上传贡献统计应能识别“高贡献低回报”peer；若要评估策略客户端风险，需要可观测的每 peer 上传/下载归一化对比。

### 2.7 S-PAPER-Stutzbach2006 — Understanding Churn in Peer-to-Peer Networks（IMC 2006）
- 研究问题：P2P churn 会话长度/到达过程/可用性到底服从什么分布，为何历史结论冲突。
- 方法：Gnutella/Kad 快照 + 三个 BitTorrent tracker 日志；识别并处理测量陷阱。
- 数据可得性：Cruiser 爬虫；tracker 日志第三方提供。
- 样本/年代：Gnutella 2004-10~12；Kad 2005-04；BT Red Hat 2003、Debian 2005、FlatOut 2004。
- 客户端/协议：多系统对比；TCP。
- 对照/重复：多数据集交叉；指出历史“中位 session 1 min–1 h”分歧。
- 效应/不确定性：session 长度属 Weibull/log-normal（非指数/非重尾）；BT peer 会长期“逗留”。
- 作者局限：false negative 导致长 session 被低估；单系统样本。
- **反证**：与“session 长度指数/重尾”的旧假设冲突（§4，contradicts 指向依赖该假设的建模工作，本 slice 未纳入其全文，记为缺口）。
- 迁移 FD 前提：FD 的“自然停滞 / 无有效进度”判据必须区分 session 结束与网络停滞；老 churn 参数不能直接设 FD 超时。

### 2.8 S-PAPER-Stutzbach2007 — On Unbiased Sampling for Unstructured P2P Networks（IEEE/ACM ToN 2007）
- 研究问题：如何在动态、异构的 P2P overlay 上无偏采样。
- 方法：提出 Metropolized Random Walk with Backtracking（MRWB），仿真 + Gnutella 实测（ion-sampler）。
- 数据可得性：ion-sampler 工具；Gnutella 完整快照作 ground truth。
- 样本/年代：Gnutella 快照（2004–2005）；仿真多种图模型。
- 客户端/协议：无结构 P2P；假设可查询邻居。
- 对照/重复：MRWB vs BFS/RW vs Oracle；KS 检验；10 万次 walk。
- 效应/不确定性：base case D=0.0019/0.0023/0.0037；中位 session <2 min 时误差显著。
- 作者局限：仿真主导；结构化 P2P 另行处理。
- **反证**：直接挑战早期 P2P 测量研究的采样效度（§4）。
- 迁移 FD 前提：FD 内部统计（如“窗口贡献 peer 数”“历史 useful peer”）若来自本地视角，不能宣称等于 swarm 无偏采样；需明确 scope。

### 2.9 S-PAPER-Cuevas2011 — Deep Diving into BitTorrent Locality（INFOCOM 2011）
- 研究问题：locality 在 ISP 级、多 torrent 规模下的真实收益与代价。
- 方法：爬取 40K torrent / 3.9M 并发 peer；概率上下界 + 确定性流量矩阵（b-matching 建模 unchoke）；原型验证。
- 数据可得性：Mininova/Piratebay 抓取；速度来自 Ookla/iPlane。
- 样本/年代：2009-08~10；11K ASes；top-100 ISP。
- 客户端/协议：BT unchoke/LRF/seed；建模保留关键机制。
- 对照/重复：Locality vs LOIF vs Random；多 ISP 案例。
- 效应/不确定性：半数 ISP 本地化 42–72%；LOIF 快 ISP −30%、慢 ISP −10%；~90% 跨境流量来自 ~10% 用户。
- 作者局限：模型化，非全网实测；静态需求快照。
- **反证**：Choffnes 2008 实测指出 locality 收益取决于 ISP 带宽策略（统一分配时优势消失）（§4）。
- 迁移 FD 前提：FD 若引入 locality/邻居偏好，需先确认对端 ISP 策略与 FD 的“有效吞吐”是否受损；不能只按“同 ISP”优化。

### 2.10 S-PAPER-Choffnes2008 — Taming the Torrent（SIGCOMM 2008）
- 研究问题：不依赖 ISP-用户信任的偏向邻居选择能否降低跨境流量且不损性能。
- 方法：用 CDN DNS 重定向行为（Ono 插件）作低成本 oracle，在 Azureus 上大规模部署。
- 数据可得性：Ono 源码 + 原始数据公开。
- 样本/年代：>120,000 用户、2,800+ 网络；数据 2007-12。
- 客户端/协议：Azureus 插件；TCP；公网。
- 对照/重复：Ono-recommended vs 随机 peer；6 h 窗口聚合。
- 效应/不确定性：>33% 路径不出 AS；中位 RTT 6 ms vs 530 ms；下载 +31%/+207% 视场景。
- 作者局限：单一客户端族；部分场景受 ISP 带宽策略限制。
- **反证**：Cuevas 2011 认为 locality 在多数 ISP 有净收益，两者条件不同（§4）。
- 迁移 FD 前提：FD 若要复用 CDN-oracle 思路，需先验证对目标接入网络的收益/代价，不能把 2008 结果当普适。

### 2.11 S-PAPER-Rossi2010 — LEDBAT: the new BitTorrent congestion control protocol（ICCCN 2010）
- 研究问题：LEDBAT（uTP）是否能既高效又对 TCP/交互流友好。
- 方法：按 draft 实现 LEDBAT 于 ns-2，包级仿真。
- 数据可得性：源码应请求提供；无公开数据集。
- 样本/年代：2010；单瓶颈 10 Mbps、50 ms RTT、drop-tail。
- 客户端/协议：uTP/LEDBAT vs TCP；未含 peer selection/tit-for-tat。
- 对照/重复：LEDBAT-TCP、LEDBAT-LEDBAT；多种 buffer/起始时差；100 次仿真。
- 效应/不确定性：对 TCP 让路（F=0.65，TCP 6×）；LEDBAT 间 t=0 公平 F>0.99；大 buffer 下后到者优势，slow-start 修复 0.53→0.99。
- 作者局限：仿真；未评估 BT 下载完成时间、与 peer selection/tit-for-tat 的交互。
- **反证**：Copa 2018 指出延迟型算法与 buffer-filling 流共存时性能差，需专门竞争模式（§4，邻接领域）。
- 迁移 FD 前提：FD 同时用 TCP 与 uTP，需实测混合流下 goodput/交互延迟 Pareto；不能引用“uTP 一定友好”的单一仿真。

### 2.12 S-PAPER-Arun2018 — Copa（NSDI 2018）
- 研究问题：能否用简单、通用、延迟导向的算法同时取得高吞吐、低时延、公平。
- 方法：目标函数 1/(δ·d_q) 的解析推导 + 用户态实现 + ns-2 + Pantheon 真实路径。
- 数据可得性：用户态实现开源；Pantheon 平台公开。
- 样本/年代：2018；Pantheon 六国节点，30 s/run。
- 客户端/协议：通用传输层（非 BT 专用）；与 Cubic/BBR/PCC/Vegas 等对比。
- 对照/重复：多平台多场景；flow churn 实验。
- 效应/不确定性：中位 Jain 0.86（Cubic 0.81, BBR 0.61, PCC 0.35）；真实路径延迟低 2–10×；RTT 公平 0.76 vs Cubic 0.12。
- 作者局限：RTT 差异极大时 mode switching 会误判；原型非内核部署。
- **反证**：延迟型方案在 buffer-filling 竞争下的既有弱点（LEDBAT 2010）被 Copa 以竞争模式缓解，说明“延迟导向即友好”并不自动成立（§4，邻接领域）。
- 迁移 FD 前提：**邻接领域，尚未在 BT 验证**；若考虑替换/调优 FD 的拥塞响应（含 uTP），必须先做 BT 负载下的对照，不得用传输层通用结果直接下结论。

### 2.13 S-PAPER-Richter2016 — A Multi-perspective Analysis of Carrier-Grade NAT Deployment（IMC 2016）
- 研究问题：CGN（运营商级 NAT）的部署规模、行为及其对端到端连通性的影响。
- 方法：BT DHT 爬取（内部地址泄漏检测）+ Netalyzr 主动测试 + 75 家运营商问卷。
- 数据可得性：数据集/方法公开；原始 DHT 爬取数据受限。
- 样本/年代：2016-03-03 起一周；探测 >21M peer，覆盖 ~19K ASes。
- 客户端/协议：BT DHT；NAT444/CGN；蜂窝。
- 对照/重复：三种方法交叉；保守阈值降低误判。
- 效应/不确定性：17–18% eyeball AS 部署 CGN；蜂窝 >90%；>700K peer 泄漏内部信息。
- 作者局限：DHT 不覆盖移动；AS 级粒度；阈值偏保守偏低。
- **反证**：与“NAT 只是家庭设备问题”的旧认知冲突（§4）。
- 迁移 FD 前提：FD 需区分“对端不可达”是 NAT 映射/过滤、端口分配还是 tracker/DHT 发现失败；CGN 环境下主动入站不可期待。

### 2.14 S-PAPER-Sermpezis2023 — Bias in Internet Measurement Platforms（2023）
- 研究问题：测量平台（IMP）因 vantage point 非均匀部署产生的多维偏差。
- 方法：形式化偏差框架（KS 检验 + 有界 KL），22 个维度，用 CAIDA/PeeringDB 等公开数据。
- 数据可得性：代码/数据/API 公开。
- 样本/年代：2023；>100K ASN。
- 客户端/协议：RIPE Atlas/RIS/RouteViews（非 BT）。
- 对照/重复：100 次重采样给 95% CI；IMP vs 随机采样。
- 效应/不确定性：Atlas 偏 0.06 vs RIS 0.16、RV 0.15；随机同等规模 VPs 偏差≈0；Atlas 自带选择算法偏 ~2×。
- 作者局限：AS 级粒度；输入数据自身不完整/有偏。
- **反证**：与“更多 vantage point 就能降低偏差”的直觉冲突（§4）。
- 迁移 FD 前提：**邻接领域（网络测量方法学）**；FD 自建/BT 生态数据的小样本结论需声明选择偏差，不得当无偏总体推断。

---

## 3. 主张—原文证据—适用条件—当前 FluxDown 实现差异—可检验预测

| # | 主张（来自上文精读） | 原文证据(locator) | 适用条件 | 与当前 FluxDown 实现差异 | 可检验预测 |
|---|---|---|---|---|---|
| C1 | rarest-first 在真实 torrent 上接近理想熵；endgame 影响小 | S-PAPER-Legout2006 §4.1.1、§2.3.1 | 高连通用户、mainline 2005 版、单插桩客户端 | FD 走 rqbit 的 piece picker；是否 rarest-first/hotness 混合需以 pinned rqbit `6f9d64b3` 源码为准（另由 RqbitFluxdown slice 覆盖） | 受控 swarm 中位 piece 可用度（Interest ratio）接近 1 |
| C2 | rate-based TFT 不防不公平；block-level TFT 有代价 | S-PAPER-Bharambe2006 §VII-B/C | 异构带宽仿真、d 小、seed 带宽受限 | FD 上传/公平由 rqbit choker 决定；FD 目前无 block-level TFT | 多任务竞争下归一化上传/下载分布出现长尾（部分 peer 上传远多于下载） |
| C3 | choke 版本差异导致公平性结论相反 | S-PAPER-Legout2006 §2.3.2/§4.2 vs S-PAPER-Bharambe2006 §VII | 2005–2006 mainline vs 仿真 | pinned rqbit 已把 rechoke 判定窗口收缩到一分钟内（commit `6f9d64b3`） | 以当前 rqbit 实现实测 rechoke 频率对互惠率/公平性的影响 |
| C4 | optimistic unchoke 给零上传 peer ≈20% 带宽 | S-PAPER-Qiu2004 §5.1 | 全局知识模型 | FD/rqbit 的 optimistic unchoke 槽位与轮换需以源码确认 | 受控 free-rider 客户端可获得约 1/(n_u+1) 量级下载 |
| C5 | 完成比例可低至 ~19%，长尾/未完成占比大 | S-PAPER-Izal2004 Table 1 | 2003 单一 1.77 GB 热门种子 | FD 的“完成”定义应为验证完成 + 文件可用（见研究包 metrics 口径） | 若以 session 而非任务计，未完成/放弃比例显著；但按字节计占比小 |
| C6 | seed 是稀缺资源，重复块浪费显著 | S-PAPER-Bharambe2006 §VI-B | 低带宽 seed 仿真 | FD 无 smartseed 逻辑；需确认 rqbit seed 端策略 | 低带宽 seed 场景下可观测到重复块比例（wire 字节 > 唯一字节） |
| C7 | 连接/邻居规模存在收益递减且开销可控 | S-PAPER-Piatek2007 §4.1/§4.2 | 2006 容量分布、Azureus | FD：BT_PEER_LIMIT=256、BT_PEER_CONNECT_LIMIT=64、BT_SESSION_PEER_LIMIT=1024、bt_session_connect_limit=(session/4).clamp(1,128) | 提高 peer 上限对 goodput 的边际收益随规模递减，协议开销非线性上升 |
| C8 | 大邻居池可提升高带宽 peer 上限 | S-PAPER-Piatek2007 §4.2 | 2006 swarm | rqbit `PeerAdmissionConfig` 保留量（global_reserve/connecting_reserve/outbound_budget）限制有效出站 | effective outbound cap 顶格时新增 peer 不再提升 goodput |
| C9 | session 长度非指数/非重尾，且 peer 长期逗留 | S-PAPER-Stutzbach2006 §5.2/§5.3 | 2003–2005 系统 | FD 做种/逗留行为是产品策略，非机制默认 | FD 会话长度分布应呈 Weibull/log-normal 型，而非指数 |
| C10 | P2P 采样易偏向短寿命/高度数节点 | S-PAPER-Stutzbach2007 §I/§V | 无结构 P2P | FD 统计来自本地视角（单客户端） | “历史 useful peer 数”不等于 swarm 无偏总体，含选择偏差 |
| C11 | locality 在 ISP 级收益大但有代价 | S-PAPER-Cuevas2011 §2；S-PAPER-Choffnes2008 §6.4 | 2007–2009 公网 | FD 未做 locality 邻居偏好 | 若引入同 ISP 偏好，在统一带宽策略 ISP 下 goodput 可能不变或下降 |
| C12 | uTP 对 TCP 让路，但内部公平需 slow-start | S-PAPER-Rossi2010 §II-C/§III-C/D | ns-2 仿真、2010 draft | FD 同时启用 TCP 与 uTP（MSE/uTP 调优） | 混合流下交互 RTT 增加被抑制，但 uTP 自身 goodput 下降；需报 Pareto |
| C13 | 延迟导向算法需显式竞争模式 | S-PAPER-Arun2018 §1/§5.2 | 通用传输层（邻接领域，未在 BT 验证） | FD 拥塞控制由 rqbit/TCP 与 uTP 承担 | 若在 BT 负载下引入延迟导向调优，需先验证与 buffer-filling 共存的吞吐代价 |
| C14 | CGN/多级 NAT 普遍，影响入站可达性 | S-PAPER-Richter2016 §1/§5；§3 | 2016 公网、蜂窝尤甚 | FD 需按 NAT 类型区分失败阶段（映射/过滤/端口） | CGN 环境下主动入站失败率高，应更多依赖出站与打洞/中继 |

> 注：C 系列为“机制解释”，不是“FD 已实现某算法”的证明。凡涉及 FD 实现，均需以 pinned rqbit `6f9d64b3`（Cargo.lock 9.0.1）与 `native/engine` 源码核对（另见 RqbitFluxdown slice）。

---

## 4. 反例清单（含与主结论冲突的文献及 contradicts_ids）

1. **激励是否带来稳健性**
   - 主张（Cohen 2003）：tit-for-tat 使系统稳健、抗 free riding。
   - 反例（Piatek 2007）：BitTyrant 中位 +70%，说明激励可被策略客户端利用；universal 策略客户端会降低整体性能。`contradicts_ids = S-PAPER-Cohen2003`。
   - 主张方行亦标 `S-PAPER-Piatek2007`。
2. **rate-based TFT 是否公平**
   - 主张（Legout 2006）：最新 choke 版本公平、抗 free rider；bit-level tit-for-tat 不合适。
   - 反例（Bharambe 2006）：仿真中部分节点上传 6.26× 下载，rate-based TFT 不防不公平，需 block-level TFT。双向 `contradicts_ids`。
   - 附加（Qiu 2004）：optimistic unchoke 给 free rider ~20% 带宽，与“抗 free rider”矛盾。`contradicts_ids = S-PAPER-Legout2006`。
3. **模型 vs 实测的上下行关系**
   - 主张（Qiu 2004）：流体模型/激励使上下行相关、T 与 λ 无关。
   - 反例（Izal 2004）：实测上下行吞吐相关系数≈0；完成比例仅 19%。`contradicts_ids = S-PAPER-Qiu2004`（在 Izal 行体现为与 Qiu 模型冲突）。
4. **locality 收益是否普适**
   - 主张（Cuevas 2011）：Locality 跨境降 42–72%、惩罚 <6%。
   - 反例（Choffnes 2008）：收益取决于 ISP 带宽策略；统一分配时优势消失。双向 `contradicts_ids`。
5. **采样是否无偏**
   - 主张（旧 P2P 测量）：爬取/轮询可代表总体。
   - 反例（Stutzbach 2007）：朴素采样偏向短寿命/高度数节点。`contradicts_ids = S-PAPER-Stutzbach2007`（对早期测量工作；本 slice 未纳入其时全文，记为缺口）。
   - 邻接（Sermpezis 2023）：即便是成熟 IMP 也有多维偏差；与“更多 VP 即更准”直觉冲突。`contradicts_ids = S-PAPER-Stutzbach2007`。
6. **网络编码替代 rarest-first**
   - 主张（Gkantsidis 2005、Miller 2006）：network coding 可提升大规模分发。
   - 反例（Legout 2006）：真实 torrent 上替换 rarest-first 不被实验支持。`contradicts_ids = S-PAPER-Legout2006`（在 Gkantsidis/Miller 行）。
7. **延迟导向拥塞控制是否自动友好**
   - 主张（宽泛）：延迟型算法天然对交互流友好。
   - 反例（Copa 2018）：延迟型方案在 buffer-filling 共存时吞吐差，必须加竞争模式；揭示“友好”与“吞吐”非自动统一。仅记录为邻接领域反例，未在 BT 验证，故本 slice 未强标 contradicts_ids 到 BT 条目。

> 冲突处理原则：不使用“多数文献”投票消解冲突，而是记录各自实验条件（客户端版本、是否仿真、带宽分布、年份），在实验协议里作为**待测协变量**。

---

## 5. literature 缺口清单

1. **BT 磁盘/校验背压的直接测量文献缺失**：Q5/Q26 两组检索式（`BitTorrent request pipeline disk bottleneck performance model`、`BitTorrent disk I/O bottleneck piece verification upload throughput measurement study`）未命中隔离“piece 校验/磁盘 I/O 限制上传吞吐”的原始研究。现有来源只有仿真中忽略磁盘的说明（S-PAPER-Bharambe2006 §IV-A）。→ 该主题仅由 S-PAPER-Bharambe2006 / S-PAPER-Qiu2004 间接覆盖，**磁盘瓶颈属证据缺口**。
2. **现代（2021+）直接 BT 论文稀缺**：本轮全文以 2003–2018 为主。已纳入的现代工作（Copa 2018、IMP Bias 2023）均为**邻接领域、尚未在 BT 验证**，已在 CSV/notes 显式标记。
3. **uTP 与 TCP 在真实接入网的混合公平性**：仅有 2010 ns-2 仿真（S-PAPER-Rossi2010）与通用传输层研究（S-PAPER-Arun2018）；缺 2021 后的 BT/uTP 公网测量全文。
4. **IPv6 下的 BT 可达性**：CGNAT 论文覆盖 IPv4 NAT/CGN（S-PAPER-Richter2016），但未覆盖 IPv6 BT peer 可达性测量全文。→ 证据缺口。
5. **MSE/PE 加密对握手成功率的影响**：仅规范层（BEP/PE）与工程文档，无学术全文。→ 转由 SpecsBepRfc slice 覆盖。
6. **Web seed（BEP 19）贡献评估**：无学术来源命中。→ 缺口。
7. **中文来源**：本轮 27 组检索式均为英文，未系统性检索中文学位论文/中文数据库；如需中文题录，需补检索（本 slice 明确标注未覆盖）。
8. **全文不可得/失败登记**：HAL（`inria.hal.science` 两个 URL）ECONNRESET；CACM CoDel 页 403。均已登记 `metadata_only` + limitations，未用摘要补写结论。
9. **Pouwelse 2005 / Fan 2006 / Tan 2007 / Guo 2005 / Karagiannis 2004** 等为 `metadata_only`：题录已核对但未读全文，其 claim 不承载实验结论。
10. **老 trace 外推**：理解 churn（2003–2005）与 locality（2007–2009）的样本年代久远；现代网络/客户端生态差异使外推需显式前提（已在 §3 表中逐条标注）。

---

## 6. 使用与引用约定

- 引用 format：`[S-PAPER-<id> §<locator>]`；代码/规范引用走对应 slice 的 `source_id`。
- 论文与代码证据**不混排权威等级**：论文证明“某条件下某效应”，不证明“FD 当前实现采用了某算法”。
- 不同文件大小/接入带宽/计时口径/完成定义的 MB/s **不汇总**，不做伪元分析；本 notes 只报原文效应及其条件。
- AI 辅助整理：检索、抓取与文本提取由本会话工具完成；所有 claim 均附原文 locator，未读全文者不写结论。
