# BT 性能差异修复归属（FluxDown 宿主 vs rqbit 引擎）

## Context

`~/code/bt-engine-peer-perf-comparison.md` 产出 10 条 librqbit 相对 libtorrent/libtransmission 的差异结论。需判定每条修复的落点：**fd（FluxDown 宿主，`native/engine`）** 还是 **rqbit（引擎库，`lingdiansr/rqbit` fork）**。

判定原则：**引擎内部调度/协议逻辑缺失 → rqbit 改（fd 无法从库外注入）；引擎已暴露配置项、只需宿主选值 → fd 改。**

## Approach

分四组，逐条给归属、落点锚点、改动动作。

### A 组：fd 已落地（无改动，仅作为现状基线记录）

以下三条在 fd 侧已完成，靠 rqbit `mse-dev` 线（fd 当前 `Cargo.toml` rev=`3860dbe0`）暴露的配置项生效：

- **#7 MSE 默认启用**：`native/engine/src/bt_downloader.rs:718` `mse_mode: bt_config.mse_mode.into()`；`BtMseMode` 枚举（`:313-341`）默认 `Enabled`（`:433`）；DB 默认 `bt_mse_mode=enabled`（`db.rs:2016`）。已解决报告 #7 的「默认关闭」。
- **#4 握手超时 + 重连退避（配置层）**：`bt_downloader.rs:694` `handshake_timeout=10s`、`:722-727` `peer_backoff = {min 5s, factor 4, max 600s, total 1800s}`。已解决报告 #4 的连接级超时/退避（不等同逐 block 超时，见 rqbit 引擎计划）。
- **#5 连接节流（配置层）**：`bt_downloader.rs:697` `connect_rate=Some(30)`；rqbit mse-dev 已把 `connect_rate` 接进 `task_peer_adder` 的 sleep 限速（`live/mod.rs` diff：`connect_interval = 1_000_000 / connect_rate`）。已解决报告 #5 的「节流」，但「首波加速」缺失（见 rqbit 引擎计划）。

### B 组：fd 侧改（引擎已支持，宿主选值）—— 1 条

- **#10 uTP 默认启用**：改 `native/engine/src/bt_downloader.rs:670` `mode: ListenerMode::TcpOnly` → `ListenerMode::TcpAndUtp`。rqbit `listen.rs` 枚举已含 `TcpAndUtp`（`utp_enabled()` 返回 true），库层无需改。此改动让 fd 同时监听 TCP + uTP，出站 TCP/uTP 竞速逻辑（`stream_connect.rs`）随之获得 `utp_socket` 而真正生效。uTP 上游标记 experimental，本次不改 UI、不暴露开关（可覆盖，见 Assumptions）。

rqbit 引擎侧改动见 `~/code/Rust/rqbit/RQBIT_PERF_IMPROVEMENT_PLAN.md`。

## Critical files & anchors

1. `~/code/Rust/FluxDown/native/engine/src/bt_downloader.rs:638-729` — SessionOptions 构造：B 组 #10 落点（`:670`）+ A 组现状证据（`:694,697,718,722`）。

## Verification

- **分类正确性**：B 组 #10 有 rqbit `listen.rs:25-29` `enum ListenerMode { TcpOnly, UtpOnly, TcpAndUtp }` 支撑（`utp_enabled()` 对 `TcpAndUtp` 返回 true）；A 组三项有 fd `bt_downloader.rs` 行号支撑。
- **端到端（若执行 B 组 #10）**：`cargo check -p fluxdown_engine` 通过后，`cargo run -p fluxdown_cli -- add <magnet> --local` 启动下载，`ss -lunp | grep <port>` 应同时出现该端口的 UDP（uTP）监听行；此前（TcpOnly）只有 TCP。

## Assumptions & contingencies

- **#10 是否暴露 UI 开关**：默认不暴露、直接硬改 `TcpAndUtp`（uTP 上游 experimental，隐藏启用最保守）。若需用户可开关，另加 `bt_config.enable_utp` 字段 + Dart 设置项，属独立改动，不在本计划。
