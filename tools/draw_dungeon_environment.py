#!/usr/bin/env python3
"""In-house dungeon environment sprites at the 32 px module.

Authored pixel-by-pixel at the target size (Pillow 11.3). Deterministic:
no RNG, no resample, no upscale of the old 16 px sheets. Re-running
rewrites the same PNGs.

Palette is weathered brown, dark gray, neon purple piping, electric-blue
circuitry, and brass. Edges are hard 1 px. Alpha is only where a sprite
needs a hole (ladder, stake, lip, ground slam).
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ENV = ROOT / "assets" / "dungeon" / "environment"
VFX = ROOT / "assets" / "dungeon" / "vfx"

# Palette. Alpha is 255 unless a call site passes a hole.
INK = (16, 14, 20, 255)
MORTAR = (54, 42, 36, 255)
MORTAR_DK = (32, 26, 28, 255)
BROWN = (118, 82, 54, 255)
BROWN_LT = (156, 114, 74, 255)
BROWN_DK = (74, 50, 34, 255)
BROWN_DEEP = (52, 36, 28, 255)
GRAY = (68, 66, 76, 255)
GRAY_LT = (104, 102, 114, 255)
GRAY_DK = (40, 38, 48, 255)
IRON = (48, 46, 56, 255)
IRON_DK = (30, 28, 38, 255)
BRASS = (214, 164, 70, 255)
BRASS_LT = (246, 214, 128, 255)
BRASS_DK = (132, 90, 36, 255)
PURPLE = (186, 70, 255, 255)
PURPLE_LT = (230, 170, 255, 255)
PURPLE_DK = (96, 28, 150, 255)
BLUE = (70, 220, 255, 255)
BLUE_LT = (186, 246, 255, 255)
BLUE_DK = (22, 86, 170, 255)
VOID = (8, 6, 14, 255)
VOID_MID = (14, 12, 26, 255)
RED = (214, 48, 24, 255)
RED_DK = (112, 18, 14, 255)
ORANGE = (255, 132, 36, 255)
YELLOW = (255, 214, 96, 255)
CLEAR = (0, 0, 0, 0)


class Canvas:
    def __init__(self, width: int, height: int, fill: tuple[int, int, int, int] = CLEAR) -> None:
        self.width = width
        self.height = height
        self.pixels = [fill] * (width * height)

    def put(self, x: int, y: int, color: tuple[int, int, int, int]) -> None:
        if 0 <= x < self.width and 0 <= y < self.height:
            self.pixels[y * self.width + x] = color

    def fill_rect(
        self,
        x0: int,
        y0: int,
        x1: int,
        y1: int,
        color: tuple[int, int, int, int],
    ) -> None:
        for y in range(y0, y1):
            for x in range(x0, x1):
                self.put(x, y, color)

    def image(self) -> Image.Image:
        img = Image.new("RGBA", (self.width, self.height))
        img.putdata(self.pixels)
        return img


def clamp_channel(value: int) -> int:
    return 0 if value < 0 else 255 if value > 255 else value


def shade(color: tuple[int, int, int, int], delta: int) -> tuple[int, int, int, int]:
    red, green, blue, alpha = color
    return (
        clamp_channel(red + delta),
        clamp_channel(green + delta),
        clamp_channel(blue + delta),
        alpha,
    )


def speck(x: int, y: int) -> int:
    """Stable -1 / 0 / +1 weather. Not a resample of any source image."""
    pick = (x * 13 + y * 29 + x * y) % 11
    if pick == 0:
        return -18
    if pick == 1:
        return 14
    return 0


def rivet(canvas: Canvas, x: int, y: int) -> None:
    canvas.put(x, y, BRASS_DK)
    canvas.put(x + 1, y, BRASS)
    canvas.put(x, y + 1, BRASS)
    canvas.put(x + 1, y + 1, BRASS_LT)


def draw_floor() -> Canvas:
    canvas = Canvas(32, 32, MORTAR)
    stones = (
        (1, 1, 15, 15, BROWN),
        (17, 1, 31, 15, BROWN_DK),
        (1, 17, 15, 31, shade(BROWN, -8)),
        (17, 17, 31, 31, shade(BROWN_DK, 10)),
    )
    for x0, y0, x1, y1, base in stones:
        canvas.fill_rect(x0, y0, x1, y1, base)
        for y in range(y0 + 1, y1 - 1):
            for x in range(x0 + 1, x1 - 1):
                canvas.put(x, y, shade(base, speck(x, y)))
        for x in range(x0, x1):
            canvas.put(x, y0, shade(base, 28))
            canvas.put(x, y1 - 1, shade(base, -28))
        for y in range(y0, y1):
            canvas.put(x0, y, shade(base, 18))
            canvas.put(x1 - 1, y, shade(base, -22))

    # Chips so the blocks are not perfect rectangles. Mortar shows through.
    for x, y in (
        (2, 2),
        (14, 3),
        (18, 13),
        (29, 4),
        (3, 29),
        (13, 18),
        (28, 28),
        (19, 19),
    ):
        canvas.put(x, y, MORTAR_DK)

    # Cracks.
    for x, y in (
        (4, 6),
        (5, 7),
        (6, 7),
        (7, 8),
        (8, 9),
        (21, 5),
        (22, 5),
        (23, 6),
        (24, 6),
        (25, 6),
        (5, 22),
        (6, 23),
        (7, 24),
        (8, 24),
        (22, 22),
        (23, 23),
        (24, 24),
        (25, 25),
        (26, 25),
    ):
        canvas.put(x, y, BROWN_DEEP)

    # Brass stud and a short blue circuit in the lower-left block.
    rivet(canvas, 24, 7)
    for x in range(5, 12):
        canvas.put(x, 26, BLUE_DK)
    canvas.put(5, 26, BLUE)
    canvas.put(8, 25, BLUE)
    canvas.put(8, 26, BLUE_LT)
    canvas.put(8, 27, BLUE)

    # Dark vertical joint so the four blocks stay separate when the tile repeats.
    for y in range(32):
        if y in (15, 16):
            continue
        canvas.put(15, y, MORTAR_DK)
        canvas.put(16, y, MORTAR)

    # Purple pipe in the horizontal joint. Full width so neighboring tiles connect.
    for x in range(32):
        canvas.put(x, 15, PURPLE_DK if x % 8 else PURPLE)
        canvas.put(x, 16, MORTAR_DK)
    canvas.put(8, 15, PURPLE_LT)
    canvas.put(24, 15, PURPLE_LT)
    # Coupling.
    canvas.put(15, 15, BRASS_DK)
    canvas.put(16, 15, BRASS)
    return canvas


def draw_platform() -> Canvas:
    canvas = Canvas(32, 32, IRON)
    for y in range(32):
        for x in range(32):
            canvas.put(x, y, shade(IRON, speck(x, y) // 2))

    for x in range(32):
        end = x < 2 or x > 29
        canvas.put(x, 0, BRASS_LT if end else PURPLE_LT)
        canvas.put(x, 1, BRASS if end else PURPLE)
        canvas.put(x, 2, BRASS_DK if end else PURPLE_DK)

    for y in range(3, 32):
        canvas.put(0, y, GRAY_LT)
        canvas.put(1, y, GRAY)
        canvas.put(30, y, GRAY_DK)
        canvas.put(31, y, INK)

    canvas.fill_rect(4, 6, 28, 28, IRON_DK)
    for x in range(4, 28):
        canvas.put(x, 6, GRAY)
        canvas.put(x, 27, INK)
    for y in range(6, 28):
        canvas.put(4, y, GRAY_DK)
        canvas.put(27, y, INK)

    for x, y in ((6, 9), (23, 9), (6, 23), (23, 23)):
        rivet(canvas, x, y)

    for x in range(8, 24):
        canvas.put(x, 17, BLUE_DK)
        if x % 5 == 0:
            canvas.put(x, 17, BLUE)
            canvas.put(x, 18, BLUE_DK)
    canvas.put(15, 16, BLUE)
    canvas.put(15, 17, BLUE_LT)
    canvas.put(15, 18, BLUE)
    canvas.put(16, 17, BLUE)

    for x in range(32):
        canvas.put(x, 31, INK)
        canvas.put(x, 30, GRAY_DK)
    return canvas


def draw_wall() -> Canvas:
    canvas = Canvas(32, 32, MORTAR)
    for y in range(32):
        row = y // 8
        in_brick = y % 8 != 0
        offset = 8 if row % 2 else 0
        for x in range(32):
            if not in_brick:
                canvas.put(x, y, MORTAR if (x + y) % 5 else MORTAR_DK)
                continue
            local = (x + offset) % 16
            if local >= 14:
                canvas.put(x, y, MORTAR_DK if local == 15 else MORTAR)
                continue
            col = ((x + offset) // 16) % 2
            base = GRAY if (col + row) % 2 == 0 else GRAY_DK
            color = shade(base, speck(x, y) // 2)
            if y % 8 == 1 or local == 0:
                color = shade(color, 22)
            elif y % 8 == 7 or local == 13:
                color = shade(color, -20)
            canvas.put(x, y, color)

    # Electric-blue cracks, kept off the mortar so the brick course still wraps.
    for x, y in (
        (3, 3),
        (4, 4),
        (5, 4),
        (6, 5),
        (18, 11),
        (19, 12),
        (20, 12),
        (9, 19),
        (10, 20),
        (11, 21),
        (22, 27),
        (23, 28),
        (24, 28),
    ):
        canvas.put(x, y, BLUE if (x + y) % 2 == 0 else BLUE_DK)

    rivet(canvas, 18, 4)
    return canvas


def draw_pit() -> Canvas:
    canvas = Canvas(32, 32, VOID)
    for y in range(32):
        for x in range(32):
            # Vertical streak so a stack of tiles reads as a shaft, not a flat stamp.
            column = 10 if x % 8 == 3 else 0
            canvas.put(x, y, shade(VOID, column + speck(x, y) // 3))

    # Cross meets the next pit tile. Branches stay inside the cell.
    for x in range(32):
        canvas.put(x, 16, BLUE if x % 4 == 0 else BLUE_DK)
    for y in range(32):
        if y != 16:
            canvas.put(16, y, BLUE if y % 4 == 0 else BLUE_DK)
    for step in range(1, 9):
        for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
            color = PURPLE_DK if step % 2 == 0 else BLUE_DK
            canvas.put(16 + sx * step, 16 + sy * step, color)
    for x, y in ((8, 8), (23, 8), (8, 23), (23, 23)):
        canvas.put(x, y, BLUE_LT)
        canvas.put(x + 1, y, BLUE)
        canvas.put(x, y + 1, BLUE)
    canvas.put(16, 16, BLUE_LT)
    canvas.put(15, 16, PURPLE)
    canvas.put(17, 16, PURPLE)
    canvas.put(16, 15, PURPLE)
    canvas.put(16, 17, PURPLE)
    # Dim depth lights. Opaque: the pit tile has no holes.
    for x, y in ((3, 5), (27, 7), (6, 28), (26, 27)):
        canvas.put(x, y, PURPLE_DK)
    return canvas


def draw_ladder() -> Canvas:
    canvas = Canvas(32, 64, CLEAR)
    for y in range(64):
        canvas.put(7, y, BRASS_LT)
        canvas.put(8, y, BRASS)
        canvas.put(9, y, BRASS_DK)
        canvas.put(22, y, BRASS_LT)
        canvas.put(23, y, BRASS)
        canvas.put(24, y, BRASS_DK)
        # Cable tied to the right rail.
        canvas.put(26, y, PURPLE_DK)
        canvas.put(27, y, PURPLE if y % 8 else PURPLE_LT)

    for rung, y in enumerate(range(6, 58, 8)):
        dark = GRAY_DK if rung % 2 == 0 else INK
        for x in range(10, 22):
            canvas.put(x, y, GRAY_LT)
            canvas.put(x, y + 1, dark)
        canvas.put(10, y, BRASS_DK)
        canvas.put(21, y, BRASS_DK)

    # Feet bolted to the floor. The sprite bottom is the floor line.
    for y in range(58, 64):
        for x in range(5, 12):
            canvas.put(x, y, BRASS_DK if y > 60 else BRASS)
        for x in range(20, 28):
            canvas.put(x, y, BRASS_DK if y > 60 else BRASS)
    rivet(canvas, 6, 60)
    rivet(canvas, 22, 60)
    return canvas


def draw_stake() -> Canvas:
    canvas = Canvas(8, 40, CLEAR)
    # Point, then a 4 px shaft, purple band, flared foot.
    canvas.put(3, 0, BRASS_LT)
    canvas.put(4, 0, BRASS)
    for x in range(2, 6):
        canvas.put(x, 1, BRASS_LT if x == 2 else BRASS)
        canvas.put(x, 2, BRASS)
    for y in range(3, 36):
        canvas.put(2, y, BRASS_LT)
        canvas.put(3, y, BRASS)
        canvas.put(4, y, BRASS_DK)
        canvas.put(5, y, shade(BRASS_DK, -16))
    for y in range(8, 13):
        for x in range(2, 6):
            canvas.put(x, y, PURPLE_LT if y == 8 else PURPLE if y < 11 else PURPLE_DK)
    for y in range(36, 40):
        for x in range(1, 7):
            canvas.put(x, y, BRASS if y == 36 else BRASS_DK)
    canvas.put(2, 37, BRASS_LT)
    return canvas


def draw_lip() -> Canvas:
    """One connected slab. Top rows tuck under the floor; the underside is jagged."""
    canvas = Canvas(32, 16, CLEAR)
    depths = (
        14, 12, 13, 10, 11, 15, 13, 9,
        12, 16, 14, 11, 13, 15, 12, 10,
        11, 15, 16, 13, 12, 14, 10, 13,
        15, 11, 9, 12, 14, 13, 11, 12,
    )
    for x, depth in enumerate(depths):
        for y in range(depth):
            base = BROWN if (x // 8) % 2 == 0 else GRAY_DK
            if y < 3:
                base = shade(base, 18)
            canvas.put(x, y, shade(base, speck(x, y)))
    for x, y in ((6, 6), (7, 7), (8, 7), (20, 8), (21, 8), (22, 9), (23, 9)):
        canvas.put(x, y, BROWN_DEEP)
    rivet(canvas, 14, 1)
    canvas.put(3, 2, BRASS)
    canvas.put(28, 2, BRASS_DK)
    return canvas


def draw_ground_slam() -> Canvas:
    canvas = Canvas(96, 24, CLEAR)
    for x in range(96):
        distance = abs(x - 47.5) / 47.5
        arch = int(round((1.0 - distance * distance) * 18))
        jag = (x * 3) % 5 - 2
        height = max(2, min(22, arch + jag))
        if x % 7 == 0:
            height = min(23, height + 3)
        for i in range(height):
            y = 23 - i
            nx = (x - 47.5) / 12.0
            ny = (i - 9) / 7.0
            core = nx * nx + ny * ny
            if core < 1.0 and distance < 0.4:
                color = PURPLE_LT if core < 0.2 else PURPLE if core < 0.55 else PURPLE_DK
            elif i < 2:
                color = RED_DK
            elif i < 7:
                color = RED
            elif i < 12:
                color = ORANGE
            else:
                color = YELLOW
            canvas.put(x, y, color)
        if 18 <= x <= 77:
            canvas.put(x, 23, YELLOW if 40 <= x <= 55 else ORANGE)
    return canvas


def save(canvas: Canvas, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    canvas.image().save(path, format="PNG", optimize=False)


def assert_size(path: Path, width: int, height: int, opaque: bool) -> None:
    img = Image.open(path)
    if img.size != (width, height):
        raise SystemExit(f"{path} is {img.size}, expected {(width, height)}")
    if img.mode != "RGBA":
        raise SystemExit(f"{path} mode {img.mode}, expected RGBA")
    alphas = {pixel[3] for pixel in img.getdata()}
    if opaque and alphas != {255}:
        raise SystemExit(f"{path} is not fully opaque: {sorted(alphas)[:6]}")
    if not opaque and 0 not in alphas:
        raise SystemExit(f"{path} has no transparent pixel")
    if not opaque and 255 not in alphas:
        raise SystemExit(f"{path} has no opaque pixel")


def main() -> None:
    targets = (
        (draw_floor(), ENV / "floor_ground.png", 32, 32, True),
        (draw_platform(), ENV / "floor_platform.png", 32, 32, True),
        (draw_wall(), ENV / "wall.png", 32, 32, True),
        (draw_pit(), ENV / "floor_pit.png", 32, 32, True),
        (draw_ladder(), ENV / "floor_ladder.png", 32, 64, False),
        (draw_stake(), ENV / "pit_stake.png", 8, 40, False),
        (draw_lip(), ENV / "pit_lip.png", 32, 16, False),
        (draw_ground_slam(), VFX / "ground_slam.png", 96, 24, False),
    )
    for canvas, path, width, height, opaque in targets:
        if (canvas.width, canvas.height) != (width, height):
            raise SystemExit(f"canvas {path.name} is {(canvas.width, canvas.height)}")
        save(canvas, path)
        assert_size(path, width, height, opaque)
        print(f"wrote {path.relative_to(ROOT)} {width}x{height}")


if __name__ == "__main__":
    main()
