#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image, PngImagePlugin

from art_common import Canvas, add_outline, light_contour, rejecting_canvas, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "assets" / "ui" / "skills"
SIZE = 32

OUTLINE = require_opaque_color((16, 10, 14, 255))
RIM_LIT = require_opaque_color((196, 176, 140, 255))

STEEL_LT = require_opaque_color((232, 240, 248, 255))
STEEL = require_opaque_color((168, 180, 194, 255))
STEEL_DK = require_opaque_color((74, 86, 98, 255))

CYAN = require_opaque_color((57, 214, 255, 255))
CYAN_DK = require_opaque_color((18, 104, 128, 255))

PURPLE = require_opaque_color((214, 75, 255, 255))
PURPLE_DK = require_opaque_color((122, 34, 176, 255))

BRASS_LT = require_opaque_color((244, 214, 140, 255))
BRASS = require_opaque_color((198, 158, 84, 255))
BRASS_DK = require_opaque_color((122, 88, 40, 255))

LEATHER_LT = require_opaque_color((196, 138, 82, 255))
LEATHER = require_opaque_color((122, 78, 48, 255))
LEATHER_DK = require_opaque_color((74, 44, 24, 255))
SOLE = require_opaque_color((36, 28, 26, 255))

IRON_LT = require_opaque_color((158, 152, 144, 255))
IRON = require_opaque_color((96, 92, 88, 255))
IRON_DK = require_opaque_color((52, 48, 46, 255))

ORTHO = ((0, -1), (0, 1), (-1, 0), (1, 0))

IRON_TONES = {IRON_LT, IRON, IRON_DK}
BRASS_TONES = {BRASS_LT, BRASS, BRASS_DK}


def new_canvas() -> Canvas:
    return rejecting_canvas(SIZE, SIZE)


def put(
    canvas: Canvas,
    x: int,
    y: int,
    color: tuple[int, int, int, int],
) -> None:
    canvas.put(x, y, color)


def fills_span(
    canvas: Canvas,
    y: int,
    x0: int,
    x1: int,
    color: tuple[int, int, int, int],
) -> None:
    for x in range(x0, x1 + 1):
        put(canvas, x, y, color)


def neighbor(
    canvas: Canvas, x: int, y: int, dx: int, dy: int
) -> tuple[int, int, int, int] | None:
    nx, ny = x + dx, y + dy
    if 0 <= nx < SIZE and 0 <= ny < SIZE:
        return canvas[ny][nx]
    return None


def touches_empty(canvas: Canvas, x: int, y: int) -> bool:
    return any(neighbor(canvas, x, y, dx, dy) is None for dx, dy in ORTHO)


def outline_icon(canvas: Canvas) -> None:
    add_outline(canvas, OUTLINE)


def shade_outline(canvas: Canvas) -> None:
    light_contour(canvas, OUTLINE, RIM_LIT)


def recolor_rim(
    canvas: Canvas,
    lit: tuple[int, int, int, int],
    mid: tuple[int, int, int, int],
    shadow: tuple[int, int, int, int],
) -> None:
    edge = [
        (x, y)
        for y in range(SIZE)
        for x in range(SIZE)
        if canvas[y][x] is not None and touches_empty(canvas, x, y)
    ]
    for x, y in edge:
        above_empty = neighbor(canvas, x, y, 0, -1) is None
        left_empty = neighbor(canvas, x, y, -1, 0) is None
        below_empty = neighbor(canvas, x, y, 0, 1) is None
        right_empty = neighbor(canvas, x, y, 1, 0) is None
        if above_empty or (left_empty and not right_empty and not below_empty):
            canvas[y][x] = lit
        elif below_empty or right_empty:
            canvas[y][x] = shadow
        else:
            canvas[y][x] = mid


