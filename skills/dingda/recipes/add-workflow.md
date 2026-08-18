# Recipe: Add Workflow

## 修改顺序

1. Contract: workflow dto + ipc
2. `crates/workflow/` + `python/packages/workflow/`
3. Event: `workflow.step.completed` 等
4. 前端 `features/workflow/`

## 禁止

- workflow crate 直接 use agent crate

## 相关

[add-event.md](add-event.md) · [add-feature.md](add-feature.md)
