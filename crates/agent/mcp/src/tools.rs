//! OpenDesk 只读数据库 MCP 工具。
//!
//! 工具函数统一返回 `Result<String, String>`：成功时 `Ok(JSON 字符串)`（工具级
//! 成功），业务失败（库不存在、SQL 被拒等）时 `Err(描述)`（工具级错误），调用
//! 方都能看到说明。

use std::path::PathBuf;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::paths::{db_path, Db};
use crate::readonly::ReadOnlyDb;

/// 默认查询行数上限。
const DEFAULT_LIMIT: usize = 100;
/// 查询行数硬上限。
const MAX_LIMIT: usize = 500;

/// MCP 工具参数中的数据库标识。
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DbName {
    /// 客户 / 邮件 / 报价 / 工作流主库。
    Opendesk,
    /// 爬虫频道 / 关键词库。
    Crawler,
}

impl DbName {
    fn as_db(&self) -> Db {
        match self {
            Self::Opendesk => Db::Opendesk,
            Self::Crawler => Db::Crawler,
        }
    }

    fn name(&self) -> &'static str {
        self.as_db().name()
    }
}

/// `list_tables` 参数。
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListTablesInput {
    /// 数据库标识。
    pub db: DbName,
}

/// `table_schema` 参数。
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TableSchemaInput {
    /// 数据库标识。
    pub db: DbName,
    /// 表名。
    pub table: String,
}

/// `run_query` 参数。
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunQueryInput {
    /// 数据库标识。
    pub db: DbName,
    /// 只读 SQL（SELECT / WITH / EXPLAIN / PRAGMA）。
    pub sql: String,
    /// 返回行数上限，默认 100，最大 500。
    pub limit: Option<usize>,
}

/// MCP server：持有数据目录，工具方法按需打开只读连接。
pub struct OpendeskMcp {
    data_dir: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl OpendeskMcp {
    /// 以指定数据目录创建 server。
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            tool_router: Self::tool_router(),
        }
    }
}

/// 以只读方式打开指定库；失败时返回错误描述（包含修复提示）。
fn open_readonly(db: Db, data_dir: &std::path::Path) -> Result<ReadOnlyDb, String> {
    let path = db_path(db, data_dir);
    ReadOnlyDb::open(&path).map_err(|error| {
        format!(
            "无法读取 {} 数据库（{}）：{error}\n\n数据库不存在时请先运行桌面应用，或用 --data-dir / OPENDESK_DATA_DIR 指定正确目录。",
            db.name(),
            path.display()
        )
    })
}

#[tool_router]
impl OpendeskMcp {
    /// 列出 OpenDesk 的两个 SQLite 数据库及其状态（路径、是否存在、大小、表数量）。
    #[tool(
        description = "列出 OpenDesk 的数据库：opendesk（客户/邮件/报价/工作流主库）与 crawler（爬虫频道/关键词库），含路径、是否存在、大小、表数量"
    )]
    async fn list_databases(&self) -> Result<String, String> {
        let mut databases = Vec::new();
        for db in [Db::Opendesk, Db::Crawler] {
            let path = db_path(db, &self.data_dir);
            let metadata = std::fs::metadata(&path).ok();
            let exists = metadata.is_some();
            let size_bytes = metadata.as_ref().map(|m| m.len());
            let table_count = if exists {
                open_readonly(db, &self.data_dir)
                    .ok()
                    .and_then(|conn| conn.table_count().ok())
            } else {
                None
            };
            databases.push(json!({
                "db": db.name(),
                "description": db.description(),
                "path": path.display().to_string(),
                "exists": exists,
                "size_bytes": size_bytes,
                "table_count": table_count,
            }));
        }
        Ok(json!({ "databases": databases }).to_string())
    }

    /// 列出某数据库的所有用户表。
    #[tool(description = "列出指定数据库（opendesk 或 crawler）中的所有用户表名")]
    async fn list_tables(
        &self,
        Parameters(input): Parameters<ListTablesInput>,
    ) -> Result<String, String> {
        let conn = open_readonly(input.db.as_db(), &self.data_dir)?;
        let tables = conn
            .list_tables()
            .map_err(|error| format!("查询表列表失败: {error}"))?;
        Ok(json!({
            "db": input.db.name(),
            "tables": tables,
        })
        .to_string())
    }

    /// 查看单张表的列、索引与外键定义。
    #[tool(
        description = "查看指定数据库某张表的列（name/type/notnull/pk/default）、索引与外键定义"
    )]
    async fn table_schema(
        &self,
        Parameters(input): Parameters<TableSchemaInput>,
    ) -> Result<String, String> {
        let conn = open_readonly(input.db.as_db(), &self.data_dir)?;
        let schema = conn
            .table_schema(&input.table)
            .map_err(|error| format!("获取表结构失败（表不存在或不可读）: {error}"))?;
        Ok(schema.to_string())
    }

    /// 只读执行 SQL 查询并返回 JSON 结果。
    ///
    /// 仅允许 SELECT / WITH / EXPLAIN / PRAGMA；连接以 SQLite 只读模式打开，
    /// 任何写入都会被拒绝。敏感列（密码 / API key / token 等）自动脱敏。
    #[tool(
        description = "在指定数据库上只读执行一条 SQL（仅 SELECT/WITH/EXPLAIN/PRAGMA，任何写操作都会被拒绝），返回 JSON；limit 控制行数上限（默认 100，最大 500）"
    )]
    async fn run_query(
        &self,
        Parameters(input): Parameters<RunQueryInput>,
    ) -> Result<String, String> {
        let limit = input.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let conn = open_readonly(input.db.as_db(), &self.data_dir)?;
        let (columns, mut rows) = conn
            .query_rows(&input.sql, limit)
            .map_err(|error| format!("查询失败（{}）: {error}", input.sql))?;

        for row in &mut rows {
            redact_sensitive(row);
        }

        Ok(json!({
            "db": input.db.name(),
            "columns": columns,
            "rows": rows,
            "row_count": rows.len(),
            "limit": limit,
            "truncated": rows.len() >= limit,
        })
        .to_string())
    }
}

