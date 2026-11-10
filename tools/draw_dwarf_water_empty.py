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
HANDLE_LENGTH = 8.0
CAN_RADIUS = 3.2
SPOUT_LENGTH = 5.5

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
WOOD = require_opaque_color((119, 92, 85, 255))
TIN = require_opaque_color((148, 152, 156, 255))
TIN_EDGE = require_opaque_color((96, 100, 106, 255))

POSES = [
    {"lean_x": -1, "drop_y": 0, "grip": (21.5, 28.5), "angle": -35.0, "in_front": True},
    {"lean_x": 1, "drop_y": 0, "grip": (25.5, 28.5), "angle": 35.0, "in_front": True},
    {"lean_x": -1, "drop_y": 0, "grip": (22.5, 29.5), "angle": -25.0, "in_front": True},
]


def main():
    for index, pose in enumerate(POSES):
        draw_frame(pose).save(FOLDER / f"dwarf_m_water_empty_f{index}.png")


def draw_frame(pose):
    can = can_pixels(pose["grip"], pose["angle"])
    layers = [dwarf_pixels(pose), can] if pose["in_front"] else [can, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    return composite_layers(CANVAS_WIDTH, CANVAS_HEIGHT, layers)


def can_pixels(grip, angle_degrees):
    radians = math.radians(angle_degrees)
    along = (math.sin(radians), -math.cos(radians))
    across = (along[1] * -1, along[0])
    grip_x, grip_y = grip
    body = (
        grip_x + along[0] * HANDLE_LENGTH,
        grip_y + along[1] * HANDLE_LENGTH,
    )
    spout = (
        body[0] + across[0] * SPOUT_LENGTH,
        body[1] + across[1] * SPOUT_LENGTH,
    )
    layer = {}
    stroke(layer, [(grip, body)], 0.55, WOOD)
    stroke(layer, [(body, spout)], 0.7, TIN)
    for offset in (-1.2, 0.0, 1.2):
        ring_start = (body[0] + across[0] * offset - along[0] * CAN_RADIUS, body[1] + across[1] * offset - along[1] * CAN_RADIUS)
        ring_end = (body[0] + across[0] * offset + along[0] * CAN_RADIUS, body[1] + across[1] * offset + along[1] * CAN_RADIUS)
        stroke(layer, [(ring_start, ring_end)], 1.1, TIN)
    stroke(layer, [(body, (body[0] - along[0] * CAN_RADIUS, body[1] - along[1] * CAN_RADIUS))], 0.9, TIN_EDGE)
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
    span_x = segment_end_x - segment_start_x
    span_y = segment_end_y - segment_start_y
    length_squared = span_x * span_x + span_y * span_y
    if length_squared == 0:
        fraction = 0.0
    else:
        fraction = max(
            0.0,
            min(
                1.0,
                ((point_x - segment_start_x) * span_x + (point_y - segment_start_y) * span_y) / length_squared,
            ),
        )
    closest_x = segment_start_x + span_x * fraction
    closest_y = segment_start_y + span_y * fraction
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
    layer = {
        (grip_x + delta_x, grip_y + delta_y): SKIN
        for delta_x in (-1, 0)
        for delta_y in (0, 1)
    }
    return outline_around(layer)


if __name__ == "__main__":
    main()
