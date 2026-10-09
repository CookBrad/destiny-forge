#!/usr/bin/env python3
"""In-house dungeon environment sprites, drawn at hunter scale.

The hunter is ~160 px tall at 1x nearest-neighbor (5 tiles of 32 px). The
environment is drawn to that body: 32x16 bricks, 64 px floor slabs and girder
bays, a 256 px exit ladder, a 512 px pillar, and warning posts, torches and
chains sized against him. TILE stays 32: repeat textures are 128 or 256 px patterns
and the game samples one 32x32 cell per tile, so seams never land on a tile
edge.

Authored pixel-by-pixel at the target size (Pillow). Deterministic: no RNG,
no resample, no upscale of the old 16 px sheets. Re-running rewrites the same
PNGs.

Palette is weathered brown, dark gray, neon purple piping, electric-blue
circuitry, and brass. Edges are hard 1 px. Alpha is only where a sprite
needs a hole (platform underside, ladder, stake, lip, props, ground slam).
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



TILE_PX = 32


def hash2(x: int, y: int, salt: int = 0) -> int:
    """Stable 0..255 pick per cell. Deterministic, no RNG state."""
    value = (x * 374761393 + y * 668265263 + salt * 2246822519) & 0xFFFFFFFF
    value = ((value ^ (value >> 13)) * 1274126177) & 0xFFFFFFFF
    return (value ^ (value >> 16)) & 0xFF


def bevel_block(
    canvas: Canvas,
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    base: tuple[int, int, int, int],
    wrap: bool = False,
) -> None:
    """A stone or iron block with a 2 px lit top-left and a 2 px shadowed bottom-right.

    `wrap` puts pixels that fall off the canvas back on the other side, so a
    running-bond course tiles without a seam.
    """
    for y in range(y0, y1):
        for x in range(x0, x1):
            color = shade(base, speck(x, y))
            if y < y0 + 2 or x < x0 + 2:
                color = shade(color, 20 if (y == y0 or x == x0) else 10)
            if y >= y1 - 2 or x >= x1 - 2:
                color = shade(color, -24 if (y == y1 - 1 or x == x1 - 1) else -12)
            if wrap:
                canvas.put(x % canvas.width, y % canvas.height, color)
            else:
                canvas.put(x, y, color)


def crack(canvas: Canvas, points: tuple[tuple[int, int], ...], color, glow) -> None:
    """Hard 1 px polyline. Neon cracks read as the electric-blue circuitry."""
    for (ax, ay), (bx, by) in zip(points, points[1:]):
        steps = max(abs(bx - ax), abs(by - ay), 1)
        for step in range(steps + 1):
            x = ax + (bx - ax) * step // steps
            y = ay + (by - ay) * step // steps
            canvas.put(x, y, glow if step % 3 == 0 else color)


# Hunter is ~160 px tall (5 tiles). Bricks are 32x16 so a course reads at
# about one tenth of the body. The old 14x7 brick read as gravel next to him.
WALL_BRICK_W = 32
WALL_BRICK_H = 16
WALL_PATTERN = 256


def draw_wall() -> Canvas:
    """256x256 repeat (eight tiles each way). The game samples one 32x32 cell
    per tile, so the bond and the cracks do not repeat every few steps."""
    canvas = Canvas(WALL_PATTERN, WALL_PATTERN, MORTAR_DK)
    stones = (shade(GRAY, -10), shade(GRAY_DK, 4), shade(GRAY, -18), shade(GRAY_DK, -4))
    for course in range(WALL_PATTERN // WALL_BRICK_H):
        y0 = course * WALL_BRICK_H
        offset = WALL_BRICK_W // 2 if course % 2 else 0
        for brick in range(WALL_PATTERN // WALL_BRICK_W + 1):
            x0 = brick * WALL_BRICK_W - offset
            pick = hash2(brick, course, 3)
            base = stones[pick % 4] if pick > 20 else shade(BROWN_DK, -10)
            bevel_block(
                canvas,
                x0 + 1,
                y0 + 1,
                x0 + WALL_BRICK_W - 1,
                y0 + WALL_BRICK_H - 1,
                base,
                wrap=True,
            )
            if pick % 9 == 0:
                # Chipped corner shows the mortar behind.
                for d in range(4):
                    canvas.put((x0 + 1 + d) % WALL_PATTERN, y0 + 1, MORTAR_DK)
                    canvas.put((x0 + 1) % WALL_PATTERN, y0 + 1 + d, MORTAR_DK)
    # Mortar lines catch a little light on the top edge.
    for y in range(0, WALL_PATTERN, WALL_BRICK_H):
        for x in range(WALL_PATTERN):
            canvas.put(x, y, MORTAR if (x // 4) % 3 else MORTAR_DK)
    crack(canvas, ((10, 20), (16, 26), (22, 27), (27, 33), (30, 40), (36, 44)), BLUE_DK, BLUE)
    crack(canvas, ((150, 66), (156, 70), (164, 70), (169, 77), (176, 79), (180, 86)), BLUE_DK, BLUE)
    crack(canvas, ((70, 180), (76, 186), (82, 186), (85, 192), (92, 197)), PURPLE_DK, PURPLE)
    crack(canvas, ((210, 200), (214, 208), (220, 210), (222, 220)), BLUE_DK, BLUE_DK)
    # One brass vent plate per repeat, about a hand wide on the hunter.
    canvas.fill_rect(104, 97, 120, 111, BRASS_DK)
    for x in range(105, 119):
        canvas.put(x, 98, BRASS_LT)
    for y in range(101, 109, 3):
        for x in range(106, 118):
            canvas.put(x, y, INK)
    rivet(canvas, 105, 108)
    rivet(canvas, 117, 108)
    return canvas


SLAB_W = 64
GROUND_PATTERN_W = 128


def draw_floor() -> Canvas:
    """Walkway slab row, 128x32 repeat of two 64 px flagstones.

    Row 0 is the floor line the hunter's sole sits on."""
    canvas = Canvas(GROUND_PATTERN_W, TILE_PX, MORTAR_DK)
    for slab in range(GROUND_PATTERN_W // SLAB_W):
        x0 = slab * SLAB_W
        base = BROWN if slab % 2 == 0 else shade(BROWN_DK, 12)
        bevel_block(canvas, x0 + 1, 4, x0 + SLAB_W - 1, TILE_PX - 1, base)
        rivet(canvas, x0 + 6, 9)
        rivet(canvas, x0 + SLAB_W - 8, 9)
        # Inlaid circuit groove down the slab.
        for x in range(x0 + 14, x0 + SLAB_W - 14):
            canvas.put(x, 20, BLUE_DK if x % 6 else BLUE)
        canvas.put(x0 + 14, 19, BLUE)
        canvas.put(x0 + SLAB_W - 15, 21, BLUE)
    # Worn brass lip along the walking surface.
    for x in range(GROUND_PATTERN_W):
        canvas.put(x, 0, BRASS_LT if x % 16 else BRASS)
        canvas.put(x, 1, BRASS)
        canvas.put(x, 2, BRASS_DK)
        canvas.put(x, 3, INK)
    for x in (18, 19, 20, 85, 86, 101):
        canvas.put(x, 1, BRASS_DK)
    return canvas


def draw_ground_fill() -> Canvas:
    """Foundation under the floor and above the ceiling beam. 128x128 repeat
    of 64x32 blocks, darker than the wall so the room reads as carved out."""
    canvas = Canvas(GROUND_PATTERN_W, GROUND_PATTERN_W, INK)
    for course in range(GROUND_PATTERN_W // 32):
        offset = 32 if course % 2 else 0
        for block in range(GROUND_PATTERN_W // SLAB_W + 1):
            x0 = block * SLAB_W - offset
            pick = hash2(block, course, 11)
            base = (BROWN_DEEP, shade(BROWN_DEEP, -10), shade(GRAY_DK, -6))[pick % 3]
            bevel_block(canvas, x0 + 1, course * 32 + 1, x0 + SLAB_W - 1, course * 32 + 31, base, wrap=True)
    crack(canvas, ((20, 40), (28, 48), (36, 49), (44, 58)), PURPLE_DK, PURPLE_DK)
    crack(canvas, ((90, 100), (96, 104), (104, 104), (110, 112)), BLUE_DK, BLUE_DK)
    return canvas


def draw_ceiling_beam() -> Canvas:
    """Iron header where the wall meets the rock. 128x32 repeat."""
    canvas = Canvas(GROUND_PATTERN_W, TILE_PX, IRON)
    bevel_block(canvas, 0, 0, GROUND_PATTERN_W, TILE_PX - 4, IRON)
    for x in range(GROUND_PATTERN_W):
        canvas.put(x, 3, GRAY)
        canvas.put(x, TILE_PX - 8, PURPLE_DK if x % 8 else PURPLE)
        canvas.put(x, TILE_PX - 7, PURPLE if x % 8 else PURPLE_LT)
        canvas.put(x, TILE_PX - 6, PURPLE_DK)
        for y in range(TILE_PX - 4, TILE_PX):
            canvas.put(x, y, INK if y > TILE_PX - 3 else IRON_DK)
    for x in range(8, GROUND_PATTERN_W, 32):
        rivet(canvas, x, 9)
        rivet(canvas, x + 16, 15)
    # Hook rings for the hanging chains (the game hangs one every few tiles).
    for x in (30, 94):
        canvas.put(x, TILE_PX - 3, BRASS)
        canvas.put(x + 1, TILE_PX - 3, BRASS_LT)
        canvas.put(x, TILE_PX - 2, BRASS_DK)
        canvas.put(x + 1, TILE_PX - 2, BRASS)
    return canvas


def draw_platform() -> Canvas:
    """Iron girder, 128x32 repeat of two 64 px bays with X bracing."""
    canvas = Canvas(GROUND_PATTERN_W, TILE_PX, CLEAR)
    for bay in range(GROUND_PATTERN_W // SLAB_W):
        x0 = bay * SLAB_W
        canvas.fill_rect(x0, 0, x0 + SLAB_W, TILE_PX, IRON_DK)
        # Top chord with neon strip, bottom chord.
        for x in range(x0, x0 + SLAB_W):
            canvas.put(x, 0, PURPLE_LT)
            canvas.put(x, 1, PURPLE)
            canvas.put(x, 2, PURPLE_DK)
            for y in range(3, 8):
                canvas.put(x, y, shade(IRON, 14 if y == 3 else speck(x, y)))
            for y in range(TILE_PX - 6, TILE_PX):
                canvas.put(x, y, shade(IRON, -6 if y < TILE_PX - 1 else -30))
        # X bracing between the chords.
        for step in range(TILE_PX - 14):
            for dx in (0, 1, 2):
                left = x0 + 4 + step * (SLAB_W - 8) // (TILE_PX - 14) + dx
                right = x0 + SLAB_W - 5 - step * (SLAB_W - 8) // (TILE_PX - 14) - dx
                canvas.put(left, 8 + step, GRAY if dx == 0 else GRAY_DK)
                canvas.put(right, 8 + step, GRAY if dx == 0 else GRAY_DK)
        # Brass posts at each bay end.
        for y in range(3, TILE_PX):
            for dx, color in ((0, BRASS_LT), (1, BRASS), (2, BRASS), (3, BRASS_DK)):
                canvas.put(x0 + dx, y, color)
        rivet(canvas, x0 + 1, 5)
        rivet(canvas, x0 + 1, TILE_PX - 5)
        canvas.put(x0 + SLAB_W // 2, 17, BLUE_LT)
        canvas.put(x0 + SLAB_W // 2 - 1, 17, BLUE)
        canvas.put(x0 + SLAB_W // 2 + 1, 17, BLUE)
    return canvas


def draw_pit() -> Canvas:
    """Opaque shaft, 128x128 repeat. Depth streaks and slow circuit veins."""
    canvas = Canvas(GROUND_PATTERN_W, GROUND_PATTERN_W, VOID)
    for y in range(GROUND_PATTERN_W):
        for x in range(GROUND_PATTERN_W):
            streak = 8 if (x // 4) % 9 == 2 else 0
            canvas.put(x, y, shade(VOID, streak + speck(x, y) // 3))
    for x0 in (22, 88):
        for y in range(GROUND_PATTERN_W):
            wobble = (y // 16) % 2
            canvas.put(x0 + wobble, y, BLUE_DK if y % 12 else BLUE)
    for y0 in (40, 104):
        for x in range(23, 89):
            canvas.put(x, y0, PURPLE_DK if x % 10 else PURPLE)
    for x, y in ((23, 40), (89, 104), (89, 40), (23, 104)):
        canvas.put(x, y, BLUE_LT)
    for x, y in ((8, 12), (60, 74), (110, 20), (44, 120), (100, 64)):
        canvas.put(x, y, PURPLE_DK)
    return canvas


LADDER_W = 64
LADDER_H = 256


def draw_ladder() -> Canvas:
    """Exit ladder, 64x256: 1.6x the hunter so it reads as the way out."""
    canvas = Canvas(LADDER_W, LADDER_H, CLEAR)
    rails = (10, 48)
    for y in range(LADDER_H):
        for rail in rails:
            for dx, color in enumerate((BRASS_LT, BRASS, BRASS, BRASS_DK, shade(BRASS_DK, -20), INK)):
                canvas.put(rail + dx, y, color)
        # Neon cable lashed to the right rail.
        canvas.put(56, y, PURPLE_DK)
        canvas.put(57, y, PURPLE if y % 16 else PURPLE_LT)
        canvas.put(58, y, PURPLE_DK)
    for rung, y in enumerate(range(20, LADDER_H - 16, 28)):
        for x in range(16, 48):
            canvas.put(x, y, GRAY_LT)
            canvas.put(x, y + 1, GRAY)
            canvas.put(x, y + 2, GRAY_DK)
            canvas.put(x, y + 3, INK)
        rivet(canvas, 16, y + 1)
        rivet(canvas, 46, y + 1)
        if rung % 2 == 0:
            canvas.put(56, y, BRASS)
            canvas.put(57, y, BRASS_LT)
    # Glowing hatch at the top: the exit reads from across the boss arena.
    canvas.fill_rect(4, 0, 60, 14, IRON_DK)
    for x in range(4, 60):
        canvas.put(x, 0, BRASS_LT)
        canvas.put(x, 1, BRASS)
        canvas.put(x, 12, PURPLE)
        canvas.put(x, 13, PURPLE_DK)
    for x in range(10, 54):
        for y in range(4, 10):
            canvas.put(x, y, BLUE_LT if y in (6, 7) else BLUE)
    # Feet bolted to the floor. The sprite bottom is the floor line.
    for y in range(LADDER_H - 10, LADDER_H):
        for x in range(6, 20):
            canvas.put(x, y, BRASS_DK if y > LADDER_H - 4 else BRASS)
        for x in range(44, 58):
            canvas.put(x, y, BRASS_DK if y > LADDER_H - 4 else BRASS)
    rivet(canvas, 8, LADDER_H - 7)
    rivet(canvas, 52, LADDER_H - 7)
    return canvas


STAKE_W = 24
STAKE_H = 112


def draw_stake() -> Canvas:
    """Pit warning post, 24x112: about hip-to-shoulder on the hunter."""
    canvas = Canvas(STAKE_W, STAKE_H, CLEAR)
    # Warning lamp on top.
    for y in range(0, 12):
        for x in range(5, 19):
            if (x - 11.5) ** 2 + (y - 7) ** 2 <= 36:
                canvas.put(x, y, BLUE_LT if y < 6 else BLUE)
    canvas.fill_rect(4, 12, 20, 16, BRASS_DK)
    for x in range(4, 20):
        canvas.put(x, 12, BRASS_LT)
    for y in range(16, STAKE_H - 8):
        for x, color in ((7, BRASS_LT), (8, BRASS), (9, BRASS), (10, BRASS), (11, BRASS), (12, BRASS_DK), (13, BRASS_DK), (14, INK)):
            canvas.put(x + 1, y, color)
        # Hazard bands, purple and ink, every 16 px.
        if 24 <= y < 72 and (y // 8) % 2 == 0:
            for x in range(8, 15):
                canvas.put(x, y, PURPLE if x < 12 else PURPLE_DK)
    for y in range(STAKE_H - 8, STAKE_H):
        for x in range(2, 22):
            canvas.put(x, y, BRASS if y < STAKE_H - 3 else BRASS_DK)
    rivet(canvas, 4, STAKE_H - 6)
    rivet(canvas, 18, STAKE_H - 6)
    return canvas


LIP_W = 24
LIP_H = 96


def draw_lip() -> Canvas:
    """Crumbling pit wall, 24x96. Left column is flush with the pit edge;
    the right side is jagged where the floor broke away. Mirrored for the far edge."""
    canvas = Canvas(LIP_W, LIP_H, CLEAR)
    for y in range(LIP_H):
        reach = 10 + (hash2(0, y // 6, 5) % 12) - y // 12
        reach = max(4, min(LIP_W, reach))
        for x in range(reach):
            base = BROWN_DEEP if (y // 24) % 2 == 0 else shade(GRAY_DK, -4)
            color = shade(base, speck(x, y))
            if x >= reach - 2:
                color = shade(color, -20)
            if x < 2:
                color = shade(color, 14)
            canvas.put(x, y, color)
    for x in range(LIP_W):
        canvas.put(x, 0, BRASS if x < 18 else BRASS_DK)
        canvas.put(x, 1, BRASS_DK)
    for y in range(10, 70, 9):
        canvas.put(3 + y % 5, y, BLUE_DK)
    return canvas


PILLAR_W = 64
PILLAR_H = 512


def draw_pillar() -> Canvas:
    """Floor-to-beam column, 64x512 (the full room height). Three hunters tall."""
    canvas = Canvas(PILLAR_W, PILLAR_H, CLEAR)
    shaft_x0, shaft_x1 = 8, 56
    for y in range(PILLAR_H):
        for x in range(shaft_x0, shaft_x1):
            # Rounded shading across the shaft.
            t = (x - shaft_x0) / (shaft_x1 - shaft_x0)
            delta = int(18 - 44 * abs(t - 0.35))
            canvas.put(x, y, shade(shade(GRAY, delta), speck(x, y)))
        # Drum joints every 48 px.
        if y % 48 == 0:
            for x in range(shaft_x0, shaft_x1):
                canvas.put(x, y, MORTAR_DK)
                canvas.put(x, y + 1, shade(GRAY, 24))
        # Neon conduit up the face.
        canvas.put(38, y, PURPLE_DK)
        canvas.put(39, y, PURPLE if y % 24 else PURPLE_LT)
        canvas.put(40, y, PURPLE_DK)
    # Capital and base: wider brass-banded blocks.
    for y0, y1 in ((0, 28), (PILLAR_H - 32, PILLAR_H)):
        bevel_block(canvas, 0, y0, PILLAR_W, y1, shade(GRAY_DK, 6))
        for x in range(PILLAR_W):
            canvas.put(x, y0 + 6, BRASS)
            canvas.put(x, y0 + 7, BRASS_DK)
            canvas.put(x, y1 - 8, BRASS_LT)
            canvas.put(x, y1 - 7, BRASS)
        for x in range(6, PILLAR_W, 16):
            rivet(canvas, x, y0 + 12)
    # Brass bands mid-shaft.
    for y0 in (160, 320):
        for y in range(y0, y0 + 8):
            for x in range(shaft_x0 - 2, shaft_x1 + 2):
                canvas.put(x, y, BRASS_LT if y == y0 else BRASS if y < y0 + 6 else BRASS_DK)
        rivet(canvas, 14, y0 + 3)
        rivet(canvas, 48, y0 + 3)
    crack(canvas, ((18, 210), (22, 220), (20, 232), (26, 244)), BLUE_DK, BLUE)
    return canvas


TORCH_W = 32
TORCH_H = 64


def draw_torch() -> Canvas:
    """Wall sconce with a neon flame, 32x64. Mounted above the hunter's head."""
    canvas = Canvas(TORCH_W, TORCH_H, CLEAR)
    flame = (
        (16, 4, 3, PURPLE_LT),
        (16, 12, 7, PURPLE),
        (16, 18, 9, PURPLE_DK),
    )
    for cx, cy, radius, color in reversed(flame):
        for y in range(cy - radius, cy + radius + 1):
            for x in range(cx - radius, cx + radius + 1):
                if (x - cx) ** 2 + ((y - cy) * 1.4) ** 2 <= radius * radius:
                    canvas.put(x, y, color)
    for y in range(8, 22):
        canvas.put(16, y, BLUE_LT if y < 16 else BLUE)
        canvas.put(15, y + 2, BLUE)
    # Brass cup and wall bracket.
    for y in range(24, 32):
        inset = (y - 24) // 2
        for x in range(6 + inset, 26 - inset):
            canvas.put(x, y, BRASS_LT if y == 24 else BRASS if x < 20 - inset else BRASS_DK)
    for y in range(32, 56):
        for x, color in ((14, BRASS_LT), (15, BRASS), (16, BRASS), (17, BRASS_DK)):
            canvas.put(x, y, color)
    canvas.fill_rect(8, 52, 24, 64, IRON)
    for x in range(8, 24):
        canvas.put(x, 52, GRAY_LT)
        canvas.put(x, 63, INK)
    rivet(canvas, 10, 56)
    rivet(canvas, 20, 56)
    return canvas


CHAIN_W = 16
CHAIN_H = 160


def draw_chain() -> Canvas:
    """Hanging chain with a hook, 16x160. Links alternate face and edge."""
    canvas = Canvas(CHAIN_W, CHAIN_H, CLEAR)
    link_h = 12
    for index, y0 in enumerate(range(0, CHAIN_H - 24, link_h - 2)):
        if index % 2 == 0:
            for y in range(y0, y0 + link_h):
                for x in range(3, 13):
                    edge = x in (3, 12) or y in (y0, y0 + link_h - 1)
                    inner = x in (4, 11) or y in (y0 + 1, y0 + link_h - 2)
                    if edge:
                        canvas.put(x, y, IRON_DK)
                    elif inner:
                        canvas.put(x, y, GRAY_LT if x < 8 else GRAY)
        else:
            for y in range(y0, y0 + link_h):
                canvas.put(7, y, GRAY_LT)
                canvas.put(8, y, GRAY)
                canvas.put(9, y, IRON_DK)
    # Hook.
    hook_top = CHAIN_H - 24
    for y in range(hook_top, CHAIN_H - 6):
        canvas.put(7, y, BRASS_LT)
        canvas.put(8, y, BRASS)
    for x in range(3, 9):
        canvas.put(x, CHAIN_H - 6, BRASS)
        canvas.put(x, CHAIN_H - 5, BRASS_DK)
    for y in range(CHAIN_H - 12, CHAIN_H - 5):
        canvas.put(3, y, BRASS)
        canvas.put(2, y, BRASS_DK)
    canvas.put(3, CHAIN_H - 13, BRASS_LT)
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
        (draw_wall(), ENV / "wall.png", WALL_PATTERN, WALL_PATTERN, True),
        (draw_floor(), ENV / "floor_ground.png", GROUND_PATTERN_W, TILE_PX, True),
        (draw_ground_fill(), ENV / "ground_fill.png", GROUND_PATTERN_W, GROUND_PATTERN_W, True),
        (draw_ceiling_beam(), ENV / "ceiling_beam.png", GROUND_PATTERN_W, TILE_PX, True),
        (draw_platform(), ENV / "floor_platform.png", GROUND_PATTERN_W, TILE_PX, True),
        (draw_pit(), ENV / "floor_pit.png", GROUND_PATTERN_W, GROUND_PATTERN_W, True),
        (draw_ladder(), ENV / "floor_ladder.png", LADDER_W, LADDER_H, False),
        (draw_stake(), ENV / "pit_stake.png", STAKE_W, STAKE_H, False),
        (draw_lip(), ENV / "pit_lip.png", LIP_W, LIP_H, False),
        (draw_pillar(), ENV / "pillar.png", PILLAR_W, PILLAR_H, False),
        (draw_torch(), ENV / "torch.png", TORCH_W, TORCH_H, False),
        (draw_chain(), ENV / "chain.png", CHAIN_W, CHAIN_H, False),
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
