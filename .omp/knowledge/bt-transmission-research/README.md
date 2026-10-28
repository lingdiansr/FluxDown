# BT 传输体验研究包

> 目标不是证明某个参数最优，而是建立 **来源证据 → 机制解释 → 指标口径 → 可证伪假设 → 实验协议** 的闭环，为「优秀 BT 传输体验」的工程实验做知识储备。
>
> **AI 辅助整理说明（一行）**：本包的检索、源码定位与初稿整理由 AI 助手在人工指定的版本边界与只读约束下完成；所有结论均带 `source_id` 或 `文件:行号` 定位，`work/lead-verification.md` 记录了由主责人独立复核并**推翻**过的子任务结论。

## 文件

| 文件 | 内容 |
|---|---|
| `README.md` | 六问题证据地图、理论分析、机制对照、生态分层、适用边界、缺口 |
| `sources.csv` | **419 条 claim / 257 个来源**，15 列（见下「列约定」） |
| `metrics.csv` | **74 项指标定义**，16 列，含单位/分子分母/窗口/重置删失/缺口 |
| `experiments.md` | E0–E3 协议、观测前置条件、统计与中断规则、授权清单 |
| `work/` | 分片原始产出与主责人独立复核台账（`lead-verification.md`），随包保留以便追溯 |

**列约定**：CSV 字段内**不使用** ASCII 逗号（用 `;` 或 `、` 分隔），因此无字段需要引号包裹；`sources.csv` 为 **15 列**（`source_id,type,title,authors,year,url,doi_or_commit,retrieved_at,access_status,topic,locator,claim,conditions,limitations,contradicts_ids`），`metrics.csv` 为 **16 列**。`sources.csv` 中同一 `source_id` 可有多行（每行一个 claim）。

---

## 0. 本轮范围与不做的事

**做**：案头资料研究与研究包编制 —— 规范、四款参考实现源码、pinned rqbit 与 FluxDown 适配层、学术文献、网络生态公开数据入口、内核/移动平台约束，以及指标字典与实验协议。

**不做**（本轮完全未执行）：不安装/修改任何下载客户端，不改配置与传输代码，不重启服务，不运行下载、announce、DHT 爬取、peer 握手、抓包或网络整形实验。真实实验需**另行授权**（清单见 `experiments.md` §6）。上述边界同样适用于现有测试脚本与远端主机。

---

## 1. 证据分层

研究围绕六个问题组织，不以峰值速度替代体验：

1. 可用数据源**在哪里丢失**：DNS、tracker/DHT/PEX/LSD、候选准入、TCP/uTP、MSE/BT 握手、兴趣/unchoke、首个 payload 各阶段如何区分？
2. 在带宽、时延、丢包、可达率与 swarm 供给不同的条件下，哪些调度限制**会成为瓶颈**，哪些只是相关现象？
3. 连接管理、piece/block 选择、请求流水线、拥塞控制、上传互惠与磁盘背压如何协同，**何时产生负作用**？
4. 如何兼顾完成时间、低排队延迟、多任务公平、资源占用与长期做种贡献？
5. 暂停恢复、重启、失效来源、坏块与长尾阶段如何影响**用户感知与数据正确性**？
6. 不同地域/运营商/接入方式与客户端生态下，哪些经验**可迁移**，哪些只能局部成立？

### 三条互不替代的证据轨

| 轨 | 回答什么 | 不能回答什么 |
|---|---|---|
| **规范**（BEP / RFC / MSE wiki） | 互操作**义务**与字段语义 | 实际实现是否照做；公网是否有收益 |
| **精确版本源码**（固定 commit） | **实际实现**是什么 | 在真实网络中的效果 |
| **学术实验与本地测量** | 效果与**适用边界** | 当前实现是否采用某算法 |

**推论规则（本包强制）**：论文不能证明当前实现已采用某算法；源码存在不能证明公网有收益；issue/博客仅用于提出假设。
每条结论按 **已验证 / 待验证 / 冲突 / 缺口** 标记，并附支持它的原文页码、源码行或数据窗口。

### 版本冻结（一切结论的边界）

| 组件 | 版本 | 备注 |
|---|---|---|
| FluxDown 引擎后端 | `librqbit` @ `6f9d64b310ecb560990315af3c60df3111eec8e7`（version `9.0.1`） | `Cargo.toml:52` + `Cargo.lock`；本地 checkout 与之逐字符一致 |
| FluxDown 仓库 | `602be2a67ddc98339e8e172e8786899be547da30`（2026-10-16） | 只读引用 |
| libtorrent | `54bc4e924b34e34212c0af9584921039e438ea84`（branch `RC_2_1`，2026-08-24） | 文档页自标 2.1.0，源码 `LIBTORRENT_VERSION` 为 2.1.1.0（见 `S-DOC-LT-001/002`） |
| qBittorrent | `607c9881e540510e0baaad28bc9ab5607ad20cd4`（master） | 是 libtorrent **宿主**，不另计为独立引擎 |
| transmission | `c22a63d52f01d301f2131e4e2102ecc0ac3bb2df`，版本 4.2.0 | |
| aria2 | `9e7273583f83e881e3ec067b523ba88724088d2f`，版本 1.37.0 | 非 libtorrent 的独立实现 |

**限制（必须连同结论一起引用）**：四个参考仓库均为 **grafted shallow clone**，无 tag 可达、submodule 未初始化；因此**不得**用可变网页上的版本号替代上述 commit。

