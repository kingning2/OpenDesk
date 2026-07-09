# Developer C（Python）规则

适用范围：`python/**`

## 职责边界

- Python 是 **AI Runtime**（gateway → queue → worker → provider）
- Python 不知道 React；不直接调用 Tauri IPC
- Python 不直连 SQLite/关系库（需要数据由 Rust 注入上下文或通过 Rust 查询后传入）

## 运行时分层（必须）

- `gateway`：HTTP API（契约校验、鉴权、限流、SSE）
- `queue`：任务队列与背压（可替换实现）
- `worker`：Worker 池与并发控制
- `provider`：模型 Provider 注册表与路由（新增 Provider 不改业务 executor）
- `llm/rag/ocr/agent/browser/workflow`：能力包（短名）

## 命名规范

- package 用短名名词：`gateway`、`queue`、`worker`、`provider`
- 禁止：`*_engine_provider_service`、`browser_automation_runtime`

## 契约与兼容

- Request/Response/Events/Errors 必须来源 `contracts/`（codegen 产物）
- 字段新增以可选为主；破坏性变更必须走 `schema/v2` + 迁移说明

