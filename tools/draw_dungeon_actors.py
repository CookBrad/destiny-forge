#!/usr/bin/env python3
"""In-house dungeon actors at the art-direction canvases.

Pixel-authored at 1×. Every opaque RGBA is copied from a signed hunter
cell. Re-running writes the same bytes. One cell per file.

Slime green is not in the signed sheets. Gel uses the sword cyan and the
few sage pixels that are. King Slime is a crowned body, not a big slime.
"""

from __future__ import annotations

import hashlib
import io
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SIGNED = ROOT / "assets" / "source" / "hunter_signed"
ENEMIES = ROOT / "assets" / "dungeon" / "enemies"
PROJECTILES = ROOT / "assets" / "dungeon" / "projectiles"

# Same pins as tools/draw_hunter_frames.py. The script stops if those
# bytes move, so a new actor cannot be painted from the wrong palette.
SIGNED_SHA256 = {
    "knight_idle_side.png": "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e",
    "knight_run_side.png": "1e05a64f76044a614d8a080e234778ce8eab3c9bff819fa99de3e34b816466e7",
    "knight_attack_side.png": "ca089a0e93524161f020d9ef91f736e330d36f3530749b8296e918b088620b04",
}

NEIGHBORS = tuple((dx, dy) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dx or dy)


def reject_resample_names() -> None:
    """Stop if this file names a resample call. Tokens are split so the
    check can see them without the source containing the joined word."""
    text = Path(__file__).read_text(encoding="utf-8")
    banned = (
        "res" + "ize",
        "thumb" + "nail",
        "trans" + "form",
        "aff" + "ine",
        "quant" + "ize",
    )
    for word in banned:
        if word in text:
            raise SystemExit(f"draw script must not mention {word}")


def load_allowed_colors() -> set[tuple[int, int, int, int]]:
    allowed: set[tuple[int, int, int, int]] = set()
    for name, digest in SIGNED_SHA256.items():
        path = SIGNED / name
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest:
            raise SystemExit(f"{name} hash {actual} != pinned source")
        image = Image.open(path).convert("RGBA")
        for red, green, blue, alpha in image.getdata():
            if alpha == 0:
                continue
            if alpha != 255:
                raise SystemExit(f"partial alpha in {name}")
            allowed.add((red, green, blue, 255))
    return allowed


class Palette:
    def __init__(self, allowed: set[tuple[int, int, int, int]]) -> None:
        def take(red: int, green: int, blue: int) -> tuple[int, int, int, int]:
            color = (red, green, blue, 255)
            if color not in allowed:
                raise SystemExit(f"color {color} is not in the signed hunter")
            return color

        self.ink = take(0, 0, 0)
        self.ink_warm = take(32, 23, 21)
        self.leather_deep = take(43, 23, 15)
        self.leather_dk = take(55, 31, 21)
        self.leather = take(74, 41, 26)
        self.leather_lt = take(97, 55, 35)
        self.leather_hi = take(107, 64, 33)
        self.skin = take(222, 152, 98)
        self.brass_dk = take(203, 141, 45)
        self.brass = take(226, 166, 64)
        self.brass_lt = take(244, 172, 13)
        self.brass_hi = take(251, 214, 119)
        self.gel_deep = take(0, 64, 69)
        self.gel_dk = take(45, 170, 152)
        self.gel = take(3, 190, 174)
        self.gel_lt = take(0, 213, 215)
        self.gel_hot = take(166, 253, 254)
        self.sage_dk = take(67, 94, 77)
        self.sage = take(80, 127, 100)
        self.sage_lt = take(84, 122, 95)
        self.purple_dk = take(108, 21, 120)
        self.purple = take(184, 118, 200)
        self.purple_lt = take(244, 136, 254)
        self.steel_deep = take(13, 13, 13)
        self.steel_dk = take(38, 38, 43)
        self.steel = take(48, 48, 53)
        self.steel_mid = take(135, 133, 137)
        self.steel_lt = take(168, 168, 169)
        self.bone_dk = take(150, 149, 144)
        self.bone = take(245, 244, 243)
        self.bone_lt = take(255, 255, 255)
        self.dead_dk = take(52, 36, 31)
        self.dead = take(54, 37, 32)
        self.white = take(255, 255, 255)
        self.gel_ramp = (self.gel_deep, self.gel, self.gel_lt)
        self.sage_ramp = (self.sage_dk, self.sage, self.sage_lt)
        self.leather_ramp = (self.leather_deep, self.leather, self.leather_lt)
        self.brass_ramp = (self.brass_dk, self.brass, self.brass_lt)
        self.bone_ramp = (self.bone_dk, self.bone, self.bone_lt)
        self.steel_ramp = (self.steel_deep, self.steel_dk, self.steel)
        # Pallid gray, not the coat brown. Signed sheets have no corpse green.
        self.dead_ramp = (self.dead_dk, self.steel_mid, self.steel_lt)
        self.skin_ramp = (self.leather, self.skin, self.brass_hi)
        self.wing_ramp = (self.steel_deep, self.steel_dk, self.steel)


