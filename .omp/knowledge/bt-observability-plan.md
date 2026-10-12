# BT peer 观测扩展实施计划

## 目标

同时补齐两层能力：

1. **FluxDown 观测面**：把 pinned `librqbit` 已经能提供的逐 peer 数据安全、按需地暴露给 daemon/API/Web/GPUI/原生移动端诊断面。
2. **pinned rqbit 数据面**：在 peer 发现、连接和握手链路中保留来源与阶段信息，使系统能够区分 tracker、DHT、PEX、LSD，并计算真正的握手成功率、失败原因和来源贡献。

目标是诊断与可重复 A/B 实验，不是立即启用自动 peer 调优。不得把诊断数据加入匿名遥测。

## 已确认事实

- pinned checkout：`zerx-lab/rqbit` rev `00374a6d772d4123f079f5adf6a1fe013e363381`（2026-10-12 随上游 #825 更新）。
- `librqbit::LiveTorrent::per_peer_stats_snapshot()` 已能返回逐 peer 记录。
- pinned rqbit 的 `PeerCounters` 已有：
  - `fetched_bytes`
  - `uploaded_bytes`
  - `connection_attempts`
  - `connections`
  - `errors`
  - `total_time_connecting_ms`
  - `downloaded_and_checked_pieces`
  - `total_piece_download_ms`
- pinned rqbit 的 `PeerStats` 已有：
  - peer state
  - `conn_kind`（TCP/uTP/SOCKS）
  - remote client name
- 当前 `native/engine/src/bt_downloader.rs::bt_peer_stats_from_handle()` 只把逐 peer 数据聚合成：
  - `useful_peers`（`fetched_bytes >= 1 MiB`）
  - connection attempts/successes/errors
  - total connecting time
  - aggregate fetched bytes/piece counters
- 当前 rqbit peer record 没有 tracker/DHT/PEX/LSD source 字段。发现路径最终主要以 `SocketAddr` 进入 peer admission，来源在该处丢失。
- 当前 `connections`/`errors` 不是严格的 BitTorrent handshake success/failure 计数；握手失败没有独立、稳定的阶段和原因枚举。
- rqbit 有 DHT stats/routing-table 能力，但 FluxDown 当前没有把它接入 daemon wire API。

## 范围与不变式

### 包含

- 逐 peer 下载/上传贡献统计。
- peer transport、client name、状态和稳定 peer key。
- tracker、DHT、PEX、LSD、多来源去重后的来源位集合。
- outgoing/incoming、transport connect、BitTorrent handshake、MSE/extension handshake 阶段统计。
- handshake 延迟与可归类失败原因。
- 聚合指标、按来源聚合指标、受限的 on-demand 逐 peer 诊断 API。
- Rust engine、daemon、protocol、Web、GPUI、Android、iOS 的 wire/镜像同步。

### 不包含

- 不迁移到 libtorrent。
- 不修改 Flutter UI；只保持旧 wire 兼容。
- 不新增匿名遥测。
- 不根据这些指标自动覆盖用户设置、带宽上限、文件选择或做种策略。
- 不通过删除并重新添加 torrent 模拟 live peer-limit 调整。
- 不在没有 A/B 结果前修改 DHT bootstrap、tracker 静态列表、timeout、request window 或 backoff 默认值。

## 设计决策

### 1. FluxDown 观测面

#### 1.1 保留高频 runtime 的轻量聚合

继续由 `TaskRuntime`/`BtPeerStats` 携带低成本、固定大小的聚合字段：

```text
seen
queued
connecting
live
dead
liveTcp/liveUtp/liveSocks
usefulPeers
fetchedBytes/downloadedPieces
totalPieceDownloadMs
connectionAttempts
connectionSuccesses
connectionErrors
totalConnectingMs
```

新增的聚合字段采用 `Option`，遵循现有旧 daemon/非 BT/后端不支持时为 `None` 的语义，不把“不可观测”编码成零。

建议增加：

```text
handshakeAttempts
handshakeSuccesses
handshakeFailures
handshakeFailuresByReason
handshakeLatencyMs: { count, p50, p95, max }
peerSources: {
  initial, tracker, dht, pex, lsd, multiple, unknown
}
sourceContribution: {
  tracker: { peers, usefulPeers, fetchedBytes },
  dht: ...,
  pex: ...,
  lsd: ...,
  unknown: ...
}
```

`sourceContribution` 必须明确统计口径：peer 若同时由多个来源发现，默认计入 `multiple`，不能把同一字节重复归因给多个来源。若实验需要按“来源触达”分析，另提供非加和的 `discoveredBySource`，名称和文档必须区分。

#### 1.2 逐 peer 数据走按需诊断接口

不把完整 peer 列表放进每 450/500ms 的 runtime event。新增 daemon RPC/API：

