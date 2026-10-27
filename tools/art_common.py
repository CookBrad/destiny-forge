from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path

from PIL import Image

Color = tuple[int, int, int, int]
Point = tuple[int, int]
CLEAR: Color = (0, 0, 0, 0)
NEIGHBORS_8 = tuple(
    (column_offset, row_offset)
    for row_offset in (-1, 0, 1)
    for column_offset in (-1, 0, 1)
    if column_offset or row_offset
)
NEIGHBORS_4 = ((1, 0), (-1, 0), (0, 1), (0, -1))


def pack_strip(frames: list, cell_width: int, cell_height: int) -> Image.Image:
    image = Image.new("RGBA", (cell_width * len(frames), cell_height), CLEAR)
    pixels = image.load()
    width, height = image.size
    for index, frame in enumerate(frames):
        origin_x = index * cell_width
        for (x, y), color in _frame_pixels(frame).items():
            pixel_x, pixel_y = x + origin_x, y
            if not (0 <= pixel_x < width and 0 <= pixel_y < height):
                raise SystemExit(f"blit clipped {pixel_x},{pixel_y}")
            pixels[pixel_x, pixel_y] = color
    return image


def _frame_pixels(frame: dict[Point, Color]) -> dict[Point, Color]:
    if isinstance(frame, dict):
        return frame
    return frame.pixels


def composite_layers(width: int, height: int, layers: list[dict[Point, Color]]) -> Image.Image:
    image = Image.new("RGBA", (width, height), CLEAR)
    for layer in layers:
        for position, color in layer.items():
            image.putpixel(position, color)
    return image


def composite_images(size: tuple[int, int], layers: list[tuple[Image.Image, Point]]) -> Image.Image:
    image = Image.new("RGBA", size, CLEAR)
    for layer, origin in layers:
        image.alpha_composite(layer, origin)
    return image


def add_outline(canvas: Canvas, color: Color) -> None:
    width = canvas.width
    height = canvas.height
    extra: list[Point] = []
    for y in range(height):
        row = y * width
        for x in range(width):
            if canvas.pixels[row + x][3] == 0:
                continue
            for column_step, row_step in NEIGHBORS_8:
                neighbor_x = x + column_step
                neighbor_y = y + row_step
                if (
                    0 <= neighbor_x < width
                    and 0 <= neighbor_y < height
                    and canvas.pixels[neighbor_y * width + neighbor_x][3] == 0
                ):
                    extra.append((neighbor_x, neighbor_y))
    for x, y in extra:
        canvas.put(x, y, color)


def add_cast_rim(canvas: Canvas, color: Color) -> None:
    width = canvas.width
    height = canvas.height
    rim: list[Point] = []
    for y in range(height):
        row = y * width
        for x in range(width):
            if canvas.pixels[row + x][3] == 0:
                continue
            for column_step, row_step in ((1, 0), (0, 1), (1, 1)):
                neighbor_x = x + column_step
                neighbor_y = y + row_step
                if (
                    not (0 <= neighbor_x < width and 0 <= neighbor_y < height)
                    or canvas.pixels[neighbor_y * width + neighbor_x][3] == 0
                ):
                    rim.append((x, y))
                    break
    for x, y in rim:
        canvas.put(x, y, color)


def light_contour(canvas: Canvas, outline: Color, lit: Color) -> None:
    caught: list[Point] = []
    for y in range(canvas.height):
        for x in range(canvas.width):
            if canvas.read(x, y) != outline:
                continue
            above_empty = canvas.read(x, y - 1) is None
            left_empty = canvas.read(x - 1, y) is None
            below_empty = canvas.read(x, y + 1) is None
            if above_empty or (left_empty and not below_empty):
                caught.append((x, y))
    for x, y in caught:
        canvas.put(x, y, lit)


def outline_layer(layer: dict[Point, Color], color: Color, width: int, height: int) -> dict[Point, Color]:
    outlined = dict(layer)
    for x, y in layer:
        for column_step, row_step in NEIGHBORS_4:
            neighbour = (x + column_step, y + row_step)
            if neighbour not in layer and 0 <= neighbour[0] < width and 0 <= neighbour[1] < height:
                outlined[neighbour] = color
    return outlined


def paste_pixels(
    target: dict[Point, Color],
    layer: dict[Point, Color],
    offset_x: int = 0,
    offset_y: int = 0,
) -> None:
    for (x, y), color in layer.items():
        target[(x + offset_x, y + offset_y)] = color


def translate_pixels(pixels: dict[Point, Color], offset_x: int, offset_y: int) -> dict[Point, Color]:
    if offset_x == 0 and offset_y == 0:
        return dict(pixels)
    return {(x + offset_x, y + offset_y): color for (x, y), color in pixels.items()}

