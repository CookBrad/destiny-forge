#!/usr/bin/env python3
"""Rebuild only the AI mine entrance sprite."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from ai_art_common import prepare_raw, raw_path
from generate_sprites import save
from mine_art import build_mine_entrance


def main() -> None:
    if raw_path("mine_entrance").exists():
        prepare_raw("mine_entrance")
    save("mine_entrance.png", build_mine_entrance())


if __name__ == "__main__":
    main()