# Error Guide

- Domain/Application 使用可判定的 Rust 错误类型。
- Infrastructure 保留底层原因并映射为领域语义。
- Tauri 边界转换为 Contract 定义的稳定错误码与用户可行动消息。
- React 根据错误码展示状态，不解析 Rust 文本。

禁止：

- `unwrap`、`expect`、`panic!` 处理业务失败；
- 吞错或只返回“失败”；
- 暴露内部堆栈、SQL、路径、密钥或服务端原始敏感响应。

重试只用于明确可恢复的网络/并发失败，并设置上限与退避。
