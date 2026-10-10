#!/usr/bin/env python3
"""In-house homestead dwarf pickaxe swing frames.

Pixel-authored at 1x on top of the 0x72 dwarf_m idle frame 0. Every colour
comes from the dwarf palette. The 16x28 dwarf sits centred on a 48x60 canvas
so the Bevy sprite centre stays on the dwarf and the pickaxe has room to arc.
Re-running writes the same bytes.

Source: assets/player/non-combat/dwarf_m_idle_anim_f0.png
Outputs: assets/player/non-combat/dwarf_m_pickaxe_swing_f0..3.png
"""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets" / "player" / "non-combat"
DWARF = FOLDER / "dwarf_m_idle_anim_f0.png"

CANVAS_W = 48
CANVAS_H = 60
DWARF_X = 16
DWARF_Y = 16
HEAD_LAST_ROW = 21
HANDLE_LENGTH = 13.0
PICK_HALF_SPAN = 5.0
PICK_SWEEP_BACK = 1.6

OUTLINE = (34, 34, 34, 255)
SKIN = (216, 165, 125, 255)
WOOD = (119, 92, 85, 255)
STEEL = (123, 137, 148, 255)

POSES = [
    {"lean_x": -1, "drop_y": 0, "grip": (20.5, 27.5), "angle": -55.0, "in_front": False},
    {"lean_x": 0, "drop_y": 0, "grip": (24.5, 24.5), "angle": 5.0, "in_front": False},
    {"lean_x": 1, "drop_y": 0, "grip": (31.5, 29.5), "angle": 70.0, "in_front": True},
    {"lean_x": 1, "drop_y": 1, "grip": (33.5, 35.5), "angle": 140.0, "in_front": True},
]


def distance_to_segment(px, py, ax, ay, bx, by):
    abx, aby = bx - ax, by - ay
    length_squared = abx * abx + aby * aby
    t = 0.0 if length_squared == 0 else max(0.0, min(1.0, ((px - ax) * abx + (py - ay) * aby) / length_squared))
    cx, cy = ax + abx * t, ay + aby * t
    return math.hypot(px - cx, py - cy)


def stroke(layer, segments, core_radius, colour):
    for y in range(CANVAS_H):
        for x in range(CANVAS_W):
            cx, cy = x + 0.5, y + 0.5
            if any(distance_to_segment(cx, cy, *a, *b) <= core_radius for a, b in segments):
                layer[(x, y)] = colour


def outline_around(layer):
    outlined = dict(layer)
    for (x, y) in layer:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            neighbour = (x + dx, y + dy)
            if neighbour not in layer and 0 <= neighbour[0] < CANVAS_W and 0 <= neighbour[1] < CANVAS_H:
                outlined[neighbour] = OUTLINE
    return outlined


def pickaxe_pixels(grip, angle_degrees):
    radians = math.radians(angle_degrees)
    along = (math.sin(radians), -math.cos(radians))
    across = (along[1] * -1, along[0])
    gx, gy = grip
    tip = (gx + along[0] * HANDLE_LENGTH, gy + along[1] * HANDLE_LENGTH)
    prong_forward = (
        tip[0] + across[0] * PICK_HALF_SPAN - along[0] * PICK_SWEEP_BACK,
        tip[1] + across[1] * PICK_HALF_SPAN - along[1] * PICK_SWEEP_BACK,
    )
    prong_back = (
        tip[0] - across[0] * PICK_HALF_SPAN - along[0] * PICK_SWEEP_BACK,
        tip[1] - across[1] * PICK_HALF_SPAN - along[1] * PICK_SWEEP_BACK,
    )
    layer = {}
    stroke(layer, [(grip, tip)], 0.55, WOOD)
    stroke(layer, [(prong_back, tip), (tip, prong_forward)], 0.75, STEEL)
    return outline_around(layer)


def fist_pixels(grip):
    gx, gy = int(grip[0]), int(grip[1])
    layer = {(gx + dx, gy + dy): SKIN for dx in (-1, 0) for dy in (0, 1)}
    return outline_around(layer)


def dwarf_pixels(pose):
    dwarf = Image.open(DWARF).convert("RGBA")
    layer = {}
    for y in range(dwarf.height):
        for x in range(dwarf.width):
            if y > HEAD_LAST_ROW:
                colour = dwarf.getpixel((x, y))
                if colour[3]:
                    layer[(DWARF_X + x, DWARF_Y + y)] = colour
    for y in range(HEAD_LAST_ROW + 1):
        for x in range(dwarf.width):
            colour = dwarf.getpixel((x, y))
            if colour[3]:
                layer[(DWARF_X + x + pose["lean_x"], DWARF_Y + y + pose["drop_y"])] = colour
    return layer


def draw_frame(pose):
    image = Image.new("RGBA", (CANVAS_W, CANVAS_H), (0, 0, 0, 0))
    pickaxe = pickaxe_pixels(pose["grip"], pose["angle"])
    layers = [dwarf_pixels(pose), pickaxe] if pose["in_front"] else [pickaxe, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    for layer in layers:
        for position, colour in layer.items():
            image.putpixel(position, colour)
    return image


def main():
    for index, pose in enumerate(POSES):
        draw_frame(pose).save(FOLDER / f"dwarf_m_pickaxe_swing_f{index}.png")


if __name__ == "__main__":
    main()
