from __future__ import annotations

import math
from pathlib import Path

from art_common import rejecting_canvas, require_opaque_color

FOLDER = Path(__file__).resolve().parents[1] / "assets" / "ui" / "action_bar"
FRAME_WIDTH = 498
FRAME_HEIGHT = 78
SLOT_SIZE = 54
SLOT_NUMBERS = (1, 2, 3, 4, 5, 6, 7, 8)
BEVEL_THICKNESS = 6
HIGHLIGHT_THICKNESS = 3
PARCHMENT_SHADOW_THICKNESS = 3
MINIMUM_FRAME_SPAN = 12
MINIMUM_HIGHLIGHT_SPAN = 6
SLOT_NUMBER_LEFT = 6
SLOT_NUMBER_TOP = 6
STROKE_THICKNESS = 3
DIGIT_WIDTH = 15
DIGIT_HEIGHT = 21
MIDDLE_ROW = 9
BOTTOM_ROW = 18
RIGHT_COLUMN = 12
KNOT_RADIUS = 2
GRAIN_LONG_WAVELENGTH = 18
GRAIN_SHORT_WAVELENGTH = 8
GRAIN_LONG_AMPLITUDE = 3.0
GRAIN_SHORT_AMPLITUDE = 1.5
WOOD = require_opaque_color((139, 90, 48, 255))
WOOD_LIGHT = require_opaque_color((186, 132, 78, 255))
WOOD_DARK = require_opaque_color((86, 52, 28, 255))
GRAIN_DARK = require_opaque_color((112, 70, 36, 255))
GRAIN_MID = require_opaque_color((124, 80, 42, 255))
GRAIN_LIGHT = require_opaque_color((154, 102, 56, 255))
KNOT = require_opaque_color((96, 58, 30, 255))
PARCHMENT = require_opaque_color((244, 232, 206, 255))
PARCHMENT_SHADOW = require_opaque_color((214, 196, 164, 255))
HIGHLIGHT = require_opaque_color((220, 78, 36, 255))
INK = require_opaque_color((62, 38, 24, 255))
TOP_BAR = (0, 0, DIGIT_WIDTH, STROKE_THICKNESS)
MIDDLE_BAR = (0, MIDDLE_ROW, DIGIT_WIDTH, MIDDLE_ROW + STROKE_THICKNESS)
BOTTOM_BAR = (0, BOTTOM_ROW, DIGIT_WIDTH, DIGIT_HEIGHT)
UPPER_LEFT_BAR = (0, 0, STROKE_THICKNESS, MIDDLE_ROW + STROKE_THICKNESS)
UPPER_RIGHT_BAR = (RIGHT_COLUMN, 0, DIGIT_WIDTH, MIDDLE_ROW + STROKE_THICKNESS)
LOWER_LEFT_BAR = (0, MIDDLE_ROW, STROKE_THICKNESS, DIGIT_HEIGHT)
LOWER_RIGHT_BAR = (RIGHT_COLUMN, MIDDLE_ROW, DIGIT_WIDTH, DIGIT_HEIGHT)
ONE_STEM = (6, 0, 9, BOTTOM_ROW)
ONE_SERIF = (3, 0, 6, STROKE_THICKNESS)
ONE_BASE = (3, BOTTOM_ROW, RIGHT_COLUMN, DIGIT_HEIGHT)
SLOT_NUMBER_STROKES = {
    1: (ONE_STEM, ONE_SERIF, ONE_BASE),
    2: (TOP_BAR, UPPER_RIGHT_BAR, MIDDLE_BAR, LOWER_LEFT_BAR, BOTTOM_BAR),
    3: (TOP_BAR, UPPER_RIGHT_BAR, MIDDLE_BAR, LOWER_RIGHT_BAR, BOTTOM_BAR),
    4: (UPPER_LEFT_BAR, UPPER_RIGHT_BAR, MIDDLE_BAR, LOWER_RIGHT_BAR),
    5: (TOP_BAR, UPPER_LEFT_BAR, MIDDLE_BAR, LOWER_RIGHT_BAR, BOTTOM_BAR),
    6: (TOP_BAR, UPPER_LEFT_BAR, MIDDLE_BAR, LOWER_LEFT_BAR, LOWER_RIGHT_BAR, BOTTOM_BAR),
    7: (TOP_BAR, UPPER_RIGHT_BAR, LOWER_RIGHT_BAR),
    8: (TOP_BAR, UPPER_LEFT_BAR, UPPER_RIGHT_BAR, MIDDLE_BAR, LOWER_LEFT_BAR, LOWER_RIGHT_BAR, BOTTOM_BAR),
}
GRAIN_LINES = (
    (9, 0.0, GRAIN_DARK),
    (27, 1.3, GRAIN_MID),
    (42, 2.1, GRAIN_DARK),
    (60, 0.7, GRAIN_MID),
    (72, 2.78, GRAIN_DARK),
    (18, 3.5, GRAIN_LIGHT),
    (51, 4.4, GRAIN_LIGHT),
)
KNOT_CENTERS = ((450, 8), (90, 8), (291, 69))


