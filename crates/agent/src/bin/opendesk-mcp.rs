//! OpenDesk 只读数据库 MCP Server 入口。
//!
//! 以 stdio transport 运行，供 Claude Code 等 MCP 客户端通过 `cargo run
//! -p agent --bin opendesk-mcp` 启动。数据库路径默认 `{data_local}/OpenDesk`，可用
//! `--data-dir <path>` 或 `OPENDESK_DATA_DIR` 覆盖。

use agent::mcp::paths::data_dir;
use agent::mcp::OpendeskMcp;
use rmcp::serve_server;

const USAGE: &str = "\
opendesk-mcp — OpenDesk 只读数据库 MCP Server

用法:
  opendesk-mcp [--data-dir <path>]

选项:
  --data-dir <path>  指定 OpenDesk 数据目录（默认 %LOCALAPPDATA%\\OpenDesk，
                     或环境变量 OPENDESK_DATA_DIR 覆盖）
  -h, --help         显示帮助
";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    let data_dir_override = parse_args()?;
    let dir = data_dir(data_dir_override.as_deref());
    eprintln!("[opendesk-mcp] 数据目录: {}", dir.display());

    let server = OpendeskMcp::new(dir);
    let service = serve_server(server, rmcp::transport::stdio()).await?;
    // 保持服务运行，直到客户端关闭 stdin（QuitReason::Closed）。
    service.waiting().await?;
    Ok(())
}

/// 初始化 stderr 日志（MCP stdio 通道上 stdout 是协议流量，日志一律走 stderr）。
fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("rmcp=info,agent=debug"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

/// 解析 CLI 参数，返回 `--data-dir` 覆盖值。
fn parse_args() -> Result<Option<String>, String> {
    let mut args = std::env::args().skip(1);
    let mut data_dir_override = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                let value = args.next().ok_or("--data-dir 需要一个路径参数")?;
                data_dir_override = Some(value);
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("未知参数: {other}\n\n{USAGE}")),
        }
    }
    Ok(data_dir_override)
}