```text
daemon.task.peerStats
```

请求至少包含：

```json
{
  "taskId": "...",
  "limit": 256,
  "cursor": "..."
}
```

响应包含：

```text
sampledAtMs
sampleSequence
truncated
peers[]
aggregate
```

单个 `PeerObservation` 最少包含：

```text
peerKey                 // 稳定但不可逆的任务内 key
state
transport               // tcp / utp / socks
clientName
sourceFlags             // initial/tracker/dht/pex/lsd/unknown
fetchedBytes
uploadedBytes
fetchedPieces
pieceDownloadMs
connectionAttempts
connectionSuccesses
connectionErrors
handshakeAttempts
handshakeSuccesses
handshakeFailures
handshakeLatencyMs
lastErrorReason
firstSeenAtMs
lastStateChangeAtMs
```

安全与资源约束：

- 默认不向普通 UI 下发原始 IP、peer ID 或完整握手 payload；如确有诊断需求，单独设计管理员诊断权限和脱敏字段。
- `peerKey` 使用任务范围内稳定的不可逆哈希，避免把地址直接变成长期标识。
- 默认返回当前 live peer，`limit` 有服务端硬上限；需要 dead/历史 peer 时显式请求并分页。
- 逐 peer 统计不落任务数据库，不进入同步快照，不进入匿名遥测。
- 读取 snapshot 时避免每个 runtime tick 为 UI 复制完整 peer 表；排序、截断和分页尽量在 engine/daemon 侧完成。

#### 1.3 FluxDown 映射边界

按以下顺序同步：

1. `native/engine/src/transfer_activity.rs`：扩展 engine-owned aggregate/peer observation 类型。
2. `native/engine/src/bt_downloader.rs`：将 rqbit aggregate/per-peer snapshot 转为 engine 类型；不得把 rqbit 类型泄漏到 protocol/UI。
3. `native/protocol/src/task_activity.rs`、`native/protocol/src/event.rs`、`native/protocol/src/method.rs`：新增可选 DTO、RPC 方法和返回类型；旧字段缺失必须可解码。
4. `native/daemon/src/actor.rs` / API handler：实现按需查询、权限、上限、采样清理。
5. `web/src/lib/rpc/protocol/task.ts`、`web/src/lib/rpc/apply.ts`：镜像 DTO 和 reducer；空/过期/删除任务不得残留 peer 统计。
6. GPUI settings/diagnostic model：只读诊断页或开发诊断面，不把统计误做成普通下载设置。
7. Android/iOS protocol/model/HostSession 镜像：保持字段逐一对应；不改 Flutter UI。
8. OpenAPI/协议 fixture/旧 wire 测试：新增字段保持 optional，旧 snapshot 可继续 decode。

## 2. pinned rqbit 数据面

### 2.1 Peer 来源模型

在 pinned rqbit 内新增公开但最小化的来源类型。一个 peer 可能由多个路径发现，因此不能使用单一 enum 覆盖历史：

```rust
bitflags-like source flags:
    INITIAL
    TRACKER
    DHT
    PEX
    LSD
    UNKNOWN
```

如果当前依赖策略不允许新增 bitflags dependency，使用内部 `u8`/`u16` 新类型和手写 bit 操作；不得新增 crate。

来源语义：

- `INITIAL`：`AddTorrentOptions.initial_peers` 或恢复状态注入。
- `TRACKER`：HTTP/HTTPS/UDP tracker 返回。
- `DHT`：DHT `get_peers`/metadata peer stream 返回。
- `PEX`：peer extension message `ut_pex` 返回。
- `LSD`：local service discovery 返回。
- `UNKNOWN`：旧恢复数据、无法追溯来源或第三方注入。

来源传播路径：

```text
tracker / DHT / PEX / LSD / initial_peers
    -> candidate peer + source flags
    -> torrent peer admission
    -> Peer record
    -> per-peer snapshot + aggregate source counters
```

同一地址再次由另一个发现机制返回时只合并 bit，不重复建立 peer；发现次数另设累计计数，不能把一次 peer 计成多个 live peer。

需要审计 pinned fork 中以下边界：

- `LiveTorrent::add_peer_if_not_seen()` 及其所有调用者。
- tracker announce response 到 torrent peer queue 的转换。
- DHT `RequestPeersStream` 到 torrent peer queue 的转换。
- `ut_pex` 消息处理到 peer queue 的转换。
- LSD session/response 到 torrent peer queue 的转换。
- fast-resume/session persistence 恢复的 peer 是否只能标记 `UNKNOWN`，不得伪造来源。

### 2.2 Handshake 生命周期

不要把 TCP connect 成功当作 BitTorrent handshake 成功。对每个 outgoing/incoming peer 记录阶段：