def paint_sword(canvas: Canvas) -> None:
    put(canvas, 15, 3, STEEL_LT)
    put(canvas, 16, 3, STEEL)
    put(canvas, 14, 4, STEEL_LT)
    put(canvas, 15, 4, STEEL)
    put(canvas, 16, 4, STEEL_DK)
    put(canvas, 17, 4, STEEL)
    for y in range(5, 18):
        put(canvas, 13, y, STEEL_LT)
        put(canvas, 14, y, STEEL)
        put(canvas, 15, y, STEEL_DK)
        put(canvas, 16, y, STEEL)
        put(canvas, 17, y, CYAN if 7 <= y <= 16 else STEEL)
    put(canvas, 17, 17, STEEL_DK)

    for x in range(7, 25):
        put(canvas, x, 18, BRASS_LT if x < 16 else BRASS)
        put(canvas, x, 19, BRASS if x < 18 else BRASS_DK)
        put(canvas, x, 20, BRASS_DK)
    put(canvas, 15, 19, PURPLE)
    put(canvas, 16, 19, PURPLE_DK)
    put(canvas, 14, 19, BRASS_LT)

    for y in range(21, 27):
        band = y in (22, 24)
        put(canvas, 13, y, LEATHER_LT if not band else LEATHER_DK)
        put(canvas, 14, y, LEATHER_DK if band else LEATHER)
        put(canvas, 15, y, LEATHER_DK if band else LEATHER)
        put(canvas, 16, y, LEATHER_DK)
        put(canvas, 17, y, LEATHER_DK)

    for x in range(12, 19):
        put(canvas, x, 27, BRASS_LT if x < 16 else BRASS)
        put(canvas, x, 28, BRASS if x < 16 else BRASS_DK)
    put(canvas, 15, 28, CYAN)


def shield_spans() -> list[tuple[int, int, int]]:
    spans = [
        (6, 10, 21),
        (7, 8, 23),
        (8, 7, 24),
    ]
    spans.extend((y, 6, 25) for y in range(9, 18))
    spans.extend(
        [
            (18, 7, 24),
            (19, 8, 23),
            (20, 9, 22),
            (21, 10, 21),
            (22, 11, 20),
            (23, 12, 19),
            (24, 13, 18),
            (25, 14, 17),
            (26, 15, 16),
        ]
    )
    return spans


def paint_shield(canvas: Canvas) -> None:
    for y, x0, x1 in shield_spans():
        for x in range(x0, x1 + 1):
            shade = x + y
            if shade < 22:
                color = IRON_LT
            elif shade > 36:
                color = IRON_DK
            else:
                color = IRON
            put(canvas, x, y, color)

    recolor_rim(canvas, BRASS_LT, BRASS, BRASS_DK)

    piping: list[tuple[int, int]] = []
    for y in range(SIZE):
        for x in range(SIZE):
            tone = canvas[y][x]
            if tone not in IRON_TONES:
                continue
            if any(neighbor(canvas, x, y, dx, dy) in BRASS_TONES for dx, dy in ORTHO):
                piping.append((x, y))
    for x, y in piping:
        above = neighbor(canvas, x, y, 0, -1)
        left = neighbor(canvas, x, y, -1, 0)
        if above in BRASS_TONES or left in BRASS_TONES:
            canvas[y][x] = PURPLE
        else:
            canvas[y][x] = PURPLE_DK

    stamp_boss(canvas)
    for x, y in ((11, 11), (20, 11), (12, 16), (19, 18)):
        if canvas[y][x] in IRON_TONES:
            canvas[y][x] = IRON_DK

    for x, y in ((10, 10), (21, 10)):
        if canvas[y][x] in IRON_TONES or canvas[y][x] in {PURPLE, PURPLE_DK}:
            canvas[y][x] = BRASS_LT


def stamp_boss(canvas: Canvas) -> None:
    cx, cy = 15, 14
    for dy in range(-4, 5):
        for dx in range(-4, 5):
            x, y = cx + dx, cy + dy
            if canvas[y][x] not in IRON_TONES:
                continue
            dist2 = dx * dx + dy * dy
            if dist2 <= 2:
                canvas[y][x] = CYAN
            elif dist2 <= 6:
                canvas[y][x] = CYAN_DK
    put(canvas, cx - 1, cy - 1, STEEL_LT)
    for y in range(18, 24):
        if canvas[y][16] in IRON_TONES:
            canvas[y][16] = CYAN_DK
        if y % 2 == 0 and canvas[y][15] in IRON_TONES:
            canvas[y][15] = CYAN


