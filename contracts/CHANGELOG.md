# Changelog

## 0.1.19

- `channel/sidecar/qr_start|qr_check|qr_cancel` 请求新增可选 `platform`（`xianyu` / `ali1688`），支持分站扫码

## 0.1.18

- 新增 `channel/sidecar/cookie_renew`：风控（验证码 / RGV587）时由 Playwright 临时续期 Cookie

## 0.1.17

- 新增 `plugin` 契约：插件条目 DTO、`list` / `install` / `uninstall` IPC、`progress` 事件（设置页 OCR tessdata 按需下载）

## 0.1.16

- `channel/ipc/open_site.request` 新增必填布局字段 `x` / `y` / `width` / `height`：主窗口内嵌子 WebView 的逻辑像素 bounds

## 0.1.15

- `qr_check` IPC / sidecar 响应新增可选 `qr_base64`：二维码过期时侧车原地刷新并返回新图

## 0.1.14

- 移除 `channel` 快照登录契约：`login` IPC + `sidecar/login`（扫码登录成为唯一登录方式）
- `qr_start` IPC 请求新增可选 `name` / `kind`：无账号时扫码登录成功自动创建账号

## 0.1.13

- 新增 `channel` 契约：`qr_start` / `qr_check` / `qr_cancel` IPC + sidecar（Playwright 扫码登录）

## 0.1.12

- 新增 `channel` 契约：`cookie` DTO、`login` / `open_site` / `close_site` IPC、`sidecar/login`（Playwright 浏览器登录）
- `account.credential` 语义升级：可存浏览器快照 JSON / cookies 数组 / 旧 cookie 字符串（兼容）
- codegen：Rust 生成 camelCase JSON 字段为 snake_case + `#[serde(rename)]`，消除 clippy 警告

## 0.1.11

- 新增 `channel` 契约：account / conversation / message / settings DTO
- 新增 channel IPC：`state` / `connect` / `disconnect` / `send`
- 新增 channel 事件：`message` / `status`
- 新增 `llm` 契约与 sidecar：`chat` / `classify`（纯大模型接入，渠道无关）

## 0.1.10

- crawler IPC：`job.results` — 查询任务已收录频道（`results_json` 数组）

## 0.1.9

- crawler IPC：`keywords.import` / `keywords.batches`（CSV → SQLite）
- `job.start` IPC：`keywords` 改为可选，由 `batch_id` 从 Rust DB 解析

## 0.1.8

- crawler `job.status` / `job.progress` 增加运营进度字段：`message`、`current_keyword`、`keyword_scanned` / `keyword_accepted`、`quota_used`、`keyword_stats_json`、`error_message`
- 供桌面 UI 展示「当前关键词 / 已爬数量 / 失败或配额停」，替代技术 phase 日志面板

## 0.1.7

- 新增 crawler `job.logs`（IPC + Sidecar）：返回 `logs_json` 过程日志数组字符串，供前端任务日志面板轮询

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
