"""OpenDesk AI sidecar entrypoint."""

from __future__ import annotations

import argparse
import logging

from sidecar.api import serve
from sidecar.logging_config import configure_logging

logger = logging.getLogger("opendesk.sidecar")


def main() -> None:
    configure_logging()
    parser = argparse.ArgumentParser(description="OpenDesk Python sidecar")
    parser.add_argument("--port", type=int, default=8787)
    args = parser.parse_args()
    logger.info(
        "sidecar starting",
        extra={"event": "sidecar.starting", "feature": "runtime", "port": args.port},
    )
    serve(args.port)


if __name__ == "__main__":
    main()
