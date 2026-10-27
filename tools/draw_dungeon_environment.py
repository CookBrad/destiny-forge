#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from PIL import Image
from art_common import CLEAR, Canvas, clipping_canvas, require_opaque_color
ROOT = Path(__file__).resolve().parents[1]
ENV = ROOT / "assets" / "dungeon" / "environment"
VFX = ROOT / "assets" / "dungeon" / "vfx"
INK = require_opaque_color((16, 14, 20, 255))
MORTAR = require_opaque_color((54, 42, 36, 255))
MORTAR_DARK = require_opaque_color((32, 26, 28, 255))
BROWN = require_opaque_color((118, 82, 54, 255))
BROWN_LIGHT = require_opaque_color((156, 114, 74, 255))
BROWN_DARK = require_opaque_color((74, 50, 34, 255))
BROWN_DEEP = require_opaque_color((52, 36, 28, 255))
GRAY = require_opaque_color((68, 66, 76, 255))
GRAY_LIGHT = require_opaque_color((104, 102, 114, 255))
GRAY_DARK = require_opaque_color((40, 38, 48, 255))
IRON = require_opaque_color((48, 46, 56, 255))
IRON_DARK = require_opaque_color((30, 28, 38, 255))
BRASS = require_opaque_color((214, 164, 70, 255))
BRASS_LIGHT = require_opaque_color((246, 214, 128, 255))
BRASS_DARK = require_opaque_color((132, 90, 36, 255))
PURPLE = require_opaque_color((186, 70, 255, 255))
PURPLE_LIGHT = require_opaque_color((230, 170, 255, 255))
PURPLE_DARK = require_opaque_color((96, 28, 150, 255))
BLUE = require_opaque_color((70, 220, 255, 255))
BLUE_LIGHT = require_opaque_color((186, 246, 255, 255))
BLUE_DARK = require_opaque_color((22, 86, 170, 255))
VOID = require_opaque_color((8, 6, 14, 255))
VOID_MID = require_opaque_color((14, 12, 26, 255))
RED = require_opaque_color((214, 48, 24, 255))
RED_DARK = require_opaque_color((112, 18, 14, 255))
ORANGE = require_opaque_color((255, 132, 36, 255))
YELLOW = require_opaque_color((255, 214, 96, 255))
TILE_PX = 32
WALL_BRICK_WIDTH = 32
WALL_BRICK_HEIGHT = 16
WALL_PATTERN = 256
SLAB_WIDTH = 64
GROUND_PATTERN_W = 128
LADDER_W = 64
LADDER_H = 256
STAKE_W = 24
STAKE_H = 112
LIP_W = 24
LIP_H = 96
PILLAR_W = 64
PILLAR_H = 512
TORCH_W = 32
TORCH_H = 64
CHAIN_W = 16
CHAIN_H = 160

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
        start_x: int,
        start_y: int,
        end_x: int,
        end_y: int,
        color: tuple[int, int, int, int],
    ) -> None:
        for y in range(start_y, end_y):
            for x in range(start_x, end_x):
                self.put(x, y, color)

    def image(self) -> Image.Image:
        image = Image.new("RGBA", (self.width, self.height))
        image.putdata(self.pixels)
        return image

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
    pick = (x * 13 + y * 29 + x * y) % 11
    if pick == 0:
        return -18
    if pick == 1:
        return 14
    return 0

def rivet(canvas: Canvas, x: int, y: int) -> None:
    canvas.put(x, y, BRASS_DARK)
    canvas.put(x + 1, y, BRASS)
    canvas.put(x, y + 1, BRASS)
    canvas.put(x + 1, y + 1, BRASS_LIGHT)

TILE_PX = 32

def hash2(x: int, y: int, salt: int = 0) -> int:
    value = (x * 374761393 + y * 668265263 + salt * 2246822519) & 0xFFFFFFFF
    value = ((value ^ (value >> 13)) * 1274126177) & 0xFFFFFFFF
    return (value ^ (value >> 16)) & 0xFF

