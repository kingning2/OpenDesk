# Recipe: Add LLM Provider

1. 先确认 Provider 是否兼容 OpenAI Chat Completions 或 Anthropic Messages。
2. 兼容协议只在 `crates/agent/src/llm` 增加默认 base URL/策略映射，不新建平行客户端。
3. 新协议才增加最小请求/响应类型与解析测试。
4. 设置与密钥仍由 Rust 安全存储管理；IPC 只返回配置状态。
5. 使用 mock HTTP 测试鉴权头、路径、错误状态和空响应。

验证：`cargo test -p llm && pnpm lint:rust`。