def main() -> None:
    FOLDER.mkdir(parents=True, exist_ok=True)
    for file_name, image in action_bar_images():
        image.save(FOLDER / file_name)


def action_bar_images():
    named_images = [("frame.png", draw_frame()), ("highlight.png", draw_highlight())]
    named_images.extend(slot_images())
    require_unique_action_bar_images(named_images)
    return named_images


def draw_frame():
    require_canvas_can_hold(FRAME_WIDTH, FRAME_HEIGHT, MINIMUM_FRAME_SPAN)
    canvas = rejecting_canvas(FRAME_WIDTH, FRAME_HEIGHT)
    paint_wooden_bevel(canvas)
    return canvas.image()


def require_canvas_can_hold(width, height, minimum_span) -> None:
    if width < minimum_span or height < minimum_span:
        raise ValueError("canvas is too small to draw")


def paint_wooden_bevel(canvas) -> None:
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    paint_wood_grain(canvas)
    canvas.fill_rect(0, 0, canvas.width - 1, BEVEL_THICKNESS, WOOD_LIGHT)
    canvas.fill_rect(0, 0, BEVEL_THICKNESS, canvas.height - 1, WOOD_LIGHT)
    canvas.fill_rect(
        1,
        canvas.height - BEVEL_THICKNESS,
        canvas.width,
        canvas.height,
        WOOD_DARK,
    )
    canvas.fill_rect(
        canvas.width - BEVEL_THICKNESS,
        1,
        canvas.width,
        canvas.height,
        WOOD_DARK,
    )


def paint_wood_grain(canvas) -> None:
    for column, row in KNOT_CENTERS:
        paint_knot(canvas, column, row)
    for base, phase, color in GRAIN_LINES:
        paint_grain_line(canvas, base, phase, color)


def paint_knot(canvas, column, row) -> None:
    for offset in range(1, KNOT_RADIUS + 1):
        paint_on_wood_body(canvas, column + offset, row, GRAIN_DARK)
        paint_on_wood_body(canvas, column - offset, row, GRAIN_DARK)
        paint_on_wood_body(canvas, column, row + offset, GRAIN_DARK)
        paint_on_wood_body(canvas, column, row - offset, GRAIN_DARK)
    paint_on_wood_body(canvas, column, row, KNOT)


def paint_on_wood_body(canvas, column, row, color) -> None:
    if pixel_is_still_on_the_wood_body(canvas, column, row):
        canvas.put(column, row, color)


def pixel_is_still_on_the_wood_body(canvas, column, row) -> bool:
    if column < BEVEL_THICKNESS or column >= canvas.width - BEVEL_THICKNESS:
        return False
    if row < BEVEL_THICKNESS or row >= canvas.height - BEVEL_THICKNESS:
        return False
    return canvas.read(column, row) == WOOD