def bevel_block(
    canvas: Canvas,
    start_x: int,
    start_y: int,
    end_x: int,
    end_y: int,
    base: tuple[int, int, int, int],
    wrap: bool = False,
) -> None:
    for y in range(start_y, end_y):
        for x in range(start_x, end_x):
            color = shade(base, speck(x, y))
            if y < start_y + 2 or x < start_x + 2:
                color = shade(color, 20 if (y == start_y or x == start_x) else 10)
            if y >= end_y - 2 or x >= end_x - 2:
                color = shade(color, -24 if (y == end_y - 1 or x == end_x - 1) else -12)
            if wrap:
                canvas.put(x % canvas.width, y % canvas.height, color)
            else:
                canvas.put(x, y, color)

def crack(canvas: Canvas, points: tuple[tuple[int, int], ...], color, glow) -> None:
    for (ax, ay), (bx, by) in zip(points, points[1:]):
        steps = max(abs(bx - ax), abs(by - ay), 1)
        for step in range(steps + 1):
            x = ax + (bx - ax) * step // steps
            y = ay + (by - ay) * step // steps
            canvas.put(x, y, glow if step % 3 == 0 else color)

WALL_BRICK_WIDTH = 32
WALL_BRICK_HEIGHT = 16
WALL_PATTERN = 256

