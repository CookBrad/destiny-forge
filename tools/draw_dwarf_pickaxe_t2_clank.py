from __future__ import annotations

import math
from pathlib import Path

from PIL import Image

from art_common import composite_layers, outline_layer, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets" / "player" / "non-combat"
DWARF = FOLDER / "dwarf_m_idle_anim_f0.png"

CANVAS_WIDTH = 48
CANVAS_HEIGHT = 60
DWARF_X = 16
DWARF_Y = 16
HEAD_LAST_ROW = 21
HANDLE_LENGTH = 13.0
PICK_HALF_SPAN = 6.2
PICK_SWEEP_BACK = 2.0

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
WOOD = require_opaque_color((119, 92, 85, 255))
IRON = require_opaque_color((132, 140, 148, 255))
IRON_EDGE = require_opaque_color((72, 78, 86, 255))
IRON_HIGHLIGHT = require_opaque_color((180, 188, 196, 255))

POSES = [
    {"lean_x": 1, "drop_y": 0, "grip": (30.5, 30.5), "angle": 75.0, "in_front": True},
    {"lean_x": -1, "drop_y": 0, "grip": (18.5, 28.5), "angle": -40.0, "in_front": False},
    {"lean_x": 0, "drop_y": 1, "grip": (21.5, 29.5), "angle": -20.0, "in_front": False},
]

def main():
    frames = [draw_frame(pose) for pose in POSES]
    for index, frame in enumerate(frames):
        frame.save(FOLDER / f"dwarf_m_pickaxe_t2_clank_f{index}.png")

def draw_frame(pose):
    pickaxe = pickaxe_pixels(pose["grip"], pose["angle"])
    layers = [dwarf_pixels(pose), pickaxe] if pose["in_front"] else [pickaxe, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    return composite_layers(CANVAS_WIDTH, CANVAS_HEIGHT, layers)

def pickaxe_pixels(grip, angle_degrees):
    radians = math.radians(angle_degrees)
    along = (math.sin(radians), -math.cos(radians))
    across = (along[1] * -1, along[0])
    grip_x, grip_y = grip
    tip = (grip_x + along[0] * HANDLE_LENGTH, grip_y + along[1] * HANDLE_LENGTH)
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
    stroke(layer, [(prong_back, tip), (tip, prong_forward)], 0.9, IRON)
    stroke(layer, [(prong_back, prong_forward)], 0.4, IRON_EDGE)
    stroke(layer, [(tip, (tip[0] + along[0] * 1.2, tip[1] + along[1] * 1.2))], 0.45, IRON_HIGHLIGHT)
    return outline_around(layer)

def stroke(layer, segments, core_radius, color):
    for y in range(CANVAS_HEIGHT):
        for x in range(CANVAS_WIDTH):
            center_x, center_y = x + 0.5, y + 0.5
            if any(distance_to_segment(center_x, center_y, *segment_start, *segment_end) <= core_radius for segment_start, segment_end in segments):
                layer[(x, y)] = color

def distance_to_segment(point_x, point_y, segment_start_x, segment_start_y, segment_end_x, segment_end_y):
    segment_span_x, segment_span_y = segment_end_x - segment_start_x, segment_end_y - segment_start_y
    length_squared = segment_span_x * segment_span_x + segment_span_y * segment_span_y
    fraction = 0.0 if length_squared == 0 else max(0.0, min(1.0, ((point_x - segment_start_x) * segment_span_x + (point_y - segment_start_y) * segment_span_y) / length_squared))
    closest_x, closest_y = segment_start_x + segment_span_x * fraction, segment_start_y + segment_span_y * fraction
    return math.hypot(point_x - closest_x, point_y - closest_y)

def outline_around(layer):
    return outline_layer(layer, OUTLINE, CANVAS_WIDTH, CANVAS_HEIGHT)

def dwarf_pixels(pose):
    dwarf = Image.open(DWARF).convert("RGBA")
    layer = {}
    for y in range(dwarf.height):
        for x in range(dwarf.width):
            if y > HEAD_LAST_ROW:
                color = dwarf.getpixel((x, y))
                if color[3]:
                    layer[(DWARF_X + x, DWARF_Y + y)] = color
    for y in range(HEAD_LAST_ROW + 1):
        for x in range(dwarf.width):
            color = dwarf.getpixel((x, y))
            if color[3]:
                layer[(DWARF_X + x + pose["lean_x"], DWARF_Y + y + pose["drop_y"])] = color
    return layer

def fist_pixels(grip):
    grip_x, grip_y = int(grip[0]), int(grip[1])
    layer = {(grip_x + delta_x, grip_y + delta_y): SKIN for delta_x in (-1, 0) for delta_y in (0, 1)}
    return outline_around(layer)

if __name__ == "__main__":
    main()
