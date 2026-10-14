from __future__ import annotations

import hashlib
import math
from pathlib import Path

from PIL import Image

Color = tuple[int, int, int, int]
Point = tuple[int, int]
CLEAR: Color = (0, 0, 0, 0)
NEIGHBORS_8 = tuple((dx, dy) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dx or dy)
NEIGHBORS_4 = ((1, 0), (-1, 0), (0, 1), (0, -1))


def iround(value: float) -> int:
    return int(math.floor(value + 0.5))


def reject_resample_names(source: Path) -> None:
    text = source.read_text(encoding="utf-8")
    banned = (
        "res" + "ize",
        "thumb" + "nail",
        "trans" + "form",
        "aff" + "ine",
        "quant" + "ize",
    )
    for word in banned:
        if word in text:
            raise SystemExit(f"draw script must not mention {word}")


def require_sha256(path: Path, digest: str) -> None:
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != digest:
        raise SystemExit(f"{path.name} hash {actual} != pinned source")


def load_pinned_colors(folder: Path, digests: dict[str, str]) -> set[Color]:
    allowed: set[Color] = set()
    for name, digest in digests.items():
        path = folder / name
        require_sha256(path, digest)
        image = Image.open(path).convert("RGBA")
        for red, green, blue, alpha in image.getdata():
            if alpha == 0:
                continue
            if alpha != 255:
                raise SystemExit(f"partial alpha in {name}")
            allowed.add((red, green, blue, 255))
    return allowed


class Palette:
    def add(self, name: str, color: Color, allowed: set[Color] | None = None) -> Color:
        if color[3] not in (0, 255):
            raise SystemExit("partial alpha")
        if allowed is not None and color not in allowed:
            raise SystemExit(f"color {color} is not in the signed hunter")
        setattr(self, name, color)
        return color