**内部文档 pin 已漂移（缺口 G19）**：`.omp/knowledge/bt-observability-plan.md:14` 与 `.omp/knowledge/bt-backend-migration.md:999` 记录的 pin 是 `d995b0c9…`，而实际 pin 为 `6f9d64b3…`；`git log` 显示前者是后者**祖先**（其后至少还有 `c17ed3ae`/`70a73c2e`/`076d8a06`/`e2b7e62a`/`6f9d64b3`）。引用这两份文档时必须标注「文档所述 rev，早于当前 pin」。

---

## 2. 六问题证据地图

「落点」列给出该问题在 `metrics.csv` 的指标族与 `experiments.md` 的实验臂。

### Q1 · 数据源在哪里丢失

| 阶段 | 规范义务 | 当前实现（pinned rqbit + FluxDown） | 指标 | 落点 |
|---|---|---|---|---|
| DNS / 解析 | 主机名解析族选择 | — | — | E0-C |
| tracker | BEP 12 multitracker、BEP 15 UDP（重传 `15·2^n` 秒、`connection_id` 1/2 分钟作用域）、BEP 23 紧凑格式（IPv4 **6** 字节 / IPv6 **18** 字节）、BEP 7 `peers6` | 已支持 tracker/DHT/PEX/LSD；**来源在进入 admission 时丢失**（rqbit peer record 无来源字段） | `M-DISC-01/02/03` 全为 `missing` | E0-C、E1-1、E3-2 |
| DHT | BEP 5 token、`get_peers`/`announce_peer`、BEP 32 `nodes6` | 有 DHT；bootstrap 与后续 lookup 的差异、6/18 字节混排健壮性**未验证** | `M-DISC-*`、`M-PEER-07` | E0-C |
| 候选准入 | — | 四个信号量 + 入站保留（V9）；`connecting_outbound` 默认 **112**（`bt_session_connect_limit(1024)=128` 减 `connecting_reserve=16`） | `M-CONN-01/05`、`M-BUDGET-01..06` | E0-A、E1-1 |
| TCP/uTP | BEP 29 | 两者皆开；uTP 与 TCP 竞速会使同一 peer 产生两次 attempt | `M-CONN-02` | E2-3 |
| MSE + BT 握手 | MSE 规范（**非 BEP 编号**，原 wiki 已 404，仅存归档） | `transport success` 与 `handshake success` **当前不可区分**；无握手阶段枚举 | `M-CONN-03/04/07` 全为 `missing` | E0-D、E2-4 |
| 兴趣 / unchoke | BEP 3 规范算法：每 10 s 重掐、按速率挑 4 个、30 s 轮换 optimistic、新连接 3× 权重 | **pinned rqbit 没有 choker**：连接建立即无条件 `Unchoke`，此后永不下发 `Choke`（V13） | `M-PEER-04` 为 `missing` | 前提已变，见 Q3 |
| 首个 payload | — | 无首字节/首校验时间戳 | `M-START-02/03` 为 `partial` | E0-D、E1 |

**结论（已验证）**：当前**无法**回答「数据源在哪里丢失」，因为来源标记与握手阶段在实现中都不存在（缺口 G1–G3）。这正是 `experiments.md` E0 必须先做、且 E1/E2 在 E0 完成前不得开跑的原因。

### Q2 · 哪些调度限制会成为瓶颈

规范给出的分层（已验证）：
- **硬上限 vs 运行时额度是两个概念**（`experiments.md` 要求核对，此处已落地）：
  - libtorrent：`max_out_request_queue` 是**硬上限**，`m_desired_queue_size` 是**运行时额度**，后者按 `request_queue_time × download_rate / block_size` 动态计算并夹在 `[min_request_queue, max_out_request_queue]`（`src/peer_connection.cpp:4799-4848`，本地 commit `54bc4e9`）；`send_buffer_watermark` 同样按 `uploaded_last_second × factor / 100` 动态求值并夹在低/高水位之间（`:5274-5287`）；`request_timeout()` 由请求 RTT 的均值与偏差自适应（`avg + deviation×4`），下限 2 s、上限 `request_timeout`（`:4571-4598`）。
  - pinned rqbit：per-peer 请求窗口是可配项但**FluxDown 未配置**，恒定走默认 **128**，且被对端 `reqq` **只能下调**（V6）。
  - FluxDown：`bt_peer_limit` 128..512、`bt_peer_connect_limit` 32..256、`bt_session_peer_limit` 128..4096。
- **满额 ≠ 瓶颈**：`M-BUDGET-04`（满额时长比例）只证明某闸门绑定。在 (a) 候选不足、(b) 有效额度不足、(c) 排放速率受限 三种情况下 occupancy 都可能顶格，而只有 (b) 抬闸门才有收益。当前实现下**这三者不可区分**（缺口 G4/G5）。

**关键前提（已验证，会改写实验设计）**：pinned rqbit 只在 `PeerAdmission` 里做准入，未见任何**基于反馈的运行时 peer 数调整**。因此本包不假设「自动调参会赢」，E1-1 的第一目的是**识别闸门**，而不是找最优值。

### Q3 · 协同与负作用

