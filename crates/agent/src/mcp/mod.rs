//! OpenDesk 只读数据库 MCP server 库。
//!
//! 独立二进制 `opendesk-mcp`（stdio transport，供 Claude Code 等客户端）与
//! 桌面应用进程内 `serve_server` 复用同一套工具实现。

pub mod paths;
pub mod readonly;
pub mod tools;

pub use tools::OpendeskMcp;
