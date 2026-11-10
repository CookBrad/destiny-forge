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
HANDLE_LENGTH = 14.0
BLADE_LENGTH = 7.6
BLADE_WIDTH = 1.4

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
WOOD = require_opaque_color((119, 92, 85, 255))
BRASS = require_opaque_color((204, 164, 72, 255))
BRASS_EDGE = require_opaque_color((148, 112, 36, 255))
BRASS_HIGHLIGHT = require_opaque_color((236, 208, 120, 255))

POSES = [
    {"lean_x": 0, "drop_y": 0, "grip": (22.5, 26.5), "angle": -25.0, "in_front": False},
    {"lean_x": -1, "drop_y": -1, "grip": (20.5, 22.5), "angle": -70.0, "in_front": False},
    {"lean_x": -1, "drop_y": -1, "grip": (19.5, 18.5), "angle": -110.0, "in_front": False},
]


def main():
    frames = [draw_frame(pose) for pose in POSES]
    for index, frame in enumerate(frames):
        frame.save(FOLDER / f"dwarf_m_hoe_charge_t4_f{index}.png")


def draw_frame(pose):
    hoe = hoe_pixels(pose["grip"], pose["angle"])
    layers = [dwarf_pixels(pose), hoe] if pose["in_front"] else [hoe, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    return composite_layers(CANVAS_WIDTH, CANVAS_HEIGHT, layers)


def hoe_pixels(grip, angle_degrees):
    radians = math.radians(angle_degrees)
    handle_direction = (math.sin(radians), -math.cos(radians))
    blade_direction = (handle_direction[1] * -1, handle_direction[0])
    grip_x, grip_y = grip
    neck = (
        grip_x + handle_direction[0] * HANDLE_LENGTH,
        grip_y + handle_direction[1] * HANDLE_LENGTH,
    )
    blade_tip = (
        neck[0] + blade_direction[0] * BLADE_LENGTH,
        neck[1] + blade_direction[1] * BLADE_LENGTH,
    )
    blade_back = (
        neck[0] - blade_direction[0] * 1.2,
        neck[1] - blade_direction[1] * 1.2,
    )
    cutting_edge = (
        blade_tip[0] + handle_direction[0] * 1.4,
        blade_tip[1] + handle_direction[1] * 1.4,
    )
    layer = {}
    stroke(layer, [(grip, neck)], 0.55, WOOD)
    stroke(layer, [(blade_back, neck), (neck, blade_tip)], BLADE_WIDTH, BRASS)
    stroke(layer, [(blade_tip, cutting_edge)], 0.75, BRASS_EDGE)
    stroke(layer, [(blade_back, blade_tip)], 0.4, BRASS_HIGHLIGHT)
    place_charge_level_mark(layer, blade_tip)
    return outline_around(layer)


def stroke(layer, segments, core_radius, color):
    for y in range(CANVAS_HEIGHT):
        for x in range(CANVAS_WIDTH):
            center_x, center_y = x + 0.5, y + 0.5
            if any(
                distance_to_segment(center_x, center_y, *segment_start, *segment_end) <= core_radius
                for segment_start, segment_end in segments
            ):
                layer[(x, y)] = color


def distance_to_segment(point_x, point_y, segment_start_x, segment_start_y, segment_end_x, segment_end_y):
    segment_span_x = segment_end_x - segment_start_x
    segment_span_y = segment_end_y - segment_start_y
    length_squared = segment_span_x * segment_span_x + segment_span_y * segment_span_y
    if length_squared == 0:
        fraction = 0.0
    else:
        fraction = max(
            0.0,
            min(
                1.0,
                (
                    (point_x - segment_start_x) * segment_span_x
                    + (point_y - segment_start_y) * segment_span_y
                )
                / length_squared,
            ),
        )
    closest_x = segment_start_x + segment_span_x * fraction
    closest_y = segment_start_y + segment_span_y * fraction
    return math.hypot(point_x - closest_x, point_y - closest_y)


def place_charge_level_mark(layer, blade_tip):
    mark_x = round(blade_tip[0])
    mark_y = round(blade_tip[1] - 2)
    layer[(mark_x, mark_y)] = BRASS_HIGHLIGHT
    layer[(mark_x, mark_y - 1)] = BRASS_EDGE


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
    layer = {
        (grip_x + column_offset, grip_y + row_offset): SKIN
        for column_offset in (-1, 0)
        for row_offset in (0, 1)
    }
    return outline_around(layer)


if __name__ == "__main__":
    main()
