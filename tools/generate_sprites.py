#!/usr/bin/env python3
"""Regenerate AI-processed pixel art for Destiny Forge."""

from __future__ import annotations

from pathlib import Path

from PIL import Image

from asset_art import (
    build_dungeon_background,
    build_dungeon_sheet,
    build_forge,
    build_hub_background,
    build_hub_tiles,
)
from character_art import build_original_player_sheet
from mine_art import build_mine_entrance

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "sprites"


def save(name: str, image: Image.Image) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / name
    image.save(path)
    print(f"wrote {path} ({image.size[0]}x{image.size[1]})")


def main() -> None:
    save("hub_tiles.png", build_hub_tiles())
    save("dungeon_sheet.png", build_dungeon_sheet())
    save("forge.png", build_forge())
    save("mine_entrance.png", build_mine_entrance())
    save("hub_background.png", build_hub_background())
    save("dungeon_background.png", build_dungeon_background())
    save("player.png", build_original_player_sheet())


if __name__ == "__main__":
    main()