//! 只读 SQLite 访问：只读连接 + SQL 只读校验（双保险）。
//!
//! 防线 1（SQLite 层）：连接以 `SQLITE_OPEN_READ_ONLY` 打开，任何写语句由
//! SQLite 直接拒绝。
//!
//! 防线 2（校验层）：执行前先 `prepare`，用 `Statement::readonly()`（SQLite
//! `sqlite3_stmt_readonly`）判定该语句是否只读，非只读语句直接拒绝。同时拒绝
//! 空语句与多语句（防止 `SELECT 1; DROP ...` 形态的注入）。

use std::path::Path;

use rusqlite::{Connection, OpenFlags, Statement};
use serde_json::{json, Value};
use thiserror::Error;

/// 只读访问错误。
#[derive(Debug, Error)]
pub enum Error {
    /// 数据库文件不存在或无法以只读方式打开。
    #[error("无法以只读方式打开数据库 {path}: {source}")]
    CannotOpen {
        /// 数据库路径。
        path: String,
        /// 底层错误。
        source: rusqlite::Error,
    },
    /// SQL 校验未通过。
    #[error("SQL 被拒绝: {0}")]
    Rejected(String),
    /// 语句准备失败（语法错误等）。
    #[error("SQL 无法解析: {0}")]
    Sql(#[from] rusqlite::Error),
}

/// 只读数据库连接。
pub struct ReadOnlyDb {
    conn: Connection,
}

impl ReadOnlyDb {
    /// 以只读模式打开 SQLite 数据库。文件不存在或无法打开时返回
    /// [`Error::CannotOpen`]。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|source| Error::CannotOpen {
            path: path.display().to_string(),
            source,
        })?;
        Ok(Self { conn })
    }

    /// 检查一个语句是否只读；非法（空 / 多语句 / 非只读）时返回
    /// [`Error::Rejected`]。
    fn prepare_read_only(&self, sql: &str) -> Result<Statement<'_>, Error> {
        let normalized = validate_sql(sql)?;
        let statement = self.conn.prepare(&normalized)?;
        if !statement.readonly() {
            return Err(Error::Rejected(format!(
                "该语句不是只读操作（{normalized}）"
            )));
        }
        Ok(statement)
    }

    /// 只读执行一条 SELECT/WITH/PRAGMA 查询，返回列名与最多 `limit` 行。
    pub fn query_rows(&self, sql: &str, limit: usize) -> Result<(Vec<String>, Vec<Value>), Error> {
        let mut statement = self.prepare_read_only(sql)?;
        let columns: Vec<String> = statement
            .column_names()
            .iter()
            .map(|c| c.to_string())
            .collect();
        let mut rows = Vec::with_capacity(limit.min(64));
        let mut query = statement.query([])?;
        while let Some(row) = query.next()? {
            if rows.len() >= limit {
                break;
            }
            let mut object = serde_json::Map::new();
            for (index, column) in columns.iter().enumerate() {
                let value = row.get_ref(index)?;
                object.insert(column.clone(), sqlite_value_to_json(&value));
            }
            rows.push(Value::Object(object));
        }
        Ok((columns, rows))
    }

    /// 列出所有用户表名。
    pub fn list_tables(&self) -> Result<Vec<String>, Error> {
        let (_, rows) = self.query_rows(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            500,
        )?;
        Ok(rows
            .into_iter()
            .filter_map(|row| row.get("name").and_then(Value::as_str).map(str::to_string))
            .collect())
    }

    /// 返回单张表的列、索引与外键定义。
    pub fn table_schema(&self, table: &str) -> Result<Value, Error> {
        let quoted = format!(r#""{}""#, table.replace('"', "\"\""));
        let (_, columns) = self.query_rows(&format!("PRAGMA table_info({quoted})"), 200)?;
        let (_, indexes) = self.query_rows(&format!("PRAGMA index_list({quoted})"), 200)?;
        let (_, foreign_keys) =
            self.query_rows(&format!("PRAGMA foreign_key_list({quoted})"), 200)?;
        Ok(json!({
            "table": table,
            "columns": columns,
            "indexes": indexes,
            "foreign_keys": foreign_keys,
        }))
    }

    /// 统计用户表数量。
    pub fn table_count(&self) -> Result<usize, Error> {
        Ok(self.list_tables()?.len())
    }
}