def draw_wall() -> Canvas:
    canvas = Canvas(WALL_PATTERN, WALL_PATTERN, MORTAR_DARK)
    stones = (shade(GRAY, -10), shade(GRAY_DARK, 4), shade(GRAY, -18), shade(GRAY_DARK, -4))
    for course in range(WALL_PATTERN // WALL_BRICK_HEIGHT):
        start_y = course * WALL_BRICK_HEIGHT
        offset = WALL_BRICK_WIDTH // 2 if course % 2 else 0
        for brick in range(WALL_PATTERN // WALL_BRICK_WIDTH + 1):
            start_x = brick * WALL_BRICK_WIDTH - offset
            pick = hash2(brick, course, 3)
            base = stones[pick % 4] if pick > 20 else shade(BROWN_DARK, -10)
            bevel_block(
                canvas,
                start_x + 1,
                start_y + 1,
                start_x + WALL_BRICK_WIDTH - 1,
                start_y + WALL_BRICK_HEIGHT - 1,
                base,
                wrap=True,
            )
            if pick % 9 == 0:
                for d in range(4):
                    canvas.put((start_x + 1 + d) % WALL_PATTERN, start_y + 1, MORTAR_DARK)
                    canvas.put((start_x + 1) % WALL_PATTERN, start_y + 1 + d, MORTAR_DARK)
    for y in range(0, WALL_PATTERN, WALL_BRICK_HEIGHT):
        for x in range(WALL_PATTERN):
            canvas.put(x, y, MORTAR if (x // 4) % 3 else MORTAR_DARK)
    crack(canvas, ((10, 20), (16, 26), (22, 27), (27, 33), (30, 40), (36, 44)), BLUE_DARK, BLUE)
    crack(canvas, ((150, 66), (156, 70), (164, 70), (169, 77), (176, 79), (180, 86)), BLUE_DARK, BLUE)
    crack(canvas, ((70, 180), (76, 186), (82, 186), (85, 192), (92, 197)), PURPLE_DARK, PURPLE)
    crack(canvas, ((210, 200), (214, 208), (220, 210), (222, 220)), BLUE_DARK, BLUE_DARK)
    canvas.fill_rect(104, 97, 120, 111, BRASS_DARK)
    for x in range(105, 119):
        canvas.put(x, 98, BRASS_LIGHT)
    for y in range(101, 109, 3):
        for x in range(106, 118):
            canvas.put(x, y, INK)
    rivet(canvas, 105, 108)
    rivet(canvas, 117, 108)
    return canvas

SLAB_WIDTH = 64
GROUND_PATTERN_W = 128

def draw_floor() -> Canvas:
    canvas = Canvas(GROUND_PATTERN_W, TILE_PX, MORTAR_DARK)
    for slab in range(GROUND_PATTERN_W // SLAB_WIDTH):
        start_x = slab * SLAB_WIDTH
        base = BROWN if slab % 2 == 0 else shade(BROWN_DARK, 12)
        bevel_block(canvas, start_x + 1, 4, start_x + SLAB_WIDTH - 1, TILE_PX - 1, base)
        rivet(canvas, start_x + 6, 9)
        rivet(canvas, start_x + SLAB_WIDTH - 8, 9)
        for x in range(start_x + 14, start_x + SLAB_WIDTH - 14):
            canvas.put(x, 20, BLUE_DARK if x % 6 else BLUE)
        canvas.put(start_x + 14, 19, BLUE)
        canvas.put(start_x + SLAB_WIDTH - 15, 21, BLUE)
    for x in range(GROUND_PATTERN_W):
        canvas.put(x, 0, BRASS_LIGHT if x % 16 else BRASS)
        canvas.put(x, 1, BRASS)
        canvas.put(x, 2, BRASS_DARK)
        canvas.put(x, 3, INK)
    for x in (18, 19, 20, 85, 86, 101):
        canvas.put(x, 1, BRASS_DARK)
    return canvas

def draw_ground_fill() -> Canvas:
    canvas = Canvas(GROUND_PATTERN_W, GROUND_PATTERN_W, INK)
    for course in range(GROUND_PATTERN_W // 32):
        offset = 32 if course % 2 else 0
        for block in range(GROUND_PATTERN_W // SLAB_WIDTH + 1):
            start_x = block * SLAB_WIDTH - offset
            pick = hash2(block, course, 11)
            base = (BROWN_DEEP, shade(BROWN_DEEP, -10), shade(GRAY_DARK, -6))[pick % 3]
            bevel_block(canvas, start_x + 1, course * 32 + 1, start_x + SLAB_WIDTH - 1, course * 32 + 31, base, wrap=True)
    crack(canvas, ((20, 40), (28, 48), (36, 49), (44, 58)), PURPLE_DARK, PURPLE_DARK)
    crack(canvas, ((90, 100), (96, 104), (104, 104), (110, 112)), BLUE_DARK, BLUE_DARK)
    return canvas

def draw_ceiling_beam() -> Canvas:
    canvas = Canvas(GROUND_PATTERN_W, TILE_PX, IRON)
    bevel_block(canvas, 0, 0, GROUND_PATTERN_W, TILE_PX - 4, IRON)
    for x in range(GROUND_PATTERN_W):
        canvas.put(x, 3, GRAY)
        canvas.put(x, TILE_PX - 8, PURPLE_DARK if x % 8 else PURPLE)
        canvas.put(x, TILE_PX - 7, PURPLE if x % 8 else PURPLE_LIGHT)
        canvas.put(x, TILE_PX - 6, PURPLE_DARK)
        for y in range(TILE_PX - 4, TILE_PX):
            canvas.put(x, y, INK if y > TILE_PX - 3 else IRON_DARK)
    for x in range(8, GROUND_PATTERN_W, 32):
        rivet(canvas, x, 9)
        rivet(canvas, x + 16, 15)
    for x in (30, 94):
        canvas.put(x, TILE_PX - 3, BRASS)
        canvas.put(x + 1, TILE_PX - 3, BRASS_LIGHT)
        canvas.put(x, TILE_PX - 2, BRASS_DARK)
        canvas.put(x + 1, TILE_PX - 2, BRASS)
    return canvas

def draw_platform() -> Canvas:
    canvas = Canvas(GROUND_PATTERN_W, TILE_PX, CLEAR)
    for bay in range(GROUND_PATTERN_W // SLAB_WIDTH):
        start_x = bay * SLAB_WIDTH
        canvas.fill_rect(start_x, 0, start_x + SLAB_WIDTH, TILE_PX, IRON_DARK)
        for x in range(start_x, start_x + SLAB_WIDTH):
            canvas.put(x, 0, PURPLE_LIGHT)
            canvas.put(x, 1, PURPLE)
            canvas.put(x, 2, PURPLE_DARK)
            for y in range(3, 8):
                canvas.put(x, y, shade(IRON, 14 if y == 3 else speck(x, y)))
            for y in range(TILE_PX - 6, TILE_PX):
                canvas.put(x, y, shade(IRON, -6 if y < TILE_PX - 1 else -30))
        for step in range(TILE_PX - 14):
            for delta_x in (0, 1, 2):
                left = start_x + 4 + step * (SLAB_WIDTH - 8) // (TILE_PX - 14) + delta_x
                right = start_x + SLAB_WIDTH - 5 - step * (SLAB_WIDTH - 8) // (TILE_PX - 14) - delta_x
                canvas.put(left, 8 + step, GRAY if delta_x == 0 else GRAY_DARK)
                canvas.put(right, 8 + step, GRAY if delta_x == 0 else GRAY_DARK)
        for y in range(3, TILE_PX):
            for delta_x, color in ((0, BRASS_LIGHT), (1, BRASS), (2, BRASS), (3, BRASS_DARK)):
                canvas.put(start_x + delta_x, y, color)
        rivet(canvas, start_x + 1, 5)
        rivet(canvas, start_x + 1, TILE_PX - 5)
        canvas.put(start_x + SLAB_WIDTH // 2, 17, BLUE_LIGHT)
        canvas.put(start_x + SLAB_WIDTH // 2 - 1, 17, BLUE)
        canvas.put(start_x + SLAB_WIDTH // 2 + 1, 17, BLUE)
    return canvas

def draw_pit() -> Canvas:
    canvas = Canvas(GROUND_PATTERN_W, GROUND_PATTERN_W, VOID)
    for y in range(GROUND_PATTERN_W):
        for x in range(GROUND_PATTERN_W):
            streak = 8 if (x // 4) % 9 == 2 else 0
            canvas.put(x, y, shade(VOID, streak + speck(x, y) // 3))
    for start_x in (22, 88):
        for y in range(GROUND_PATTERN_W):
            wobble = (y // 16) % 2
            canvas.put(start_x + wobble, y, BLUE_DARK if y % 12 else BLUE)
    for start_y in (40, 104):
        for x in range(23, 89):
            canvas.put(x, start_y, PURPLE_DARK if x % 10 else PURPLE)
    for x, y in ((23, 40), (89, 104), (89, 40), (23, 104)):
        canvas.put(x, y, BLUE_LIGHT)
    for x, y in ((8, 12), (60, 74), (110, 20), (44, 120), (100, 64)):
        canvas.put(x, y, PURPLE_DARK)
    return canvas

LADDER_W = 64
LADDER_H = 256

def draw_ladder() -> Canvas:
    canvas = Canvas(LADDER_W, LADDER_H, CLEAR)
    rails = (10, 48)
    for y in range(LADDER_H):
        for rail in rails:
            for delta_x, color in enumerate((BRASS_LIGHT, BRASS, BRASS, BRASS_DARK, shade(BRASS_DARK, -20), INK)):
                canvas.put(rail + delta_x, y, color)
        canvas.put(56, y, PURPLE_DARK)
        canvas.put(57, y, PURPLE if y % 16 else PURPLE_LIGHT)
        canvas.put(58, y, PURPLE_DARK)
    for rung, y in enumerate(range(20, LADDER_H - 16, 28)):
        for x in range(16, 48):
            canvas.put(x, y, GRAY_LIGHT)
            canvas.put(x, y + 1, GRAY)
            canvas.put(x, y + 2, GRAY_DARK)
            canvas.put(x, y + 3, INK)
        rivet(canvas, 16, y + 1)
        rivet(canvas, 46, y + 1)
        if rung % 2 == 0:
            canvas.put(56, y, BRASS)
            canvas.put(57, y, BRASS_LIGHT)
    canvas.fill_rect(4, 0, 60, 14, IRON_DARK)
    for x in range(4, 60):
        canvas.put(x, 0, BRASS_LIGHT)
        canvas.put(x, 1, BRASS)
        canvas.put(x, 12, PURPLE)
        canvas.put(x, 13, PURPLE_DARK)
    for x in range(10, 54):
        for y in range(4, 10):
            canvas.put(x, y, BLUE_LIGHT if y in (6, 7) else BLUE)
    for y in range(LADDER_H - 10, LADDER_H):
        for x in range(6, 20):
            canvas.put(x, y, BRASS_DARK if y > LADDER_H - 4 else BRASS)
        for x in range(44, 58):
            canvas.put(x, y, BRASS_DARK if y > LADDER_H - 4 else BRASS)
    rivet(canvas, 8, LADDER_H - 7)
    rivet(canvas, 52, LADDER_H - 7)
    return canvas

STAKE_W = 24
STAKE_H = 112

def draw_stake() -> Canvas:
    canvas = Canvas(STAKE_W, STAKE_H, CLEAR)
    for y in range(0, 12):
        for x in range(5, 19):
            if (x - 11.5) ** 2 + (y - 7) ** 2 <= 36:
                canvas.put(x, y, BLUE_LIGHT if y < 6 else BLUE)
    canvas.fill_rect(4, 12, 20, 16, BRASS_DARK)
    for x in range(4, 20):
        canvas.put(x, 12, BRASS_LIGHT)
    for y in range(16, STAKE_H - 8):
        for x, color in ((7, BRASS_LIGHT), (8, BRASS), (9, BRASS), (10, BRASS), (11, BRASS), (12, BRASS_DARK), (13, BRASS_DARK), (14, INK)):
            canvas.put(x + 1, y, color)
        if 24 <= y < 72 and (y // 8) % 2 == 0:
            for x in range(8, 15):
                canvas.put(x, y, PURPLE if x < 12 else PURPLE_DARK)
    for y in range(STAKE_H - 8, STAKE_H):
        for x in range(2, 22):
            canvas.put(x, y, BRASS if y < STAKE_H - 3 else BRASS_DARK)
    rivet(canvas, 4, STAKE_H - 6)
    rivet(canvas, 18, STAKE_H - 6)
    return canvas

LIP_W = 24
LIP_H = 96

def draw_lip() -> Canvas:
    canvas = Canvas(LIP_W, LIP_H, CLEAR)
    for y in range(LIP_H):
        reach = 10 + (hash2(0, y // 6, 5) % 12) - y // 12
        reach = max(4, min(LIP_W, reach))
        for x in range(reach):
            base = BROWN_DEEP if (y // 24) % 2 == 0 else shade(GRAY_DARK, -4)
            color = shade(base, speck(x, y))
            if x >= reach - 2:
                color = shade(color, -20)
            if x < 2:
                color = shade(color, 14)
            canvas.put(x, y, color)
    for x in range(LIP_W):
        canvas.put(x, 0, BRASS if x < 18 else BRASS_DARK)
        canvas.put(x, 1, BRASS_DARK)
    for y in range(10, 70, 9):
        canvas.put(3 + y % 5, y, BLUE_DARK)
    return canvas

PILLAR_W = 64
PILLAR_H = 512

def draw_pillar() -> Canvas:
    canvas = Canvas(PILLAR_W, PILLAR_H, CLEAR)
    shaft_start_x, shaft_end_x = 8, 56
    for y in range(PILLAR_H):
        for x in range(shaft_start_x, shaft_end_x):
            t = (x - shaft_start_x) / (shaft_end_x - shaft_start_x)
            delta = int(18 - 44 * abs(t - 0.35))
            canvas.put(x, y, shade(shade(GRAY, delta), speck(x, y)))
        if y % 48 == 0:
            for x in range(shaft_start_x, shaft_end_x):
                canvas.put(x, y, MORTAR_DARK)
                canvas.put(x, y + 1, shade(GRAY, 24))
        canvas.put(38, y, PURPLE_DARK)
        canvas.put(39, y, PURPLE if y % 24 else PURPLE_LIGHT)
        canvas.put(40, y, PURPLE_DARK)
    for start_y, end_y in ((0, 28), (PILLAR_H - 32, PILLAR_H)):
        bevel_block(canvas, 0, start_y, PILLAR_W, end_y, shade(GRAY_DARK, 6))
        for x in range(PILLAR_W):
            canvas.put(x, start_y + 6, BRASS)
            canvas.put(x, start_y + 7, BRASS_DARK)
            canvas.put(x, end_y - 8, BRASS_LIGHT)
            canvas.put(x, end_y - 7, BRASS)
        for x in range(6, PILLAR_W, 16):
            rivet(canvas, x, start_y + 12)
    for start_y in (160, 320):
        for y in range(start_y, start_y + 8):
            for x in range(shaft_start_x - 2, shaft_end_x + 2):
                canvas.put(x, y, BRASS_LIGHT if y == start_y else BRASS if y < start_y + 6 else BRASS_DARK)
        rivet(canvas, 14, start_y + 3)
        rivet(canvas, 48, start_y + 3)
    crack(canvas, ((18, 210), (22, 220), (20, 232), (26, 244)), BLUE_DARK, BLUE)
    return canvas

TORCH_W = 32
TORCH_H = 64

def draw_torch() -> Canvas:
    canvas = Canvas(TORCH_W, TORCH_H, CLEAR)
    flame = (
        (16, 4, 3, PURPLE_LIGHT),
        (16, 12, 7, PURPLE),
        (16, 18, 9, PURPLE_DARK),
    )
    for cx, cy, radius, color in reversed(flame):
        for y in range(cy - radius, cy + radius + 1):
            for x in range(cx - radius, cx + radius + 1):
                if (x - cx) ** 2 + ((y - cy) * 1.4) ** 2 <= radius * radius:
                    canvas.put(x, y, color)
    for y in range(8, 22):
        canvas.put(16, y, BLUE_LIGHT if y < 16 else BLUE)
        canvas.put(15, y + 2, BLUE)
    for y in range(24, 32):
        inset = (y - 24) // 2
        for x in range(6 + inset, 26 - inset):
            canvas.put(x, y, BRASS_LIGHT if y == 24 else BRASS if x < 20 - inset else BRASS_DARK)
    for y in range(32, 56):
        for x, color in ((14, BRASS_LIGHT), (15, BRASS), (16, BRASS), (17, BRASS_DARK)):
            canvas.put(x, y, color)
    canvas.fill_rect(8, 52, 24, 64, IRON)
    for x in range(8, 24):
        canvas.put(x, 52, GRAY_LIGHT)
        canvas.put(x, 63, INK)
    rivet(canvas, 10, 56)
    rivet(canvas, 20, 56)
    return canvas

CHAIN_W = 16
CHAIN_H = 160

def draw_chain() -> Canvas:
    canvas = Canvas(CHAIN_W, CHAIN_H, CLEAR)
    link_h = 12
    for index, start_y in enumerate(range(0, CHAIN_H - 24, link_h - 2)):
        if index % 2 == 0:
            for y in range(start_y, start_y + link_h):
                for x in range(3, 13):
                    edge = x in (3, 12) or y in (start_y, start_y + link_h - 1)
                    inner = x in (4, 11) or y in (start_y + 1, start_y + link_h - 2)
                    if edge:
                        canvas.put(x, y, IRON_DARK)
                    elif inner:
                        canvas.put(x, y, GRAY_LIGHT if x < 8 else GRAY)
        else:
            for y in range(start_y, start_y + link_h):
                canvas.put(7, y, GRAY_LIGHT)
                canvas.put(8, y, GRAY)
                canvas.put(9, y, IRON_DARK)
    hook_top = CHAIN_H - 24
    for y in range(hook_top, CHAIN_H - 6):
        canvas.put(7, y, BRASS_LIGHT)
        canvas.put(8, y, BRASS)
    for x in range(3, 9):
        canvas.put(x, CHAIN_H - 6, BRASS)
        canvas.put(x, CHAIN_H - 5, BRASS_DARK)
    for y in range(CHAIN_H - 12, CHAIN_H - 5):
        canvas.put(3, y, BRASS)
        canvas.put(2, y, BRASS_DARK)
    canvas.put(3, CHAIN_H - 13, BRASS_LIGHT)
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
                color = PURPLE_LIGHT if core < 0.2 else PURPLE if core < 0.55 else PURPLE_DARK
            elif i < 2:
                color = RED_DARK
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
    image = Image.open(path)
    if image.size != (width, height):
        raise SystemExit(f"{path} is {image.size}, expected {(width, height)}")
    if image.mode != "RGBA":
        raise SystemExit(f"{path} mode {image.mode}, expected RGBA")
    alphas = {pixel[3] for pixel in image.getdata()}
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
