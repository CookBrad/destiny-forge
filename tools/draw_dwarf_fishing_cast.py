from __future__ import annotations

import math
from pathlib import Path

from PIL import Image

from art_common import composite_layers, outline_layer, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets" / "player" / "non-combat"
DWARF = FOLDER / "dwarf_m_idle_anim_f0.png"

CANVAS_WIDTH = 64
CANVAS_HEIGHT = 60
DWARF_X = 16
DWARF_Y = 16
HEAD_LAST_ROW = 21
HANDLE_LENGTH = 22.0
TIP_LENGTH = 3.0

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
BAMBOO = require_opaque_color((166, 140, 72, 255))
BAMBOO_SHADE = require_opaque_color((128, 102, 48, 255))
LINE = require_opaque_color((210, 210, 200, 255))

CAST_CHARGE = [
    {"lean_x": -1, "drop_y": 0, "grip": (22.5, 27.5), "angle": -70.0, "in_front": False, "line": False},
    {"lean_x": -1, "drop_y": 0, "grip": (21.5, 26.5), "angle": -95.0, "in_front": False, "line": False},
    {"lean_x": -2, "drop_y": 0, "grip": (20.5, 25.5), "angle": -110.0, "in_front": False, "line": False},
]

CAST_RELEASE = [
    {"lean_x": -1, "drop_y": 0, "grip": (22.5, 26.5), "angle": -80.0, "in_front": False, "line": False},
    {"lean_x": 0, "drop_y": 0, "grip": (26.5, 27.5), "angle": -10.0, "in_front": False, "line": True},
    {"lean_x": 1, "drop_y": 0, "grip": (30.5, 29.5), "angle": 35.0, "in_front": True, "line": True},
    {"lean_x": 1, "drop_y": 0, "grip": (32.5, 31.5), "angle": 55.0, "in_front": True, "line": True},
]

WAIT_IDLE = [
    {"lean_x": 0, "drop_y": 0, "grip": (31.5, 31.0), "angle": 50.0, "in_front": True, "line": True, "tip_bob": 0.0},
    {"lean_x": 0, "drop_y": 0, "grip": (31.5, 31.0), "angle": 52.0, "in_front": True, "line": True, "tip_bob": 1.0},
    {"lean_x": 0, "drop_y": 1, "grip": (31.5, 32.0), "angle": 54.0, "in_front": True, "line": True, "tip_bob": 0.0},
    {"lean_x": 0, "drop_y": 0, "grip": (31.5, 31.0), "angle": 51.0, "in_front": True, "line": True, "tip_bob": -1.0},
]

SETS = (
    ("dwarf_m_rod_bamboo_cast_charge", CAST_CHARGE),
    ("dwarf_m_rod_bamboo_cast_release", CAST_RELEASE),
    ("dwarf_m_rod_bamboo_wait", WAIT_IDLE),
)


def main():
    for stem, poses in SETS:
        for index, pose in enumerate(poses):
            draw_frame(pose).save(FOLDER / f"{stem}_f{index}.png")


def draw_frame(pose):
    rod = rod_pixels(pose)
    layers = [dwarf_pixels(pose), rod] if pose["in_front"] else [rod, dwarf_pixels(pose)]
    layers.append(fist_pixels(pose["grip"]))
    return composite_layers(CANVAS_WIDTH, CANVAS_HEIGHT, layers)


def rod_pixels(pose):
    grip_x, grip_y = pose["grip"]
    radians = math.radians(pose["angle"])
    along = (math.sin(radians), -math.cos(radians))
    tip_bob = pose.get("tip_bob", 0.0)
    tip = (
        grip_x + along[0] * HANDLE_LENGTH,
        grip_y + along[1] * HANDLE_LENGTH + tip_bob,
    )
    whip = (
        tip[0] + along[0] * TIP_LENGTH,
        tip[1] + along[1] * TIP_LENGTH + tip_bob * 0.5,
    )
    layer = {}
    stroke(layer, [(pose["grip"], tip)], 0.7, BAMBOO)
    stroke(layer, [(tip, whip)], 0.45, BAMBOO_SHADE)
    if pose.get("line"):
        line_end = (whip[0] + 6.0, whip[1] + 10.0)
        stroke(layer, [(whip, line_end)], 0.35, LINE)
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
                ((point_x - segment_start_x) * span_x + (point_y - segment_start_y) * span_y)
                / length_squared,
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
