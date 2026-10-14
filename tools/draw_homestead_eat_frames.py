#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image

from art_common import composite_images, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
SPRITES = ROOT / "assets" / "player" / "non-combat"
BASE = SPRITES / "dwarf_m_idle_anim_f0.png"

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
SKIN_SHADE = require_opaque_color((195, 141, 112, 255))
MEAT = require_opaque_color((156, 84, 40, 255))
MEAT_LIGHT = require_opaque_color((204, 128, 64, 255))
BONE = require_opaque_color((238, 228, 204, 255))

PALETTE = {
    "a": OUTLINE,
    "s": SKIN,
    "d": SKIN_SHADE,
    "m": MEAT,
    "h": MEAT_LIGHT,
    "b": BONE,
}

DRUMSTICK = [
    ".aaa...",
    "ahmma..",
    "ammmaba",
    "ammmabb",
    ".aaa.a.",
]

BITTEN_DRUMSTICK = [
    "..aa...",
    ".ahma..",
    "a.mmaba",
    ".ammabb",
    "..aa.a.",
]

HAND = [
    "aa",
    "sa",
]

OPEN_MOUTH = [
    "aa",
]

CHEEK_BULGE = [
    "s",
    "d",
]


def stamp(image: Image.Image, pattern: list[str], left: int, top: int) -> None:
    for row, line in enumerate(pattern):
        for col, key in enumerate(line):
            if key == ".":
                continue
            x = left + col
            y = top + row
            if 0 <= x < image.width and 0 <= y < image.height:
                image.putpixel((x, y), PALETTE[key])


def dip_head(image: Image.Image, rows: int) -> Image.Image:
    neck = 20
    top = image.crop((0, 0, image.width, neck))
    bottom = image.crop((0, neck, image.width, image.height))
    return composite_images(image.size, [(bottom, (0, neck)), (top, (0, rows))])


def close_eyes(image: Image.Image, head_offset: int) -> None:
    for x in (8, 9):
        image.putpixel((x, 18 + head_offset), SKIN)
        image.putpixel((x, 19 + head_offset), OUTLINE)


def reach(base: Image.Image) -> Image.Image:
    frame = base.copy()
    stamp(frame, DRUMSTICK, 9, 21)
    stamp(frame, HAND, 13, 24)
    return frame


def lift(base: Image.Image) -> Image.Image:
    frame = base.copy()
    stamp(frame, OPEN_MOUTH, 9, 21)
    stamp(frame, DRUMSTICK, 11, 19)
    stamp(frame, HAND, 14, 22)
    return frame


def bite(base: Image.Image) -> Image.Image:
    frame = dip_head(base, 1)
    stamp(frame, DRUMSTICK, 10, 20)
    stamp(frame, HAND, 14, 23)
    return frame


def chew(base: Image.Image) -> Image.Image:
    frame = dip_head(base, 1)
    close_eyes(frame, 1)
    stamp(frame, CHEEK_BULGE, 11, 21)
    stamp(frame, BITTEN_DRUMSTICK, 11, 22)
    stamp(frame, HAND, 15, 25)
    return frame


def main() -> None:
    base = Image.open(BASE).convert("RGBA")
    frames = [pose(base) for pose in (reach, lift, bite, chew)]
    for index, frame in enumerate(frames):
        frame.save(SPRITES / f"dwarf_m_eat_anim_f{index}.png", optimize=True)


if __name__ == "__main__":
    main()