def hunter_palette(allowed: set[Color]) -> Palette:
    palette = Palette()

    def take(red: int, green: int, blue: int) -> Color:
        return palette.add(f"_{red}_{green}_{blue}", (red, green, blue, 255), allowed)

    palette.ink = take(0, 0, 0)
    palette.ink_warm = take(32, 23, 21)
    palette.leather_shadow = take(36, 21, 13)
    palette.leather_rim = take(42, 23, 15)
    palette.leather_deep = take(43, 23, 15)
    palette.leather_soot = take(54, 30, 20)
    palette.leather_dk = take(55, 31, 21)
    palette.leather_md = take(63, 35, 23)
    palette.leather_mid2 = take(67, 38, 25)
    palette.leather = take(74, 41, 26)
    palette.leather_fold = take(76, 43, 28)
    palette.leather_lt = take(97, 55, 35)
    palette.leather_lt2 = take(98, 56, 35)
    palette.leather_hi = take(107, 64, 33)
    palette.skin_dk = take(149, 92, 47)
    palette.skin_md = take(187, 119, 69)
    palette.skin = take(222, 152, 98)
    palette.skin_warm = take(217, 150, 99)
    palette.skin_lt = take(252, 187, 127)
    palette.brass_shadow = take(161, 103, 26)
    palette.brass_md = take(167, 108, 29)
    palette.brass_dk = take(203, 141, 45)
    palette.brass = take(226, 166, 64)
    palette.brass_lt = take(244, 172, 13)
    palette.brass_hi = take(251, 214, 119)
    palette.gel_shadow = take(3, 40, 43)
    palette.gel_deep = take(0, 64, 69)
    palette.gel_well = take(0, 76, 81)
    palette.gel_pool = take(0, 97, 106)
    palette.gel_tide = take(2, 118, 129)
    palette.gel_sea = take(4, 126, 133)
    palette.gel_foam = take(40, 137, 138)
    palette.gel_mid = take(42, 140, 147)
    palette.gel_kelp = take(39, 150, 147)
    palette.gel_lagoon = take(42, 161, 166)
    palette.gel_dk = take(45, 170, 152)
    palette.gel = take(3, 190, 174)
    palette.gel_core = take(0, 210, 211)
    palette.gel_lt = take(0, 213, 215)
    palette.gel_bright = take(0, 229, 227)
    palette.gel_hi = take(0, 251, 250)
    palette.gel_hot = take(166, 253, 254)
    palette.sage_shadow = take(7, 35, 33)
    palette.sage_mid = take(28, 103, 99)
    palette.sage_dk = take(67, 94, 77)
    palette.sage = take(80, 127, 100)
    palette.sage_lt = take(84, 122, 95)
    palette.purple_dk = take(108, 21, 120)
    palette.purple = take(184, 118, 200)
    palette.purple_lt = take(244, 136, 254)
    palette.steel_deep = take(13, 13, 13)
    palette.steel_void = take(16, 16, 16)
    palette.steel_shadow = take(22, 22, 26)
    palette.steel_dk = take(38, 38, 43)
    palette.steel = take(48, 48, 53)
    palette.steel_cloth = take(37, 36, 37)
    palette.steel_mid = take(135, 133, 137)
    palette.steel_lt = take(168, 168, 169)
    palette.bone_shadow = take(126, 126, 116)
    palette.bone_dust = take(148, 154, 143)
    palette.bone_dk = take(150, 149, 144)
    palette.bone_md = take(174, 172, 175)
    palette.bone_mid = take(179, 178, 182)
    palette.bone = take(245, 244, 243)
    palette.bone_lt = take(255, 255, 255)
    palette.dead_dk = take(52, 36, 31)
    palette.dead = take(54, 37, 32)
    palette.white = take(255, 255, 255)
    palette.gel_ramp = (
        palette.gel_shadow,
        palette.gel_deep,
        palette.gel_well,
        palette.gel_pool,
        palette.gel_tide,
        palette.gel_sea,
        palette.gel_foam,
        palette.gel_mid,
        palette.gel_kelp,
        palette.gel_lagoon,
        palette.gel_dk,
        palette.gel,
        palette.gel_core,
        palette.gel_lt,
        palette.gel_bright,
        palette.gel_hi,
    )
    palette.gel_under = palette.gel_ramp[:8]
    palette.gel_body = palette.gel_ramp[4:13]
    palette.gel_shine = palette.gel_ramp[10:]
    palette.sage_ramp = (
        palette.sage_shadow,
        palette.sage_dk,
        palette.sage_mid,
        palette.sage,
        palette.sage_lt,
    )
    palette.leather_ramp = (
        palette.leather_shadow,
        palette.leather_deep,
        palette.leather_soot,
        palette.leather_dk,
        palette.leather_md,
        palette.leather_mid2,
        palette.leather,
        palette.leather_fold,
        palette.leather_lt,
        palette.leather_lt2,
        palette.leather_hi,
    )
    palette.brass_ramp = (
        palette.brass_shadow,
        palette.brass_md,
        palette.brass_dk,
        palette.brass,
        palette.brass_lt,
        palette.brass_hi,
    )
    palette.bone_ramp = (
        palette.bone_shadow,
        palette.bone_dk,
        palette.bone_dust,
        palette.bone_md,
        palette.bone_mid,
        palette.bone,
        palette.bone_lt,
    )
    palette.bone_form = (
        palette.bone_shadow,
        palette.bone_dk,
        palette.bone_dust,
        palette.bone_md,
        palette.bone_mid,
    )
    palette.steel_ramp = (
        palette.steel_deep,
        palette.steel_void,
        palette.steel_shadow,
        palette.steel_cloth,
        palette.steel_dk,
        palette.steel,
    )
    palette.wing_ramp = palette.steel_ramp
    palette.dead_ramp = (
        palette.dead_dk,
        palette.dead,
        palette.steel_mid,
        palette.bone_dust,
        palette.steel_lt,
        palette.bone_md,
    )
    palette.skin_ramp = (
        palette.skin_dk,
        palette.skin_md,
        palette.skin_warm,
        palette.skin,
        palette.skin_lt,
    )
    palette.cloth_ramp = (
        palette.steel_deep,
        palette.steel_void,
        palette.steel_shadow,
        palette.steel_cloth,
        palette.steel_dk,
        palette.steel,
        palette.steel_mid,
    )
    return palette