**必须先把「互惠」这条假设划掉一半**（已验证）：
- 规范确实规定按速率互惠的 choke 算法（`S-BEP-3`，10 s/4 peers/30 s/3×）。
- 但 pinned rqbit **不实现 choker**（V13）：所有连接一律 unchoked。上传侧的公平与防滥用改由**上传请求队列**承担 —— per-peer 128、per-torrent 4096、peer 间轮转、peer 内 FIFO、重复 chunk 去重、`PeerBacklogExceeded` 视为请求洪水并断连、`generation` 守卫防止跨连接写错（V8，`S-CODE-RQBIT-UP-01/02/03`）。
- ⇒ 在 E1-5 中，「上传限速换取下载速率（互惠）」这一机制**在当前实现里没有对应的执行者**。观察到的相关性只可能来自 (a) 对端的 choker 对我们的行为、(b) 上传占用带宽挤占下行、（c）队列背压。**把队列效应读成互惠效应是本项目最容易犯的错误。**

**piece 选择**（已验证）：规范只说「一般随机顺序」，**没有**要求 rarest-first（`S-BEP-3`，见 `work/specs.addendum.csv`）。pinned rqbit 的实际策略是：
`try_steal(10×)` → streaming `priority_pieces` → 排队序（文件按字母序；每文件内**首片、末片、其余升序**）→ `try_steal(3×)`（V7）。
这不是 rarest-first、也不是随机；它与「中继抢慢对端」机制叠加。**因此文献中关于 rarest-first 的收益不能直接迁移到 FluxDown。**

**请求流水线**：规范给出义务（SHOULD 级，`S-BEP-3`：「一次排多个请求」），文献给出早期数值锚点（Cohen 2003 报告每连接约 5 个 16 kB 子块请求，`S-PAPER-Cohen2003`）。libtorrent 用自适应额度（上文），rqbit 用固定窗口 128（被 `reqq` 夹住）。三者口径不同，**不可直接比较**。

**磁盘背压**：`M-GUARD-07/08`；libtorrent 侧存在 `max_queued_disk_bytes`、`coalesce_reads/writes`、`aio_threads` 等运行时额度（`S-CODE-LT-*`），rqbit 侧存储与写队列机制存在但缺背压口径。

### Q4 · 多目标兼顾

四个目标互不完全兼容，且规范/文献/实现三方给出的取舍不同：

| 目标 | 冲突对象 | 证据 |
|---|---|---|
| 完成时间 | 长期做种贡献 | `S-PAPER-Legout2006`（rarest-first 使 piece 熵接近理想）↔ `S-PAPER-Bharambe2006`（上传利用率 91% 但负载高度不均，部分客户端上传 6.26× 于其下载） |
| 交互延迟 | 吞吐 | RFC 6817 的 LEDBAT 目标是 **TARGET ≤ 100 ms** 且「不比标准 TCP 更激进」（`S-RFC-6817`，**Experimental**）；实测需分实现验证 |
| 任务公平 | 单任务吞吐 | session 级预算被多任务共享（V9）；「每任务各享额度」不成立 |
| 资源占用 | 并发度 | 抬预算的代价是 RSS/FD/CPU；`S-CODE-FD-*` 与 #825 记录 RSS +约 15 MB |

**在目标/容量可比或已归一化时才用 Jain 指数**（`M-RATIO-06`）；未归一化的公平性结论无效。

### Q5 · 恢复与用户感知

- **口径先行**：`t0` 为执行指令受理、文件选择**预先固定**；`V(t)` 是选定范围内**当前有效且唯一**的已验证字节（排除 padding 与未选文件），**允许真实失效时下降**；连接/fetched 计数重置**不改变** `V(t)`；`t_done` 要求选定内容校验**且**最终文件可用。
- **暂停语义**：`active` 时间只扣显式用户暂停的**实际生效**区间，不扣网络故障、排队、退避、初检或自然停滞；另报暂停请求→生效的响应时延与暂停期间流量（`M-RECOVER-01`）。
- **不可用现成数据回答的部分（缺口）**：panic 恢复、断电恢复、resume data 写入中途被杀的语义 —— 内部文档（`bt-backend-migration.md` §4.1）以 TODO 形式列出 rqbit 的 `.rqbit-session.json` 可能写坏、**总上传字节尚未持久化**，因此「跨 epoch 上传统计守恒」（`M-RECOVER-04`）当前**不可假设成立**（缺口 G15）。
- **pause/resume 的机制线索（已验证）**：`PieceTracker` 保证一个 piece 同时只归一个对端，暂停时把在途 piece 重新排队（`crates/librqbit/src/piece_tracker.rs:104-110`）—— 这是「UI 进度回落 vs 真实丢数据」必须区分的原因（`M-RECOVER-02`）。

### Q6 · 可迁移性

见 §5 与 §6。要点：**受控实验能识别因果，公网只能验证边界**；E3 不替代 E1/E2。

---

## 3. 理论分析（四个可审查模型）

每个模型都给出「能用它算什么」与「不能拿它说什么」，避免把公式当结论。

### 模型 1 · 排队与 Little 关系

在稳定窗口中，**在途量 ≈ 尝试率 × 平均阶段耗时**。
- 用途：把「连接数顶格」翻译为「某个阶段的平均驻留时间 × 吞吐率」，从而定位是**质量**（成功率低）、**速率**（排放慢）还是**并发**（预算小）在约束。
- **不可说**：连接数顶格**只证明某闸门绑定**，不证明抬闸门会改善 goodput。长超时 + 低成功率 与 高占槽 并存时，必须分别验证「提高并发」「降低超时」「提高尝试率」三种干预，不能只做一种就下结论。
- 采集：`M-CONN-05`（在途）、`M-CONN-06`（拒绝分原因）、`M-BUDGET-03/05`。

### 模型 2 · BDP 与流水线

