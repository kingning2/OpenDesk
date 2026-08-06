//! OpenDesk 数据库文件路径解析。
//!
//! 默认与主应用一致：`{data_local}/OpenDesk/{opendesk,crawler}.db`（Windows 即
//! `%LOCALAPPDATA%\OpenDesk\*`）。可用 `--data-dir` 参数或 `OPENDESK_DATA_DIR`
//! 环境变量覆盖数据目录。

use std::path::PathBuf;

/// 逻辑数据库标识，用于工具参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Db {
    /// 主业务库：客户 / 邮件 / 报价 / 工作流。
    Opendesk,
    /// 爬虫库：频道 / 关键词 / 设置。
    Crawler,
}

impl Db {
    /// 数据库名称（用于展示与 MCP 参数）。
    pub fn name(self) -> &'static str {
        match self {
            Self::Opendesk => "opendesk",
            Self::Crawler => "crawler",
        }
    }

    /// 一句话说明。
    pub fn description(self) -> &'static str {
        match self {
            Self::Opendesk => "客户 / 邮件 / 报价 / 工作流主库",
            Self::Crawler => "爬虫频道 / 关键词库",
        }
    }
}

/// 解析数据目录：CLI 参数 > 环境变量 > 平台默认。
pub fn data_dir(cli_override: Option<&str>) -> PathBuf {
    cli_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("OPENDESK_DATA_DIR")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| {
            let mut dir = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
            dir.push("OpenDesk");
            dir
        })
}

/// 返回某数据库的绝对路径。
pub fn db_path(db: Db, data_dir: &std::path::Path) -> PathBuf {
    match db {
        Db::Opendesk => data_dir.join("opendesk.db"),
        Db::Crawler => data_dir.join("crawler.db"),
    }
}
