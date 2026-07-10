# Feature Template

垂直 Feature 的标准目录结构（无业务逻辑）。

## 目录结构

```
crates/<feature>/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── app/
│   │   └── mod.rs
│   └── domain/
│       └── mod.rs
└── tests/
    └── scaffold_test.rs

apps/desktop/src/features/<feature>/
├── index.ts
├── pages/
├── components/
└── hooks/

contracts/schema/v1/<feature>/
├── dto/
├── ipc/
├── event/
└── error/
```

## TODO

- [ ] 定义 contract schema
- [ ] 注册 workspace member
- [ ] 注册 Tauri commands（空实现）
- [ ] 添加 platform IPC wrapper
- [ ] 注册 sidebar / route 元数据

## 生成

```bash
python skills/opendesk/scripts/create_feature.py --name <feature>
```

## 禁止

- 跨 feature import
- Feature UI 使用 @tauri-apps/api
- 在骨架阶段写业务逻辑
