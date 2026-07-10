# Code Review Guide

## Review 顺序

1. **架构** — 跨层？跨 Feature？
2. **契约** — 是否先改 contracts？
3. **六边形** — UseCase 有无 IO？
4. **命名** — 符合 naming guide？
5. **范围** — 是否改了无关文件？
6. **Lint** — 是否通过？

## Checklist

```
□ 跨层调用检查（React/Python/SQLite）
□ Feature 边界（无直接 import）
□ Contract 变更顺序正确
□ 无 unwrap/expect/panic（Rust 生产路径）
□ Feature UI 无 @tauri-apps/api
□ 无业务逻辑（Skeleton 阶段）
□ 无 Demo 代码
□ pnpm lint 通过
□ check_architecture.py 通过
□ 角色职责正确（A/B/C）
```

## 严重级别

| 级别 | 说明 | 处理 |
|------|------|------|
| P0 | 绕架构（React→Python 等） | 必须阻断 |
| P1 | 未先改 Contract | 必须阻断 |
| P2 | Feature 互 import | 必须修复 |
| P3 | 命名/风格 | 建议修复 |

## Contract PR

- 2+ Approve
- CHANGELOG 已更新
- Breaking → MIGRATION.md

## 自动化

```bash
python skills/opendesk/scripts/check_architecture.py
python skills/opendesk/scripts/lint_all.py
```

## 相关

- [../architecture/principles.md](../architecture/principles.md)
- [release.md](release.md)
