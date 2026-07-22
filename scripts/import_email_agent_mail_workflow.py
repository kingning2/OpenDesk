#!/usr/bin/env python3
"""
Import email-agent mailbox accounts and script library into OpenDesk's local opendesk.db.
"""

from __future__ import annotations

import json
import os
import sqlite3
from datetime import UTC, datetime
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parents[1]
EMAIL_AGENT_ROOT = WORKSPACE_ROOT.parent / "email-agent"
EMAIL_ACCOUNTS_PATH = EMAIL_AGENT_ROOT / "data" / "email-accounts.json"
SCRIPTS_PATH = EMAIL_AGENT_ROOT / "data" / "scripts.json"
_local_app_data = os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local")
DB_PATH = Path(_local_app_data) / "OpenDesk" / "opendesk.db"
MAIL_TABLE_MIGRATION = (
    WORKSPACE_ROOT
    / "crates"
    / "storage"
    / "migrations-opendesk"
    / "2026-07-21-173000_create_mail_tables"
    / "up.sql"
)
SCRIPT_TABLE_MIGRATION = (
    WORKSPACE_ROOT
    / "crates"
    / "storage"
    / "migrations-opendesk"
    / "2026-07-21-181500_create_script_snippet_table"
    / "up.sql"
)


def now_ms() -> str:
    return str(int(datetime.now(tz=UTC).timestamp() * 1000))


def ensure_db_dir() -> None:
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)


def ensure_script_table(conn: sqlite3.Connection) -> None:
    conn.executescript(MAIL_TABLE_MIGRATION.read_text(encoding="utf-8"))
    migration_sql = SCRIPT_TABLE_MIGRATION.read_text(encoding="utf-8")
    conn.executescript(migration_sql)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def import_accounts(conn: sqlite3.Connection) -> int:
    payload = load_json(EMAIL_ACCOUNTS_PATH)
    accounts = payload.get("accounts", [])
    now = now_ms()
    rows = []
    for item in accounts:
        rows.append(
            (
                item["id"],
                item.get("label") or item.get("email") or item["auth"]["user"],
                item.get("email") or item["auth"]["user"],
                None,
                item["smtp"]["host"],
                int(item["smtp"]["port"]),
                1 if item["smtp"].get("ssl", True) else 0,
                item["auth"]["user"],
                f"inline:{item['id']}",
                item["auth"]["pass"],
                item.get("imap", {}).get("host"),
                item.get("imap", {}).get("port"),
                1 if item.get("imap", {}).get("ssl", True) else 0,
                1 if item.get("imap", {}).get("host") else 0,
                now,
                now,
            )
        )

    conn.executemany(
        """
        INSERT INTO mail_account (
            id, label, from_address, from_name, smtp_host, smtp_port, use_tls,
            username, password_ref, password_value, imap_host, imap_port,
            imap_use_tls, imap_sync_enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            label = excluded.label,
            from_address = excluded.from_address,
            from_name = excluded.from_name,
            smtp_host = excluded.smtp_host,
            smtp_port = excluded.smtp_port,
            use_tls = excluded.use_tls,
            username = excluded.username,
            password_ref = excluded.password_ref,
            password_value = excluded.password_value,
            imap_host = excluded.imap_host,
            imap_port = excluded.imap_port,
            imap_use_tls = excluded.imap_use_tls,
            imap_sync_enabled = excluded.imap_sync_enabled,
            updated_at = excluded.updated_at
        """,
        rows,
    )
    return len(rows)


def build_script_title(item: dict) -> str:
    return item.get("description") or item.get("trigger") or item.get("id") or "Untitled"


def import_scripts(conn: sqlite3.Connection) -> int:
    payload = load_json(SCRIPTS_PATH)
    scripts = payload.get("scripts", [])
    now = now_ms()
    rows = []
    for index, item in enumerate(scripts, start=1):
        rows.append(
            (
                f"script_{item['id']}",
                item["id"],
                build_script_title(item),
                item.get("stage"),
                item.get("trigger"),
                item.get("description"),
                item.get("from_stage"),
                item.get("to_stage"),
                json.dumps(item.get("tags", []), ensure_ascii=False),
                item.get("content", ""),
                item.get("category1"),
                item.get("category2"),
                1 if item.get("needs_boss_input") else 0,
                item.get("boss_input_hint"),
                index,
                now,
                now,
            )
        )

    conn.executemany(
        """
        INSERT INTO script_snippet (
            id, source_id, title, stage, trigger_text, description, from_stage,
            to_stage, tags_json, body_text, category_l1, category_l2,
            needs_boss_input, boss_input_hint, sort_order, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(source_id) DO UPDATE SET
            title = excluded.title,
            stage = excluded.stage,
            trigger_text = excluded.trigger_text,
            description = excluded.description,
            from_stage = excluded.from_stage,
            to_stage = excluded.to_stage,
            tags_json = excluded.tags_json,
            body_text = excluded.body_text,
            category_l1 = excluded.category_l1,
            category_l2 = excluded.category_l2,
            needs_boss_input = excluded.needs_boss_input,
            boss_input_hint = excluded.boss_input_hint,
            sort_order = excluded.sort_order,
            updated_at = excluded.updated_at
        """,
        rows,
    )
    return len(rows)


def main() -> int:
    ensure_db_dir()
    with sqlite3.connect(DB_PATH) as conn:
        conn.execute("PRAGMA foreign_keys = ON;")
        ensure_script_table(conn)
        account_count = import_accounts(conn)
        script_count = import_scripts(conn)
        conn.commit()
    print(f"Imported {account_count} mail accounts into {DB_PATH}")
    print(f"Imported {script_count} script snippets into {DB_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
