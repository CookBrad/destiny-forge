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


def outline_layer(layer: dict[Point, Color], color: Color, width: int, height: int) -> dict[Point, Color]:
    outlined = dict(layer)
    for x, y in layer:
        for column_step, row_step in NEIGHBORS_4:
            neighbor = (x + column_step, y + row_step)
            if neighbor not in layer and 0 <= neighbor[0] < width and 0 <= neighbor[1] < height:
                outlined[neighbor] = color
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


def clipping_canvas(width: int, height: int, fill: Color = CLEAR) -> Canvas:
    return Canvas(width, height, fill)


class Canvas:
    def __init__(self, width: int, height: int, fill: Color = CLEAR) -> None:
        self.width = width
        self.height = height
        self.pixels = [fill] * (width * height)

    def put(self, x: int, y: int, color: Color) -> None:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return
        if color[3] not in (0, 255):
            raise SystemExit("partial alpha")
        self.pixels[y * self.width + x] = color

    def read(self, x: int, y: int) -> Color | None:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return None
        color = self.pixels[y * self.width + x]
        if color[3] == 0:
            return None
        return color

    def opaque_at(self, x: int, y: int) -> bool:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return False
        return self.pixels[y * self.width + x][3] == 255

