"""AI-generated hub, dungeon, forge, and background assets."""

from __future__ import annotations

from PIL import Image

from ai_art_common import fit_to_canvas, load_clean, resize_exact, slice_tile_sheet

HUB_TILES_W = 128
HUB_TILES_H = 64
HUB_TILE_COLS = 8

DUNGEON_SHEET_W = 192
DUNGEON_SHEET_H = 64
DUNGEON_TILE_COLS = 12

FORGE_W = 64
FORGE_H = 48

BACKGROUND_W = 320
BACKGROUND_H = 180


def build_hub_tiles() -> Image.Image:
    img = load_clean("hub_tiles")
    return slice_tile_sheet(
        img,
        HUB_TILE_COLS,
        sheet_width=HUB_TILES_W,
        sheet_height=HUB_TILES_H,
    )


def _fit_dungeon_tile(
    crop: Image.Image,
    *,
    tile: bool = False,
    ground_y: int = 15,
) -> Image.Image:
    if tile:
        return resize_exact(crop, 16, 16)
    return fit_to_canvas(crop, 16, 16, ground_y=ground_y)


def build_dungeon_sheet() -> Image.Image:
    """Composite dungeon sprites from the AI sheet's separate bands.

    The clean source is not a single horizontal icon row: floors and ceiling
    strips sit at the top, enemies and props are laid out in lower bands.
    """
    img = load_clean("dungeon_sheet")
    w, _ = img.size
    third = w // 3

    slots: list[tuple[tuple[int, int, int, int], bool]] = [
        # CaveFloorA, CaveFloorB, StonePlatform
        ((20, 40, 20 + third, 120), True),
        ((third + 20, 40, 2 * third + 20, 120), True),
        ((20, 260, 20 + third, 360), True),
        # Slime, Bat, Corpse
        ((17, 445, 225, 636), False),
        ((310, 500, 440, 590), False),
        ((501, 446, 763, 655), False),
        # LadderExit, Slash, Torch
        ((44, 685, 168, 936), False),
        ((250, 682, 420, 820), False),
        ((480, 700, 580, 900), False),
    ]

    sheet = Image.new("RGBA", (DUNGEON_SHEET_W, DUNGEON_SHEET_H), (0, 0, 0, 0))
    for index, (box, is_tile) in enumerate(slots):
        tile = _fit_dungeon_tile(img.crop(box), tile=is_tile)
        sheet.paste(tile, (index * 16, 0))

    return sheet


def build_forge() -> Image.Image:
    img = load_clean("forge")
    return fit_to_canvas(
        img,
        FORGE_W,
        FORGE_H,
        ground_y=FORGE_H - 1,
        pad=(12, 12, 12, 20),
    )


def build_hub_background() -> Image.Image:
    img = load_clean("hub_background", opaque=True)
    return resize_exact(img, BACKGROUND_W, BACKGROUND_H)


def build_dungeon_background() -> Image.Image:
    img = load_clean("dungeon_background", opaque=True)
    return resize_exact(img, BACKGROUND_W, BACKGROUND_H)