def colors_used(*groups: dict[Point, Color]) -> set[Color]:
    colors: set[Color] = set()
    for group in groups:
        colors.update(group.values())
    return colors


class Pin:
    def __init__(self, x: float, y: float) -> None:
        self.x = x
        self.y = y


class _Row:
    def __init__(self, canvas: Canvas, y: int) -> None:
        self._canvas = canvas
        self._y = y

    def __getitem__(self, x: int) -> Color | None:
        return self._canvas.read(x, self._y)

    def __setitem__(self, x: int, color: Color | None) -> None:
        if color is None:
            self._canvas.put(x, self._y, CLEAR)
            return
        self._canvas.put(x, self._y, color)


class Canvas:
    def __init__(
        self,
        width: int,
        height: int,
        fill: Color = CLEAR,
        *,
        strict: bool = False,
        pin: Pin | None = None,
    ) -> None:
        self.width = width
        self.height = height
        self.pixels = [fill] * (width * height)
        self.strict = strict
        self.pin = Pin(0.0, 0.0) if pin is None else pin

    def put(self, x: int, y: int, color: Color) -> None:
        if color[3] not in (0, 255):
            raise SystemExit("partial alpha")
        if not (0 <= x < self.width and 0 <= y < self.height):
            if self.strict:
                raise ValueError(f"pixel outside the icon: {(x, y)}")
            return
        self.pixels[y * self.width + x] = color

    def read(self, x: int, y: int) -> Color | None:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return None
        color = self.pixels[y * self.width + x]
        if color[3] == 0:
            return None
        return color

    def __getitem__(self, y: int) -> _Row:
        return _Row(self, y)

    def fill(self, points: list[Point], color: Color) -> None:
        for x, y in points:
            self.put(x, y, color)

    def fill_rect(self, x0: int, y0: int, x1: int, y1: int, color: Color) -> None:
        for y in range(y0, y1):
            for x in range(x0, x1):
                self.put(x, y, color)

    def blit(self, shaded_pixels: dict[Point, Color]) -> None:
        for (x, y), color in shaded_pixels.items():
            self.put(x, y, color)

    def opaque_at(self, x: int, y: int) -> bool:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return False
        return self.pixels[y * self.width + x][3] == 255

    def image(self) -> Image.Image:
        image = Image.new("RGBA", (self.width, self.height), (0, 0, 0, 0))
        image.putdata(self.pixels)
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
            for dx, dy in NEIGHBORS_8:
                nx = x + dx
                ny = y + dy
                if 0 <= nx < width and 0 <= ny < height and canvas.pixels[ny * width + nx][3] == 0:
                    extra.append((nx, ny))
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
            for dx, dy in ((1, 0), (0, 1), (1, 1)):
                nx = x + dx
                ny = y + dy
                if not (0 <= nx < width and 0 <= ny < height) or canvas.pixels[ny * width + nx][3] == 0:
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
        for dx, dy in NEIGHBORS_4:
            neighbour = (x + dx, y + dy)
            if neighbour not in layer and 0 <= neighbour[0] < width and 0 <= neighbour[1] < height:
                outlined[neighbour] = color
    return outlined


def paste_pixels(target: dict[Point, Color], layer: dict[Point, Color], dx: int = 0, dy: int = 0) -> None:
    for (x, y), color in layer.items():
        target[(x + dx, y + dy)] = color


def translate_pixels(pixels: dict[Point, Color], dx: int, dy: int) -> dict[Point, Color]:
    if dx == 0 and dy == 0:
        return dict(pixels)
    return {(x + dx, y + dy): color for (x, y), color in pixels.items()}


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


def place_on_pin(target: Canvas, source: Canvas, pin: Pin) -> None:
    dx = iround(pin.x - source.pin.x)
    dy = iround(pin.y - source.pin.y)
    for index, color in enumerate(source.pixels):
        if color[3] == 0:
            continue
        x = index % source.width + dx
        y = index // source.width + dy
        target.put(x, y, color)


