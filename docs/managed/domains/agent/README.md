# Agent Domain（AI 基建）

## 职责

`crates/agent` 是 **AI 基建 crate**，不承载业务用例。按目录拆分能力：

- `llm/`：模型 HTTP 协议、Provider 策略与连接测试；
- `prompt/`：通用提示词请求结构；
- `skills/`：可复用 Skill 的注册与查找基建；
- Feature（`mail` / `crawler` 等）：业务 Prompt 与用例，调用上述基建；
- `crates/app`：LLM 设置、安全存储、命令编排；
- Query Port：只读客户、价目、纠错规则等业务上下文。

```text
React → Tauri IPC → Rust Feature/Application → crates/agent::{llm,prompt,skills} → Model API
```

## 边界

- API Key 存 OS keyring，IPC 只返回 `configured`/`has_api_key`。
- 领域 Prompt、解析与业务规则不放进 `crates/agent`。
- AI 不直接写客户、报价或合作状态；发送动作保持人工确认。
- 长任务返回 run/task id，状态经 Tauri Event 到 React。

## 当前事实

- LLM Provider 设置与连接测试已落地。
- 邮件 HTML、爬虫关键词等用例留在 Feature，经 `agent::llm` / `agent::prompt` 调模型。
- `skills/` 提供注册表基建；具体业务 Skill 尚未集中注册。
- `workflow_runtime` 已具备 DAG、状态机、Executor Registry、检查点与恢复骨架。
