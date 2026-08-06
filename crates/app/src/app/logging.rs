//! Application tracing subscriber initialization.
//!
//! 复用 kernel 的文件日志（写 `{data}/OpenDesk/logs/opendesk.log.*`），
//! 与 worker 进程对齐，便于排查下载 / 安装等后台任务。

pub fn init_tracing() {
    kernel::logging::init_tracing("opendesk");
}
