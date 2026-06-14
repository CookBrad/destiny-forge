#!/usr/bin/env python3
"""Rebuild every AI-processed sprite asset."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from ai_art_common import prepare_raw, raw_path
from asset_art import (
    build_dungeon_background,
    build_dungeon_sheet,
    build_forge,
    build_hub_background,
    build_hub_tiles,
)
from character_art import build_original_player_sheet
from generate_sprites import save
from mine_art import build_mine_entrance

ASSETS = [
    ("player.png", build_original_player_sheet, "player"),
    ("mine_entrance.png", build_mine_entrance, "mine_entrance"),
    ("hub_tiles.png", build_hub_tiles, "hub_tiles"),
    ("dungeon_sheet.png", build_dungeon_sheet, "dungeon_sheet"),
    ("forge.png", build_forge, "forge"),
    ("hub_background.png", build_hub_background, "hub_background"),
    ("dungeon_background.png", build_dungeon_background, "dungeon_background"),
]


OPAQUE_SOURCES = {"hub_background", "dungeon_background"}


def main() -> None:
    for _, _, source_name in ASSETS:
        if raw_path(source_name).exists():
            prepare_raw(source_name, opaque=source_name in OPAQUE_SOURCES)
            print(f"prepared ai_{source_name}_clean.png")

    for filename, builder, _ in ASSETS:
        save(filename, builder())


if __name__ == "__main__":
    main()