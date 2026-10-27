#!/usr/bin/env python3
from __future__ import annotations

import io
from pathlib import Path

from PIL import Image

from art_common import Canvas, Palette, add_cast_rim, add_outline, clipping_canvas, hunter_palette, load_pinned_colors, reject_resample_names

ROOT = Path(__file__).resolve().parents[1]
SIGNED = ROOT / "assets" / "source" / "hunter_signed"
ENEMIES = ROOT / "assets" / "dungeon" / "enemies"
PROJECTILES = ROOT / "assets" / "dungeon" / "projectiles"

SIGNED_SHA256 = {
    "knight_idle_side.png": "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e",
    "knight_run_side.png": "1e05a64f76044a614d8a080e234778ce8eab3c9bff819fa99de3e34b816466e7",
    "knight_attack_side.png": "ca089a0e93524161f020d9ef91f736e330d36f3530749b8296e918b088620b04",
}

BAYER4 = (
    (0, 8, 2, 10),
    (12, 4, 14, 6),
    (3, 11, 1, 9),
    (15, 7, 13, 5),
)

def main() -> None:
    reject_resample_names(Path(__file__))
    allowed = load_pinned_colors(SIGNED, SIGNED_SHA256)
    palette = hunter_palette(allowed)
    sheets = draw_all(palette)
    check_all(sheets, allowed)
    for name, path in output_paths().items():
        save_png(to_image(sheets[name]), path)

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

def draw_slime(palette: Palette) -> Canvas:
    canvas = clipping_canvas(64, 64)
    blit_ellipse(canvas, 33, 42, 23, 20, palette.gel_under, 10)
    blit_ellipse(canvas, 32, 36, 21, 18, palette.gel_body, 10)
    blit_ellipse(canvas, 32, 16, 7, 10, palette.gel_body, 5)
    blit_ellipse(canvas, 14, 52, 7, 9, palette.gel_under, 5)
    blit_ellipse(canvas, 36, 51, 13, 8, palette.sage_ramp, 6)
    canvas.blit(volume(ellipse_pixels(24, 30, 8, 6), 22, 28, palette.gel_shine, 4))
    canvas.fill(ellipse_pixels(22, 28, 3, 2), palette.gel_hot)
    canvas.put(21, 27, palette.white)
    canvas.fill(ellipse_pixels(28, 46, 5, 5), palette.gel_deep)
    canvas.fill(ellipse_pixels(40, 38, 3, 3), palette.gel_well)
    canvas.fill([(x, 62) for x in range(14, 52)], palette.gel_deep)
    canvas.fill([(x, 63) for x in range(16, 50)], palette.gel_shadow)
    add_cast_rim(canvas, palette.gel_shadow)
    canvas.fill(ellipse_pixels(43, 47, 6, 2), palette.ink)
    canvas.fill(ellipse_pixels(43, 46, 4, 1), palette.gel_deep)
    paint_eye(canvas, 36, 35, palette, palette.gel_deep)
    paint_eye(canvas, 48, 36, palette, palette.purple_dark)
    add_outline(canvas, palette.ink)
    canvas.put(35, 34, palette.white)
    canvas.put(47, 35, palette.white)
    canvas.put(26, 22, palette.gel_hi)
    return canvas

def blit_ellipse(
    canvas: Canvas,
    cx: int,
    cy: int,
    rx: int,
    ry: int,
    ramp: tuple[tuple[int, int, int, int], ...],
    split: int,
) -> list[tuple[int, int]]:
    points = ellipse_pixels(cx, cy, rx, ry)
    canvas.blit(volume(points, cx, cy, ramp, split))
    return points

def ellipse_pixels(cx: int, cy: int, rx: int, ry: int) -> list[tuple[int, int]]:
    if rx < 1 or ry < 1:
        return []
    rx2 = rx * rx
    ry2 = ry * ry
    points = []
    for y in range(cy - ry, cy + ry + 1):
        delta_y = y - cy
        for x in range(cx - rx, cx + rx + 1):
            delta_x = x - cx
            if delta_x * delta_x * ry2 + delta_y * delta_y * rx2 <= rx2 * ry2:
                points.append((x, y))
    return points

