# Changelog

## 0.1.6

- crawler `job_config` / `job.start`（IPC + Sidecar）增加可选字段 `api_key`
- 说明：由前端写入，经 Rust 下发；Python 真 YouTube Adapter 使用；禁止写入日志

## 0.1.5

- 新增 `crawler` 契约：job-config / channel-result DTO
- 新增 crawler IPC + sidecar：`job.start` / `job.cancel` / `job.status`
- 新增 crawler 事件：`job.started` / `job.progress` / `job.log` / `job.completed` / `job.failed`
- 首版 platform 仅约定 `youtube`，枚举可扩展

## 0.1.4

- 新增 Python Sidecar stdout JSON Lines 日志契约 `runtime/log/entry/v1`

## 0.1.3

- Sidecar 崩溃/health 失败自动重启 + `runtime.sidecar.restarted` 事件
- `InMemoryEventBus` / `InMemoryTaskScheduler` 可用骨架
- `RecordStore` CRUD port + 内存占位实现
- Sidecar 管理面 `/stats` `/tasks/active` `/metrics` `/debug/dump`
- react-router 路由 + agent Feature 垂直切片模板

## 0.1.2

- Rust 接管 Python sidecar 生命周期（启动 / 健康检查 / 停止）
- 新增 `kernel::event` / `kernel::task` 与 `ports::RecordStore` 骨架

## 0.1.1

- 新增 `agent/ipc/ping` Tauri IPC 契约
- `sync_contracts.py` 生成三端 DTO 类型
- 打通 `agent/ping` 端到端骨架（React → Rust → Python sidecar）

## 0.1.0

- 初始化契约目录结构
