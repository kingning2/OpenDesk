# OpenDesk 项目解析

## 1. 项目是什么

OpenDesk 是一个面向企业场景的本地优先 AI 智能客服桌面平台。

它计划把客服会话、邮件、知识库、AI Agent、自动化工作流、浏览器操作、OCR、插件和模型调用等能力整合到同一个桌面应用中。用户通过桌面界面操作，Rust 负责协调业务与本地资源，Python 负责执行大模型及其他 AI 任务。

> 当前仓库处于 **Architecture Skeleton（架构骨架）** 阶段。现阶段主要成果是目录、模块边界、契约和工程工具，尚未形成可投入使用的完整客服产品。

## 2. 项目的主要用途

从现有模块和架构设计看，OpenDesk 主要面向以下用途。

### 2.1 统一客服工作台

计划将不同来源的客户沟通统一到桌面端处理，例如：

- 在线聊天；
- 邮件；
- 不同客服渠道；
- 用户与租户管理；
- 会话上下文和处理状态。

对应的核心模块包括 `chat`、`mail`、`channel`、`user` 和 `tenant`。

### 2.2 AI 辅助客服

Python AI Runtime 计划为客服提供：

- 大模型问答与回复建议；
- 基于企业资料的 RAG 检索增强；
- Agent 任务执行；
- Prompt 管理；
- 模型与 Provider 路由；
- 上下文记忆；
- OCR 图片文字识别；
- 浏览器自动化能力。

这些能力不会由界面直接调用。所有请求都必须经过 Rust，以便统一处理权限、超时、日志、任务状态和生命周期。

### 2.3 工作流自动化

`workflow` 模块计划把重复客服操作编排为工作流，例如：

1. 收到客户问题；
2. 查询知识库；
3. 调用模型生成建议；
4. 根据规则转人工或继续执行；
5. 将结果返回客服界面。

目前仓库只有工作流编辑器和相关模块的骨架，还没有实际的工作流执行能力。

### 2.4 本地优先的企业 AI

项目强调 Local First 和 Offline First：

- 桌面程序在本地运行；
- Rust 管理本地存储和 Python Runtime；
- Python 不直接操作业务数据库；
- 核心能力应尽量支持离线；
- 可以通过 Provider 抽象切换云端或本地模型。

这种设计适合重视数据控制、审计和离线可用性的企业环境。

### 2.5 可扩展能力平台

仓库包含 `plugin`、`mcp`、`provider` 等模块，说明项目不仅计划提供固定功能，也希望支持：

- 插件扩展；
- MCP 工具接入；
- 多模型 Provider；
- 可替换的存储、队列和执行组件；
- 独立演进的业务 Feature。

## 3. 最终用户可能如何使用

项目完成后，一条典型使用流程可能是：

1. 客服人员打开 OpenDesk 桌面应用；
2. 在统一工作台查看聊天或邮件；
3. 选择一条客户问题；
4. 请求 AI 查询知识库并生成回复建议；
5. Rust 将任务和上下文发送给 Python Runtime；
6. Python 执行 RAG、LLM 或 Agent；
7. 结果返回 Rust，再转发给 React 界面；
8. 客服审核、修改并发送回复；
9. Rust 保存业务状态并记录日志。

需要注意：这是根据项目架构和模块划分推断出的目标流程，不代表当前代码已经实现。

## 4. 整体架构

```text
┌──────────────────────────────────────────┐
│ React 展示层                              │
│ 页面、交互、状态、主题、动画               │
└───────────────────┬──────────────────────┘
                    │ Tauri IPC
┌───────────────────▼──────────────────────┐
│ Rust Application Core                    │
│ 唯一协调者：业务、存储、事件、任务、权限、 │
│ 日志以及 Python Sidecar 生命周期          │
└───────────────────┬──────────────────────┘
                    │ 本机 IPC / HTTP
┌───────────────────▼──────────────────────┐
│ Python AI Runtime                        │
│ LLM、RAG、Agent、OCR、Browser、Worker     │
└──────────────────────────────────────────┘
```

最重要的限制是：

- React 不能直接访问 Python；
- React 和 Python 都不能直接访问 SQLite；
- Python 不能调用 Tauri 或操作界面；
- Rust 是三端之间的唯一协调者；
- 跨端数据必须以 `contracts/` 为准。

## 5. 三端分别负责什么

### 5.1 React：展示与交互

主要目录：`apps/desktop/src`、`packages/ui`、`packages/platform`。

React 负责：

- 页面和组件；
- 用户交互；
- 主题、布局和动画；
- 通过 `@desk/platform/ipc` 调用 Rust。

React 不应包含 AI 执行逻辑、SQL、文件系统操作或跨 Feature 业务逻辑。

### 5.2 Rust：应用核心与协调者

主要目录：`crates/`、`apps/desktop/src-tauri`。

Rust 负责：