class Sprite:
    def __init__(self, pixels: dict[Point, Color] | None = None, pin: Pin | None = None) -> None:
        self.pixels = {} if pixels is None else dict(pixels)
        self.pin = Pin(0.0, 0.0) if pin is None else Pin(pin.x, pin.y)

    def copy(self) -> Sprite:
        return Sprite(self.pixels, self.pin)

    def paste(self, other: Sprite, dx: int = 0, dy: int = 0) -> Sprite:
        paste_pixels(self.pixels, other.pixels, dx, dy)
        return self

    def shift(self, dx: int, dy: int) -> Sprite:
        moved = Sprite(translate_pixels(self.pixels, dx, dy), self.pin)
        moved.pin = Pin(self.pin.x + dx, self.pin.y + dy)
        return moved

    def crop(self, x0: int, y0: int, x1: int, y1: int) -> Sprite:
        kept = {}
        for (x, y), color in self.pixels.items():
            if x0 <= x < x1 and y0 <= y < y1:
                kept[(x - x0, y - y0)] = color
        return Sprite(kept)

    def erase_rect(self, x0: int, y0: int, x1: int, y1: int) -> Sprite:
        self.pixels = {
            (x, y): color
            for (x, y), color in self.pixels.items()
            if not (x0 <= x < x1 and y0 <= y < y1)
        }
        return self

    def erase_if(self, predicate) -> Sprite:
        self.pixels = {
            (x, y): color
            for (x, y), color in self.pixels.items()
            if not predicate(x, y, color)
        }
        return self

    def bounds(self) -> tuple[int, int, int, int]:
        xs = [x for x, _ in self.pixels]
        ys = [y for _, y in self.pixels]
        if not xs:
            raise SystemExit("empty sprite")
        return min(xs), min(ys), max(xs), max(ys)

    def contained(self, dx: int = 0, dy: int = 0, *, width: int, height: int) -> None:
        for x, y in self.pixels:
            xx, yy = x + dx, y + dy
            if not (0 <= xx < width and 0 <= yy < height):
                raise SystemExit(f"opaque pixel would leave the cell at {xx},{yy}")


def sprite_from_image(image: Image.Image) -> Sprite:
    pixels = image.load()
    width, height = image.size
    kept = {}
    for y in range(height):
        for x in range(width):
            red, green, blue, alpha = pixels[x, y]
            if alpha == 0:
                continue
            if alpha != 255:
                raise SystemExit(f"partial alpha at {x},{y}")
            kept[(x, y)] = (red, green, blue, 255)
    return Sprite(kept)


def long_runs(xs: list[int]) -> list[tuple[int, int]]:
    if not xs:
        return []
    ordered = sorted(set(xs))
    runs: list[tuple[int, int]] = []
    start = previous = ordered[0]
    for x in ordered[1:]:
        if x == previous + 1:
            previous = x
            continue
        runs.append((start, previous))
        start = previous = x
    runs.append((start, previous))
    return [(a, b) for a, b in runs if b - a + 1 >= 8]


def stance_contact(sprite: Sprite) -> tuple[int, float]:
    by_row: dict[int, list[int]] = {}
    for x, y in sprite.pixels:
        by_row.setdefault(y, []).append(x)
    for y in sorted(by_row, reverse=True):
        runs = long_runs(by_row[y])
        if not runs:
            continue
        runs.sort(key=lambda run: (run[1] - run[0], -run[0]), reverse=True)
        if len(runs) >= 2:
            first = (runs[0][0] + runs[0][1]) / 2
            second = (runs[1][0] + runs[1][1]) / 2
            return y, (first + second) / 2
        left, right = runs[0]
        return y, (left + right) / 2
    raise SystemExit("no stance contact")


def bottom_band(sprite: Sprite) -> tuple[float, int]:
    max_y = max(y for _, y in sprite.pixels)
    xs = [x for x, y in sprite.pixels if y == max_y]
    if len(xs) < 2:
        xs = [x for x, y in sprite.pixels if y >= max_y - 1]
    return (min(xs) + max(xs)) / 2, max_y