在途有效请求字节需覆盖「路径反馈时延 × 速率需求」：
$$B_{\text{inflight}} \gtrsim \text{RTT} \times \text{rate target}$$
- 用途：判断在高 RTT 下**请求窗口**是否先于带宽成为瓶颈。
- **不可说**：这个公式给的是**下界需求**，**不是**最优 peer 数，也不是最优请求窗口。用它推「应该开多少连接」会系统性过度拨号。
- **必须分开的四类限制**：请求窗口（per-peer 在途请求数）、拥塞窗口、接收端/发送端缓冲、磁盘队列。它们量纲不同、可在同一 run 内同时绑定。
- 采集：`M-REQ-01/02/03`、`M-GUARD-07`。

### 模型 3 · swarm 供需、piece 覆盖与激励

可服务副本数、稀缺 piece、peer 上传供给与 churn 共同决定**上限**。
- 用途：在低 seed 稀缺 swarm 上，**只有增加来源才有收益**，调调度参数无收益（E3-2 的假设）。
- **文献冲突（必须保留）**：`S-PAPER-Legout2006` 认为 rarest-first + 现有 choke 已足够且公平稳健；`S-PAPER-Bharambe2006` 实测出显著不公平并提出块级 pairwise tit-for-tat；`S-PAPER-Qiu2004` 的流体模型给出 $T = 1/(\theta+\beta)$（与到达率无关），而 `S-PAPER-Izal2004` 的实测不支持该简化；`S-PAPER-Piatek2007` 表明激励可被策略性利用（BitTyrant 中位 +70%，且约 14 KB/s 之后上传近乎无回报）。
  ⇒ **「互惠/激励有效」在文献层面就不是共识**，本包不以任一单篇为前提。
- **不可说**：互惠、公平、文件完成与全 swarm 健康**可能不一致**；不能用一个指标代表整体。

### 模型 4 · 控制稳定性与多目标取舍

把参数分三层：**静态硬边界**（配置上限）、**动态工作额度**（运行时自适应）、**用户策略**（限速、队列、做种规则）。
- 用途：评估反馈时滞、hysteresis、冷却、资源压力与多任务公平，输出 goodput/时延/资源的 **Pareto 比较**。
- **不可说**：不预设「自动调所有参数」必胜。当前 pinned rqbit **没有**运行时 peer 数控制器（已验证），所以本包的所有 E1/E2 臂都是**开环**对照。
- **资源护栏优先**：任何 goodput 提升都必须在护栏未触发的子集内成立（`experiments.md` §5.5）。

---

## 4. 工程机制对照

**读法**：每格给 `source_id` 或 `文件:行号@commit`。标「✓自验」的行由主责人亲自读过原文/源码（详见 `work/lead-verification.md`）。