def paint_grain_line(canvas, base, phase, color) -> None:
    for column in range(canvas.width):
        row = grain_row(column, base, phase)
        paint_on_wood_body(canvas, column, row, color)
        paint_on_wood_body(canvas, column, row + 1, color)


def grain_row(column, base, phase):
    long_wave = GRAIN_LONG_AMPLITUDE * math.sin(column / GRAIN_LONG_WAVELENGTH + phase)
    short_wave = GRAIN_SHORT_AMPLITUDE * math.sin(column / GRAIN_SHORT_WAVELENGTH + 2 * phase)
    return round(base + long_wave + short_wave)


def draw_highlight():
    require_canvas_can_hold(SLOT_SIZE, SLOT_SIZE, MINIMUM_HIGHLIGHT_SPAN)
    canvas = rejecting_canvas(SLOT_SIZE, SLOT_SIZE)
    paint_highlight_border(canvas)
    return canvas.image()


def paint_highlight_border(canvas) -> None:
    canvas.fill_rect(0, 0, canvas.width, HIGHLIGHT_THICKNESS, HIGHLIGHT)
    canvas.fill_rect(
        0,
        canvas.height - HIGHLIGHT_THICKNESS,
        canvas.width,
        canvas.height,
        HIGHLIGHT,
    )
    canvas.fill_rect(0, 0, HIGHLIGHT_THICKNESS, canvas.height, HIGHLIGHT)
    canvas.fill_rect(
        canvas.width - HIGHLIGHT_THICKNESS,
        0,
        canvas.width,
        canvas.height,
        HIGHLIGHT,
    )


def slot_images():
    return [
        (slot_file_name(slot_number), draw_slot(slot_number))
        for slot_number in SLOT_NUMBERS
    ]


def slot_file_name(slot_number) -> str:
    valid_slot_number = require_slot_number(slot_number)
    return f"slot_{valid_slot_number}.png"


def require_slot_number(slot_number):
    if (
        isinstance(slot_number, bool)
        or not isinstance(slot_number, int)
        or slot_number not in SLOT_NUMBERS
        or slot_number not in SLOT_NUMBER_STROKES
    ):
        raise ValueError("action bar slot number must be 1 through 8")
    return slot_number


def draw_slot(slot_number):
    valid_slot_number = require_slot_number(slot_number)
    canvas = rejecting_canvas(SLOT_SIZE, SLOT_SIZE)
    paint_parchment_slot(canvas)
    paint_slot_number(canvas, valid_slot_number)
    return canvas.image()


def paint_parchment_slot(canvas) -> None:
    canvas.fill_rect(0, 0, canvas.width, canvas.height, PARCHMENT)
    canvas.fill_rect(
        0,
        canvas.height - PARCHMENT_SHADOW_THICKNESS,
        canvas.width,
        canvas.height,
        PARCHMENT_SHADOW,
    )
    canvas.fill_rect(
        canvas.width - PARCHMENT_SHADOW_THICKNESS,
        0,
        canvas.width,
        canvas.height,
        PARCHMENT_SHADOW,
    )


def paint_slot_number(canvas, slot_number) -> None:
    valid_slot_number = require_slot_number(slot_number)
    for left, top, right, bottom in SLOT_NUMBER_STROKES[valid_slot_number]:
        canvas.fill_rect(
            SLOT_NUMBER_LEFT + left,
            SLOT_NUMBER_TOP + top,
            SLOT_NUMBER_LEFT + right,
            SLOT_NUMBER_TOP + bottom,
            INK,
        )


def require_unique_action_bar_images(named_images) -> None:
    image_bytes = [image.tobytes() for _file_name, image in named_images]
    if len(image_bytes) != len(set(image_bytes)):
        raise ValueError("action bar images are not unique")


if __name__ == "__main__":
    main()
