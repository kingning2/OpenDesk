# Query Port Template

仅用于跨域只读查询。trait 保持最小，返回稳定 DTO，由装配层注入实现。

写操作改用拥有方 UseCase 或 Event；步骤见 [`../../recipes/add-query-port.md`](../../recipes/add-query-port.md)。
