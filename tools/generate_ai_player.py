#!/usr/bin/env python3
"""Rebuild only the AI player sprite."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from ai_art_common import prepare_raw, raw_path
from character_art import build_original_player_sheet
from generate_sprites import save


def main() -> None:
    if raw_path("player").exists():
        prepare_raw("player")
    save("player.png", build_original_player_sheet())


if __name__ == "__main__":
    main()