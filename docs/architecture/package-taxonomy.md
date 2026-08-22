# 包分类（Package Taxonomy）

**原则：每个 crate / 包自包含、单一职责。`crates/` 内只依赖共享叶子（`common` / `ports`）+ 外部 crate，
不依赖兄弟 crate；若存在兄弟依赖则合并。**

## Rust（crates + business + apps/desktop/src-tauri）

| 包 | 角色 | 一句话职责 | 依赖 |
|---|---|---|---|
| `common` | 共享叶子 | 共享 DTO / 契约 / 错误 / 事件 | — |
| `ports` | 共享叶子 | Port traits（repository / sidecar / license） | common |
| `agent` | 能力包 | AI 编排层：`model` / `knowledge` / `prompt` / `intent` / `reply` 各功能文件夹，lib 编排 | 外部 |
| `platform` | 能力包 | 渠道平台：协议 seam + 领域层 + 共享底座 + 闲鱼/1688 Provider + SQLite 存储 | common, ports |
| `infra` | 能力包 | 基础设施：事件总线 + sidecar 运行时 + 适配器（agent_sidecar / license） | common, ports |
| `macros` | 工具 | 过程宏（`#[timed]`） | — |
| `business` | 应用胶水 | 桌面壳胶水（日志 / 配置 / 渠道存储 / 事件桥 / 耗时计时），领域在 `platform::domain` | common, infra, platform |
| `dingda` (src-tauri) | 应用壳 | Tauri 胶水：IPC 命令、状态注册、Builder | 全部 |

### agent 内部文件夹

| 文件夹 | 职责 |
|---|---|
| `agent::model` | LLM 模型家族（seam + 4 provider：OpenAI 兼容 / Anthropic / Gemini / DashScope） |
| `agent::knowledge` | 知识库（商品信息提取与注入） |
| `agent::prompt` | 提示词模板（模板独立文件 + 意图索引 + 变量插值） |
| `agent::intent` | 本地意图检测（price / tech / default / no_reply） |
| `agent::reply` | 回复引擎（编排入口） |

`subscription/` 为独立激活/授权工具（vendored OpenSSL），**不属于** workspace。

### platform 内部模块

| 模块 | 职责 |
|---|---|
| `platform::protocol` | 渠道协议 seam（`ChannelProtocol` + dispatcher + 能力清单 + 编译期平台选择） |
| `platform::domain` | 领域层（模型 + Store Ports + 领域服务） |
| `platform::shared` | Provider 共享底座（账号派生 / Cookie 工具 / 业务 SQLite 数据层） |
| `platform::xianyu` | 闲鱼协议 Provider（`cfg(platform_xianyu)`） |
| `platform::ali1688` | 1688 账号 Provider（`cfg(platform_ali1688)`） |
| `platform::storage` | 通用 SQLite 记录存储（`SqliteDb` + `RecordStore`） |

## 前端 pnpm 包

| 包 | 角色 | 一句话职责 | 依赖 |
|---|---|---|---|
| `@desk/platform` | 平台层 | 编译期平台选择、Tauri IPC 封装、事件订阅、错误分类、窗口 | @desk/contracts, @tauri-apps/api |
| `@desk/ui` | UI 库 | 组件 / 图标 / motion / theme / tokens | 第三方 UI 库 |
| `@desk/store` | 状态 | zustand store 工厂（`createDeskStore`） | zustand |
| `@desk/contracts` | 契约层 | 跨端契约的 TS 生成类型（自动生成） | — |
| `@desk/utils` | 工具 | 零依赖纯函数（日期/错误/格式化） | — |
| `@desk/desktop` (apps/desktop) | 应用 | Tauri + React 桌面应用 | 全部 |

## 不变量

- `crates/**` 不依赖 Tauri；**不依赖 `business`**；只依赖共享叶子 `common` / `ports` + 外部 crate。
- `business` 与 `src-tauri` 位于 `crates/**` 之上：`dingda → business → crates/**`。
- `platform_*` cfg 由 `crates/platform` 的 `build.rs` 注入（共享
  `tooling/build/channel_platform_cfg.rs`），**勿移动** `tooling/build/` 与 `tooling/config/`。
- `contracts/`（根）是三端契约唯一真相源；`packages/contracts` 为其 TS 生成产物。