#[tool_handler]
impl rmcp::ServerHandler for OpendeskMcp {}

/// 将结果行中的敏感列值替换为脱敏占位。
fn redact_sensitive(row: &mut Value) {
    let Value::Object(map) = row else { return };
    for (column, value) in map.iter_mut() {
        if is_sensitive_column(column) && !value.is_null() {
            *value = Value::String("***redacted***".into());
        }
    }
}

/// 判断列名是否为凭证类敏感列（password / api_key / token / secret / credential）。
fn is_sensitive_column(column: &str) -> bool {
    let lower = column.to_ascii_lowercase();
    [
        "password",
        "api_key",
        "apikey",
        "token",
        "secret",
        "credential",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_credential_columns_only() {
        let mut row = json!({
            "id": 1,
            "password_value": "secret",
            "api_key_ref": "k",
            "name": "ok",
            "subscriber_count": 10,
        });
        redact_sensitive(&mut row);
        assert_eq!(row["password_value"], "***redacted***");
        assert_eq!(row["api_key_ref"], "***redacted***");
        assert_eq!(row["name"], "ok");
        assert_eq!(row["id"], 1);
        assert_eq!(row["subscriber_count"], 10);
    }

    fn seed_db(dir: &std::path::Path) {
        let conn = rusqlite::Connection::open(dir.join("opendesk.db")).expect("open");
        conn.execute_batch(
            "CREATE TABLE customer (id TEXT PRIMARY KEY, email TEXT, password_value TEXT);\
             INSERT INTO customer VALUES ('1','a@b.com','pw');\
             INSERT INTO customer VALUES ('2','c@d.com','pw2');",
        )
        .expect("seed");
    }

    #[tokio::test]
    async fn run_query_reads_real_db_and_redacts() {
        let dir = tempfile::tempdir().expect("tempdir");
        seed_db(dir.path());
        let server = OpendeskMcp::new(dir.path().to_path_buf());
        let out = server
            .run_query(Parameters(RunQueryInput {
                db: DbName::Opendesk,
                sql: "SELECT * FROM customer".into(),
                limit: None,
            }))
            .await
            .expect("run_query");
        let parsed: Value = serde_json::from_str(&out).expect("json");
        assert_eq!(parsed["row_count"], 2);
        assert_eq!(parsed["rows"][0]["password_value"], "***redacted***");
        assert_eq!(parsed["rows"][0]["email"], "a@b.com");
        assert_eq!(parsed["truncated"], false);
    }

    #[tokio::test]
    async fn run_query_honors_limit() {
        let dir = tempfile::tempdir().expect("tempdir");
        seed_db(dir.path());
        let server = OpendeskMcp::new(dir.path().to_path_buf());
        let out = server
            .run_query(Parameters(RunQueryInput {
                db: DbName::Opendesk,
                sql: "SELECT * FROM customer".into(),
                limit: Some(1),
            }))
            .await
            .expect("run_query");
        let parsed: Value = serde_json::from_str(&out).expect("json");
        assert_eq!(parsed["row_count"], 1);
        assert_eq!(parsed["truncated"], true);
    }

    #[tokio::test]
    async fn run_query_rejects_write_sql() {
        let dir = tempfile::tempdir().expect("tempdir");
        seed_db(dir.path());
        let server = OpendeskMcp::new(dir.path().to_path_buf());
        let err = server
            .run_query(Parameters(RunQueryInput {
                db: DbName::Opendesk,
                sql: "UPDATE customer SET email='x'".into(),
                limit: None,
            }))
            .await
            .expect_err("should reject write");
        assert!(
            err.contains("拒绝") || err.contains("不是只读"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn list_tables_and_schema_tools() {
        let dir = tempfile::tempdir().expect("tempdir");
        seed_db(dir.path());
        let server = OpendeskMcp::new(dir.path().to_path_buf());
        let tables = server
            .list_tables(Parameters(ListTablesInput {
                db: DbName::Opendesk,
            }))
            .await
            .expect("list_tables");
        assert!(tables.contains("customer"));

        let schema = server
            .table_schema(Parameters(TableSchemaInput {
                db: DbName::Opendesk,
                table: "customer".into(),
            }))
            .await
            .expect("table_schema");
        assert!(schema.contains("password_value"));
    }

    #[tokio::test]
    async fn missing_database_reports_friendly_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let server = OpendeskMcp::new(dir.path().to_path_buf());
        let err = server
            .list_tables(Parameters(ListTablesInput {
                db: DbName::Opendesk,
            }))
            .await
            .expect_err("db does not exist");
        assert!(err.contains("无法读取"), "unexpected error: {err}");
    }
}
