# Recipe: Add Contract

## 修改顺序

1. `python skills/opendesk/scripts/create_contract.py --feature X --name Y --kind dto|ipc|event|error`
2. 编辑 schema 字段
3. 更新 `contracts/CHANGELOG.md`
4. `python skills/opendesk/scripts/sync_contracts.py`
5. 更新三端引用（类型导入骨架）
6. `python skills/opendesk/scripts/check_contracts.py`

## 禁止

- 先改 TS/Rust/Python 再补 schema
- 未文档化字段
- Breaking 变更不写 MIGRATION

## Checklist

- [ ] `$id` 正确
- [ ] `required` 完整
- [ ] 2+ reviewer（PR）

## 模板

[../templates/contract/](../templates/contract/)