class Canvas:
    def __init__(self, width: int, height: int) -> None:
        self.width = width
        self.height = height
        self.pixels = [(0, 0, 0, 0)] * (width * height)

    def put(self, x: int, y: int, color: tuple[int, int, int, int]) -> None:
        if color[3] not in (0, 255):
            raise SystemExit("partial alpha")
        if 0 <= x < self.width and 0 <= y < self.height:
            self.pixels[y * self.width + x] = color

    def fill(self, points: list[tuple[int, int]], color: tuple[int, int, int, int]) -> None:
        for x, y in points:
            self.put(x, y, color)

    def blit(self, shaded_pixels: dict[tuple[int, int], tuple[int, int, int, int]]) -> None:
        for (x, y), color in shaded_pixels.items():
            self.put(x, y, color)


def ellipse_pixels(cx: int, cy: int, rx: int, ry: int) -> list[tuple[int, int]]:
    if rx < 1 or ry < 1:
        return []
    rx2 = rx * rx
    ry2 = ry * ry
    points = []
    for y in range(cy - ry, cy + ry + 1):
        dy = y - cy
        for x in range(cx - rx, cx + rx + 1):
            dx = x - cx
            if dx * dx * ry2 + dy * dy * rx2 <= rx2 * ry2:
                points.append((x, y))
    return points


def capsule_pixels(x0: int, y0: int, x1: int, y1: int, radius: int) -> list[tuple[int, int]]:
    min_x = min(x0, x1) - radius
    max_x = max(x0, x1) + radius
    min_y = min(y0, y1) - radius
    max_y = max(y0, y1) + radius
    vx = x1 - x0
    vy = y1 - y0
    length2 = vx * vx + vy * vy
    radius2 = radius * radius
    points = []
    for y in range(min_y, max_y + 1):
        for x in range(min_x, max_x + 1):
            if _near_segment(x, y, x0, y0, x1, y1, vx, vy, length2, radius2):
                points.append((x, y))
    return points


def _near_segment(
    x: int,
    y: int,
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    vx: int,
    vy: int,
    length2: int,
    radius2: int,
) -> bool:
    if length2 == 0:
        dx = x - x0
        dy = y - y0
        return dx * dx + dy * dy <= radius2
    wx = x - x0
    wy = y - y0
    dot = wx * vx + wy * vy
    if dot <= 0:
        return wx * wx + wy * wy <= radius2
    if dot >= length2:
        dx = x - x1
        dy = y - y1
        return dx * dx + dy * dy <= radius2
    dist2_num = (wx * wx + wy * wy) * length2 - dot * dot
    return dist2_num <= radius2 * length2


def chain(points: list[tuple[int, int]], radius: int) -> list[tuple[int, int]]:
    drawn: list[tuple[int, int]] = []
    for start, end in zip(points, points[1:]):
        drawn.extend(capsule_pixels(start[0], start[1], end[0], end[1], radius))
    return drawn


def polygon_pixels(points: list[tuple[int, int]]) -> list[tuple[int, int]]:
    if len(points) < 3:
        return []
    y_min = min(point[1] for point in points)
    y_max = max(point[1] for point in points)
    drawn = []
    count = len(points)
    for y in range(y_min, y_max + 1):
        crossings = _scanline_crossings(points, count, y)
        crossings.sort()
        pair = 0
        while pair + 1 < len(crossings):
            left = crossings[pair]
            right = crossings[pair + 1]
            for x in range(left, right + 1):
                drawn.append((x, y))
            pair += 2
    return drawn


