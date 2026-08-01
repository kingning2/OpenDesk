# Recipe: Add Query Port

1. 只有跨域只读查询才新增 Port；同 Feature 内直接调用。
2. 在 `crates/ports` 或拥有方定义最小 trait。
3. 返回稳定 DTO，避免暴露数据库模型。
4. Infrastructure 实现查询；调用 Feature 只依赖 trait。
5. 在装配层注入并测试权限、空结果和脱敏。

禁止通过 Query Port 写库。模板见 [`../templates/query-port/`](../templates/query-port/)。
