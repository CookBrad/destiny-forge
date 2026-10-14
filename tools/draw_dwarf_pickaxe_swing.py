#!/usr/bin/env python3


from __future__ import annotations

import math
from pathlib import Path

from PIL import Image

from art_common import Palette, composite_layers, contact_sheet, outline_layer

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

_tones = Palette()
OUTLINE = _tones.add("outline", (34, 34, 34, 255))
SKIN = _tones.add("skin", (216, 165, 125, 255))
WOOD = _tones.add("wood", (119, 92, 85, 255))
STEEL = _tones.add("steel", (123, 137, 148, 255))

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
    return outline_layer(layer, OUTLINE, CANVAS_W, CANVAS_H)


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
    pickaxe = pickaxe_pixels(pose["grip"], pose["angle"])
    layers = [dwarf_pixels(pose), pickaxe] if pose["in_front"] else [pickaxe, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    return composite_layers(CANVAS_W, CANVAS_H, layers)


def main():
    frames = [draw_frame(pose) for pose in POSES]
    contact_sheet(frames, len(frames), gutter=0, margin=0)
    for index, frame in enumerate(frames):
        frame.save(FOLDER / f"dwarf_m_pickaxe_swing_f{index}.png")


if __name__ == "__main__":
    main()