def _scanline_crossings(points: list[tuple[int, int]], count: int, y: int) -> list[int]:
    crossings = []
    for index in range(count):
        x0, y0 = points[index]
        x1, y1 = points[(index + 1) % count]
        if y0 == y1:
            continue
        low, high = (y0, y1) if y0 < y1 else (y1, y0)
        if y < low or y >= high:
            continue
        crossings.append(x0 + (x1 - x0) * (y - y0) // (y1 - y0))
    return crossings


def shaded(
    points: list[tuple[int, int]],
    cx: int,
    cy: int,
    ramp: tuple[tuple[int, int, int, int], tuple[int, int, int, int], tuple[int, int, int, int]],
    split: int,
) -> dict[tuple[int, int], tuple[int, int, int, int]]:
    """Three flat bands from a light sitting up and to the left of center.

    A straight diagonal cut reads as a slice. Distance from one lamp keeps
    the bands rounded, which is the gel and the leather.
    """
    if not points:
        return {}
    offset = max(split, 4)
    lamp_x = cx - offset
    lamp_y = cy - offset
    dark, mid, light = ramp
    distances = []
    for x, y in points:
        dx = x - lamp_x
        dy = y - lamp_y
        distances.append(dx * dx + dy * dy)
    nearest = min(distances)
    span = max(distances) - nearest
    painted = {}
    if span <= 0:
        for x, y in points:
            painted[(x, y)] = mid
        return painted
    for (x, y), dist in zip(points, distances):
        band = (dist - nearest) * 3 // (span + 1)
        if band <= 0:
            painted[(x, y)] = light
        elif band == 1:
            painted[(x, y)] = mid
        else:
            painted[(x, y)] = dark
    return painted


def add_outline(canvas: Canvas, color: tuple[int, int, int, int]) -> None:
    """1 px contour just outside the body so the silhouette stays readable."""
    width = canvas.width
    height = canvas.height
    extra: list[tuple[int, int]] = []
    for y in range(height):
        row = y * width
        for x in range(width):
            if canvas.pixels[row + x][3] == 0:
                continue
            for dx, dy in NEIGHBORS:
                nx = x + dx
                ny = y + dy
                if 0 <= nx < width and 0 <= ny < height and canvas.pixels[ny * width + nx][3] == 0:
                    extra.append((nx, ny))
    for x, y in extra:
        canvas.put(x, y, color)


def paint_eye(canvas: Canvas, x: int, y: int, palette: Palette, iris: tuple[int, int, int, int]) -> None:
    canvas.fill(ellipse_pixels(x, y, 3, 3), palette.white)
    canvas.put(x + 1, y, iris)
    canvas.put(x + 1, y + 1, palette.ink)
    canvas.put(x - 1, y - 1, palette.white)


def draw_slime(palette: Palette) -> Canvas:
    canvas = Canvas(64, 64)
    canvas.blit(shaded(ellipse_pixels(32, 18, 6, 9), 32, 16, palette.gel_ramp, 5))
    canvas.blit(shaded(ellipse_pixels(33, 40, 22, 20), 30, 34, palette.gel_ramp, 12))
    canvas.blit(shaded(ellipse_pixels(14, 50, 6, 8), 14, 48, palette.gel_ramp, 5))
    canvas.blit(shaded(ellipse_pixels(36, 50, 12, 7), 34, 48, palette.sage_ramp, 5))
    canvas.fill([(x, 62) for x in range(16, 50)], palette.gel_deep)
    paint_eye(canvas, 36, 36, palette, palette.gel_deep)
    paint_eye(canvas, 48, 37, palette, palette.purple_dk)
    canvas.fill(ellipse_pixels(43, 46, 5, 2), palette.ink)
    add_outline(canvas, palette.ink)
    canvas.put(35, 35, palette.white)
    canvas.put(47, 36, palette.white)
    return canvas


def draw_blob(palette: Palette) -> Canvas:
    canvas = Canvas(32, 32)
    canvas.blit(shaded(ellipse_pixels(16, 16, 14, 14), 16, 16, palette.gel_ramp, 8))
    canvas.fill(ellipse_pixels(16, 18, 4, 4), palette.gel_deep)
    canvas.fill(ellipse_pixels(12, 12, 3, 2), palette.gel_hot)
    add_outline(canvas, palette.ink)
    canvas.put(11, 11, palette.white)
    return canvas


def draw_bolt(palette: Palette) -> Canvas:
    canvas = Canvas(16, 48)
    canvas.blit(shaded(ellipse_pixels(8, 10, 6, 7), 8, 8, palette.gel_ramp, 4))
    tail: list[tuple[int, int]] = []
    for y in range(16, 46):
        half = 4 - (y - 16) * 3 // 30
        if half < 1:
            half = 1
        for x in range(8 - half, 9 + half):
            tail.append((x, y))
    canvas.blit(shaded(tail, 8, 18, palette.gel_ramp, 6))
    canvas.fill(ellipse_pixels(8, 9, 2, 3), palette.gel_hot)
    canvas.put(7, 8, palette.white)
    add_outline(canvas, palette.ink)
    return canvas


def draw_arrow(palette: Palette) -> Canvas:
    canvas = Canvas(16, 48)
    head_rows = {
        1: (7, 8),
        2: (6, 9),
        3: (5, 10),
        4: (4, 11),
        5: (5, 10),
        6: (6, 9),
        7: (7, 8),
    }
    for y, (x0, x1) in head_rows.items():
        for x in range(x0, x1 + 1):
            canvas.put(x, y, palette.brass_lt if x >= 8 else palette.brass)
    canvas.put(7, 2, palette.brass_hi)
    canvas.put(8, 3, palette.brass_hi)
    canvas.put(3, 6, palette.brass_dk)
    canvas.put(3, 7, palette.brass_dk)
    canvas.put(12, 6, palette.brass_dk)
    canvas.put(12, 7, palette.brass_dk)
    for y in range(8, 34):
        canvas.put(7, y, palette.leather_deep)
        canvas.put(8, y, palette.leather)
    for step, y in enumerate(range(30, 44)):
        inset = step // 3
        canvas.put(6 - inset, y, palette.bone)
        canvas.put(5 - min(inset, 2), y, palette.bone_dk)
        canvas.put(9 + inset, y, palette.bone)
        canvas.put(10 + min(inset, 2), y, palette.steel_lt)
    canvas.put(7, 44, palette.leather_deep)
    canvas.put(8, 44, palette.leather_deep)
    canvas.put(7, 45, palette.ink_warm)
    canvas.put(8, 45, palette.ink_warm)
    add_outline(canvas, palette.ink)
    return canvas


def draw_bat(palette: Palette) -> Canvas:
    canvas = Canvas(96, 48)
    left_wing = polygon_pixels([(46, 24), (22, 8), (6, 14), (4, 26), (16, 34), (40, 30)])
    right_wing = polygon_pixels([(52, 22), (78, 6), (90, 12), (92, 24), (78, 34), (58, 30)])
    canvas.blit(shaded(left_wing, 24, 18, palette.wing_ramp, 10))
    canvas.blit(shaded(right_wing, 74, 16, palette.wing_ramp, 10))
    canvas.fill(chain([(44, 22), (18, 12), (8, 20)], 1), palette.purple_dk)
    canvas.fill(chain([(46, 26), (12, 26), (18, 32)], 1), palette.purple)
    canvas.fill(chain([(54, 20), (80, 10), (88, 18)], 1), palette.purple)
    canvas.fill(chain([(54, 26), (84, 24), (74, 32)], 1), palette.purple_dk)
    canvas.blit(shaded(ellipse_pixels(49, 25, 10, 11), 49, 24, palette.leather_ramp, 7))
    canvas.fill(polygon_pixels([(42, 18), (38, 6), (48, 16)]), palette.leather_dk)
    canvas.fill(polygon_pixels([(54, 16), (58, 4), (62, 18)]), palette.leather)
    canvas.fill(polygon_pixels([(42, 16), (40, 10), (46, 15)]), palette.purple_dk)
    canvas.fill(polygon_pixels([(56, 15), (58, 9), (60, 16)]), palette.purple)
    canvas.fill(capsule_pixels(44, 34, 41, 40, 1), palette.bone_dk)
    canvas.fill(capsule_pixels(54, 34, 57, 40, 1), palette.bone_dk)
    add_outline(canvas, palette.ink)
    canvas.put(45, 24, palette.gel_hot)
    canvas.put(46, 24, palette.ink)
    canvas.put(53, 24, palette.gel_hot)
    canvas.put(54, 24, palette.ink)
    canvas.put(44, 23, palette.white)
    return canvas


def draw_goblin(palette: Palette) -> Canvas:
    canvas = Canvas(80, 112)
    # Ear overlaps the skull so the outline joins them. A tall spike read as a hat.
    canvas.fill(polygon_pixels([(42, 24), (28, 16), (34, 22), (48, 32)]), palette.leather_dk)
    canvas.fill(polygon_pixels([(40, 24), (32, 18), (44, 28)]), palette.purple_dk)
    canvas.blit(shaded(ellipse_pixels(50, 30, 13, 12), 50, 28, palette.skin_ramp, 8))
    canvas.blit(shaded(ellipse_pixels(64, 34, 8, 5), 64, 32, palette.skin_ramp, 4))
    torso = polygon_pixels([(32, 48), (46, 42), (62, 50), (58, 80), (36, 84), (26, 64)])
    canvas.blit(shaded(torso, 44, 58, palette.leather_ramp, 14))
    canvas.fill([(x, 76) for x in range(36, 58)], palette.brass_dk)
    canvas.fill([(x, 77) for x in range(36, 58)], palette.brass)
    canvas.blit(shaded(capsule_pixels(40, 54, 24, 72, 3), 32, 60, palette.leather_ramp, 6))
    canvas.blit(shaded(capsule_pixels(56, 54, 70, 66, 3), 64, 58, palette.skin_ramp, 6))
    canvas.blit(shaded(capsule_pixels(68, 66, 76, 44, 2), 74, 52, palette.brass_ramp, 6))
    canvas.fill(capsule_pixels(64, 66, 72, 66, 1), palette.brass_dk)
    canvas.fill(ellipse_pixels(76, 44, 2, 2), palette.brass_hi)
    canvas.blit(shaded(capsule_pixels(42, 82, 32, 106, 4), 36, 94, palette.leather_ramp, 8))
    canvas.blit(shaded(capsule_pixels(54, 82, 64, 106, 4), 58, 94, palette.leather_ramp, 8))
    canvas.fill(ellipse_pixels(30, 107, 7, 3), palette.leather_deep)
    canvas.fill(ellipse_pixels(66, 107, 8, 3), palette.leather_dk)
    canvas.fill([(x, 110) for x in range(24, 38)], palette.ink_warm)
    canvas.fill([(x, 110) for x in range(58, 75)], palette.ink_warm)
    add_outline(canvas, palette.ink)
    paint_eye(canvas, 56, 28, palette, palette.purple)
    canvas.put(68, 34, palette.brass_lt)
    return canvas


def draw_skeleton(palette: Palette) -> Canvas:
    canvas = Canvas(64, 144)
    bow = chain([(20, 28), (10, 50), (8, 74), (12, 98), (22, 116)], 2)
    canvas.blit(shaded(bow, 12, 72, palette.brass_ramp, 12))
    canvas.fill(capsule_pixels(24, 34, 24, 112, 1), palette.steel_dk)
    canvas.blit(shaded(ellipse_pixels(38, 20, 12, 14), 38, 18, palette.bone_ramp, 8))
    canvas.blit(shaded(ellipse_pixels(40, 32, 8, 5), 40, 32, palette.bone_ramp, 4))
    canvas.fill(ellipse_pixels(41, 33, 4, 2), palette.ink)
    canvas.fill(ellipse_pixels(32, 18, 3, 4), palette.ink)
    canvas.fill(ellipse_pixels(44, 18, 3, 4), palette.ink)
    canvas.fill(capsule_pixels(36, 38, 34, 100, 2), palette.bone)
    rib_half = (12, 15, 14, 11, 7)
    for index, y in enumerate((50, 62, 74, 86, 98)):
        half = rib_half[index]
        tone = palette.bone_lt if index % 2 == 0 else palette.bone
        canvas.fill(capsule_pixels(36 - half, y, 36 + half, y, 1), tone)
    canvas.fill(capsule_pixels(30, 52, 16, 78, 2), palette.bone)
    canvas.fill(capsule_pixels(44, 50, 52, 76, 2), palette.bone)
    canvas.blit(shaded(ellipse_pixels(36, 106, 8, 4), 36, 106, palette.bone_ramp, 3))
    canvas.fill(capsule_pixels(32, 108, 26, 134, 2), palette.bone)
    canvas.fill(capsule_pixels(40, 108, 48, 134, 2), palette.bone)
    canvas.fill(ellipse_pixels(24, 138, 6, 3), palette.bone_dk)
    canvas.fill(ellipse_pixels(50, 138, 6, 3), palette.bone)
    canvas.fill([(x, 142) for x in range(18, 31)], palette.ink_warm)
    canvas.fill([(x, 142) for x in range(44, 57)], palette.ink_warm)
    add_outline(canvas, palette.ink)
    canvas.put(45, 18, palette.gel_hot)
    canvas.put(33, 19, palette.steel_deep)
    return canvas


def draw_zombie(palette: Palette) -> Canvas:
    canvas = Canvas(96, 128)
    canvas.blit(shaded(capsule_pixels(40, 54, 20, 98, 5), 28, 74, palette.dead_ramp, 10))
    coat = polygon_pixels(
        [
            (32, 46),
            (50, 40),
            (74, 48),
            (82, 78),
            (74, 102),
            (66, 94),
            (58, 108),
            (48, 96),
            (40, 110),
            (30, 98),
            (24, 78),
        ]
    )
    canvas.blit(shaded(coat, 52, 64, palette.leather_ramp, 18))
    canvas.blit(shaded(capsule_pixels(70, 56, 88, 72, 5), 80, 62, palette.dead_ramp, 8))
    canvas.fill(capsule_pixels(88, 68, 92, 60, 1), palette.bone)
    canvas.fill(capsule_pixels(90, 72, 93, 70, 1), palette.bone_dk)
    canvas.fill(capsule_pixels(86, 76, 92, 80, 1), palette.bone)
    canvas.blit(shaded(ellipse_pixels(66, 32, 13, 14), 66, 30, palette.dead_ramp, 8))
    canvas.fill(ellipse_pixels(72, 42, 6, 4), palette.ink)
    canvas.fill([(70, 39), (72, 38), (74, 39), (76, 40)], palette.bone)
    # A rip, not a badge: dark hole, purple thread along one edge.
    canvas.fill(polygon_pixels([(52, 66), (60, 58), (66, 70), (58, 82), (48, 74)]), palette.ink)
    canvas.fill(chain([(50, 70), (56, 64), (62, 74)], 1), palette.purple)
    canvas.blit(shaded(capsule_pixels(46, 100, 36, 120, 6), 40, 110, palette.steel_ramp, 8))
    canvas.blit(shaded(capsule_pixels(64, 100, 76, 120, 6), 70, 110, palette.steel_ramp, 8))
    canvas.fill(ellipse_pixels(34, 122, 8, 4), palette.leather_deep)
    canvas.fill(ellipse_pixels(78, 122, 8, 4), palette.leather_dk)
    canvas.fill([(x, 126) for x in range(26, 43)], palette.ink_warm)
    canvas.fill([(x, 126) for x in range(70, 88)], palette.ink_warm)
    add_outline(canvas, palette.ink)
    canvas.fill(ellipse_pixels(60, 30, 3, 2), palette.bone_dk)
    canvas.put(61, 30, palette.ink)
    canvas.fill(ellipse_pixels(72, 32, 2, 2), palette.ink)
    canvas.put(72, 32, palette.sage)
    return canvas


def draw_king(palette: Palette) -> Canvas:
    canvas = Canvas(192, 192)
    canvas.blit(shaded(ellipse_pixels(96, 108, 64, 60), 88, 96, palette.gel_ramp, 26))
    canvas.blit(shaded(ellipse_pixels(98, 160, 78, 30), 98, 148, palette.gel_ramp, 16))
    canvas.blit(shaded(ellipse_pixels(102, 152, 54, 30), 102, 146, palette.sage_ramp, 14))
    canvas.blit(shaded(ellipse_pixels(30, 148, 18, 40), 30, 140, palette.gel_ramp, 14))
    canvas.blit(shaded(ellipse_pixels(164, 156, 14, 30), 164, 150, palette.gel_ramp, 10))
    # Spikes sink into the dome so the crown is part of the body, not three icons.
    canvas.blit(shaded(polygon_pixels([(96, 1), (74, 78), (118, 78)]), 96, 30, palette.brass_ramp, 16))
    canvas.blit(shaded(polygon_pixels([(56, 18), (38, 76), (78, 72)]), 54, 40, palette.brass_ramp, 10))
    canvas.blit(shaded(polygon_pixels([(140, 14), (116, 76), (162, 70)]), 140, 36, palette.brass_ramp, 10))
    canvas.blit(shaded(ellipse_pixels(96, 74, 46, 14), 96, 70, palette.brass_ramp, 12))
    canvas.fill(ellipse_pixels(96, 30, 5, 7), palette.purple)
    canvas.fill(ellipse_pixels(96, 28, 2, 3), palette.purple_lt)
    canvas.fill(ellipse_pixels(52, 42, 3, 4), palette.purple_dk)
    canvas.fill(ellipse_pixels(146, 36, 3, 4), palette.gel)
    canvas.fill(capsule_pixels(72, 88, 90, 100, 2), palette.purple_dk)
    canvas.fill(capsule_pixels(74, 86, 88, 98, 1), palette.purple)
    canvas.fill(ellipse_pixels(114, 136, 30, 14), palette.ink)
    for x in range(94, 138, 8):
        canvas.fill(ellipse_pixels(x, 126, 2, 4), palette.bone)
        canvas.fill(ellipse_pixels(x + 4, 146, 2, 3), palette.bone_dk)
    ring = set(ellipse_pixels(100, 98, 18, 18)) - set(ellipse_pixels(100, 98, 13, 13))
    canvas.fill(list(ring), palette.brass)
    canvas.blit(shaded(ellipse_pixels(100, 98, 11, 11), 100, 98, palette.gel_ramp, 6))
    canvas.fill(ellipse_pixels(100, 98, 4, 4), palette.gel_hot)
    canvas.fill(ellipse_pixels(68, 84, 7, 7), palette.purple_dk)
    canvas.fill(ellipse_pixels(68, 84, 3, 3), palette.purple_lt)
    canvas.fill(ellipse_pixels(138, 90, 6, 6), palette.gel_dk)
    canvas.fill(ellipse_pixels(138, 90, 2, 2), palette.gel_hot)
    crack = chain([(46, 76), (58, 94), (50, 112), (66, 126)], 1)
    canvas.fill(crack, palette.gel_hot)
    canvas.fill([(x, 190) for x in range(36, 160)], palette.gel_deep)
    add_outline(canvas, palette.ink)
    canvas.put(98, 94, palette.white)
    canvas.put(94, 16, palette.brass_hi)
    canvas.put(142, 28, palette.gel_hot)
    return canvas


def to_image(canvas: Canvas) -> Image.Image:
    image = Image.new("RGBA", (canvas.width, canvas.height), (0, 0, 0, 0))
    image.putdata(canvas.pixels)
    return image


def png_bytes(image: Image.Image) -> bytes:
    buffer = io.BytesIO()
    image.save(buffer, format="PNG", compress_level=9)
    return buffer.getvalue()


def save_png(image: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    first = png_bytes(image)
    path.write_bytes(first)
    if png_bytes(Image.open(path).convert("RGBA")) != first:
        raise SystemExit(f"{path.name} did not round-trip")


def opaque_stats(canvas: Canvas) -> tuple[int, int, int, int, int, set[tuple[int, int, int, int]]]:
    count = 0
    min_x, min_y = canvas.width, canvas.height
    max_x, max_y = 0, 0
    colors: set[tuple[int, int, int, int]] = set()
    for index, color in enumerate(canvas.pixels):
        alpha = color[3]
        if alpha == 0:
            continue
        if alpha != 255:
            raise SystemExit("partial alpha")
        x = index % canvas.width
        y = index // canvas.width
        count += 1
        min_x = min(min_x, x)
        min_y = min(min_y, y)
        max_x = max(max_x, x)
        max_y = max(max_y, y)
        colors.add(color)
    return count, min_x, min_y, max_x, max_y, colors


def require_sheet(
    name: str,
    canvas: Canvas,
    allowed: set[tuple[int, int, int, int]],
    min_opaque: int,
    touch_bottom: bool,
    touch_top: bool,
    min_span_x: int,
    min_span_y: int,
) -> None:
    count, min_x, min_y, max_x, max_y, colors = opaque_stats(canvas)
    if count < min_opaque:
        raise SystemExit(f"{name} has only {count} opaque pixels")
    invented = colors - allowed
    if invented:
        raise SystemExit(f"{name} invented {next(iter(invented))}")
    if touch_bottom and max_y != canvas.height - 1:
        raise SystemExit(f"{name} sole ends on row {max_y}")
    if touch_top and min_y > 1:
        raise SystemExit(f"{name} top row is {min_y}")
    if max_x - min_x < min_span_x:
        raise SystemExit(f"{name} span x is {max_x - min_x}")
    if max_y - min_y < min_span_y:
        raise SystemExit(f"{name} span y is {max_y - min_y}")


def draw_all(palette: Palette) -> dict[str, Canvas]:
    return {
        "slime": draw_slime(palette),
        "bat": draw_bat(palette),
        "goblin": draw_goblin(palette),
        "skeleton": draw_skeleton(palette),
        "zombie": draw_zombie(palette),
        "king_slime": draw_king(palette),
        "arrow": draw_arrow(palette),
        "slime_bolt": draw_bolt(palette),
        "slime_blob": draw_blob(palette),
    }


def check_all(sheets: dict[str, Canvas], allowed: set[tuple[int, int, int, int]]) -> None:
    expected = {
        "slime": (64, 64, 700, True, False, 36, 40),
        "bat": (96, 48, 400, False, False, 70, 28),
        "goblin": (80, 112, 700, True, False, 40, 90),
        "skeleton": (64, 144, 500, True, False, 28, 120),
        "zombie": (96, 128, 900, True, False, 60, 90),
        "king_slime": (192, 192, 8000, True, True, 140, 180),
        "arrow": (16, 48, 40, False, False, 8, 36),
        "slime_bolt": (16, 48, 40, False, False, 8, 30),
        "slime_blob": (32, 32, 200, False, False, 24, 24),
    }
    masks = []
    for name, (width, height, min_opaque, bottom, top, span_x, span_y) in expected.items():
        canvas = sheets[name]
        if canvas.width != width or canvas.height != height:
            raise SystemExit(f"{name} canvas {canvas.width}x{canvas.height}")
        require_sheet(name, canvas, allowed, min_opaque, bottom, top, span_x, span_y)
        masks.append(tuple(pixel[3] for pixel in canvas.pixels))
    if len(set(masks)) != len(masks):
        raise SystemExit("two actors share a silhouette")
    if sheets["arrow"].pixels == sheets["slime_bolt"].pixels:
        raise SystemExit("arrow and slime bolt match")


def output_paths() -> dict[str, Path]:
    return {
        "slime": ENEMIES / "slime.png",
        "bat": ENEMIES / "bat.png",
        "goblin": ENEMIES / "goblin.png",
        "skeleton": ENEMIES / "skeleton.png",
        "zombie": ENEMIES / "zombie.png",
        "king_slime": ENEMIES / "king_slime.png",
        "arrow": PROJECTILES / "arrow.png",
        "slime_bolt": PROJECTILES / "slime_bolt.png",
        "slime_blob": PROJECTILES / "slime_blob.png",
    }


def main() -> None:
    reject_resample_names()
    allowed = load_allowed_colors()
    palette = Palette(allowed)
    sheets = draw_all(palette)
    check_all(sheets, allowed)
    for name, path in output_paths().items():
        save_png(to_image(sheets[name]), path)


if __name__ == "__main__":
    main()
