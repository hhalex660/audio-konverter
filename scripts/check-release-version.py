#!/usr/bin/env python3
"""Reject a release tag that does not match the Cargo package version."""

import sys
import tomllib
from pathlib import Path


def main() -> int:
    try:
        manifest, tag = sys.argv[1:]
        version = tomllib.loads(Path(manifest).read_text(encoding="utf-8"))["package"]["version"]
        if not isinstance(version, str) or tag != f"v{version}":
            raise ValueError(f"tag {tag!r} does not match Cargo version {version!r}")
    except (ValueError, KeyError, OSError, tomllib.TOMLDecodeError) as error:
        print(f"Release version check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