/// 将 SQLite 单元格值转为 JSON 值；Blob 按 UTF-8 宽容解码。
fn sqlite_value_to_json(value: &rusqlite::types::ValueRef<'_>) -> Value {
    use rusqlite::types::ValueRef;
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => {
            let text = std::str::from_utf8(t).unwrap_or_default();
            json!(text)
        }
        ValueRef::Blob(b) => json!(String::from_utf8_lossy(b)),
    }
}

/// 校验 SQL 是否为只读查询语句，返回去掉尾部 `;` 的归一化语句。
///
/// 规则：非空；禁止多语句（语句内部不允许 `;`）；首个关键字必须是
/// `SELECT` / `WITH` / `EXPLAIN` / `PRAGMA`。语句是否真的只读由
/// [`ReadOnlyDb::prepare_read_only`] 在 `prepare` 后权威判定。
pub fn validate_sql(sql: &str) -> Result<String, Error> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(Error::Rejected("SQL 为空".into()));
    }
    let without_trailing_semicolon = trimmed.strip_suffix(';').unwrap_or(trimmed).trim();
    if without_trailing_semicolon.is_empty() {
        return Err(Error::Rejected("SQL 为空".into()));
    }
    if without_trailing_semicolon.contains(';') {
        return Err(Error::Rejected("不允许一次执行多条语句".into()));
    }
    let first_word = without_trailing_semicolon
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if !matches!(
        first_word.as_str(),
        "SELECT" | "WITH" | "EXPLAIN" | "PRAGMA"
    ) {
        return Err(Error::Rejected(format!(
            "仅允许 SELECT / WITH / EXPLAIN / PRAGMA，收到 {first_word}"
        )));
    }
    Ok(without_trailing_semicolon.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_readonly_sql() {
        assert!(validate_sql("SELECT 1").is_ok());
        assert!(validate_sql("  select * from customer  ").is_ok());
        assert!(validate_sql("WITH x AS (SELECT 1) SELECT * FROM x").is_ok());
        assert!(validate_sql("PRAGMA table_info(customer)").is_ok());
        assert!(validate_sql("EXPLAIN SELECT 1").is_ok());
        assert!(validate_sql("SELECT 1;").is_ok());
    }

    #[test]
    fn rejects_writes_and_multistatement() {
        assert!(validate_sql("").is_err());
        assert!(validate_sql("   ;").is_err());
        assert!(validate_sql("INSERT INTO customer (id) VALUES ('x')").is_err());
        assert!(validate_sql("UPDATE customer SET notes='x'").is_err());
        assert!(validate_sql("DELETE FROM customer").is_err());
        assert!(validate_sql("DROP TABLE customer").is_err());
        assert!(validate_sql("ATTACH 'x' AS y").is_err());
        assert!(validate_sql("SELECT 1; DROP TABLE customer").is_err());
        assert!(validate_sql("SELECT 1; SELECT 2").is_err());
    }

    #[test]
    fn readonly_connection_rejects_writes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        {
            let conn = Connection::open(&path).expect("open write conn");
            conn.execute_batch(
                "CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT);\
                 INSERT INTO t (name) VALUES ('a'), ('b'), ('c');",
            )
            .expect("seed");
        }
        let db = ReadOnlyDb::open(&path).expect("open readonly");
        let (columns, rows) = db.query_rows("SELECT * FROM t", 100).expect("query");
        assert_eq!(columns, vec!["id", "name"]);
        assert_eq!(rows.len(), 3);
        assert!(db
            .query_rows("INSERT INTO t (name) VALUES ('x')", 10)
            .is_err());
        assert!(db.query_rows("PRAGMA journal_mode = WAL", 10).is_err());
    }

    #[test]
    fn limit_truncates_rows() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        {
            let conn = Connection::open(&path).expect("open write conn");
            conn.execute_batch(
                "CREATE TABLE t (id INTEGER); INSERT INTO t VALUES (1),(2),(3),(4),(5);",
            )
            .expect("seed");
        }
        let db = ReadOnlyDb::open(&path).expect("open readonly");
        let (_, rows) = db.query_rows("SELECT * FROM t", 2).expect("query");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn list_tables_and_schema_work() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.db");
        {
            let conn = Connection::open(&path).expect("open write conn");
            conn.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)")
                .expect("seed");
        }
        let db = ReadOnlyDb::open(&path).expect("open readonly");
        assert_eq!(db.list_tables().expect("tables"), vec!["t".to_string()]);
        let schema = db.table_schema("t").expect("schema");
        assert!(schema["columns"].as_array().is_some_and(|c| c.len() == 2));
    }
}
