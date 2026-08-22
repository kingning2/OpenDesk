# business

桌面端应用胶水（不依赖 Tauri）：日志 / 配置 / 渠道存储 / 事件桥 / 耗时计时。

## Purpose

**领域层已下沉到 [`crates/platform`](../crates/platform/README.md)**（业务模型 + Store Ports +
领域服务）。本 crate 只保留桌面壳的应用胶水：

| 模块 | 职责 |
|---|---|
| `logging` | 应用日志初始化（终端 + 内存环形缓冲） |
| `config` | 应用配置（AI JSON + 插件/OCR tessdata） |
| `channel` | 渠道 SQLite 存储 + 安全过滤 |
| `event_sink` | EventBus → `common::events::EventSink` 适配 |
| `timing` | 异步耗时日志（显式 `#[timed]` 时启用） |

## 边界

- **属于**：不依赖 Tauri 的应用壳胶水。
- **不属于**：领域逻辑（`crates/platform`）；前端（`apps/desktop/src`）；
  跨平台基础设施（`crates/**`）；具体平台协议（`crates/platform-*`）。

## 依赖方向

```text
apps/desktop/src-tauri → business（胶水） + crates/platform（领域 + 渠道）
business → crates/{common, infra, platform}
```

`business` **不被** `crates/**` 依赖——领域类型已全部在 `crates/platform`，应用壳可独立演进。
