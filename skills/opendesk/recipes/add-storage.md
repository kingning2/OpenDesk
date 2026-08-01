# Recipe: Add Storage

1. 由拥有数据的 Feature 定义 Repository/Port。
2. 在 `crates/storage` 添加 migration 与实现。
3. 使用事务、唯一约束和索引表达数据不变量。
4. Application 只依赖 Port，数据库模型不穿过 IPC。
5. 为 migration、回滚边界和关键查询写测试。

凭据不存普通业务表；错误映射为领域语义。
