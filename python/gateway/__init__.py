"""gateway — Sidecar 浏览器例外能力（单层模块，无深层子包）。

| 模块 | 职责 |
|------|------|
| `handle_*` | 对 Rust 的 HTTP 薄适配 |
| `cookie_renew` / `qr` / `slider` | 续期、扫码、滑块 |
| `camoufox` / `playwright_common` / `platform_config` | 浏览器基建 |

作者：Xiaoman
创建时间：2026-08-21
"""

from gateway.handle_cookie_renew import handle_cookie_renew
from gateway.handle_qr import handle_qr_cancel, handle_qr_check, handle_qr_start

__all__ = [
    "handle_cookie_renew",
    "handle_qr_cancel",
    "handle_qr_check",
    "handle_qr_start",
]
