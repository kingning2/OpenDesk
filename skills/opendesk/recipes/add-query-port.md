# Recipe: Add Query Port

## 修改顺序

1. `create_query_port.py --name <feature>_query`
2. `crates/ports/src/<name>.rs` — trait 定义
3. `crates/ports/src/lib.rs` — `pub mod`
4. 实现 crate（如 `chat`）在 infra 层实现 trait（骨架 `todo!` 或空返回）
5. `crates/app` 组装时注入

## 禁止

- Query Port 含写操作
- Feature A 直接调用 Feature B 的 Repository

## Checklist

- [ ] trait 在 `ports`，实现在 feature/infra
- [ ] 只读方法签名
- [ ] 无 feature 间 crate 依赖

## 模板

[../templates/query-port/](../templates/query-port/)
