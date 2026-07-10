# Recipe: Add UseCase

## 修改顺序

1. `create_usecase.py --crate X --name Y`
2. 在 `crates/<crate>/src/app/` 添加模块
3. 注入 Port trait（构造函数参数）
4. 单元测试 Mock Port 骨架
5. `pnpm lint:rust`

## 禁止

UseCase 内：SQL · HTTP · FS · SQLite · Tauri · Python

## Checklist

- [ ] 仅依赖 `ports` trait 与 `domain` 类型
- [ ] 返回 `Result<T, FeatureError>`
- [ ] `#[instrument]` 日志字段

## 模板

[../templates/usecase/](../templates/usecase/)
