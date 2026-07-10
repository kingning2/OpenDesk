# Recipe: Add Python Package

新增 Python Runtime 包并注册到 uv workspace。

## 修改顺序

| 步骤 | 操作 |
|------|------|
| 1 | 运行脚手架脚本 |
| 2 | 确认根 `pyproject.toml` 已注册 workspace member |
| 3 | 确认 Ruff `known-first-party` 已包含包名 |
| 4 | `uv sync`（若使用 uv） |
| 5 | `pnpm lint:python` |

## 自动化

```bash
python skills/opendesk/scripts/create_python_package.py --name embed
python skills/opendesk/scripts/create_python_package.py --name embed --dep contracts --dep shared
python skills/opendesk/scripts/create_python_package.py --name embed --dry-run
```

## 生成内容

```
python/packages/<name>/
├── pyproject.toml
├── README.md
├── src/<name>/__init__.py
└── tests/test_scaffold.py
```

根 `pyproject.toml` 自动更新：

- `[tool.uv.workspace].members` 增加 `python/packages/<name>`
- `[tool.ruff.lint.isort].known-first-party` 增加 `<name>`

## 禁止

- GUI · SQLite · 业务状态持久化
- 未评审的对外 HTTP Server
- 跳过 workspace 注册手动建包

## 验证

```bash
pnpm lint:python
python skills/opendesk/scripts/check_boundary.py
```

## Checklist

- [ ] `pyproject.toml` 已生成
- [ ] workspace member 已注册
- [ ] Ruff first-party 已注册
- [ ] 无业务逻辑
- [ ] lint 通过

## 模板

[../templates/python-package/](../templates/python-package/)
