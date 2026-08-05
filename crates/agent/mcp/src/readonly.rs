//! 只读 SQLite 访问：只读连接 + SQL 只读校验（双保险）。
//!
//! 防线 1（SQLite 层）：连接以 `SQLITE_OPEN_READ_ONLY` 打开，任何写语句由
//! SQLite 直接拒绝。
//!
//! 防线 2（校验层）：执行前先 `prepare`，用 `Statement::readonly()`（SQLite
//! `sqlite3_stmt_readonly`）判定该语句是否只读，非只读语句直接拒绝。同时拒绝
//! 空语句与多语句（防止 `SELECT 1; DROP ...` 形态的注入）。

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags, Statement};
use serde_json::{json, Value};
use thiserror::Error;

/// 分析 SQL 实际读取表时 `EXPLAIN` 结果的行数上限（足够容纳任意复杂查询的 VDBE 程序）。
const EXPLAIN_ROW_CAP: usize = 100_000;

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

    /// 只读执行一条查询，但仅当该语句实际读取的所有表都在 `allowed_tables` 白名单内。
    ///
    /// 表引用通过 `EXPLAIN` 反查：SQLite 会把每条真实表/索引读取编译成
    /// `OpenRead` opcode，把其根页映射回 `sqlite_master` 即可拿到实际读取的表名，
    /// 子查询 / JOIN / CTE 展开后的读取都能被覆盖。任何白名单外的表被读取即拒绝。
    pub fn query_rows_if_allowed(
        &self,
        sql: &str,
        limit: usize,
        allowed_tables: &[&str],
    ) -> Result<(Vec<String>, Vec<Value>), Error> {
        self.check_table_allowlist(sql, allowed_tables)?;
        self.query_rows(sql, limit)
    }

    /// 校验语句只访问白名单内的表；子查询 / JOIN / CTE 通过 EXPLAIN 反查覆盖。
    ///
    /// PRAGMA 不走常规表读取，分两类单独校验：
    /// - `PRAGMA name(arg)` 语句：编译不产生 `OpenRead`，其括号参数按表名校验，
    ///   且必须带括号参数——无参数 PRAGMA（`table_list` / `database_list` 等）会
    ///   枚举全部表或泄露文件路径，一律拒绝。
    /// - pragma 表值函数（`SELECT * FROM pragma_table_list`、
    ///   `pragma_table_info('x')`）：编译成 `VFilter` 而非 `OpenRead`，需文本级
    ///   扫描其表参数；`pragma_table_list` 等无参数形式会列出全部表，直接拒绝。
    fn check_table_allowlist(&self, sql: &str, allowed_tables: &[&str]) -> Result<(), Error> {
        let normalized = validate_sql(sql)?;
        for tvf in pragma_tvf_tables(&normalized) {
            match tvf {
                None => {
                    return Err(Error::Rejected(
                        "不允许使用列出全部表的 pragma 表值函数（如 pragma_table_list）".into(),
                    ));
                }
                Some(table) if !allowed_tables.contains(&table.as_str()) => {
                    return Err(Error::Rejected(format!("不允许访问表 {table}（白名单外）")));
                }
                Some(_) => {}
            }
        }
        let first_word = normalized
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if first_word == "PRAGMA" {
            let Some(target) = pragma_table_arg(&normalized) else {
                return Err(Error::Rejected("不允许无表参数的 PRAGMA".into()));
            };
            if !allowed_tables.contains(&target.as_str()) {
                return Err(Error::Rejected(format!(
                    "不允许访问表 {target}（白名单外）"
                )));
            }
            return Ok(());
        }
        for table in self.referenced_tables(&normalized)? {
            if !allowed_tables.contains(&table.as_str()) {
                return Err(Error::Rejected(format!("不允许访问表 {table}（白名单外）")));
            }
        }
        Ok(())
    }

    /// 反查一条只读 SQL 实际读取到的表名（按读取顺序，去重）。
    fn referenced_tables(&self, sql: &str) -> Result<Vec<String>, Error> {
        let normalized = validate_sql(sql)?;
        let (_, rows) = self.query_rows(&format!("EXPLAIN {normalized}"), EXPLAIN_ROW_CAP)?;
        let root_to_table = self.root_page_table_map()?;
        let mut tables = Vec::new();
        for row in rows {
            let opcode = row.get("opcode").and_then(Value::as_str).unwrap_or("");
            if !matches!(opcode, "OpenRead" | "OpenWrite") {
                continue;
            }
            let Some(page) = row.get("p2").and_then(Value::as_i64) else {
                continue;
            };
            if let Some(table) = root_to_table.get(&page) {
                if !tables.iter().any(|existing| existing == table) {
                    tables.push(table.clone());
                }
            }
        }
        Ok(tables)
    }

    /// 建立 `sqlite_master` 根页 → 所属表名的映射（表和索引的根页都归到所属表）。
    fn root_page_table_map(&self) -> Result<HashMap<i64, String>, Error> {
        let (_, rows) = self.query_rows(
            "SELECT name, tbl_name, rootpage FROM sqlite_master WHERE rootpage > 0",
            10_000,
        )?;
        let mut map = HashMap::new();
        for row in rows {
            let table = row
                .get("tbl_name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if let Some(page) = row.get("rootpage").and_then(Value::as_i64) {
                if page > 0 && !table.is_empty() {
                    map.insert(page, table);
                }
            }
        }
        Ok(map)
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

/// 提取 `PRAGMA name(arg)` 中的括号参数并去掉引号；无括号参数返回 `None`。
fn pragma_table_arg(sql: &str) -> Option<String> {
    let open = sql.find('(')?;
    let rest = &sql[open + 1..];
    let close = rest.find(')')?;
    let arg = rest[..close].trim();
    if arg.is_empty() {
        return None;
    }
    Some(arg.trim_matches('"').trim_matches('\'').to_string())
}

/// 扫描 SQL 中所有 pragma 表值函数引用，返回每个引用对应的表参数。
///
/// `None` 表示该引用无表参数且会列出全部表（如 `pragma_table_list` /
/// `pragma_table_xinfo` / `pragma_function_list` 等），调用方应直接拒绝。
/// 带表参数的（如 `pragma_table_info('x')` / `pragma_table_info(x)`）返回其参数，
/// 由调用方按白名单校验。扫描会跳过字符串字面量，避免 `LIKE '%pragma_%'`
/// 之类误报。
fn pragma_tvf_tables(sql: &str) -> Vec<Option<String>> {
    const LEAK_ALL: &[&str] = &[
        "pragma_table_list",
        "pragma_table_xinfo",
        "pragma_function_list",
        "pragma_module_list",
        "pragma_database_list",
        "pragma_collation_list",
    ];
    let chars: Vec<char> = sql.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\'' | '"' | '`' => {
                let quote = chars[i];
                i += 1;
                while i < chars.len() {
                    if chars[i] == quote {
                        if i + 1 < chars.len() && chars[i + 1] == quote {
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let ident: String = chars[start..i].iter().collect();
                let lower = ident.to_ascii_lowercase();
                if LEAK_ALL.contains(&lower.as_str()) {
                    out.push(None);
                } else if lower.starts_with("pragma_") {
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    let arg = if i < chars.len() && chars[i] == '(' {
                        i += 1;
                        while i < chars.len() && chars[i].is_whitespace() {
                            i += 1;
                        }
                        if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                            let quote = chars[i];
                            i += 1;
                            let mut s = String::new();
                            while i < chars.len() && chars[i] != quote {
                                s.push(chars[i]);
                                i += 1;
                            }
                            if i < chars.len() {
                                i += 1;
                            }
                            Some(s)
                        } else {
                            let arg_start = i;
                            while i < chars.len()
                                && chars[i] != ','
                                && chars[i] != ')'
                                && !chars[i].is_whitespace()
                            {
                                i += 1;
                            }
                            let raw: String = chars[arg_start..i].iter().collect();
                            if raw.is_empty() {
                                None
                            } else {
                                Some(raw)
                            }
                        }
                    } else {
                        None
                    };
                    out.push(arg);
                }
            }
            _ => i += 1,
        }
    }
    out
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

    fn seed_allowlist_db(dir: &std::path::Path) {
        let path = dir.join("test.db");
        let conn = Connection::open(&path).expect("open");
        conn.execute_batch(
            "CREATE TABLE customer (id INTEGER);\
             CREATE TABLE mail_account (id INTEGER);\
             INSERT INTO customer VALUES (1), (2);\
             INSERT INTO mail_account VALUES (99);",
        )
        .expect("seed");
    }

    #[test]
    fn rejects_pragma_and_tvf_table_enumeration() {
        let dir = tempfile::tempdir().expect("tempdir");
        seed_allowlist_db(dir.path());
        let db = ReadOnlyDb::open(dir.path().join("test.db")).expect("open");
        let allowed = ["customer"];

        assert!(db
            .query_rows_if_allowed("PRAGMA table_list", 100, &allowed)
            .is_err());
        assert!(db
            .query_rows_if_allowed("PRAGMA database_list", 100, &allowed)
            .is_err());
        assert!(db
            .query_rows_if_allowed("SELECT * FROM pragma_table_list", 100, &allowed)
            .is_err());
        assert!(db
            .query_rows_if_allowed(
                "SELECT name FROM pragma_table_info('mail_account')",
                100,
                &allowed
            )
            .is_err());
        assert!(db
            .query_rows_if_allowed(
                "SELECT * FROM pragma_table_info(mail_account)",
                100,
                &allowed
            )
            .is_err());
        assert!(
            db.query_rows_if_allowed(
                "SELECT id FROM customer WHERE id LIKE '%pragma_table_list%'",
                100,
                &allowed
            )
            .is_ok(),
            "字符串字面量里的 pragma_ 不应误报"
        );
    }

    #[test]
    fn query_rows_if_allowed_enforces_allowlist() {
        let dir = tempfile::tempdir().expect("tempdir");
        seed_allowlist_db(dir.path());
        let db = ReadOnlyDb::open(dir.path().join("test.db")).expect("open");
        let allowed = ["customer"];

        assert!(db
            .query_rows_if_allowed("SELECT * FROM customer", 100, &allowed)
            .is_ok());
        assert!(db.query_rows_if_allowed("SELECT 1", 100, &allowed).is_ok());
        assert!(db
            .query_rows_if_allowed("PRAGMA table_info(customer)", 100, &allowed)
            .is_ok());

        let err = db
            .query_rows_if_allowed("SELECT * FROM mail_account", 100, &allowed)
            .expect_err("hidden table rejected");
        assert!(err.to_string().contains("白名单"), "unexpected: {err}");

        let err = db
            .query_rows_if_allowed(
                "SELECT * FROM customer WHERE id IN (SELECT id FROM mail_account)",
                100,
                &allowed,
            )
            .expect_err("subquery hidden table rejected");
        assert!(err.to_string().contains("白名单"), "unexpected: {err}");

        let err = db
            .query_rows_if_allowed("PRAGMA table_info(mail_account)", 100, &allowed)
            .expect_err("hidden table pragma rejected");
        assert!(err.to_string().contains("白名单"), "unexpected: {err}");
    }
}
