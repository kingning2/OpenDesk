#!/usr/bin/env python3
"""
Migrate real business data from email-agent into OpenDesk opendesk.db.

Skips logs/audit/cache (contact_logs, open_events, reply-audit, etc.).

作者：Xiaoman
创建时间：2026-07-22

Usage:
  python skills/opendesk/scripts/migrate_email_agent.py
  EMAIL_AGENT_DATA=D:/path/to/email-agent/data python skills/opendesk/scripts/migrate_email_agent.py
"""

from __future__ import annotations

import hashlib
import json
import logging
import os
import re
import sqlite3
import subprocess
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DEFAULT_EA_DATA = ROOT.parent / "email-agent" / "data"
EA_DATA = Path(os.environ.get("EMAIL_AGENT_DATA", DEFAULT_EA_DATA))
LOCAL_APP = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
DB_PATH = Path(os.environ.get("OPENDESK_DB", LOCAL_APP / "OpenDesk" / "opendesk.db"))

KEYRING_SERVICE = "OpenDesk"
LLM_KEYRING_USER = "llm_api_key"
LLM_SETTINGS_ROW_ID = "default"
SOURCE_CHANNEL = "email-agent"

LIFECYCLE_BY_STAGE = {
    "archived": "paused",
    "s_bounce": "lost",
}

LEGACY_JSON_FILES = {
    "stages.json": "email-agent:stages",
    "workflow-templates.json": "email-agent:workflow-templates",
    "workflow-stages.json": "email-agent:workflow-stages",
    "flow-rules.json": "email-agent:flow-rules",
    "pricing-config.json": "email-agent:pricing-config",
}

EMAIL_RE = re.compile(r"[\w.+-]+@[\w.-]+\.\w+")


def setup_logging() -> None:
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")


def now_ms() -> str:
    return str(int(datetime.now(tz=UTC).timestamp() * 1000))


def stable_id(prefix: str, key: str) -> str:
    digest = hashlib.sha256(key.encode("utf-8")).hexdigest()[:16]
    return f"{prefix}{digest}"


def mail_keyring_user(account_id: str) -> str:
    return f"mail_account/{account_id}"


def mail_password_ref(account_id: str) -> str:
    return f"keyring:{KEYRING_SERVICE}/{mail_keyring_user(account_id)}"


def keyring_set(service: str, user: str, secret: str) -> None:
    try:
        import keyring
    except ImportError as exc:
        raise RuntimeError("pip install keyring is required for password migration") from exc
    keyring.set_password(service, user, secret)


def extract_email(raw: str | None) -> str | None:
    if not raw:
        return None
    match = re.search(r"<([^>]+)>", raw)
    if match:
        return match.group(1).strip().lower()
    found = EMAIL_RE.search(raw)
    return found.group(0).lower() if found else None


