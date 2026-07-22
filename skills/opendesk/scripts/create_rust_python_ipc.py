#!/usr/bin/env python3
"""Scaffold Rust ↔ Python sidecar IPC (contract + runtime client + gateway handler)."""

from __future__ import annotations

import argparse
import json
import logging
from typing import Literal

from _common import (
    CONTRACTS,
    CRATES,
    PYTHON_PACKAGES,
    PYTHON_SIDECAR,
    ROOT,
    ensure_pub_mod,
    ensure_python_import,
    setup_logging,
    validate_name,
    write_text,
)

Method = Literal["GET", "POST"]


def _pascal(snake: str) -> str:
    return "".join(part.capitalize() for part in snake.split("_"))


def _route_path(feature: str, action: str) -> str:
    return f"/v1/{feature}/{action}"


def _op_id(feature: str, action: str) -> str:
    return f"{feature}_{action}"


def _schema(feature: str, action: str, direction: str) -> dict:
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": f"opendesk://{feature}/sidecar/{action}.{direction}/v1",
        "title": f"{feature}/sidecar/{action}.{direction}",
        "type": "object",
        "required": [] if direction == "request" else ["ok"],
        "properties": (
            {"trace_id": {"type": "string"}}
            if direction == "request"
            else {"ok": {"type": "boolean"}, "trace_id": {"type": "string"}}
        ),
        "additionalProperties": False,
    }


def _openapi_path_fragment(
    feature: str,
    action: str,
    method: Method,
    route: str,
) -> str:
    op = _op_id(feature, action)
    body = ""
    if method == "POST":
        body = f"""
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '../../schema/v1/{feature}/sidecar/{action}.request.schema.json'
"""
    return f"""# Auto-generated path fragment for {route}
# Rust runtime client only — React must not call this URL.

{route}:
  {method.lower()}:
    operationId: {op}
    summary: "{feature} {action} (Rust → Python)"
    tags: [{feature}]
{body}    responses:
      "200":
        description: Success
        content:
          application/json:
            schema:
              $ref: '../../schema/v1/{feature}/sidecar/{action}.response.schema.json'
      "500":
        description: Sidecar error
        content:
          application/json:
            schema:
              $ref: '../sidecar.v1.yaml#/components/schemas/SidecarError'
"""


def _rust_client_module() -> str:
    return """//! Sidecar HTTP client (Rust → Python). Skeleton only.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SidecarClientError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("sidecar: {0}")]
    Sidecar(String),
}

/// HTTP client for the local Python sidecar. Port is assigned by runtime lifecycle.
pub struct SidecarClient {
    base_url: String,
}

impl SidecarClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // TODO: inject reqwest::Client when runtime wiring is approved.
}
"""


def _rust_route_file(feature: str, action: str, method: Method, route: str) -> str:
    route_key = _op_id(feature, action)
    req = f"{_pascal(route_key)}Request"
    res = f"{_pascal(route_key)}Response"
    return f"""//! Sidecar route binding: {route} ({method})

use super::client::{{SidecarClient, SidecarClientError}};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct {req} {{
    pub trace_id: String,
}}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct {res} {{
    pub ok: bool,
    pub trace_id: String,
}}

pub async fn {route_key}(
    client: &SidecarClient,
    request: {req},
) -> Result<{res}, SidecarClientError> {{
    let _ = (client, request);
    // TODO: {method} {route} with contract DTO; parse response JSON.
    Err(SidecarClientError::Transport("not implemented (skeleton)".into()))
}}
"""


def _python_handler(feature: str, action: str, method: Method, route: str) -> str:
    fn = f"handle_{feature}_{action}"
    return f'''"""Sidecar handler: {route} ({method}) — Python ← Rust only."""

from __future__ import annotations

import logging
from typing import Any

logger = logging.getLogger("opendesk.sidecar.{feature}")


def {fn}(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/{feature}/sidecar/{action}.*.schema.json"""
    logger.info("{fn}", extra={{"trace_id": trace_id, "feature": "{feature}"}})
    _ = payload
    return {{"ok": True, "trace_id": trace_id}}
'''