def volume(
    points: list[tuple[int, int]],
    cx: int,
    cy: int,
    ramp: tuple[tuple[int, int, int, int], ...],
    split: int,
) -> dict[tuple[int, int], tuple[int, int, int, int]]:
    if not points:
        return {}
    stops = len(ramp)
    if stops == 1:
        return {point: ramp[0] for point in points}
    last = stops - 1
    offset = max(split, 4)
    lamp_x = cx - offset
    lamp_y = cy - offset
    distances = []
    for x, y in points:
        delta_x = x - lamp_x
        delta_y = y - lamp_y
        distances.append(delta_x * delta_x + delta_y * delta_y)
    nearest = min(distances)
    span = max(distances) - nearest
    painted = {}
    if span <= 0:
        mid = ramp[last // 2]
        for x, y in points:
            painted[(x, y)] = mid
        return painted
    for (x, y), dist in zip(points, distances):
        away = (dist - nearest) * last
        base = away // (span + 1)
        rem = away % (span + 1)
        threshold = BAYER4[y & 3][x & 3]
        if rem * 16 > (span + 1) * threshold and base < last:
            base += 1
        pick = (x * 13 + y * 29) % 17
        if pick == 0 and base < last:
            base += 1
        elif pick == 1 and base > 0:
            base -= 1
        painted[(x, y)] = ramp[last - base]
    return painted

def paint_eye(
    canvas: Canvas,
    x: int,
    y: int,
    palette: Palette,
    iris: tuple[int, int, int, int],
) -> None:
    canvas.fill(ellipse_pixels(x, y, 3, 3), palette.white)
    canvas.fill(ellipse_pixels(x, y, 2, 2), iris)
    canvas.put(x + 1, y, palette.ink)
    canvas.put(x + 1, y + 1, palette.ink)
    canvas.put(x - 1, y - 1, palette.white)
    canvas.put(x, y - 1, palette.white)

def draw_bat(palette: Palette) -> Canvas:
    canvas = clipping_canvas(96, 48)
    blit_poly(
        canvas,
        [(46, 24), (22, 7), (6, 13), (16, 22), (40, 26)],
        (22, 14),
        palette.wing_ramp[2:],
        8,
    )
    blit_poly(
        canvas,
        [(40, 26), (16, 22), (3, 26), (16, 35), (40, 30)],
        (20, 28),
        palette.wing_ramp[:4],
        8,
    )
    blit_poly(
        canvas,
        [(52, 22), (78, 5), (91, 12), (78, 20), (58, 24)],
        (76, 12),
        palette.wing_ramp[2:],
        8,
    )
    blit_poly(
        canvas,
        [(58, 24), (78, 20), (93, 25), (78, 35), (58, 30)],
        (80, 28),
        palette.wing_ramp[:4],
        8,
    )
    canvas.fill(chain([(44, 22), (18, 11), (8, 20)], 1), palette.purple_dark)
    canvas.fill(chain([(46, 26), (12, 26), (18, 33)], 1), palette.purple)
    canvas.fill(chain([(44, 24), (28, 18), (16, 22)], 1), palette.purple_light)
    canvas.fill(chain([(54, 20), (80, 9), (88, 18)], 1), palette.purple)
    canvas.fill(chain([(54, 26), (84, 24), (74, 33)], 1), palette.purple_dark)
    canvas.fill(chain([(56, 23), (72, 16), (86, 20)], 1), palette.purple_light)
    body = blit_ellipse(canvas, 49, 25, 11, 12, palette.leather_ramp, 7)
    canvas.fill(ellipse_pixels(49, 28, 6, 5), palette.leather_deep)
    spark(canvas, body, palette.leather_hi, 9)
    blit_poly(canvas, [(42, 18), (37, 4), (48, 16)], (42, 10), palette.leather_ramp, 5)
    blit_poly(canvas, [(54, 16), (59, 3), (63, 18)], (58, 10), palette.leather_ramp, 5)
    canvas.fill(polygon_pixels([(42, 16), (39, 8), (46, 15)]), palette.purple_dark)
    canvas.fill(polygon_pixels([(56, 15), (59, 7), (61, 16)]), palette.purple)
    canvas.put(40, 9, palette.purple_light)
    canvas.put(59, 8, palette.purple_light)
    blit_capsule(canvas, 44, 34, 40, 41, 1, palette.bone_ramp, 3)
    blit_capsule(canvas, 54, 34, 58, 41, 1, palette.bone_ramp, 3)
    canvas.put(40, 41, palette.brass_dark)
    canvas.put(58, 41, palette.brass)
    paint_gem(canvas, 49, 30, palette, 2)
    add_cast_rim(canvas, palette.ink_warm)
    add_outline(canvas, palette.ink)
    canvas.put(45, 23, palette.gel_hot)
    canvas.put(46, 23, palette.ink)
    canvas.put(45, 24, palette.gel_hi)
    canvas.put(53, 23, palette.gel_hot)
    canvas.put(54, 23, palette.ink)
    canvas.put(44, 22, palette.white)
    canvas.put(52, 22, palette.white)
    return canvas

def blit_poly(
    canvas: Canvas,
    corners: list[tuple[int, int]],
    lamp: tuple[int, int],
    ramp: tuple[tuple[int, int, int, int], ...],
    split: int,
) -> list[tuple[int, int]]:
    points = polygon_pixels(corners)
    canvas.blit(volume(points, lamp[0], lamp[1], ramp, split))
    return points

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
        start_x, start_y = points[index]
        end_x, end_y = points[(index + 1) % count]
        if start_y == end_y:
            continue
        low, high = (start_y, end_y) if start_y < end_y else (end_y, start_y)
        if y < low or y >= high:
            continue
        crossings.append(start_x + (end_x - start_x) * (y - start_y) // (end_y - start_y))
    return crossings

def paint_gem(canvas: Canvas, x: int, y: int, palette: Palette, radius: int) -> None:
    canvas.fill(ellipse_pixels(x, y, radius, radius), palette.purple_dark)
    inner = max(1, radius - 1)
    canvas.fill(ellipse_pixels(x, y, inner, inner), palette.purple)
    canvas.put(x - 1, y - 1, palette.purple_light)
    canvas.put(x, y, palette.purple_light)

def chain(points: list[tuple[int, int]], radius: int) -> list[tuple[int, int]]:
    drawn: list[tuple[int, int]] = []
    for start, end in zip(points, points[1:]):
        drawn.extend(capsule_pixels(start[0], start[1], end[0], end[1], radius))
    return drawn

def capsule_pixels(start_x: int, start_y: int, end_x: int, end_y: int, radius: int) -> list[tuple[int, int]]:
    min_x = min(start_x, end_x) - radius
    max_x = max(start_x, end_x) + radius
    min_y = min(start_y, end_y) - radius
    max_y = max(start_y, end_y) + radius
    vector_x = end_x - start_x
    vector_y = end_y - start_y
    length2 = vector_x * vector_x + vector_y * vector_y
    radius2 = radius * radius
    points = []
    for y in range(min_y, max_y + 1):
        for x in range(min_x, max_x + 1):
            if _near_segment(x, y, start_x, start_y, end_x, end_y, vector_x, vector_y, length2, radius2):
                points.append((x, y))
    return points

def spark(
    canvas: Canvas,
    points: list[tuple[int, int]],
    color: tuple[int, int, int, int],
    mod: int,
) -> None:
    for x, y in points:
        if (x * 7 + y * 13) % mod == 0:
            canvas.put(x, y, color)

def blit_capsule(
    canvas: Canvas,
    start_x: int,
    start_y: int,
    end_x: int,
    end_y: int,
    radius: int,
    ramp: tuple[tuple[int, int, int, int], ...],
    split: int,
) -> list[tuple[int, int]]:
    points = capsule_pixels(start_x, start_y, end_x, end_y, radius)
    canvas.blit(volume(points, (start_x + end_x) // 2, (start_y + end_y) // 2, ramp, split))
    return points

def paint_rivet(canvas: Canvas, x: int, y: int, palette: Palette) -> None:
    canvas.put(x, y, palette.brass_dark)
    canvas.put(x + 1, y, palette.brass)
    canvas.put(x, y + 1, palette.brass)
    canvas.put(x + 1, y + 1, palette.brass_hi)

def _near_segment(
    x: int,
    y: int,
    start_x: int,
    start_y: int,
    end_x: int,
    end_y: int,
    vector_x: int,
    vector_y: int,
    length2: int,
    radius2: int,
) -> bool:
    if length2 == 0:
        delta_x = x - start_x
        delta_y = y - start_y
        return delta_x * delta_x + delta_y * delta_y <= radius2
    wx = x - start_x
    wy = y - start_y
    dot = wx * vector_x + wy * vector_y
    if dot <= 0:
        return wx * wx + wy * wy <= radius2
    if dot >= length2:
        delta_x = x - end_x
        delta_y = y - end_y
        return delta_x * delta_x + delta_y * delta_y <= radius2
    distance_squared_numerator = (wx * wx + wy * wy) * length2 - dot * dot
    return distance_squared_numerator <= radius2 * length2

def plant_boot(canvas: Canvas, cx: int, palette: Palette, half: int) -> None:
    sole = canvas.height - 1
    canvas.fill(ellipse_pixels(cx, sole - 2, half, 3), palette.leather_deep)
    canvas.fill(ellipse_pixels(cx + 1, sole - 3, half - 2, 2), palette.leather_dark)
    canvas.fill([(x, sole) for x in range(cx - half, cx + half + 1)], palette.ink_warm)
    canvas.put(cx - half + 2, sole - 1, palette.leather_hi)

def draw_goblin(palette: Palette) -> Canvas:
    canvas = clipping_canvas(80, 112)
    blit_poly(canvas, [(44, 26), (24, 10), (30, 22), (48, 32)], (32, 18), palette.leather_ramp, 6)
    canvas.fill(polygon_pixels([(42, 24), (28, 14), (44, 28)]), palette.purple_dark)
    canvas.put(30, 16, palette.purple)
    canvas.put(31, 15, palette.purple_light)
    blit_ellipse(canvas, 50, 22, 10, 6, palette.leather_ramp, 5)
    head = blit_ellipse(canvas, 52, 32, 12, 11, palette.skin_ramp, 7)
    canvas.fill(ellipse_pixels(48, 26, 8, 3), palette.skin_dark)
    snout = blit_ellipse(canvas, 64, 35, 8, 5, palette.skin_ramp, 4)
    canvas.fill(ellipse_pixels(66, 37, 3, 2), palette.skin_dark)
    canvas.put(68, 36, palette.ink_warm)
    spark(canvas, head + snout, palette.skin_light, 8)
    blit_poly(canvas, [(36, 42), (50, 38), (62, 44), (58, 52), (40, 52)], (46, 44), palette.leather_ramp, 6)
    canvas.blit(volume(ellipse_pixels(50, 48, 8, 6), 50, 46, palette.cloth_ramp, 5))
    duster = blit_poly(
        canvas,
        [(28, 50), (46, 42), (64, 50), (66, 78), (58, 90), (34, 88), (22, 66)],
        (42, 56),
        palette.leather_ramp,
        14,
    )
    flap = blit_poly(
        canvas,
        [(32, 72), (50, 68), (60, 76), (56, 94), (34, 94), (26, 82)],
        (40, 80),
        palette.leather_ramp[2:],
        10,
    )
    canvas.fill(capsule_pixels(38, 52, 36, 84, 1), palette.leather_shadow)
    canvas.fill(capsule_pixels(50, 50, 52, 80, 1), palette.leather_deep)
    stitch(canvas, [(40, y) for y in range(54, 82)], palette.leather_shadow)
    canvas.fill(chain([(46, 44), (34, 58), (32, 78)], 1), palette.purple_dark)
    canvas.fill(chain([(47, 46), (36, 60)], 1), palette.purple)
    for x in range(34, 60):
        canvas.put(x, 74, palette.brass_shadow)
        canvas.put(x, 75, palette.brass_dark)
        canvas.put(x, 76, palette.brass)
        canvas.put(x, 77, palette.brass_light if x % 2 == 0 else palette.brass)
    paint_rivet(canvas, 44, 75, palette)
    paint_gem(canvas, 52, 76, palette, 2)
    blit_capsule(canvas, 40, 54, 20, 74, 4, palette.leather_ramp, 6)
    blit_capsule(canvas, 56, 52, 70, 64, 4, palette.skin_ramp, 6)
    blade = blit_capsule(canvas, 68, 64, 76, 38, 2, palette.brass_ramp, 6)
    canvas.fill(capsule_pixels(64, 64, 72, 66, 2), palette.brass_shadow)
    canvas.fill(ellipse_pixels(76, 38, 2, 2), palette.brass_hi)
    canvas.put(75, 37, palette.white)
    spark(canvas, blade, palette.brass_hi, 5)
    blit_capsule(canvas, 42, 86, 32, 106, 5, palette.leather_ramp, 8)
    blit_capsule(canvas, 54, 86, 64, 106, 5, palette.leather_ramp, 8)
    canvas.fill(capsule_pixels(34, 96, 30, 104, 1), palette.purple_dark)
    plant_boot(canvas, 30, palette, 7)
    plant_boot(canvas, 66, palette, 8)
    add_cast_rim(canvas, palette.ink_warm)
    add_outline(canvas, palette.ink)
    paint_eye(canvas, 56, 30, palette, palette.purple)
    canvas.put(68, 34, palette.brass_light)
    canvas.put(48, 34, palette.skin_dark)
    spark(canvas, duster + flap, palette.leather_hi, 17)
    return canvas

def stitch(canvas: Canvas, points: list[tuple[int, int]], color: tuple[int, int, int, int]) -> None:
    for index, (x, y) in enumerate(points):
        if index % 3 != 2:
            canvas.put(x, y, color)

def draw_skeleton(palette: Palette) -> Canvas:
    canvas = clipping_canvas(64, 144)
    bow = chain([(20, 28), (10, 50), (8, 74), (12, 98), (22, 116)], 3)
    canvas.blit(volume(bow, 10, 70, palette.brass_ramp, 12))
    canvas.fill(capsule_pixels(18, 36, 18, 108, 1), palette.leather_deep)
    canvas.fill(capsule_pixels(24, 34, 24, 112, 1), palette.steel_dark)
    canvas.fill(capsule_pixels(16, 70, 22, 74, 2), palette.leather)
    paint_rivet(canvas, 16, 70, palette)
    blit_poly(canvas, [(22, 38), (8, 52), (10, 88), (24, 70)], (14, 58), palette.leather_ramp, 8)
    blit_ellipse(canvas, 38, 21, 13, 14, palette.bone_form, 8)
    canvas.blit(volume(ellipse_pixels(34, 15, 6, 5), 32, 13, (palette.bone_md, palette.bone_mid, palette.bone), 3))
    blit_ellipse(canvas, 40, 33, 8, 5, palette.bone_form, 4)
    canvas.fill(ellipse_pixels(32, 18, 4, 5), palette.ink)
    canvas.fill(ellipse_pixels(44, 18, 4, 5), palette.ink)
    canvas.fill(ellipse_pixels(32, 19, 2, 3), palette.steel_deep)
    canvas.fill(ellipse_pixels(44, 19, 2, 3), palette.purple_dark)
    canvas.put(44, 18, palette.gel_hot)
    canvas.put(43, 17, palette.purple)
    canvas.put(33, 17, palette.steel_void)
    canvas.fill(ellipse_pixels(41, 34, 5, 2), palette.ink)
    canvas.put(38, 34, palette.bone_dark)
    canvas.put(40, 34, palette.bone_md)
    canvas.put(42, 34, palette.bone_dark)
    canvas.fill(capsule_pixels(28, 22, 34, 26, 1), palette.bone_shadow)
    canvas.fill(capsule_pixels(46, 22, 50, 26, 1), palette.bone_dark)
    canvas.put(32, 14, palette.bone)
    blit_poly(canvas, [(26, 40), (44, 38), (46, 52), (24, 54)], (34, 44), palette.leather_ramp, 6)
    canvas.fill(chain([(28, 44), (42, 42)], 1), palette.purple_dark)
    paint_rivet(canvas, 30, 46, palette)
    blit_capsule(canvas, 36, 38, 34, 100, 4, palette.bone_form, 8)
    rib_half = (13, 16, 15, 12, 8)
    for index, y in enumerate((50, 62, 74, 86, 98)):
        half = rib_half[index]
        blit_capsule(canvas, 36 - half, y, 36 + half, y + 1, 2, palette.bone_form, 4)
        canvas.put(36 - half, y + 1, palette.bone_shadow)
        canvas.put(36 + half, y, palette.bone_mid)
    blit_capsule(canvas, 28, 50, 14, 78, 3, palette.bone_form, 6)
    blit_capsule(canvas, 44, 48, 54, 76, 3, palette.bone_form, 6)
    canvas.fill(ellipse_pixels(14, 78, 3, 3), palette.brass_dark)
    canvas.fill(ellipse_pixels(54, 76, 3, 3), palette.brass)
    canvas.put(54, 75, palette.brass_hi)
    blit_poly(canvas, [(26, 98), (46, 96), (50, 110), (22, 112)], (36, 104), palette.leather_ramp, 6)
    canvas.fill(ellipse_pixels(36, 104, 3, 3), palette.purple_dark)
    canvas.put(36, 103, palette.purple_light)
    blit_ellipse(canvas, 36, 108, 9, 5, palette.bone_form, 4)
    blit_capsule(canvas, 31, 110, 24, 134, 4, palette.bone_form, 6)
    blit_capsule(canvas, 41, 110, 50, 134, 4, palette.bone_form, 6)
    canvas.fill(ellipse_pixels(24, 134, 3, 3), palette.steel_dark)
    canvas.fill(ellipse_pixels(50, 134, 3, 3), palette.brass_md)
    plant_boot(canvas, 24, palette, 6)
    plant_boot(canvas, 50, palette, 6)
    add_cast_rim(canvas, palette.bone_shadow)
    add_outline(canvas, palette.ink)
    canvas.put(45, 16, palette.gel_hot)
    return canvas

def draw_zombie(palette: Palette) -> Canvas:
    canvas = clipping_canvas(96, 128)
    blit_capsule(canvas, 40, 54, 18, 98, 6, palette.dead_ramp, 10)
    coat = blit_poly(
        canvas,
        [
            (32, 46),
            (50, 38),
            (74, 46),
            (84, 76),
            (78, 104),
            (68, 96),
            (60, 112),
            (50, 98),
            (42, 114),
            (30, 100),
            (22, 78),
        ],
        (50, 64),
        palette.leather_ramp,
        18,
    )
    canvas.fill(capsule_pixels(40, 58, 38, 100, 1), palette.leather_shadow)
    canvas.fill(capsule_pixels(58, 52, 62, 96, 1), palette.leather_deep)
    stitch(canvas, [(46, y) for y in range(56, 100)], palette.leather_shadow)
    canvas.fill(chain([(48, 44), (36, 62), (34, 88)], 1), palette.purple_dark)
    canvas.fill(chain([(50, 46), (38, 64)], 1), palette.purple)
    blit_capsule(canvas, 70, 56, 88, 74, 6, palette.dead_ramp, 8)
    canvas.fill(capsule_pixels(88, 68, 93, 58, 1), palette.bone_dark)
    canvas.fill(capsule_pixels(90, 72, 94, 68, 1), palette.bone_shadow)
    canvas.fill(capsule_pixels(86, 76, 93, 82, 1), palette.bone)
    canvas.put(93, 58, palette.bone_light)
    blit_ellipse(canvas, 66, 32, 14, 15, palette.dead_ramp, 8)
    canvas.fill(ellipse_pixels(60, 26, 6, 4), palette.dead_dark)
    canvas.fill(ellipse_pixels(72, 42, 6, 4), palette.ink)
    canvas.fill([(70, 39), (72, 38), (74, 39), (76, 40), (71, 40)], palette.bone_dark)
    canvas.put(73, 38, palette.bone)
    canvas.fill(capsule_pixels(58, 22, 72, 18, 2), palette.steel_dark)
    canvas.put(64, 18, palette.steel_light)
    tear = polygon_pixels([(52, 66), (62, 56), (68, 72), (58, 84), (48, 74)])
    canvas.fill(tear, palette.ink)
    canvas.blit(volume(ellipse_pixels(58, 70, 4, 5), 58, 70, palette.dead_ramp, 3))
    canvas.fill(chain([(50, 70), (56, 64), (64, 74)], 1), palette.purple)
    canvas.put(62, 66, palette.purple_light)
    paint_rivet(canvas, 54, 52, palette)
    paint_rivet(canvas, 66, 58, palette)
    blit_capsule(canvas, 46, 102, 34, 120, 6, palette.steel_ramp, 8)
    blit_capsule(canvas, 64, 102, 78, 120, 6, palette.steel_ramp, 8)
    plant_boot(canvas, 34, palette, 8)
    plant_boot(canvas, 78, palette, 8)
    add_cast_rim(canvas, palette.ink_warm)
    add_outline(canvas, palette.ink)
    canvas.fill(ellipse_pixels(60, 30, 3, 2), palette.bone_dark)
    canvas.put(61, 30, palette.ink)
    canvas.put(59, 29, palette.steel_light)
    canvas.fill(ellipse_pixels(72, 32, 2, 2), palette.ink)
    canvas.put(72, 32, palette.sage)
    canvas.put(71, 31, palette.gel_hot)
    spark(canvas, coat, palette.leather_hi, 23)
    return canvas

def draw_king(palette: Palette) -> Canvas:
    canvas = clipping_canvas(192, 192)
    blit_ellipse(canvas, 98, 164, 80, 26, palette.gel_under, 14)
    blit_ellipse(canvas, 96, 114, 66, 58, palette.gel_under, 20)
    blit_ellipse(canvas, 92, 104, 58, 50, palette.gel_body, 18)
    blit_ellipse(canvas, 102, 154, 54, 26, palette.sage_ramp, 12)
    blit_ellipse(canvas, 28, 150, 20, 40, palette.gel_body, 12)
    blit_ellipse(canvas, 166, 156, 16, 30, palette.gel_under, 10)
    canvas.blit(volume(ellipse_pixels(72, 88, 20, 16), 66, 80, palette.gel_shine, 8))
    canvas.fill(ellipse_pixels(68, 82, 6, 4), palette.gel_hot)
    canvas.put(66, 80, palette.white)
    add_cast_rim(canvas, palette.gel_shadow)
    canvas.fill(ellipse_pixels(96, 82, 56, 18), palette.steel_deep)
    blit_ellipse(canvas, 96, 80, 52, 15, palette.brass_ramp, 12)
    canvas.fill(ellipse_pixels(96, 78, 46, 8), palette.brass_dark)
    canvas.blit(volume(ellipse_pixels(96, 76, 44, 6), 90, 74, palette.brass_ramp, 8))
    for x in (58, 78, 96, 114, 134):
        paint_rivet(canvas, x, 74, palette)
    crown_spike(canvas, 96, 0, 74, 12, palette)
    crown_spike(canvas, 54, 14, 74, 10, palette)
    crown_spike(canvas, 140, 12, 74, 10, palette)
    canvas.fill(capsule_pixels(54, 70, 96, 72, 2), palette.brass_dark)
    canvas.fill(capsule_pixels(96, 70, 140, 72, 2), palette.brass)
    paint_gem(canvas, 96, 28, palette, 5)
    canvas.fill(ellipse_pixels(96, 26, 2, 2), palette.purple_light)
    paint_gem(canvas, 54, 36, palette, 4)
    paint_gem(canvas, 140, 34, palette, 4)
    paint_gem(canvas, 78, 70, palette, 3)
    paint_gem(canvas, 114, 70, palette, 3)
    canvas.put(140, 32, palette.brass_hi)
    canvas.fill(capsule_pixels(72, 90, 90, 102, 2), palette.purple_dark)
    canvas.fill(capsule_pixels(74, 88, 88, 100, 1), palette.purple)
    canvas.put(80, 92, palette.purple_light)
    canvas.fill(ellipse_pixels(114, 138, 32, 16), palette.ink)
    canvas.fill(ellipse_pixels(114, 142, 26, 10), palette.steel_deep)
    for x in range(90, 140, 8):
        tooth = ellipse_pixels(x, 126, 3, 5)
        canvas.blit(volume(tooth, x, 124, palette.bone_ramp, 3))
        lower = ellipse_pixels(x + 4, 148, 2, 4)
        canvas.blit(volume(lower, x + 4, 150, palette.bone_ramp, 2))
    ring = ring_pixels(100, 98, 20, 20, 13, 13)
    canvas.blit(volume(ring, 96, 94, palette.brass_ramp, 6))
    blit_ellipse(canvas, 100, 98, 12, 12, palette.gel_body, 6)
    canvas.fill(ellipse_pixels(100, 98, 5, 5), palette.gel_hot)
    canvas.put(98, 94, palette.white)
    canvas.put(97, 93, palette.gel_hi)
    paint_gem(canvas, 68, 84, palette, 7)
    canvas.fill(ellipse_pixels(138, 90, 7, 7), palette.gel_deep)
    canvas.fill(ellipse_pixels(138, 90, 3, 3), palette.gel_hot)
    canvas.put(136, 88, palette.white)
    canvas.fill(chain([(46, 86), (58, 104), (50, 122), (66, 136)], 1), palette.gel_hot)
    canvas.fill(chain([(48, 88), (56, 106)], 1), palette.gel_hi)
    canvas.fill([(x, 190) for x in range(32, 164)], palette.gel_deep)
    canvas.fill([(x, 191) for x in range(36, 160)], palette.gel_shadow)
    add_outline(canvas, palette.ink)
    canvas.put(96, 2, palette.brass_hi)
    canvas.put(144, 10, palette.brass_light)
    return canvas

def crown_spike(
    canvas: Canvas,
    cx: int,
    tip_y: int,
    base_y: int,
    half_base: int,
    palette: Palette,
) -> None:
    mid_y = (tip_y + base_y) // 2
    waist = max(3, half_base - 4)
    outline = [
        (cx, tip_y),
        (cx - 3, tip_y + 8),
        (cx - 2, tip_y + 16),
        (cx - waist, mid_y),
        (cx - half_base, base_y),
        (cx + half_base, base_y),
        (cx + waist, mid_y),
        (cx + 2, tip_y + 16),
        (cx + 3, tip_y + 8),
    ]
    ridge = (cx, mid_y)
    left_face = polygon_pixels(outline[:5] + [ridge])
    right_face = polygon_pixels([outline[0], ridge] + outline[5:])
    canvas.blit(volume(left_face, cx - 6, tip_y + 18, palette.brass_ramp[:4], 8))
    canvas.blit(volume(right_face, cx + 4, tip_y + 14, palette.brass_ramp[2:], 8))
    canvas.fill(capsule_pixels(cx, tip_y + 4, cx, base_y - 8, 1), palette.brass_hi)
    canvas.fill(ellipse_pixels(cx, tip_y + 3, 3, 3), palette.brass_light)
    canvas.put(cx, tip_y + 1, palette.brass_hi)

def ring_pixels(cx: int, cy: int, rx: int, ry: int, rx_in: int, ry_in: int) -> list[tuple[int, int]]:
    inner = set(ellipse_pixels(cx, cy, rx_in, ry_in))
    return [point for point in ellipse_pixels(cx, cy, rx, ry) if point not in inner]

def draw_arrow(palette: Palette) -> Canvas:
    canvas = clipping_canvas(16, 48)
    head = polygon_pixels([(8, 1), (3, 8), (6, 8), (6, 11), (10, 11), (10, 8), (13, 8)])
    canvas.blit(volume(head, 7, 4, palette.brass_ramp, 4))
    canvas.put(8, 2, palette.brass_hi)
    canvas.put(7, 3, palette.brass_light)
    canvas.put(4, 7, palette.brass_shadow)
    canvas.put(12, 7, palette.brass_dark)
    canvas.put(8, 9, palette.brass)
    for y in range(11, 34):
        left = palette.leather_shadow if y % 2 == 0 else palette.leather_deep
        right = palette.leather_md if y % 2 == 0 else palette.leather
        canvas.put(6, y, palette.ink_warm)
        canvas.put(7, y, left)
        canvas.put(8, y, right)
        canvas.put(9, y, palette.leather_hi if (y + 8) % 4 == 0 else palette.leather_dark)
    canvas.put(7, 18, palette.brass_dark)
    canvas.put(8, 18, palette.brass)
    canvas.put(7, 19, palette.brass_md)
    canvas.put(8, 19, palette.brass_hi)
    for step, y in enumerate(range(30, 44)):
        inset = step // 3
        left = 6 - inset
        right = 9 + inset
        canvas.put(left, y, palette.bone_dark if step % 2 else palette.bone_shadow)
        canvas.put(left - 1, y, palette.bone_md)
        canvas.put(right, y, palette.bone if step % 2 else palette.bone_mid)
        canvas.put(right + 1, y, palette.steel_light)
    canvas.put(7, 44, palette.leather_deep)
    canvas.put(8, 44, palette.leather)
    canvas.put(7, 45, palette.ink_warm)
    canvas.put(8, 45, palette.ink_warm)
    add_outline(canvas, palette.ink)
    return canvas

def draw_bolt(palette: Palette) -> Canvas:
    canvas = clipping_canvas(16, 48)
    blit_ellipse(canvas, 8, 10, 6, 7, palette.gel_body, 4)
    tail: list[tuple[int, int]] = []
    for y in range(15, 46):
        half = 4 - (y - 15) * 3 // 31
        if half < 1:
            half = 1
        for x in range(8 - half, 9 + half):
            tail.append((x, y))
    canvas.blit(volume(tail, 8, 22, palette.gel_under, 6))
    core: list[tuple[int, int]] = []
    for y in range(8, 36):
        core.append((8, y))
        if y < 28:
            core.append((7, y))
    canvas.blit(volume(core, 8, 12, palette.gel_shine, 4))
    canvas.fill(ellipse_pixels(8, 8, 2, 3), palette.gel_hot)
    canvas.put(7, 7, palette.white)
    canvas.put(8, 6, palette.gel_hi)
    add_outline(canvas, palette.ink)
    return canvas

def draw_blob(palette: Palette) -> Canvas:
    canvas = clipping_canvas(32, 32)
    blit_ellipse(canvas, 16, 18, 14, 12, palette.gel_under, 7)
    blit_ellipse(canvas, 16, 15, 12, 11, palette.gel_body, 6)
    canvas.blit(volume(ellipse_pixels(16, 21, 6, 5), 16, 22, palette.sage_ramp, 4))
    canvas.fill(ellipse_pixels(16, 19, 4, 4), palette.gel_deep)
    canvas.fill(ellipse_pixels(12, 12, 4, 3), palette.gel_bright)
    canvas.fill(ellipse_pixels(11, 11, 2, 2), palette.gel_hot)
    add_cast_rim(canvas, palette.gel_shadow)
    add_outline(canvas, palette.ink)
    canvas.put(10, 10, palette.white)
    return canvas

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

def save_png(image: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    first = png_bytes(image)
    path.write_bytes(first)
    if png_bytes(Image.open(path).convert("RGBA")) != first:
        raise SystemExit(f"{path.name} did not round-trip")

def png_bytes(image: Image.Image) -> bytes:
    buffer = io.BytesIO()
    image.save(buffer, format="PNG", compress_level=9)
    return buffer.getvalue()

def to_image(canvas: Canvas) -> Image.Image:
    image = Image.new("RGBA", (canvas.width, canvas.height), (0, 0, 0, 0))
    image.putdata(canvas.pixels)
    return image

if __name__ == "__main__":
    main()
