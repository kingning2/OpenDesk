# Repository Template

拥有数据的 Feature 定义 Repository/Port，`crates/storage` 实现。返回领域或 Contract DTO，不把数据库模型暴露到 IPC。

写操作明确事务与唯一约束；凭据使用系统安全存储。