def paint_boot(canvas: Canvas) -> None:
    spans = [
        (6, 17, 24, LEATHER_LT),
        (7, 16, 25, LEATHER),
        (8, 16, 25, LEATHER),
        (9, 16, 25, LEATHER),
        (10, 16, 25, LEATHER),
        (11, 16, 25, LEATHER),
        (12, 15, 26, LEATHER),
        (13, 14, 27, LEATHER),
        (14, 13, 29, LEATHER),
        (15, 12, 29, LEATHER),
        (16, 11, 28, LEATHER),
        (17, 10, 27, LEATHER_DK),
        (18, 9, 26, SOLE),
        (19, 9, 17, SOLE),
        (20, 9, 16, SOLE),
        (21, 10, 15, SOLE),
    ]
    for y, x0, x1, color in spans:
        fills_span(canvas, y, x0, x1, color)
    for y in (7, 8):
        fills_span(canvas, y, 19, 22, SOLE)
    for y in range(7, 12):
        canvas[y][16] = LEATHER_LT
        canvas[y][25] = LEATHER_DK
    for y in range(9, 12):
        for x in range(19, 23):
            canvas[y][x] = BRASS_LT if y == 9 or x == 19 else BRASS
    put(canvas, 20, 10, BRASS_DK)
    put(canvas, 21, 10, CYAN)
    for x in (16, 18, 24):
        if canvas[12][x] == LEATHER:
            canvas[12][x] = PURPLE
    for y in range(14, 17):
        canvas[y][27] = BRASS_LT
        canvas[y][28] = BRASS
        if y < 16:
            canvas[y][29] = BRASS_DK


def paint_boot_trail(canvas: Canvas) -> None:
    streaks = (
        (2, 8, CYAN_DK),
        (3, 8, CYAN),
        (4, 8, CYAN),
        (5, 8, CYAN),
        (1, 13, PURPLE_DK),
        (2, 13, PURPLE),
        (3, 13, PURPLE),
        (4, 13, PURPLE),
        (2, 18, CYAN_DK),
        (3, 18, CYAN),
        (4, 18, CYAN),
        (5, 18, CYAN),
    )
    for x, y, color in streaks:
        if canvas[y][x] is None:
            put(canvas, x, y, color)


def circle_pixels(cx: int, cy: int, radius: int) -> list[tuple[int, int]]:
    points: set[tuple[int, int]] = set()
    x, y, decision = 0, radius, 3 - 2 * radius

    def plot(px: int, py: int) -> None:
        points.add((cx + px, cy + py))
        points.add((cx - px, cy + py))
        points.add((cx + px, cy - py))
        points.add((cx - px, cy - py))
        points.add((cx + py, cy + px))
        points.add((cx - py, cy + px))
        points.add((cx + py, cy - px))
        points.add((cx - py, cy - px))

    while x <= y:
        plot(x, y)
        if decision < 0:
            decision += 4 * x + 6
        else:
            decision += 4 * (x - y) + 10
            y -= 1
        x += 1
    return sorted(points)


def paint_wand(canvas: Canvas) -> None:
    diamond = {
        6: range(16, 18),
        7: range(15, 19),
        8: range(14, 20),
        9: range(14, 20),
        10: range(15, 19),
        11: range(16, 18),
    }
    for y, xs in diamond.items():
        for x in xs:

            color = PURPLE_DK if x + y >= 28 else PURPLE
            put(canvas, x, y, color)
    put(canvas, 15, 7, STEEL_LT)

    for x in range(14, 19):
        put(canvas, x, 12, BRASS_LT if x < 17 else BRASS)
        put(canvas, x, 13, BRASS_DK)
    for y in range(14, 27):
        put(canvas, 15, y, BRASS_LT)
        put(canvas, 16, y, BRASS)
        put(canvas, 17, y, BRASS_DK)
    for x in range(14, 19):
        put(canvas, x, 27, BRASS_LT if x < 17 else BRASS)
        put(canvas, x, 28, BRASS_DK)
    put(canvas, 16, 28, CYAN_DK)


def paint_wand_orbit(canvas: Canvas) -> None:
    for x, y in circle_pixels(16, 10, 8):
        if y >= 13 or canvas[y][x] is not None:
            continue

        if x >= 20 and y <= 8:
            color = PURPLE
        elif x + y < 20:
            color = CYAN
        else:
            color = CYAN_DK
        put(canvas, x, y, color)


def finish_body(canvas: Canvas) -> None:
    outline_icon(canvas)
    shade_outline(canvas)


def to_image(canvas: Canvas) -> Image.Image:
    return canvas.image()


def draw_sword() -> Image.Image:
    canvas = new_canvas()
    paint_sword(canvas)
    finish_body(canvas)
    return to_image(canvas)


def draw_shield() -> Image.Image:
    canvas = new_canvas()
    paint_shield(canvas)
    finish_body(canvas)
    return to_image(canvas)


def draw_boot() -> Image.Image:
    canvas = new_canvas()
    paint_boot(canvas)
    finish_body(canvas)
    paint_boot_trail(canvas)
    return to_image(canvas)


