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
BLADE_LENGTH = 6.0
BLADE_WIDTH = 1.1

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
WOOD = require_opaque_color((119, 92, 85, 255))
RUST = require_opaque_color((138, 96, 62, 255))
RUST_EDGE = require_opaque_color((98, 68, 44, 255))

POSES = [
    {"lean_x": -1, "drop_y": 0, "grip": (21.5, 28.5), "angle": -40.0, "in_front": False},
    {"lean_x": 0, "drop_y": 0, "grip": (25.5, 26.5), "angle": 15.0, "in_front": False},
    {"lean_x": 1, "drop_y": 1, "grip": (30.5, 32.5), "angle": 95.0, "in_front": True},
    {"lean_x": 1, "drop_y": 2, "grip": (31.5, 38.5), "angle": 145.0, "in_front": True},
]


def main():
    frames = [draw_frame(pose) for pose in POSES]
    for index, frame in enumerate(frames):
        frame.save(FOLDER / f"dwarf_m_hoe_t1_f{index}.png")


def draw_frame(pose):
    hoe = hoe_pixels(pose["grip"], pose["angle"])
    layers = [dwarf_pixels(pose), hoe] if pose["in_front"] else [hoe, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    return composite_layers(CANVAS_WIDTH, CANVAS_HEIGHT, layers)


def hoe_pixels(grip, angle_degrees):
    radians = math.radians(angle_degrees)
    along = (math.sin(radians), -math.cos(radians))
    across = (along[1] * -1, along[0])
    grip_x, grip_y = grip
    neck = (
        grip_x + along[0] * HANDLE_LENGTH,
        grip_y + along[1] * HANDLE_LENGTH,
    )
    blade_tip = (
        neck[0] + across[0] * BLADE_LENGTH,
        neck[1] + across[1] * BLADE_LENGTH,
    )
    blade_back = (
        neck[0] - across[0] * 1.2,
        neck[1] - across[1] * 1.2,
    )
    layer = {}
    stroke(layer, [(grip, neck)], 0.55, WOOD)
    stroke(layer, [(blade_back, neck), (neck, blade_tip)], BLADE_WIDTH, RUST)
    stroke(layer, [(blade_tip, (blade_tip[0] + along[0] * 1.4, blade_tip[1] + along[1] * 1.4))], 0.7, RUST_EDGE)
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
