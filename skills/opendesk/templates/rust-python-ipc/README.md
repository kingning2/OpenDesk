# Rust ↔ Python IPC Template

## 生成

```bash
python skills/opendesk/scripts/create_rust_python_ipc.py --feature <feature> --action <action>
```

## 文件清单

| 产物 | 模板来源 |
|------|----------|
| JSON Schema | `create_rust_python_ipc.py` inline |
| OpenAPI path | `sidecar.path.yaml.tpl` |
| Rust route | `rust_route.rs.tpl` |
| Python handler | `python_handler.py.tpl` |

## 禁止

业务逻辑 · React 直连 · Python 持久化
