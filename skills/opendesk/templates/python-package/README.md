# Python Package Template

`python/packages/<name>/` 标准结构。

## 目录结构

```
python/packages/<name>/
├── pyproject.toml
├── README.md
├── src/<name>/
│   └── __init__.py
└── tests/
    └── test_scaffold.py
```

## pyproject.toml 要点

- `[project].name` — 包名（import 名）
- `requires-python = ">=3.13"`
- `[build-system]` — hatchling
- `[tool.hatch.build.targets.wheel].packages` — `src/<name>`

## Workspace 注册

根 `pyproject.toml`：

```toml
[tool.uv.workspace]
members = [
  "python/packages/<name>",
  ...
]
```

## 生成

```bash
python skills/opendesk/scripts/create_python_package.py --name <name>
```

## 禁止

业务逻辑 · SQLite · GUI · 直连 React