| 机制 | 规范（义务） | libtorrent @54bc4e9 | qBittorrent @607c988 | transmission @c22a63d / aria2 @9e72735 | pinned rqbit @6f9d64b | FluxDown @602be2a6 |
|---|---|---|---|---|---|---|
| piece 选择 | 一般随机顺序，**无** rarest 要求（`S-BEP-3` ✓自验） | 优先级常量 `dont_download=0/low=1/default=4/top=7`，`pick_pieces()` 强制 prioritize partial（`S-CODE-LT-019`） | 宿主覆盖 libtorrent | transmission `RechokePeriod` + 脉冲（`S-CODE-TR-001/002/003`）；aria2 独立实现（`S-CODE-ARIA2-001..003`） | **顺序式**：steal 10× → streaming 优先 → 文件字母序·首片/末片/升序 → steal 3×（`piece_tracker.rs:123-164`、`chunk_tracker.rs:236-249`、`file_info.rs:15-26`）✓自验 | 透传 rqbit |
| choker / 上传公平 | 10 s 重掐、按速率 4 个、30 s optimistic、新连接 3×（`S-BEP-3` ✓自验） | `choking_algorithm=fixed_slots_choker`、`seed_choking_algorithm=round_robin`、`unchoke_slots_limit=8`（`S-CODE-LT-004`、`src/choker.cpp:196-221` = `S-CODE-LT-017`） | `unchoke_slots_limit` 取自 pref `MaxUploads`（`S-CODE-QBT-003`） | transmission 有 rechoke 脉冲（`S-CODE-TR-003`） | **无 choker**：连接即无条件 `Unchoke`，永不 `Choke`（`peer_connection.rs:458-485`）✓自验 | 透传 rqbit；**公平改由上传队列承担** |
| 上传调度 | 「不能立即写出的请求应排在内存中，choke 时可整批丢弃」（`S-BEP-3` ✓自验） | 发送缓冲水位动态（`:5274-5287`）✓自验 | 宿主覆盖 | — | per-peer **128** / per-torrent **4096**，peer 轮转 + peer 内 FIFO，`Duplicate`/`PeerBacklogExceeded`(断连)/`TorrentBacklogExceeded`(真错误)，`generation` 守卫（`upload_queue.rs:28,31,51-64,99-142`）✓自验 | 透传 |
| 请求窗口 | 一次排多个请求（SHOULD；早期实测约 5×16 kB，`S-PAPER-Cohen2003`） | 硬上限 `max_out_request_queue` **vs** 运行时 `m_desired_queue_size = request_queue_time × rate / block_size` 再夹紧（`S-CODE-LT-011`、`:4799-4848`）✓自验 | 宿主覆盖 | — | 可配但**默认 128**，有效值 `min(reqq, 128)`（`live/mod.rs:1389,526-532,1618-1626`）✓自验 | **未下发**该选项 ⇒ 恒为默认（缺口 G18） |
| 请求超时 | — | 自适应：`avg + 4×deviation`，下限 2 s、上限 `request_timeout`（默认 60 s，`S-CODE-LT-001`；`src/peer_connection.cpp:4571-4598`）✓自验 | 宿主覆盖 | — | — | — |
| peer 准入 | — | `connections_limit` 满则不再拨号、按 `connection_speed=30` 发放；入站超 `connections_limit + connections_slack(200+10)` 才拒（`S-CODE-LT-020/021`）；`active_downloads=3/active_seeds=5`（`S-CODE-LT-006`）；`peer_turnover=4%`/`cutoff=90%`/`smooth_connects`（`S-CODE-LT-007`） | `connections_limit` 取自 pref `MaxConnections`（`S-CODE-QBT-001`）；每 torrent 上限 pref `MaxConnectionsPerTorrent` 默认 **100**（`S-CODE-QBT-002`） | aria2 `bt-max-peers` 默认 **55**（`S-CODE-ARIA2-001`） | 4 信号量 + 入站保留；`global_reserve`/`connecting_reserve`；`outbound_budget` 保证非零预算至少放行 1（`peer_admission.rs:80-121,150-178`）✓自验 | 注入 `peer_admission`（`bt_downloader.rs:828-837`）；`connecting_outbound` 默认 **112** |
| 拒绝计数 | — | — | — | — | `rejected` = **许可获取失败次数**，单次出站准入最多 +4；`AdmissionDenied` 四值有稳定标签但**无分列计数**（`:282-294,123-146`）✓自验 | — |
| per-peer 统计 | — | — | — | — | `fetched_bytes` 在**下行** `on_received_piece` 累加；`uploaded_bytes` 走上行钩子（`live/mod.rs:2340`、`peer_connection.rs:49,605`）✓自验 | `useful_peers` 透传 |
| `useful_peers` 口径 | — | — | — | — | **point-in-time**：当前仍连接、本连接内累计跨越 1 MiB 者；跨阈值只计一次，**断连即减**（`peers/mod.rs:378-405` 单测）✓自验。**但** `USEFUL_PEER_BYTES` 注释写「peer 从我们这里取走」属上传语义，与代码相反（文档缺陷）✓自验 | 注释为下载侧，与代码一致；文档措辞须改 |
| v1/v2/hybrid | BEP 3 拼接式；BEP 52 每文件对齐 + `piece layers` + hybrid 双 infohash（`S-BEP-52`，Status **Draft**） | 最早 v2 落地之一；强制拒绝前导零/`i-0e`/未排序键（`S-SPEC-3`） | 宿主覆盖 libtorrent | — | **部分支持且客户端未接线**：`Id32`/`urn:btmh:1220` 仅存在于 `librqbit_core`（`hash_id.rs:182-185`、`magnet.rs:81-97`，并以 libtorrent 官方 v2 测试种子作单测向量 `:200-212`）；**`crates/librqbit/src/` 对 `Id32`/`btmh`/`as_id32` 零引用**；无 `file_tree`/`piece_layers`/merkle 解析；14 个 `V2*` 错误变体全仓无构造点（`error.rs:24-54`）✓自验 | 未发现 v2 处理代码 ⇒ 归入缺口 G16 |
| session 级热更新 | — | — | — | — | — | `seed_max_active` 热生效；`bt_peer_limit` **只影响后续 add**（`bt_downloader.rs:1017-1023`）✓自验 —— 与 `download_manager.rs:4767-4771` 注释矛盾（缺口 G24） |
| 磁盘/存储 | — | `max_queued_disk_bytes=100 MiB`、`file_pool_size=40`、`aio_threads=10`、`checking_mem_usage=256`（`S-CODE-LT-005`）；`high_performance_seed()` 预设另改多项（`S-CODE-LT-023`） | 宿主覆盖 | — | sparse/partfile 适配存在；背压口径缺失 | `bt_partfile.rs`/`bt_sparse.rs` |

**关于 qBittorrent 的定位（正确性要求）**：qBittorrent 是 **libtorrent 宿主**，不能计为另一个独立引擎；其价值在于展示「库默认值被宿主覆盖」的层次。qBittorrent-Enhanced-Edition 仅作第三方分支旁证。

**跨实现的关键差异（可直接用于设实验前提）**：

- **choke 机制**：libtorrent `fixed_slots_choker` + `unchoke_slots_limit=8`、transmission 有 rechoke 脉冲、aria2 有独立 choke 实现、qBittorrent 暴露为 `MaxUploads` —— 而 **pinned rqbit 完全没有 choker**。⇒ FluxDown 与其余四者在「上传公平如何实现」上属于不同机制族，**任何跨客户端对比都必须先声明这一点**。
- **并发预算量级差异**：aria2 `bt-max-peers` 默认 **55**、qBittorrent 每 torrent 上限默认 **100**、libtorrent `connections_limit`+`active_downloads=3/active_seeds=5` —— 而 FluxDown 是 `bt_peer_limit` **256** + `bt_session_peer_limit` **1024**（`connecting_outbound` 有效 **112**）。量级差 2–5 倍，**不可把「更多连接」当作无条件更好**（§3 模型 1）。
- **uTP 参数**：libtorrent 默认 `utp_target_delay=100 ms`、`utp_loss_multiplier=50%`（`S-CODE-LT-008`），与 RFC 6817 的 `TARGET ≤ 100 ms` 及「不比 TCP 更激进」一致；aria2 默认 **关闭 LPD**（`S-CODE-ARIA2-003`）—— 说明「发现机制默认开」不是普遍默认，**发现来源必须作为实验操纵量而不是隐含常量**。