- Tauri 命令和事件；
- 业务 UseCase；
- 本地存储；
- Event Bus 和后台任务；
- 权限、缓存、日志和错误处理；
- 启动、停止、重启和检查 Python Sidecar；
- 将 Python 输出转发给 React。

### 5.3 Python：AI Runtime

主要目录：`python/`。

Python 负责：

- Gateway 请求处理；
- 任务队列和背压；
- Worker 并发执行；
- 模型 Provider 注册和路由；
- LLM、RAG、Agent、OCR、Browser 等 AI 能力。

Python 不负责业务数据库、桌面 UI、Tauri IPC 或客服业务状态。

## 6. Python Runtime 的计划结构

```text
Rust 请求
   │
   ▼
gateway     请求校验、错误映射、路由
   │
   ▼
queue       排队、背压、取消
   │
   ▼
worker      并发、超时、任务执行
   │
   ├── provider   模型选择与适配
   ├── llm        大模型能力
   ├── rag        知识检索
   ├── agent      Agent 执行
   ├── ocr        文字识别
   └── browser    浏览器能力
```

目前这些 Python package 基本都是空壳，仅建立了包结构和少量 `agent ping` 示例绑定。

## 7. 为什么需要 contracts

`contracts/` 是 React、Rust、Python 之间的数据协议唯一真相源，包含：

- Request / Response；
- HTTP API；
- Event；
- Error；
- 跨端 DTO。

正确的修改顺序是：

```text
Contract → Codegen → Rust → Python → React
```

这样可以避免三端各自定义字段，最终出现字段名称、必填规则或数据类型不一致。

破坏性修改需要创建新版本契约，并提供迁移说明。

## 8. Feature 划分

项目按照业务 Feature 垂直拆分：

| Feature | 计划职责 |
|---|---|
| `chat` | 在线会话 |
| `mail` | 邮件处理 |
| `agent` | AI Agent |
| `workflow` | 自动化工作流 |
| `knowledge` | 企业知识库 |
| `browser` | 浏览器能力 |
| `ocr` | 图片文字识别 |
| `mcp` | MCP 工具接入 |
| `plugin` | 插件系统 |
| `tenant` | 租户边界 |
| `user` | 用户信息 |
| `channel` | 客服渠道 |

Feature 之间禁止直接依赖。跨 Feature 写操作使用 Event，只读查询使用 Query Port，共享数据使用 Contract。

## 9. 当前实现状态

### 已经具备

- pnpm、Cargo 和 Python uv Workspace；
- React + Tauri 桌面应用入口；
- Rust crate 和 Python package 的基本目录；
- 三端 lint 工具；
- 架构检查脚本；
- JSON Schema / OpenAPI 契约目录；
- `agent ping` 请求和响应契约；
- Rust Sidecar Client 和路由绑定骨架；
- Python Sidecar、Gateway Handler 骨架；
- UI 设计令牌和少量基础组件。

### 尚未完成

- Python HTTP Server 实际启动；
- Rust 对 Python Sidecar 的完整生命周期管理；
- Rust 到 Python 的真实网络调用；
- 契约到 TypeScript、Rust、Python 的完整代码生成；
- Queue 和 Worker 实现；
- 真实模型 Provider；
- RAG、Agent、OCR 和 Browser 业务能力；
- Event Bus、任务调度和存储实现；
- 完整客服界面和业务流程。

当前界面的 `greet` 功能属于 Tauri 初始化模板，不是 OpenDesk 的实际客服功能。

## 10. 当前值得关注的问题

扫描仓库后发现以下架构偏差或待确认事项：

1. Tauri 启动代码使用了 `expect()`，违反 Rust 禁止 `unwrap/expect/panic` 的规则；
2. React 示例直接调用 Tauri `invoke()`，没有经过 `@desk/platform/ipc`；
3. `agent ping` 的 `trace_id` 在 JSON Schema 中可选，在 Rust DTO 中却是必填；
4. TypeScript、Rust 和 Python 的契约 codegen 产物尚未真正建立；
5. Rust `agent_ping()` 当前固定返回 `not implemented`；
6. Python Sidecar 目前只打印一行文本，没有启动服务。

因此，当前仓库应被看作“架构施工现场”，而不是功能已经完成但缺少界面的应用。

## 11. 建议的近期建设顺序

在架构评审允许的范围内，建议按以下顺序推进：

1. 统一 `agent ping` 契约；
2. 建立三端契约 codegen；
3. 定义 Python Gateway、Queue、Worker、Provider 接口；
4. 定义 Sidecar 生命周期、日志、超时和取消协议；
5. 使用 Mock 打通 Rust → Python 的最小链路；
6. 增加接口测试与契约测试；
7. 架构评审通过后，再实现真实 AI Provider 和业务 Feature。

## 12. 一句话总结

OpenDesk 的目标是成为一个本地优先、契约驱动、可扩展的企业 AI 客服桌面平台；React 提供工作台，Rust掌控业务和本地资源，Python 专注执行 AI 任务。当前项目完成的是架构骨架，实际客服和 AI 能力仍处于待实现阶段。