```text
candidate
queued
connect_started
transport_connected
handshake_started
protocol_handshake_succeeded
mse_succeeded / mse_failed      // 若该路径启用 MSE
extended_handshake_succeeded / failed
live
failed
```

至少新增以下累计计数：

```text
transportAttempts
transportSuccesses
transportFailures
handshakeAttempts
handshakeSuccesses
handshakeFailures
mseFailures
extensionHandshakeFailures
```

失败原因必须是有限、可聚合的稳定分类，而不是把完整错误文本直接作为指标键：

```text
connectTimeout
connectRefused
connectReset
transportIo
handshakeTimeout
handshakeMalformed
handshakeInfoHashMismatch
handshakePeerIdInvalid
mseFailure
extensionFailure
cancelled
unknown
```

同时保留最近一次脱敏文本供日志诊断，但不能让任意远端错误文本造成高基数字段。

记录：

- outgoing/incoming 方向；
- TCP/uTP/SOCKS transport；
- `Instant` 单调延迟；
- peer task/torrent 归属；
- 失败阶段和有限原因。

延迟统计采用固定大小的 count/sum/分位数近似或 ring buffer，不在每次握手分配任意长度历史数组。需要 p50/p95 时由固定窗口聚合计算。

### 2.3 Per-peer counters 与 snapshot

扩展 pinned rqbit 的 `PeerCounters`/`PeerStats`：

```text
source_flags
source_discovery_count
handshake_attempts
handshake_successes
handshake_failures
handshake_latency counters
last_handshake_failure_reason
```

保留现有 `fetched_bytes`、`uploaded_bytes`、`connection_attempts`、`connections`、`errors`，不要改变其既有语义。新增字段必须使用 serde 默认值，以便旧 persistence/旧 snapshot 能读取。

`AggregatePeerStats` 增加按来源和握手阶段的聚合计数，但区分：

- 当前 point-in-time 数：live/connecting/queued/dead；
- 当前 torrent 生命周期累计数：attempts/successes/failures/bytes；
- 当前窗口统计：handshake latency histogram/window。

不要把恢复后的累计计数误当作本次进程启动以来的计数；文档中明确 reset/persistence 边界。

### 2.4 rqbit API 边界

优先扩展现有 `per_peer_stats_snapshot()` 返回类型和 `stats_snapshot()`，而不是在 FluxDown 中读取私有字段或复制 rqbit 内部结构。

可以保留 rqbit 自己的 DHT stats/routing-table API，但 FluxDown 的 daemon API 只转发与下载诊断相关的稳定 DTO；不要直接把整个 rqbit HTTP API 暴露给客户端。

## 数据口径

### Handshake 成功率

默认定义：

```text
handshake_success_rate = handshake_successes / handshake_attempts
```

必须同时输出分子、分母和窗口/生命周期范围。不能只输出百分比，也不能把 transport connect success 当作 handshake success。

### Useful peer

保留现有 FluxDown 定义：当前 peer 累计 `fetched_bytes >= 1 MiB`。另外提供：

- `contributingPeers`: 本采样窗口内 fetched bytes 增量 > 0 的 peer 数；
- `usefulPeers`: 生命周期累计超过 1 MiB 的 peer 数。

两者不得混名。

### 来源贡献

提供两套互斥口径：

1. `primarySourceContribution`：每个 peer 选定一个主来源，所有字节可加和，但主来源选择规则必须固定并文档化。
2. `sourceReachability`：按来源统计发现/触达/有数据 peer，可重叠，不能把这些数相加当作 peer 总数或字节总数。

第一版推荐只做 `sourceFlags` + `sourceReachability`，避免在来源归因尚不可靠时制造虚假的加和字节结论。

## 实施阶段

### Phase A：rqbit 内部模型与测试

- 新增 source flags 和 peer admission 传播。
- 新增握手阶段、方向、transport 和失败原因统计。
- 扩展 per-peer/aggregate snapshot，并补 serde 默认值。
- 为 tracker、DHT、PEX、LSD、initial peer 各写最小离线单元测试，确认 source flags 正确合并。
- 为 connect timeout、拒绝、握手超时、malformed、info-hash mismatch、成功路径写确定性测试。
- 测试旧 persistence 数据缺少新字段仍可读取。

### Phase B：FluxDown engine adapter

- 扩展 `BtPeerStats` 与新的 `BtPeerObservation` engine 类型。
- `bt_peer_stats_from_handle()` 只负责转换和聚合，不保留 UI/wire 逻辑。
- 增加 on-demand per-peer snapshot 函数，限制排序/分页/返回数量。
- 明确非 BT、旧 rqbit、live handle 已释放、任务删除时的 `None`/清理语义。

### Phase C：daemon/protocol/API