**两处需要连带修正的文档/清单问题（本轮发现）**：

1. **文档默认值 vs 源码默认值**：`S-DOC-LT-002` ↔ `S-CODE-LT-022` 是本案**唯一**的真冲突 —— libtorrent `tuning-ref.html` 的 peer list size 写 **4000**，而源码与 `reference-Settings` 为 **3000+1000**。此外 `reference-Settings` 的默认值表由 `docs/gen_settings_doc.py` 从 `settings_pack.cpp` **生成**（同源），因此它对「源码默认值」**没有独立信息量**，只能用于核对生成是否漂移（`S-DOC-LT-001`）。
2. **不是设置项的东西**：任务清单里列的 `min_request_queue`、`aio_read`、`aio_write` 在 libtorrent 2.x 中**不是 `settings_pack` 设置**（`min_request_queue` 是 `src/peer_connection.cpp:80` 的 `constexpr 2`）。⇒ `metrics.csv`/`experiments.md` 中凡引用它们之处一律按**代码常量**对待，不得当可调项。

**`contradicts_ids` 的三种用法（本包约定，必须按此读）**：
1. **真冲突**：两来源结论互斥（如 `S-DOC-LT-002` ↔ `S-CODE-LT-022`；`S-PAPER-Legout2006` ↔ `S-PAPER-Bharambe2006`）。
2. **上限 vs 额度**：同一机制的硬边界行与运行时额度行互指（用于提醒两者不同名也不同义）。
3. **库默认 vs 宿主生效值**：库默认行与宿主覆盖行互指（如 libtorrent 默认 ↔ qBittorrent 传入的 pref 值）。
⇒ `contradicts_ids` **非空不等于结论矛盾**；判读前先看两行的 `topic` 与 `locator` 属哪一类。

---

## 5. 网络生态分层（事实层，不把地域标签当因果变量）

**结论（已验证）：本轮 10 个接入分层全部为「未覆盖」。** 只确认了公开数据入口、方法与许可，没有取得任何一层的实测样本，**不索取密钥、不使用付费数据、不用 VPN 出口冒充本地接入**。

| 分层 | 事实 | 不能据此推出 |
|---|---|---|
| 国内同运营商 / 跨运营商 / 教育网 | 入口登记；**未取得样本** | 不能用 APNIC 宏观 IPv6 覆盖率冒充任一接入层的 BT 可达率 |
| 海外东亚 / 欧洲 / 北美 | 入口登记（RIPE Atlas / RIS、CAIDA）；**未取得样本** | 跨境慢不能直接归因审查或 P2P 限速；AS 关系 ≠ 真实流量路径 |
| 跨境方向 | **未覆盖** | 地域标签不能代替实测路径 |
| IPv4 公网 / NAT / CGN / IPv6 / 双栈 | RFC 4787/5382/6598 的语义（`S-RFC-*`）；CGN 实测部署见 `S-PAPER-Richter2016` | 有 IPv6 地址 ≠ IPv6 peer 可达；一次 bootstrap 失败 ≠ DHT 整体失效 |
| 家庭路由器 / Wi-Fi / 有线 / TUN / 代理 | 内核与队列文档入口 | 进程 CPU 低或上传高不能排除下行、路由器、单线程或磁盘瓶颈 |
| public / private、热门 / 稀缺、种子年龄与 churn | 定义与分层方式 | tracker 报的 seeders ≠ 可连 seeders；peer ID 自报 ≠ 可信客户端鉴定 |
| 客户端族 / 版本 | per-peer `remote_client_name` 可得（`S-CODE-RQBIT-PEER-01`） | 客户端名不作为恶意判据 |
| 移动 / NAS 平台约束 | Android/iOS 后台限制与 NAS 空闲唤醒的官方文档入口 | 桌面结果不能外推蜂窝/后台 |

**测量方法学**：`S-PAPER-Sermpezis2023` 显示测量平台偏差主要来自**布点策略**而非观测点数量（与 `S-PAPER-Stutzbach2007` 的「无偏采样」形成对照）；时间同步方面 NTP 通常只有毫秒级精度且依赖路径对称 ⇒ **不凭未同步时间戳估计单向时延**（`M-START-*` 全部只测单主机内的单调时钟差）。

---

## 6. 结论适用边界

**可以说的**：
1. 规范层面：BEP 3 不要求 rarest-first；choke 算法有规范性描述；紧凑地址 IPv4 6 / IPv6 18 字节；MSE 不是 BEP 编号规范；RFC 6817 是 Experimental。
2. pinned rqbit 层面：无 choker（V13）；piece 选择是顺序式（V7）；请求窗口默认 128 且被 `reqq` 夹住、FluxDown 未配置（V6）；上传队列有 128/4096 硬上限与轮转（V8）；v2 只到 infohash/magnet 层（V5）。
3. 准入层面：预算分四层信号量 + 入站保留，`connecting_outbound` 默认 112；`rejected` 是失败次数而非 peer 数（V9/V14）。
4. 文档层面：内部两份 BT 文档的 pin 已漂移（V10）；`bt_peer_limit`「热生效」注释错误（V15）；`useful_peers` 注释方向写反（V12）；上游 Prometheus 输出有重名覆盖（V16）。
5. 证据层面：`bt_peer_limit` 128→512 的既有观测（~1.1 → 12.6 MB/s 稳态）是**单一公开种子、无方差、只报稳态速率、原始数据不在仓库**的动机性证据（V11）。

