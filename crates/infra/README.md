# infra

基础设施（**自包含**，只依赖 `common` / `ports`）— 事件总线 + sidecar 运行时 + 适配器。

## Purpose

跨应用通用的基础设施：进程内事件总线、Python sidecar 运行时（客户端 / 生命周期 / 日志 / 路由）、
agent sidecar 网关与 license 校验适配器。

## 结构

| 模块 | 职责 |
|---|---|
| `event` | 进程内 pub/sub 事件总线（`EventBus` / `EventHandler` / `InMemoryEventBus`） |
| `sidecar` | Python sidecar 运行时（`SidecarClient` / `SidecarLifecycle` / `log_pipe` / `routes`） |
| `agent_sidecar` | agent sidecar 网关适配器（`RuntimeAgentSidecar`，实现 `ports::sidecar`） |
| `license` | license 校验适配器（实现 `ports::license`） |

## 边界

- **属于**：通用基础设施。
- **不属于**：业务逻辑（`platform::domain`）、AI（`ai`）、渠道平台协议（`platform`）、Tauri（`src-tauri`）。

## Known Limitations

- `license` 的编译期 env vars（`DINGDA_LICENSE_*`）由 `build.rs` 从 `generated/` 读取，
  需先运行 `pnpm build:license-verifier` 生成校验数据（否则为空串）。
- `sidecar` 运行时面向 Python sidecar；若未来侧车演进，路由绑定需同步。
