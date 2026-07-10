# Contracts Guide

## 何时改 Contract

- 新增/修改 IPC 命令参数或返回值
- 新增/修改 Event payload
- 新增/修改跨端 DTO
- 新增/修改 Error 码
- 新增/修改 sidecar OpenAPI 端点

## 文件命名

```
contracts/schema/v1/<feature>/dto/<name>.schema.json
contracts/schema/v1/<feature>/ipc/<command>.schema.json
contracts/schema/v1/<feature>/event/<name>.schema.json
contracts/schema/v1/<feature>/error/<code>.schema.json
```

## Schema 规范

- `$id` 使用 `opendesk://<feature>/<kind>/<name>/v1`
- 必填字段列在 `required`
- 禁止未文档化的 `additionalProperties`（建议 `false`）
- 日期时间用 `format: date-time`
- ID 用 `format: uuid`（若适用）

## 变更工作流

1. 编辑 schema / openapi
2. 更新 `contracts/CHANGELOG.md`
3. 运行 `python skills/opendesk/scripts/sync_contracts.py`
4. 更新 Rust / Python / TS 引用（骨架阶段可仅创建类型文件）
5. PR：2+ Approve

## Breaking Change

1. 新建 `schema/v2/` 或新文件
2. 编写 `compatibility/MIGRATION.md`
3. 旧版本保留至迁移完成

## 工具

```bash
python skills/opendesk/scripts/create_contract.py --feature chat --name thread --kind dto
python skills/opendesk/scripts/check_contracts.py
python skills/opendesk/scripts/sync_contracts.py --dry-run
```

## 相关

- [../architecture/contracts.md](../architecture/contracts.md)
- [../recipes/add-contract.md](../recipes/add-contract.md)
