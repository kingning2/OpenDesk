# platform

渠道平台（**自包含**，只依赖 `common` / `ports`）— 协议 seam + 领域层 + 共享底座 + 平台 Provider + SQLite 存储。

## Purpose

多渠道平台的完整栈：渠道协议契约与调度、领域层（模型 + Store Ports + 领域服务）、
两站共享数据层、闲鱼协议实现、1688 账号实现、通用 SQLite 记录存储。

## 结构

| 模块 | 职责 |
|---|---|
| `protocol` | 渠道协议 seam（`ChannelProtocol` + dispatcher + 能力清单 + 编译期平台选择） |
| `domain` | 领域层（模型 + Store Ports + 领域服务：account/item/order/risk/setting/monitor） |
| `shared` | Provider 共享底座（账号派生 / Cookie 工具 / `SqliteBusinessDb` / `InMemory*Store`） |
| `xianyu` | 闲鱼协议 Provider（`cfg(platform_xianyu)`） |
| `ali1688` | 1688 账号 Provider（`cfg(platform_ali1688)`） |
| `storage` | 通用 SQLite 记录存储（`SqliteDb` + `RecordStore`） |

## 边界

- **属于**：渠道平台相关（协议 / 领域 / 数据 / 平台实现 / 存储）。
- **不属于**：AI（`ai` crate）、应用胶水（`business`）、Tauri（`src-tauri`）。

## Extension points

新平台三步：`protocol::ChannelKind` 加枚举 → `protocol::capabilities` 声明能力清单 →
在 `src/` 下建 Provider 模块（对齐 `xianyu`），并在 `lib.rs` 加 `#[cfg(platform_<id>)] pub mod`。

## Known Limitations

- `platform_*` cfg 由本 crate `build.rs` 注入（共享 `tooling/build/channel_platform_cfg.rs`），
  新平台需在 `tooling/config/channel-platforms.json` 登记；`ali1688`-only 构建不支持（既有）。
- `xianyu` 依赖 `wreq`（BoringSSL 原生库），本地 `cargo test` 需 CI 工具链（`pnpm lint:rust`
  会自动装 cmake/nasm）；以 `cargo check` / CI 为准。
