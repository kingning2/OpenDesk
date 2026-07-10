# Contract Template

JSON Schema 契约骨架。

## 文件

- [dto.schema.json.tpl](dto.schema.json.tpl)

## TODO

- [ ] 填写 `required` 与 `properties`
- [ ] 设置 `additionalProperties: false`
- [ ] 更新 CHANGELOG
- [ ] 运行 sync_contracts.py

## 生成

```bash
python skills/opendesk/scripts/create_contract.py --feature X --name Y --kind dto
```
