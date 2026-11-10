from __future__ import annotations

from pathlib import Path

from PIL import PngImagePlugin

from art_common import add_outline, rejecting_canvas, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets" / "ui" / "tools"
ICON_SIZE = 16
HOE_TIERS = (1, 2, 3, 4, 5)
BLADE_SPANS = {
    1: (7, 11, 2, 4),
    2: (7, 12, 2, 4),
    3: (6, 12, 2, 4),
    4: (6, 13, 2, 4),
    5: (5, 13, 2, 5),
}
HANDLE_PIXELS = (
    (8, 5), (7, 5),
    (7, 6), (6, 6),
    (6, 7), (5, 7),
    (5, 8), (4, 8),
    (4, 9), (3, 9),
    (3, 10), (2, 10),
    (2, 11), (3, 11),
    (2, 12), (3, 12), (4, 12),
    (3, 13), (4, 13),
)
COLLAR_PIXELS = ((7, 5), (8, 5))

OUTLINE = require_opaque_color((34, 34, 34, 255))
WOOD = require_opaque_color((119, 92, 85, 255))
RUST = require_opaque_color((138, 96, 62, 255))
RUST_EDGE = require_opaque_color((98, 68, 44, 255))
COPPER = require_opaque_color((196, 122, 64, 255))
COPPER_EDGE = require_opaque_color((148, 86, 42, 255))
COPPER_HIGHLIGHT = require_opaque_color((220, 156, 96, 255))
IRON = require_opaque_color((132, 140, 148, 255))
IRON_EDGE = require_opaque_color((72, 78, 86, 255))
IRON_HIGHLIGHT = require_opaque_color((180, 188, 196, 255))
BRASS = require_opaque_color((204, 164, 72, 255))
BRASS_EDGE = require_opaque_color((148, 112, 36, 255))
BRASS_HIGHLIGHT = require_opaque_color((236, 208, 120, 255))
AETHERSTEEL = require_opaque_color((148, 176, 212, 255))
AETHERSTEEL_EDGE = require_opaque_color((72, 108, 168, 255))
AETHERSTEEL_GLOW = require_opaque_color((210, 236, 255, 255))
METAL_COLORS = {
    1: (RUST, RUST_EDGE, RUST),
    2: (COPPER, COPPER_EDGE, COPPER_HIGHLIGHT),
    3: (IRON, IRON_EDGE, IRON_HIGHLIGHT),
    4: (BRASS, BRASS_EDGE, BRASS_HIGHLIGHT),
    5: (AETHERSTEEL, AETHERSTEEL_EDGE, AETHERSTEEL_GLOW),
}


def main() -> None:
    painted_icons = hoe_icon_records()
    require_unique_hoe_icons(painted_icons)
    for painted_icon in painted_icons:
        save_hoe_icon(painted_icon)


def hoe_icon_records():
    return [hoe_icon_record(tier) for tier in HOE_TIERS]


def hoe_icon_record(tier):
    checked_tier = require_hoe_tier(tier)
    return (checked_tier, draw_hoe_icon(checked_tier))


def require_hoe_tier(tier):
    if (
        isinstance(tier, bool)
        or not isinstance(tier, int)
        or tier not in HOE_TIERS
        or tier not in BLADE_SPANS
        or tier not in METAL_COLORS
    ):
        raise ValueError("hoe tier must be 1, 2, 3, 4, or 5")
    return tier


def draw_hoe_icon(tier):
    checked_tier = require_hoe_tier(tier)
    canvas = rejecting_canvas(ICON_SIZE, ICON_SIZE)
    paint_handle(canvas)
    paint_blade(canvas, checked_tier)
    add_outline(canvas, OUTLINE)
    return canvas.image()


def paint_handle(canvas) -> None:
    for column, row in HANDLE_PIXELS:
        canvas.put(column, row, WOOD)


def paint_blade(canvas, tier) -> None:
    checked_tier = require_hoe_tier(tier)
    for column, row in blade_pixels(checked_tier):
        canvas.put(column, row, blade_color(checked_tier, column, row))


def blade_pixels(tier):
    checked_tier = require_hoe_tier(tier)
    left, right, top, bottom = blade_span(checked_tier)
    pixels = [
        (column, row)
        for row in range(top, bottom + 1)
        for column in range(left, right + 1)
    ]
    if checked_tier != HOE_TIERS[-1]:
        pixels.extend(COLLAR_PIXELS)
    return pixels


def blade_span(tier):
    checked_tier = require_hoe_tier(tier)
    return BLADE_SPANS[checked_tier]


def blade_color(tier, column, row):
    checked_tier = require_hoe_tier(tier)
    if (column, row) not in blade_pixels(checked_tier):
        raise ValueError("pixel is not on the hoe blade")
    body, edge, highlight = metal_colors(checked_tier)
    blade_top = blade_span(checked_tier)[2]
    blade_bottom = blade_span(checked_tier)[3]
    if row == blade_top:
        return highlight
    if row == blade_bottom:
        return edge
    return body


def metal_colors(tier):
    checked_tier = require_hoe_tier(tier)
    return METAL_COLORS[checked_tier]


def require_unique_hoe_icons(painted_icons) -> None:
    payloads = []
    for painted_icon in painted_icons:
        tier, image = painted_icon
        require_hoe_tier(tier)
        payloads.append(image.tobytes())
    if len(set(payloads)) != len(payloads):
        raise ValueError("hoe icons are not unique")


def save_hoe_icon(painted_icon) -> None:
    tier, image = painted_icon
    checked_tier = require_hoe_tier(tier)
    require_hoe_icon_image(image)
    FOLDER.mkdir(parents=True, exist_ok=True)
    image.save(
        FOLDER / hoe_icon_file_name(checked_tier),
        format="PNG",
        pnginfo=PngImagePlugin.PngInfo(),
        compress_level=9,
    )


def require_hoe_icon_image(image) -> None:
    if image.size != (ICON_SIZE, ICON_SIZE) or image.mode != "RGBA":
        raise ValueError("hoe icon must be a 16 by 16 RGBA image")


def hoe_icon_file_name(tier) -> str:
    checked_tier = require_hoe_tier(tier)
    return f"hoe_t{checked_tier}.png"


if __name__ == "__main__":
    main()
