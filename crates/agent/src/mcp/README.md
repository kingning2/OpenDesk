# opendesk-mcp

OpenDesk 的**只读**数据库 MCP Server（位于 `crates/agent/src/mcp/`）。给 Claude Code 等 AI 提供一个可安全查询 OpenDesk 业务数据的 MCP 接口。

**核心承诺：只能读，绝不可能写。** 双重防线（见 [只读安全设计](#只读安全设计)），并自动脱敏邮件密码等凭证列。

## 定位：AI 自己写 SQL

本服务的核心是 `run_query`——**查询语句由 AI 自行编写**（任意只读 SQL：SELECT / WITH / EXPLAIN / PRAGMA）。元数据工具只用来帮 AI 摸清库里有啥（有哪些库、哪些表、表的列/索引/外键），以便写出正确查询，并非内置的固定查询。

## 工具集

| 工具 | 参数 | 说明 |
| --- | --- | --- |
| `list_databases` | — | 列出两个 SQLite 库（路径 / 是否存在 / 大小 / 表数量 / 一句话说明） |
| `list_tables` | `db` | 列出某库全部用户表 |
| `table_schema` | `db, table` | 单表列 / 索引 / 外键定义（PRAGMA） |
| `run_query` | `db, sql, limit?` | **核心**：只读执行 AI 编写的 SQL，返回 JSON；`limit` 默认 100、最大 500 |

`db` 取值：`opendesk`（客户 / 邮件 / 报价 / 工作流主库）、`crawler`（爬虫频道 / 关键词库）。

## 与 Claude Code 集成

仓库根已配置 `.mcp.json`：

```json
{
  "mcpServers": {
    "opendesk-db": {
      "command": "cargo",
      "args": ["run", "--release", "-p", "agent", "--bin", "opendesk-mcp"]
    }
  }
}
```

首次启动会编译（约几十秒）。日常使用建议先构建一次，改用秒启的二进制：

```bash
cargo build --release -p agent --bin opendesk-mcp
# 将 .mcp.json 的 args 换成 target/release/opendesk-mcp.exe
```

在 Claude Code 中重载后，即可在对话里让 AI 执行 `list_databases` → `list_tables` → `run_query` 分析业务数据。

## 数据目录解析

数据库文件位于 `{data_local}/OpenDesk/{opendesk,crawler}.db`（Windows 即 `%LOCALAPPDATA%\OpenDesk\*`），与主应用一致。可用以下方式覆盖目录：

```bash
# CLI 参数（最高优先级）
opendesk-mcp --data-dir /path/to/data
# 或环境变量
OPENDESK_DATA_DIR=/path/to/data opendesk-mcp
```

> 库不存在时会返回友好错误（"数据库不存在，请先运行桌面应用"），不会创建空库。

## 只读安全设计

1. **SQLite 层**：连接以 `SQLITE_OPEN_READ_ONLY` 打开，任何写语句由 SQLite 直接拒绝。
2. **SQL 校验层**（执行前拦截）：首个关键字必须是 `SELECT` / `WITH` / `EXPLAIN` / `PRAGMA`；拒绝多语句（含 `;`）与写关键字。
3. **敏感列脱敏**：结果中含密码 / API key / token / secret / credential 的列，值替换为 `***redacted***`（`run_query` 与 `table_schema` 均生效）。

即使校验器有疏漏，只读连接也能保证无法写库——这是最终防线。

> 安全提示：库内含明文邮件密码（`mail_account.password_value`）与 LLM key 引用，虽已脱敏，仍请避免让 AI 导出或外传查询结果。

## 开发

```bash
cargo test -p agent --lib mcp   # 单元 + 集成测试（校验器 / 只读强制 / limit / 脱敏）
cargo build -p agent --bin opendesk-mcp
```
