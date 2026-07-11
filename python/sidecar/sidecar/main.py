"""OpenDesk AI sidecar entrypoint."""

from __future__ import annotations

import argparse

from sidecar.logging_config import configure_logging
from sidecar.server import serve


def main() -> None:
    configure_logging()
    parser = argparse.ArgumentParser(description="OpenDesk Python sidecar")
    parser.add_argument("--port", type=int, default=8787)
    args = parser.parse_args()
    serve(args.port)


if __name__ == "__main__":
    main()
