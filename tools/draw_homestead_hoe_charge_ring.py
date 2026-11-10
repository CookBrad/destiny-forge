from __future__ import annotations

import math
from pathlib import Path

from art_common import add_outline, rejecting_canvas, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets" / "homestead" / "vfx"
CELL_SIZE = 32
CENTER = 16.0
INNER_RADIUS = 10.5
OUTER_RADIUS = 13.0
LIGHT_DIAGONAL = 28
DARK_DIAGONAL = 34
CHARGE_LEVELS = (1, 2, 3, 4)
CHARGE_LEVEL_MILLISECONDS = {1: 350, 2: 700, 3: 1050, 4: 1400}
PIP_CENTERS = ((15, 4), (27, 15), (15, 27), (4, 15))

OUTLINE = require_opaque_color((34, 34, 34, 255))
RING_LIGHT = require_opaque_color((244, 214, 140, 255))
RING = require_opaque_color((198, 158, 84, 255))
RING_DARK = require_opaque_color((122, 88, 40, 255))
PIP_LIT_CENTER = require_opaque_color((210, 236, 255, 255))
PIP_LIT = require_opaque_color((57, 214, 255, 255))
PIP_DARK_CENTER = require_opaque_color((32, 22, 18, 255))
PIP_DARK = require_opaque_color((62, 44, 28, 255))


def main() -> None:
    for painted_ring in charge_ring_records():
        save_charge_ring(painted_ring)


def charge_ring_records():
    return [charge_ring_record(level) for level in CHARGE_LEVELS]


def charge_ring_record(level):
    checked_level = require_charge_level(level)
    return (checked_level, seconds_for_charge_level(checked_level), draw_charge_ring(checked_level))


def require_charge_level(level):
    if level not in CHARGE_LEVELS:
        raise ValueError("charge level must be 1, 2, 3, or 4")
    return level


def seconds_for_charge_level(level):
    checked_level = require_charge_level(level)
    return CHARGE_LEVEL_MILLISECONDS[checked_level] / 1000


def draw_charge_ring(level):
    require_charge_level(level)
    canvas = rejecting_canvas(CELL_SIZE, CELL_SIZE)
    paint_ring(canvas)
    paint_pips(canvas, level)
    add_outline(canvas, OUTLINE)
    return canvas.image()


def paint_ring(canvas) -> None:
    for row in range(canvas.height):
        for column in range(canvas.width):
            if ring_contains(column, row):
                canvas.put(column, row, ring_color(column, row))


def ring_contains(column, row) -> bool:
    distance = distance_from_center(column, row)
    return INNER_RADIUS <= distance <= OUTER_RADIUS


def distance_from_center(column, row) -> float:
    return math.hypot(column + 0.5 - CENTER, row + 0.5 - CENTER)


def ring_color(column, row):
    diagonal = column + row
    if diagonal < LIGHT_DIAGONAL:
        return RING_LIGHT
    if diagonal > DARK_DIAGONAL:
        return RING_DARK
    return RING


def paint_pips(canvas, level) -> None:
    require_charge_level(level)
    for index, center in enumerate(PIP_CENTERS):
        paint_pip(canvas, center, pip_color(index, level))


def pip_color(index, level):
    require_charge_level(level)
    if index not in (0, 1, 2, 3):
        raise ValueError("pip index must be 0, 1, 2, or 3")
    if index < level:
        return PIP_LIT
    return PIP_DARK


def paint_pip(canvas, center, color) -> None:
    center_column, center_row = center
    for row_offset in (-1, 0, 1):
        for column_offset in (-1, 0, 1):
            canvas.put(
                center_column + column_offset,
                center_row + row_offset,
                pip_pixel_color(color, column_offset, row_offset),
            )


def pip_pixel_color(color, column_offset, row_offset):
    if column_offset == 0 and row_offset == 0:
        if color == PIP_LIT:
            return PIP_LIT_CENTER
        if color == PIP_DARK:
            return PIP_DARK_CENTER
    if color == PIP_LIT or color == PIP_DARK:
        return color
    raise ValueError("pip color must be lit or dark")


def save_charge_ring(painted_ring) -> None:
    level, seconds, frame = painted_ring
    checked_level = require_charge_level(level)
    if seconds != seconds_for_charge_level(checked_level):
        raise ValueError("charge ring seconds do not match the level")
    FOLDER.mkdir(parents=True, exist_ok=True)
    frame.save(FOLDER / charge_ring_file_name(checked_level))


def charge_ring_file_name(level) -> str:
    checked_level = require_charge_level(level)
    return f"hoe_charge_ring_f{checked_level - 1}.png"


if __name__ == "__main__":
    main()