def draw_wand() -> Image.Image:
    canvas = new_canvas()
    paint_wand(canvas)
    finish_body(canvas)
    paint_wand_orbit(canvas)
    return to_image(canvas)


def glyph_box(image: Image.Image) -> tuple[int, int, int, int]:
    pixels = image.load()
    min_x, min_y, max_x, max_y = SIZE, SIZE, -1, -1
    for y in range(SIZE):
        for x in range(SIZE):
            if pixels[x, y][3]:
                min_x = min(min_x, x)
                min_y = min(min_y, y)
                max_x = max(max_x, x)
                max_y = max(max_y, y)
    return min_x, min_y, max_x, max_y


def verify(image: Image.Image, name: str) -> None:
    if image.size != (SIZE, SIZE):
        raise SystemExit(f"{name} is {image.size}, expected {SIZE}×{SIZE}")
    if image.mode != "RGBA":
        raise SystemExit(f"{name} mode is {image.mode}, expected RGBA")
    opaque = 0
    for red, green, blue, alpha in image.getdata():
        if alpha not in (0, 255):
            raise SystemExit(f"{name} has partial alpha {alpha}")
        if alpha == 0 and (red, green, blue) != (0, 0, 0):
            raise SystemExit(f"{name} transparent pixel is not zero-rgb")
        if alpha == 255:
            opaque += 1
    if not 80 <= opaque <= 800:
        raise SystemExit(f"{name} opaque count {opaque} is outside 80..800")
    for corner in ((0, 0), (SIZE - 1, 0), (0, SIZE - 1), (SIZE - 1, SIZE - 1)):
        if image.getpixel(corner)[3] != 0:
            raise SystemExit(f"{name} corner {corner} is opaque")
    min_x, min_y, max_x, max_y = glyph_box(image)
    if max_x - min_x + 1 < 16 or max_y - min_y + 1 < 16:
        raise SystemExit(f"{name} glyph is too small: {(min_x, min_y, max_x, max_y)}")
    if min_x < 1 or min_y < 1 or max_x > SIZE - 2 or max_y > SIZE - 2:
        raise SystemExit(f"{name} glyph touches the canvas edge {(min_x, min_y, max_x, max_y)}")


def save_png(image: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, format="PNG", pnginfo=PngImagePlugin.PngInfo(), compress_level=9)


def ascii_preview(image: Image.Image) -> str:
    legend = {
        OUTLINE: "·",
        RIM_LIT: "+",
        STEEL_LT: "W",
        STEEL: "w",
        STEEL_DK: "x",
        CYAN: "c",
        CYAN_DK: "C",
        PURPLE: "p",
        PURPLE_DK: "P",
        BRASS_LT: "B",
        BRASS: "b",
        BRASS_DK: "n",
        LEATHER_LT: "L",
        LEATHER: "l",
        LEATHER_DK: "r",
        SOLE: "s",
        IRON_LT: "H",
        IRON: "m",
        IRON_DK: "d",
    }
    rows: list[str] = []
    for y in range(SIZE):
        chars = []
        for x in range(SIZE):
            pixel = image.getpixel((x, y))
            if pixel[3] == 0:
                chars.append(" ")
            else:
                chars.append(legend.get(pixel, "?"))
        rows.append("".join(chars).rstrip())
    return "\n".join(rows)


def main() -> None:
    check = "--check" in sys.argv
    icons = {
        "short_wep.png": draw_sword(),
        "shield.png": draw_shield(),
        "boot.png": draw_boot(),
        "wand.png": draw_wand(),
    }
    payloads = [list(image.getdata()) for image in icons.values()]
    if len({tuple(payload) for payload in payloads}) != len(icons):
        raise SystemExit("skill icons are not unique")

    for name, image in icons.items():
        verify(image, name)
        dest = OUT_DIR / name
        if check:
            if not dest.is_file():
                raise SystemExit(f"missing {dest}")
            existing = Image.open(dest).convert("RGBA")
            verify(existing, name)
            if list(existing.getdata()) != list(image.getdata()):
                raise SystemExit(f"{name} does not match tools/draw_skill_icons.py")
        else:
            save_png(image, dest)
        min_x, min_y, max_x, max_y = glyph_box(image)
        print(
            f"{name} glyph {max_x - min_x + 1}x{max_y - min_y + 1} "
            f"at ({min_x},{min_y})-({max_x},{max_y})"
        )
        print(ascii_preview(image))
        print()


if __name__ == "__main__":
    main()
