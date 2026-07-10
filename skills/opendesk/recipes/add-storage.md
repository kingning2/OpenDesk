# Recipe: Add Storage

## 修改顺序

1. 在 `crates/ports` 定义 `XxxRepo` trait
2. 在 `crates/storage/src/` 实现（SQLite 骨架）
3. UseCase 仅依赖 trait
4. 迁移脚本目录 `crates/storage/migrations/`（占位）

## 禁止

- UseCase / domain 写 SQL
- Python 访问 SQLite

## Checklist

- [ ] trait 在 ports
- [ ] 实现可替换（无硬编码连接串于 UseCase）

## 模板

[../templates/repository/](../templates/repository/)