**不可以说的**：
1. 不能说「提高预算会提升 goodput」——只知闸门可绑定，未知哪一闸门是瓶颈（§3 模型 1）。
2. 不能说「互惠换速度」——当前实现没有 choker，机制不存在（Q3）。
3. 不能说「rarest-first 有效所以 FluxDown 应该做」——当前实现不是 rarest-first，且文献本身有冲突（模型 3）。
4. 不能用 `rejected` 做「被拒 peer 数」统计（V14）。
5. 不能把 `useful_peers` 当「生命周期累计有用 peer 数」（V12）。
6. 不能把上述任何一条外推到蜂窝/后台/NAS —— 缺平台测量条件（§5）。
7. 不能用任一单篇文献外推公网收益；公共 swarm 的远端状态不可重置（`experiments.md` §0.5）。

---

## 7. 口径与演算例（口径必须先于数据）

**定义**：`t0` = 执行指令受理；`V(t)` = 选定范围内唯一有效的已验证字节；`t_done` = 选定内容校验通过 **且** 最终文件可用。

### 演算例（必须复现）

设 16 MiB 文件，`t=0` 受理，`t=10` 已验证 4 MiB，手动暂停 `t=10–30`（生效区间 20 s），`t=40` 验证完成 16 MiB，wire payload 总计 20 MiB：

| 指标 | 计算 | 结果 |
|---|---|---|
| wall goodput | `(16−0) MiB / (40−0) s` | **0.4 MiB/s** |
| active goodput | `(16−0) MiB / (40−20) s` | **0.8 MiB/s** |
| useful/wire | `16 MiB / 20 MiB` | **0.8** |

**硬规则**：即使恢复时 connection/fetched 快照归零，也**不得**在唯一验证进度 16 MiB 上再加回 4 MiB。该 run 必须标为**有干预**，不进入无人值守整程主比较。

### 阶段反例（必须成立）

- TCP connect 成功但 MSE/BT 握手失败的 attempt **只**增加 transport success（`M-CONN-02`），**不**增加 handshake success（`M-CONN-07`）。
- 同一 peer 被 tracker 与 DHT 均发现：去重后 peer 总数 = **1**；两个来源的 reachability 各可为 1；**不得**把两份字节加和。
- 无采集权限的拒绝量记为**空值 + `missing`**；**不得**由 0 推「无准入拒绝」。
- 「已收 payload 但 piece 未验证」的窗口**不得**判为停滞（`M-STAB-05`）。

### 统计反例（必须成立）

若某实现 10 个 run 仅 1 个完成、另一实现 10 个都完成，**不得**只用前者完成 run 的均速宣称更快；必须报告完成比例与删失，配对单位为**独立 run**，不是采样点。1 s 快照不是独立重复。

---

## 8. 来源追溯与自验

**做法**：每一行 claim 都带 `locator`（规范章节 / RFC 小节 / `路径:行号` + commit / 论文页码）。**原文 URL 在 `sources.csv` 的 `url` 列，固定 commit 或版本哈希在 `doi_or_commit` 列**（BEP 页用其 `Version:` 哈希如 `7392d83f…`；代码用仓库 commit；本地只读源码用「文件:行号 + commit」）。`work/lead-verification.md` 记录主责人**独立读取原文/源码**的 16 组核对，其中包括**推翻**子任务结论的实例：

| 自验项 | 结论 | 影响 |
|---|---|---|
| V2 | BEP 15 IPv6 stride **18 字节**、请求 IP 字段 32 位且应置 0、格式由 UDP 包地址族决定 | 与 `S-BEP-15` 逐字一致 |
| V3 | BEP 3 的「一般随机顺序」与「流水线 SHOULD」两条规范语句**原缺**，已补入 `sources.csv` | 修正「规范要求 rarest-first」的误读 |
| V4 | BEP 52 正文**无**测试向量引用；候选测试向量仓库 **404**（未编造）；真实现成件 = `bep_0052_torrent_creator.py` + libtorrent 官方测试种子 | 关闭「v1/v2/hybrid 测试向量」缺口 |
| V5 | pinned rqbit v2 = **部分且客户端未接线**：`Id32`/`btmh` 只在 `librqbit_core`；`crates/librqbit/src/` 对它们**零引用**；无 merkle/`piece_layers` 解析；14 个 `V2*` 变体无构造点 | 新增缺口 G16；任何「已支持 BEP 52」的说法必须限定为「magnet/infohash 层」 |
| V6 | 请求窗口默认 **128**、`min(reqq,128)`、FluxDown 未配置 | 修正「窗口可调」的过度暗示 |
| V7 | piece 选择**不是** rarest-first（顺序式 + 抢慢对端） | 修正文献迁移可行性 |
| V8 | 上传队列 128/4096 + 轮转 + generation | 确立上传公平的真实执行者 |
| V9 | 四信号量 + 保留量；`rejected` 为单计数 | 新增缺口 G4/G17 |
| **V12** | **推翻**子任务主张：`useful_peers` **是下载侧**（调用点在下行 `on_received_piece`；单测锁定语义），rqbit 的**注释方向写反** | 已回写 `metrics.csv` 的 `M-PEER-02` |
| **V13** | pinned rqbit **没有 choker** | 改写 E1-5 的前提与 `M-PEER-04` |
| **V14** | `rejected` = 许可获取失败次数（单次出站准入最多 +4） | 改写 `M-CONN-06` 的口径 |
| **V15** | `bt_peer_limit` **不**热生效（与 `download_manager.rs:4767-4771` 注释矛盾） | 新增缺口 G24；限定 E1-1 的操纵方式 |
| V16 | 上游 Prometheus 输出 `rqbit_peers_queued` 重名覆盖 `seen` | 采集方案必须先绕开 |

