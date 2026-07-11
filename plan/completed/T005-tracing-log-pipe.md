| Field | Value |
|-------|-------|
| ID | T005 |
| Priority | P0 |
| Status | completed |
| Depends on | T004 |
| Blocks | T006 |
| Milestone | M1 |

## Goal

初始化 tracing subscriber，并将 Python JSON 日志解析为带 `trace_id` 的结构化 tracing 事件。

## Scope

- `app::init_tracing()` + `tracing-subscriber`
- Python `JsonLogFormatter`（stdout JSONL）
- `runtime::sidecar::log_pipe` 解析并映射 level / trace_id / feature

## Out of scope

- 日志落盘
- OpenTelemetry 导出

## Acceptance criteria

- [x] `launch()` 首行调用 `init_tracing()`
- [x] Python handler `extra={"trace_id", "feature"}` 出现在 stdout JSON
- [x] Rust `log_pipe::emit_line` 解析 JSON 并输出 tracing 字段
- [x] 非 JSON 行降级为 unstructured 日志
- [x] 无 unwrap/expect（含测试）

## Key files

- `crates/app/src/logging.rs`
- `crates/runtime/src/sidecar/log_pipe.rs`
- `python/sidecar/sidecar/logging_config.py`
