from __future__ import annotations

from pathlib import Path

from art_common import rejecting_canvas, require_opaque_color

FOLDER = Path(__file__).resolve().parents[1] / "assets" / "ui" / "action_bar"
FRAME_WIDTH = 166
FRAME_HEIGHT = 26
SLOT_SIZE = 18
SLOT_NUMBERS = (1, 2, 3, 4, 5, 6, 7, 8)
BEVEL_THICKNESS = 2
HIGHLIGHT_THICKNESS = 1
PARCHMENT_SHADOW_THICKNESS = 1
MINIMUM_FRAME_SPAN = 4
MINIMUM_HIGHLIGHT_SPAN = 2
SLOT_NUMBER_LEFT = 2
SLOT_NUMBER_TOP = 2
INK_MARK = "#"
WOOD = require_opaque_color((139, 90, 48, 255))
WOOD_LIGHT = require_opaque_color((186, 132, 78, 255))
WOOD_DARK = require_opaque_color((86, 52, 28, 255))
PARCHMENT = require_opaque_color((244, 232, 206, 255))
PARCHMENT_SHADOW = require_opaque_color((214, 196, 164, 255))
HIGHLIGHT = require_opaque_color((220, 78, 36, 255))
INK = require_opaque_color((62, 38, 24, 255))
SLOT_NUMBER_ROWS = {
    1: (".#...", "##...", ".#...", ".#...", ".#...", ".#...", ".###."),
    2: (".###.", "#...#", "....#", "..##.", ".#...", "#....", ".###."),
    3: (".###.", "#...#", "....#", "..##.", "....#", "#...#", ".###."),
    4: ("#...#", "#...#", "#...#", ".###.", "....#", "....#", "....#"),
    5: (".###.", "#....", "#....", ".###.", "....#", "....#", ".###."),
    6: (".##..", "#....", "#....", ".###.", "#...#", "#...#", ".###."),
    7: (".###.", "#...#", "....#", "...#.", "..#..", ".#...", ".#..."),
    8: (".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."),
}


def main() -> None:
    FOLDER.mkdir(parents=True, exist_ok=True)
    for file_name, image in action_bar_images():
        image.save(FOLDER / file_name)


def action_bar_images():
    named_images = [("frame.png", draw_frame()), ("highlight.png", draw_highlight())]
    named_images.extend(slot_images())
    require_unique_action_bar_images(named_images)
    return named_images


def slot_images():
    return [
        (slot_file_name(slot_number), draw_slot(slot_number))
        for slot_number in SLOT_NUMBERS
    ]


def slot_file_name(slot_number) -> str:
    valid_slot_number = require_slot_number(slot_number)
    return f"slot_{valid_slot_number}.png"


def require_unique_action_bar_images(named_images) -> None:
    image_bytes = [image.tobytes() for _file_name, image in named_images]
    if len(image_bytes) != len(set(image_bytes)):
        raise ValueError("action bar images are not unique")


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


def draw_slot(slot_number):
    valid_slot_number = require_slot_number(slot_number)
    canvas = rejecting_canvas(SLOT_SIZE, SLOT_SIZE)
    paint_parchment_slot(canvas)
    paint_slot_number(canvas, valid_slot_number)
    return canvas.image()


def require_slot_number(slot_number):
    if (
        isinstance(slot_number, bool)
        or not isinstance(slot_number, int)
        or slot_number not in SLOT_NUMBERS
        or slot_number not in SLOT_NUMBER_ROWS
    ):
        raise ValueError("action bar slot number must be 1 through 8")
    return slot_number


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
    for column, row in slot_number_ink(slot_number):
        canvas.put(SLOT_NUMBER_LEFT + column, SLOT_NUMBER_TOP + row, INK)


def slot_number_ink(slot_number):
    valid_slot_number = require_slot_number(slot_number)
    pixels = []
    for row_index, pattern in enumerate(SLOT_NUMBER_ROWS[valid_slot_number]):
        pixels.extend(ink_pixels_in_row(row_index, pattern))
    return pixels


def ink_pixels_in_row(row_index, pattern):
    return [
        (column_index, row_index)
        for column_index, mark in enumerate(pattern)
        if mark == INK_MARK
    ]


if __name__ == "__main__":
    main()