**子任务产出的可信度分级**：`specs` / `literature` / `network-env` 三片在结构校验（15 列、无畸形行、`access_status` 取值合法、URL 非空）后通过；`rqbit-fd` / `refimpl` 两片的代码 claim 由 V5–V9、V12–V14 抽样核对，其中 1 条结论被推翻、3 条口径被修正。**未被抽样的个体 claim 仍标记为「子任务产出、未逐条复核」。**

---

## 9. 资料缺口总表

| 缺口 | 内容 | 阻塞 |
|---|---|---|
| G1 | 每 peer 首 payload / 首校验时间戳缺失 | E1/E2/E3 的 `M-START-02/03` |
| G2 | 发现来源 flags 与来源→admission 传播缺失 | `M-DISC-*`、`M-PEER-07` |
| G3 | 握手阶段与有限失败原因枚举缺失 | `M-CONN-03/04/06/07` |
| G4 | 准入拒绝**分原因**计数缺失（现仅单计数） | 预算类结论 |
| G5 | 候选排队时长与队列状态转换缺失 | `M-BUDGET-05/06` |
| G6 | wire payload / fetched / checked 三者未分离 | `M-RATIO-01/03` |
| G7 | 请求窗口占用与块级请求时延缺失 | `M-REQ-*` |
| G8 | piece 可得副本数缺失 | `M-REQ-05`、E3 |
| G9 | 校验耗时与失败归因（对端 vs 存储）缺失 | `M-GUARD-08`、`M-REQ-04` |
| G10 | 每任务需求/剩余量（归一化用）缺失 | 多任务公平 |
| G11 | 交互护栏探测端（需授权） | `M-GUARD-01/02` |
| G12 | 只读内核/磁盘采集脚本（需授权） | `M-GUARD-03..07` |
| G13 | 移动端平台指标条件未具备 | `M-NASMOB-02..05` |
| G14 | NAS 空闲/机械盘采集条件未具备 | `M-NASMOB-01` |
| G15 | 上传字节跨 epoch 持久化未完成（上游 TODO） | `M-RECOVER-04` |
| **G16** | pinned rqbit **无** v2 元数据/merkle 校验（14 个 `V2*` 变体 never-constructed） | v2/hybrid 兼容性结论 |
| **G17** | `PeerAdmission.rejected` 无分原因计数 | 同 G4 |
| **G18** | per-peer 请求窗口 FluxDown **不可配置**（固定 128，被 `reqq` 夹住） | E1-2 只能降级为诊断对照 |
| **G19** | 内部文档 pin（`d995b0c9`）早于实际 pin（`6f9d64b3`） | 引用旧文档时须标注 |
| **G20** | `bt_peer_limit` 证据链无原始数据与方差 | 阈值不可定 |
| **G21** | pinned rqbit **无 choker** | 互惠类假设不成立 |
| **G22** | `rejected` 口径为失败次数（料想为 peer 数会放大数倍） | 同 G4 |
| **G23** | `useful_peers` 文档措辞与代码方向相反 | 引用文档会得出反向结论 |
| **G24** | `bt_peer_limit`「热生效」注释错误（实际只影响后续 add） | E1-1 操纵方式 |
| G25 | 文献：BT 磁盘/校验背压**无直接测量**文献 | 磁盘背压只能本地测 |
| G26 | 文献：2021+ 现代直接 BT 论文稀缺；Copa/测量偏差仅「邻接领域、未在 BT 验证」 | 现代外推受限 |
| G27 | 生态：10 个接入分层**全部未覆盖**；国内运营商/教育网数据入口为零 | E3 只能先登记条件 |
| G28 | 生态：HDD/SSD 顺序/随机性能缺权威中立依据（只登记，不引博客） | 磁盘实验的预期值 |
| G29 | 内核文档为滚动内容未绑版本；本机 `/proc/sys`、`/sys/block` 实测值未读 | E1-6 基线 |

**全文不可得（已登记，不用摘要补写结论）**：`S-MSE-2`（canonical wiki.vuze.com **HTTP 404**，仅存归档快照）；`S-PAPER-*` 中 16 条 `metadata_only`（含 HAL ECONNRESET ×2、CACM 403）；候选仓库 `bittorrent/bittorrent-v2-test-vectors` 与 `the8472/bittorrent-v2-test-vectors` **均 404**。

---

## 10. 下一步（需另行授权）

1. **E0 先行**：按 `experiments.md` §2 完成口径审计（维护点/作用域/重置规则），产出字段映射表；`partial`/`missing` 的指标对应的实验臂**不开跑**。
2. **补采集前置**：按缺口 G1–G12 决定优先实现哪些计数器（本包只登记，不改代码）。
3. **阈值与 `tau` 冻结**：由可实现容量基线与用户体验目标共同设定，**不凭单个 ISO 试验**（V11）。
4. **E1 → E2 → E3 顺序**：E1 识别闸门 → E2 换网络条件与协议 → E3 仅验证边界。
5. 任何公网运行需遵守 tracker 频率与 private 规则，**不安排全网扫描**；公开结果不含 IP / peer ID / passkey / cookie / token / 完整 magnet。