- 新增 `daemon.task.peerStats` 方法和 DTO。
- 聚合 runtime 字段保持 optional；逐 peer 诊断不进入高频事件。
- 添加权限、任务归属检查、硬上限、cursor 和采样时间。
- 保持旧客户端/旧 daemon decode；只新增 optional 字段，不为纯新增可选字段强制升级协议版本。
- 需要新增严格事件枚举时按现有协议版本规则处理。

### Phase D：客户端镜像

- Web reducer/protocol/tests。
- GPUI 只读诊断模型和页面/调试入口。
- Android/iOS DTO、HostSession、映射和测试。
- 不改 Flutter UI；确认旧 Flutter wire 客户端忽略新增字段。

### Phase E：验证与 A/B

固定同一 Ubuntu torrent、目录、tracker/DHT、限速和时间窗口，分别测试：

- `peer_limit=128/256/384/512`；
- DHT 开关；
- tracker-only；
- Clash/TUN 开启/关闭；
- IPv6/IPv4-only；
- 单任务与多任务。

每组至少采集：

```text
throughput / completion time
peer source reachability
handshake attempts/success/failure/reason/latency
per-source contributing peers
per-peer fetched/uploaded bytes
CPU/RSS/FD/disk IO
DHT lookup and tracker announce result
```

只有当来源、握手、贡献字节和资源指标都能在同一窗口对齐，才允许讨论自动 peer 调整。第一版不加入控制器：`bt_peer_limit`（128..512，默认 256）由用户显式选择，并在 daemon 重建共享会话时同步到 session 缺省；A/B 上限 512 即该范围上界。

## 代码边界

### FluxDown

- `native/engine/src/bt_downloader.rs`
- `native/engine/src/transfer_activity.rs`
- `native/engine/src/download_manager.rs`（仅在查询/生命周期需要时）
- `native/daemon/src/actor.rs`
- `native/daemon/src/config.rs`（若新增诊断开关）
- `native/protocol/src/task_activity.rs`
- `native/protocol/src/event.rs`
- `native/protocol/src/method.rs`
- `native/protocol/src/lib.rs`
- `web/src/lib/rpc/protocol/task.ts`
- `web/src/lib/rpc/apply.ts`
- `web/src/lib/rpc/apply.test.ts`
- GPUI diagnostics model/view
- Android `core` DTO/HostSession/mapping/tests
- iOS `FluxKit` DTO/HostSession/mapping/tests

### pinned rqbit

- `crates/librqbit/src/torrent_state/live/mod.rs`
- `crates/librqbit/src/torrent_state/live/peers/mod.rs`
- `crates/librqbit/src/torrent_state/live/peers/stats/mod.rs`
- `crates/librqbit/src/torrent_state/live/peer/stats/{atomic,snapshot}.rs`
- `crates/librqbit/src/peer_connection.rs`
- `crates/librqbit/src/stream_connect.rs`
- `crates/librqbit/src/session.rs`
- pinned tracker/DHT/PEX/LSD integration call sites

不直接修改 cargo registry 上游源码；若需要 rqbit 行为变更，必须更新 FluxDown 的 pinned fork/revision，并保留版本边界说明。

## 验收标准

1. daemon API 能返回受限的逐 peer fetched/uploaded bytes、transport、client、state 和 source flags。
2. 统计能明确区分 `usefulPeers` 与窗口内 `contributingPeers`。
3. tracker/DHT/PEX/LSD 来源能在 admission 后保留；同一 peer 多来源不会重复计数。
4. handshake success rate 使用真正的 handshake 分子/分母，不再用 transport connection success 代替。
5. handshake failure 能按有限原因聚合，并能从日志看到完整错误链。
6. 旧 daemon/旧客户端/旧 persistence 缺少新字段时仍能运行，非 BT/删除/完成/重连不会残留 stale peer 数据。
7. 高频 runtime 事件没有逐 peer 大对象；逐 peer 数据只通过受限按需接口读取。
8. 诊断数据不进入匿名遥测、不写入任务持久化、不泄露原始 peer 地址给普通 UI。
9. 在至少一组固定 torrent A/B 中能同时对齐吞吐、来源、握手、贡献 peer 和资源压力，才允许形成调参结论。

## 风险与回退

- 来源传播跨多个 rqbit 子系统，任何遗漏都会产生 `unknown`；必须把 unknown 保留为显式值，不能假装准确归因。
- per-peer 列表可能在大 swarm 中造成分配和 wire 压力；默认分页、硬上限、按需查询。
- handshake 失败原因可能高基数；只允许有限 enum，原始错误放日志。
- source flags 与 fast-resume 旧数据不兼容时统一降级为 `UNKNOWN`，不能从地址反推来源。
- 新 rqbit revision 可能改变公开 API；升级后必须重新核对默认值、字段语义和调用点。
- 观测字段只用于诊断和实验；没有稳定窗口和资源指标前，不启用自动 peer-limit 控制器。