def widen_sole(sprite: Sprite, toward_right: bool) -> None:
    row, _center = stance_contact(sprite)
    xs = sorted(x for x, y in sprite.pixels if y == row)
    runs = long_runs(xs)
    runs.sort(key=lambda run: (run[1] - run[0], -run[0]), reverse=True)
    left, right = runs[0]
    if toward_right:
        sprite.pixels[(right + 1, row)] = sprite.pixels[(right, row)]
    else:
        sprite.pixels[(left - 1, row)] = sprite.pixels[(left, row)]


def shift_contact_to(sprite: Sprite, target_x: float, target_row: int) -> tuple[Sprite, int, int]:
    row, center = stance_contact(sprite)
    anchor = Pin(center, float(row))
    target = Pin(target_x, float(target_row))
    dy = int(target.y) - int(anchor.y)
    moved = sprite.shift(0, dy)
    for _ in range(4):
        _row, center = stance_contact(moved)
        anchor = Pin(center, float(_row))
        delta = target.x - anchor.x
        if abs(delta - round(delta)) < 1e-6:
            dx = iround(delta)
            moved = moved.shift(dx, 0)
            moved.pin = Pin(target.x, float(target_row))
            return moved, dx, dy
        widen_sole(moved, toward_right=delta > 0)
    raise SystemExit(f"contact {center} cannot reach {target_x}")


def plant_contact(
    sprite: Sprite,
    target_x: float,
    target_row: int,
    cell_width: int,
    cell_height: int,
) -> Sprite:
    current = sprite.copy()
    for _ in range(5):
        moved, _dx, _dy = shift_contact_to(current, target_x, target_row)
        overflow = [
            point
            for point in moved.pixels
            if not (0 <= point[0] < cell_width and 0 <= point[1] < cell_height)
        ]
        if not overflow:
            return moved
        for point in overflow:
            del moved.pixels[point]
        row, center = stance_contact(moved)
        if row == target_row and abs(center - target_x) < 1e-6:
            return moved
        current = moved
    raise SystemExit(f"foot point {target_x},{target_row} clips the cell")


def _frame_pixels(frame: Sprite | dict[Point, Color]) -> dict[Point, Color]:
    if isinstance(frame, dict):
        return frame
    return frame.pixels


def pack_strip(frames: list[Sprite | dict[Point, Color]], cell_width: int, cell_height: int) -> Image.Image:
    image = Image.new("RGBA", (cell_width * len(frames), cell_height), CLEAR)
    pixels = image.load()
    width, height = image.size
    for index, frame in enumerate(frames):
        origin_x = index * cell_width
        for (x, y), color in _frame_pixels(frame).items():
            xx, yy = x + origin_x, y
            if not (0 <= xx < width and 0 <= yy < height):
                raise SystemExit(f"blit clipped {xx},{yy}")
            pixels[xx, yy] = color
    return image


def contact_sheet(
    frames: list[Image.Image],
    columns: int,
    *,
    gutter: int = 2,
    margin: int = 4,
    background: Color = CLEAR,
) -> Image.Image:
    if columns < 1:
        raise SystemExit("contact sheet needs a column")
    if not frames:
        raise SystemExit("contact sheet needs a frame")
    cell_width = max(frame.width for frame in frames)
    cell_height = max(frame.height for frame in frames)
    rows = (len(frames) + columns - 1) // columns
    width = margin * 2 + columns * cell_width + (columns - 1) * gutter
    height = margin * 2 + rows * cell_height + (rows - 1) * gutter
    sheet = Image.new("RGBA", (width, height), background)
    for index, frame in enumerate(frames):
        column = index % columns
        row = index // columns
        x = margin + column * (cell_width + gutter)
        y = margin + row * (cell_height + gutter)
        placed = Canvas(frame.width, frame.height, pin=Pin(0.0, 0.0))
        placed.pixels = list(frame.convert("RGBA").getdata())
        target = Canvas(sheet.width, sheet.height, background, pin=Pin(float(x), float(y)))
        target.pixels = list(sheet.getdata())
        place_on_pin(target, placed, target.pin)
        sheet = target.image()
    return sheet
