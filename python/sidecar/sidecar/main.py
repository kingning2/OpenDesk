"""OpenDesk AI sidecar entrypoint."""

from __future__ import annotations

import argparse
import logging

from sidecar.logging_config import configure_logging
from sidecar.server import serve

logger = logging.getLogger("opendesk.sidecar")


def main() -> None:
    configure_logging()
    parser = argparse.ArgumentParser(description="OpenDesk Python sidecar")
    parser.add_argument("--port", type=int, default=8787)
    args = parser.parse_args()
    # 启动唯一日志：侧车就绪。
    logger.info(
        "侧车已启动",
        extra={"event": "sidecar.starting", "feature": "runtime", "port": args.port},
    )
    serve(args.port)


if __name__ == "__main__":
    main()