def run_opendesk_migrations() -> None:
    logging.info("running OpenDesk diesel migrations on %s", DB_PATH)
    cargo = os.environ.get("CARGO", "cargo")
    toolchain = os.environ.get("RUSTUP_TOOLCHAIN", "stable-x86_64-pc-windows-msvc")
    result = subprocess.run(
        [
            "rustup",
            "run",
            toolchain,
            cargo,
            "run",
            "--quiet",
            "-p",
            "storage",
            "--example",
            "opendesk_migrate",
            "--",
            str(DB_PATH),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        logging.error("migration stderr:\n%s", result.stderr)
        raise RuntimeError(f"migration failed: {result.stderr.strip() or result.stdout}")


def load_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def import_accounts(conn: sqlite3.Connection) -> int:
    accounts_path = EA_DATA / "email-accounts.json"
    if not accounts_path.is_file():
        logging.warning("skip accounts: %s missing", accounts_path)
        return 0

    payload = load_json(accounts_path)
    accounts = payload.get("accounts", []) if isinstance(payload, dict) else []
    now = now_ms()
    rows: list[tuple] = []
    for item in accounts:
        account_id = item["id"]
        password = item["auth"]["pass"]
        keyring_set(KEYRING_SERVICE, mail_keyring_user(account_id), password)
        rows.append(
            (
                account_id,
                item.get("label") or item.get("email") or item["auth"]["user"],
                item.get("email") or item["auth"]["user"],
                None,
                item["smtp"]["host"],
                int(item["smtp"]["port"]),
                1 if item["smtp"].get("ssl", True) else 0,
                item["auth"]["user"],
                mail_password_ref(account_id),
                "",
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


def lifecycle_for_stage(stage: str | None) -> str:
    return LIFECYCLE_BY_STAGE.get(stage or "", "new")


def import_customers(conn: sqlite3.Connection, ea_conn: sqlite3.Connection) -> int:
    now = now_ms()
    rows: list[tuple] = []
    for row in ea_conn.execute(
        """
        SELECT email, name, stage, budget, notes, summary_note, extra,
               first_contact, last_contact
        FROM contacts
        """
    ):
        email = (row[0] or "").strip().lower()
        if not email:
            continue
        name, stage, budget, notes, summary_note, extra, first_contact, last_contact = row[1:]
        note_parts = [part for part in (notes, summary_note) if part]
        combined_notes = "\n\n".join(note_parts) if note_parts else None
        customer_id = stable_id("cust_", email)
        source_ref = f"ea:contact:{email}"
        created_at = first_contact or now
        updated_at = last_contact or created_at
        extra_json = extra if extra and extra != "{}" else None
        rows.append(
            (
                customer_id,
                name or None,
                email,
                None,
                SOURCE_CHANNEL,
                None,
                lifecycle_for_stage(stage),
                stage or "no_stage",
                float(budget) if budget else None,
                None,
                None,
                None,
                "none",
                None,
                None,
                None,
                None,
                combined_notes,
                created_at,
                updated_at,
                extra_json,
                source_ref,
            )
        )

    conn.executemany(
        """
        INSERT INTO customer (
            id, display_name, email, whatsapp_phone, source_channel, source_meta,
            lifecycle_status, outreach_stage, quoted_price, quoted_currency, quoted_at,
            pricing_tier, cooperation_status, package_name, monthly_fee, contract_start,
            contract_end, notes, created_at, updated_at, extra_json, source_ref
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(email) DO UPDATE SET
            display_name = excluded.display_name,
            lifecycle_status = excluded.lifecycle_status,
            outreach_stage = excluded.outreach_stage,
            quoted_price = excluded.quoted_price,
            notes = excluded.notes,
            updated_at = excluded.updated_at,
            extra_json = excluded.extra_json,
            source_ref = excluded.source_ref
        """,
        rows,
    )
    return len(rows)


def build_customer_lookup(conn: sqlite3.Connection) -> dict[str, str]:
    lookup: dict[str, str] = {}
    for customer_id, email in conn.execute("SELECT id, email FROM customer"):
        lookup[email.strip().lower()] = customer_id
    return lookup


def import_email_cache(
    conn: sqlite3.Connection,
    ea_conn: sqlite3.Connection,
    customer_lookup: dict[str, str],
) -> int:
    now = now_ms()
    rows: list[tuple] = []
    for row in ea_conn.execute(
        """
        SELECT account_id, folder, uid, message_id, from_addr, subject, date, body
        FROM email_cache
        """
    ):
        account_id, folder, uid, message_id, from_addr, subject, date, body = row
        source_ref = f"ea:cache:{account_id}:{folder}:{uid}"
        msg_id = stable_id("msg_", source_ref)
        from_email = extract_email(from_addr)
        customer_id = customer_lookup.get(from_email) if from_email else None
        rfc_id = message_id if message_id else None
        received_at = date or now
        rows.append(
            (
                msg_id,
                customer_id,
                None,
                account_id,
                "received",
                "inbound",
                subject or "",
                body or "",
                None,
                None,
                None,
                received_at,
                uid,
                folder,
                rfc_id,
                None,
                None,
                0,
                None,
                now,
                now,
                None,
                from_addr or from_email,
                source_ref,
            )
        )

    conn.executemany(
        """
        INSERT INTO mail_message (
            id, customer_id, template_id, account_id, status, direction, subject,
            body_text, body_html, error_message, sent_at, received_at, imap_uid,
            imap_folder, rfc_message_id, in_reply_to, references_header, is_favorite,
            open_tracking_id, created_at, updated_at, to_address, from_address, source_ref
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            customer_id = excluded.customer_id,
            subject = excluded.subject,
            body_text = excluded.body_text,
            received_at = excluded.received_at,
            from_address = excluded.from_address,
            source_ref = excluded.source_ref,
            updated_at = excluded.updated_at
        """,
        rows,
    )
    return len(rows)


def import_sync_state(conn: sqlite3.Connection, ea_conn: sqlite3.Connection) -> int:
    rows = list(
        ea_conn.execute(
            """
            SELECT account_id, folder, uidvalidity, highest_modseq, last_uid,
                   last_sync_at, full_synced
            FROM sync_state
            """
        )
    )
    conn.executemany(
        """
        INSERT INTO mail_imap_sync_state (
            account_id, folder, uidvalidity, highest_modseq, last_uid,
            last_sync_at, full_synced
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(account_id, folder) DO UPDATE SET
            uidvalidity = excluded.uidvalidity,
            highest_modseq = excluded.highest_modseq,
            last_uid = excluded.last_uid,
            last_sync_at = excluded.last_sync_at,
            full_synced = excluded.full_synced
        """,
        rows,
    )
    return len(rows)


def build_script_title(item: dict) -> str:
    return item.get("description") or item.get("trigger") or item.get("id") or "Untitled"


def import_scripts(conn: sqlite3.Connection) -> int:
    scripts_path = EA_DATA / "scripts.json"
    if not scripts_path.is_file():
        return 0
    payload = load_json(scripts_path)
    scripts = payload.get("scripts", []) if isinstance(payload, dict) else []
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


def import_pending(conn: sqlite3.Connection) -> int:
    pending_path = EA_DATA / "pending-emails.json"
    if not pending_path.is_file():
        return 0
    payload = load_json(pending_path)
    tasks = payload.get("tasks", []) if isinstance(payload, dict) else []
    now = now_ms()
    rows = []
    for item in tasks:
        task_id = item.get("id") or stable_id("pt_", json.dumps(item, sort_keys=True))
        source_ref = f"ea:pending:{task_id}"
        settings = item.get("settings") or {}
        account_id = settings.get("accountId") or item.get("accountId")
        recipients = item.get("recipients") or []
        created_at = item.get("createdAt") or now
        updated_at = item.get("finishTime") or item.get("nextSendAt") or created_at
        rows.append(
            (
                stable_id("pend_", source_ref),
                account_id,
                json.dumps(recipients, ensure_ascii=False),
                item.get("subject") or "",
                item.get("bodyText") or "",
                item.get("bodyHtml"),
                item.get("status") or "pending",
                item.get("scheduleAt") or item.get("nextSendAt") or None,
                source_ref,
                created_at,
                updated_at,
            )
        )
    conn.executemany(
        """
        INSERT INTO mail_pending_send (
            id, account_id, recipients_json, subject, body_text, body_html,
            status, scheduled_at, source_ref, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            account_id = excluded.account_id,
            recipients_json = excluded.recipients_json,
            subject = excluded.subject,
            body_text = excluded.body_text,
            body_html = excluded.body_html,
            status = excluded.status,
            scheduled_at = excluded.scheduled_at,
            source_ref = excluded.source_ref,
            updated_at = excluded.updated_at
        """,
        rows,
    )
    return len(rows)


def import_legacy_json(conn: sqlite3.Connection) -> int:
    now = now_ms()
    count = 0
    for filename, kind in LEGACY_JSON_FILES.items():
        path = EA_DATA / filename
        if not path.is_file():
            continue
        payload = load_json(path)
        doc_id = stable_id("leg_", kind)
        conn.execute(
            """
            INSERT INTO legacy_json_doc (id, kind, payload_json, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(kind) DO UPDATE SET
                payload_json = excluded.payload_json,
                updated_at = excluded.updated_at
            """,
            (doc_id, kind, json.dumps(payload, ensure_ascii=False), now),
        )
        count += 1

    for path in sorted(EA_DATA.glob("workflow-stages-tpl_*.json")):
        kind = f"email-agent:workflow-stages-tpl:{path.stem}"
        payload = load_json(path)
        doc_id = stable_id("leg_", kind)
        conn.execute(
            """
            INSERT INTO legacy_json_doc (id, kind, payload_json, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(kind) DO UPDATE SET
                payload_json = excluded.payload_json,
                updated_at = excluded.updated_at
            """,
            (doc_id, kind, json.dumps(payload, ensure_ascii=False), now),
        )
        count += 1
    return count


def import_llm_settings(conn: sqlite3.Connection) -> int:
    config_path = EA_DATA / "ai-config.json"
    if not config_path.is_file():
        return 0
    config = load_json(config_path)
    if not isinstance(config, dict):
        return 0
    api_key = config.get("apiKey") or ""
    if api_key:
        keyring_set(KEYRING_SERVICE, LLM_KEYRING_USER, api_key)
    now = now_ms()
    conn.execute(
        """
        INSERT INTO llm_setting (
            id, provider, base_url, model_id, api_key_ref, has_api_key, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            provider = excluded.provider,
            base_url = excluded.base_url,
            model_id = excluded.model_id,
            api_key_ref = excluded.api_key_ref,
            has_api_key = excluded.has_api_key,
            updated_at = excluded.updated_at
        """,
        (
            LLM_SETTINGS_ROW_ID,
            config.get("provider") or "openai",
            config.get("baseUrl"),
            config.get("model") or "gpt-4o-mini",
            f"keyring:{KEYRING_SERVICE}/{LLM_KEYRING_USER}",
            1 if api_key else 0,
            now,
        ),
    )
    return 1


def print_counts(conn: sqlite3.Connection, ea_conn: sqlite3.Connection) -> None:
    source_counts = {
        "contacts": ea_conn.execute("SELECT COUNT(*) FROM contacts").fetchone()[0],
        "email_cache": ea_conn.execute("SELECT COUNT(*) FROM email_cache").fetchone()[0],
        "sync_state": ea_conn.execute("SELECT COUNT(*) FROM sync_state").fetchone()[0],
    }
    target_counts = {
        "mail_account": conn.execute("SELECT COUNT(*) FROM mail_account").fetchone()[0],
        "customer": conn.execute("SELECT COUNT(*) FROM customer").fetchone()[0],
        "mail_message": conn.execute("SELECT COUNT(*) FROM mail_message").fetchone()[0],
        "mail_imap_sync_state": conn.execute(
            "SELECT COUNT(*) FROM mail_imap_sync_state"
        ).fetchone()[0],
        "script_snippet": conn.execute("SELECT COUNT(*) FROM script_snippet").fetchone()[0],
        "mail_pending_send": conn.execute("SELECT COUNT(*) FROM mail_pending_send").fetchone()[0],
        "legacy_json_doc": conn.execute("SELECT COUNT(*) FROM legacy_json_doc").fetchone()[0],
    }
    inline_passwords = conn.execute(
        "SELECT COUNT(*) FROM mail_account WHERE password_value != ''"
    ).fetchone()[0]

    logging.info("source (email-agent): %s", source_counts)
    logging.info("target (OpenDesk): %s", target_counts)
    logging.info("mail_account with inline password_value: %s (expect 0)", inline_passwords)


def main() -> int:
    setup_logging()
    if not EA_DATA.is_dir():
        logging.error("email-agent data dir not found: %s", EA_DATA)
        return 1

    sync_cache = EA_DATA / "sync-cache.db"
    if not sync_cache.is_file():
        logging.error("sync-cache.db not found: %s", sync_cache)
        return 1

    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    run_opendesk_migrations()

    with sqlite3.connect(DB_PATH) as conn, sqlite3.connect(sync_cache) as ea_conn:
        conn.execute("PRAGMA foreign_keys = ON;")
        account_count = import_accounts(conn)
        customer_count = import_customers(conn, ea_conn)
        customer_lookup = build_customer_lookup(conn)
        message_count = import_email_cache(conn, ea_conn, customer_lookup)
        sync_count = import_sync_state(conn, ea_conn)
        script_count = import_scripts(conn)
        pending_count = import_pending(conn)
        legacy_count = import_legacy_json(conn)
        llm_count = import_llm_settings(conn)
        conn.commit()
        print_counts(conn, ea_conn)

    logging.info(
        "imported accounts=%s customers=%s messages=%s sync_state=%s "
        "scripts=%s pending=%s legacy=%s llm=%s -> %s",
        account_count,
        customer_count,
        message_count,
        sync_count,
        script_count,
        pending_count,
        legacy_count,
        llm_count,
        DB_PATH,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