def _streaming_rust_example() -> str:
    return """//! Streaming: Python → Rust → Tauri Events (skeleton pattern).

/// Rust reads stream chunks from sidecar HTTP body,
/// then emits Tauri events — Python must not emit to frontend directly.
pub async fn forward_stream_to_tauri(_task_id: &str) -> Result<(), String> {
    Ok(())
}
"""


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog="Example: python create_rust_python_ipc.py --feature agent --action ping",
    )
    parser.add_argument("--feature", required=True)
    parser.add_argument("--action", required=True, help="e.g. ping, run_task")
    parser.add_argument("--method", choices=("GET", "POST"), default="POST")
    parser.add_argument("--streaming", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        feature = validate_name(args.feature)
        action = validate_name(args.action)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    method: Method = args.method
    route = _route_path(feature, action)
    route_key = _op_id(feature, action)

    schema_dir = CONTRACTS / "schema" / "v1" / feature / "sidecar"
    write_text(
        schema_dir / f"{action}.request.schema.json",
        json.dumps(_schema(feature, action, "request"), indent=2) + "\n",
        dry_run=args.dry_run,
    )
    write_text(
        schema_dir / f"{action}.response.schema.json",
        json.dumps(_schema(feature, action, "response"), indent=2) + "\n",
        dry_run=args.dry_run,
    )
    write_text(
        CONTRACTS / "openapi" / "sidecar.paths" / f"{route_key}.yaml",
        _openapi_path_fragment(feature, action, method, route),
        dry_run=args.dry_run,
    )

    sidecar_dir = CRATES / "runtime" / "src" / "sidecar"
    client_file = sidecar_dir / "client.rs"
    if not client_file.exists():
        write_text(client_file, _rust_client_module(), dry_run=args.dry_run)

    write_text(
        sidecar_dir / "routes" / f"{route_key}.rs",
        _rust_route_file(feature, action, method, route),
        dry_run=args.dry_run,
    )
    ensure_pub_mod(sidecar_dir / "mod.rs", "client", dry_run=args.dry_run)
    ensure_pub_mod(sidecar_dir / "routes" / "mod.rs", route_key, dry_run=args.dry_run)
    ensure_pub_mod(sidecar_dir / "mod.rs", "routes", dry_run=args.dry_run)
    ensure_pub_mod(CRATES / "runtime" / "src" / "lib.rs", "sidecar", dry_run=args.dry_run)

    if args.streaming:
        write_text(sidecar_dir / "stream.rs", _streaming_rust_example(), dry_run=args.dry_run)
        ensure_pub_mod(sidecar_dir / "mod.rs", "stream", dry_run=args.dry_run)

    handler_path = PYTHON_PACKAGES / "gateway" / "src" / "gateway" / "handlers" / f"{route_key}.py"
    write_text(handler_path, _python_handler(feature, action, method, route), dry_run=args.dry_run)

    handlers_init = PYTHON_PACKAGES / "gateway" / "src" / "gateway" / "handlers" / "__init__.py"
    ensure_python_import(
        handlers_init,
        f"from .{route_key} import handle_{feature}_{action} as handle_{feature}_{action}",
        dry_run=args.dry_run,
    )

    routes_file = PYTHON_SIDECAR / "sidecar" / "api" / "routes.py"
    route_line = f'ROUTES["{route}"] = ("{method}", "handle_{feature}_{action}")'
    if routes_file.exists():
        content = routes_file.read_text(encoding="utf-8")
        if route not in content:
            write_text(
                routes_file,
                content.rstrip() + "\n" + route_line + "\n",
                dry_run=args.dry_run,
            )
    else:
        write_text(
            routes_file,
            '"""Sidecar 路由表（路径 → HTTP 方法 + Handler 名）。"""\n\n'
            "from __future__ import annotations\n\n"
            "ROUTES: dict[str, tuple[str, str]] = {}\n\n"
            f"{route_line}\n",
            dry_run=args.dry_run,
        )

    registry_file = PYTHON_SIDECAR / "sidecar" / "api" / "registry.py"
    handler_symbol = f"handle_{feature}_{action}"
    if registry_file.exists():
        registry_content = registry_file.read_text(encoding="utf-8")
        if f'"{handler_symbol}"' not in registry_content:
            # Extend existing `from gateway.handlers import a, b` line when present.
            import_prefix = "from gateway.handlers import "
            if import_prefix in registry_content and handler_symbol not in registry_content:
                for line in registry_content.splitlines():
                    if line.startswith(import_prefix):
                        names = [
                            part.strip()
                            for part in line[len(import_prefix) :].split(",")
                            if part.strip()
                        ]
                        if handler_symbol not in names:
                            names.append(handler_symbol)
                            names.sort()
                            new_import = import_prefix + ", ".join(names)
                            registry_content = registry_content.replace(line, new_import, 1)
                        break
            elif handler_symbol not in registry_content:
                registry_content = (
                    f"{import_prefix}{handler_symbol}\n" + registry_content
                    if not registry_content.startswith(import_prefix)
                    else registry_content
                )
                if import_prefix not in registry_content:
                    # Fallback: append import near top after future import.
                    registry_content = registry_content.replace(
                        "from __future__ import annotations\n",
                        f"from __future__ import annotations\n\n{import_prefix}{handler_symbol}\n",
                        1,
                    )

            if "HANDLERS: dict[str, HandlerFn] = {" in registry_content:
                registry_content = registry_content.replace(
                    "HANDLERS: dict[str, HandlerFn] = {\n",
                    "HANDLERS: dict[str, HandlerFn] = {\n"
                    f'    "{handler_symbol}": {handler_symbol},\n',
                    1,
                )
            write_text(registry_file, registry_content, dry_run=args.dry_run)
    else:
        write_text(
            registry_file,
            '"""Sidecar 业务 Handler 注册表。"""\n\n'
            "from __future__ import annotations\n\n"
            "from collections.abc import Callable\n"
            "from typing import Any\n\n"
            f"from gateway.handlers import {handler_symbol}\n\n"
            "HandlerFn = Callable[..., dict[str, Any]]\n\n"
            "HANDLERS: dict[str, HandlerFn] = {\n"
            f'    "{handler_symbol}": {handler_symbol},\n'
            "}\n",
            dry_run=args.dry_run,
        )

    example_dir = ROOT / "skills" / "opendesk" / "examples" / "rust-python" / route_key
    write_text(
        example_dir / "README.md",
        f"# Rust ↔ Python: `{route}`\n\n"
        "| 层 | 文件 |\n|----|------|\n"
        f"| Contract | `contracts/schema/v1/{feature}/sidecar/{action}.*.schema.json` |\n"
        f"| OpenAPI | `contracts/openapi/sidecar.paths/{route_key}.yaml` |\n"
        f"| Rust client | `crates/runtime/src/sidecar/routes/{route_key}.rs` |\n"
        f"| Python handler | `python/packages/gateway/.../handlers/{route_key}.py` |\n\n"
        "React **禁止**直连此路径。\n",
        dry_run=args.dry_run,
    )

    logging.info("rust-python ipc %s registered at %s", route_key, route)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
