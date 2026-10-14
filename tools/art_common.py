from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path

from PIL import Image

Color = tuple[int, int, int, int]
Point = tuple[int, int]
CLEAR: Color = (0, 0, 0, 0)
NEIGHBORS_8 = tuple((dx, dy) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dx or dy)
NEIGHBORS_4 = ((1, 0), (-1, 0), (0, 1), (0, -1))


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


def require_opaque_color(color: Color, allowed: set[Color] | None = None) -> Color:
    if color[3] not in (0, 255):
        raise SystemExit("partial alpha")
    if allowed is not None and color not in allowed:
        raise SystemExit(f"color {color} is not in the signed hunter")
    return color


@dataclass(frozen=True)
class Palette:
    ink: Color
    ink_warm: Color
    leather_shadow: Color
    leather_deep: Color
    leather_soot: Color
    leather_dk: Color
    leather_md: Color
    leather_above_mid: Color
    leather: Color
    leather_fold: Color
    leather_lt: Color
    leather_above_light: Color
    leather_hi: Color
    skin_dk: Color
    skin_md: Color
    skin: Color
    skin_warm: Color
    skin_lt: Color
    brass_shadow: Color
    brass_md: Color
    brass_dk: Color
    brass: Color
    brass_lt: Color
    brass_hi: Color
    gel_shadow: Color
    gel_deep: Color
    gel_well: Color
    gel_pool: Color
    gel_tide: Color
    gel_sea: Color
    gel_foam: Color
    gel_mid: Color
    gel_kelp: Color
    gel_lagoon: Color
    gel_dk: Color
    gel: Color
    gel_core: Color
    gel_lt: Color
    gel_bright: Color
    gel_hi: Color
    gel_hot: Color
    sage_shadow: Color
    sage_mid: Color
    sage_dk: Color
    sage: Color
    sage_lt: Color
    purple_dk: Color
    purple: Color
    purple_lt: Color
    steel_deep: Color
    steel_void: Color
    steel_shadow: Color
    steel_dk: Color
    steel: Color
    steel_cloth: Color
    steel_mid: Color
    steel_lt: Color
    bone_shadow: Color
    bone_dust: Color
    bone_dk: Color
    bone_md: Color
    bone_mid: Color
    bone: Color
    bone_lt: Color
    dead_dk: Color
    dead: Color
    white: Color
    gel_ramp: tuple[Color, ...]
    gel_under: tuple[Color, ...]
    gel_body: tuple[Color, ...]
    gel_shine: tuple[Color, ...]
    sage_ramp: tuple[Color, ...]
    leather_ramp: tuple[Color, ...]
    brass_ramp: tuple[Color, ...]
    bone_ramp: tuple[Color, ...]
    bone_form: tuple[Color, ...]
    steel_ramp: tuple[Color, ...]
    wing_ramp: tuple[Color, ...]
    dead_ramp: tuple[Color, ...]
    skin_ramp: tuple[Color, ...]
    cloth_ramp: tuple[Color, ...]


def hunter_palette(allowed: set[Color]) -> Palette:
    def take(red: int, green: int, blue: int) -> Color:
        return require_opaque_color((red, green, blue, 255), allowed)

    ink = take(0, 0, 0)
    ink_warm = take(32, 23, 21)
    leather_shadow = take(36, 21, 13)
    leather_deep = take(43, 23, 15)
    leather_soot = take(54, 30, 20)
    leather_dk = take(55, 31, 21)
    leather_md = take(63, 35, 23)
    leather_above_mid = take(67, 38, 25)
    leather = take(74, 41, 26)
    leather_fold = take(76, 43, 28)
    leather_lt = take(97, 55, 35)
    leather_above_light = take(98, 56, 35)
    leather_hi = take(107, 64, 33)
    skin_dk = take(149, 92, 47)
    skin_md = take(187, 119, 69)
    skin = take(222, 152, 98)
    skin_warm = take(217, 150, 99)
    skin_lt = take(252, 187, 127)
    brass_shadow = take(161, 103, 26)
    brass_md = take(167, 108, 29)
    brass_dk = take(203, 141, 45)
    brass = take(226, 166, 64)
    brass_lt = take(244, 172, 13)
    brass_hi = take(251, 214, 119)
    gel_shadow = take(3, 40, 43)
    gel_deep = take(0, 64, 69)
    gel_well = take(0, 76, 81)
    gel_pool = take(0, 97, 106)
    gel_tide = take(2, 118, 129)
    gel_sea = take(4, 126, 133)
    gel_foam = take(40, 137, 138)
    gel_mid = take(42, 140, 147)
    gel_kelp = take(39, 150, 147)
    gel_lagoon = take(42, 161, 166)
    gel_dk = take(45, 170, 152)
    gel = take(3, 190, 174)
    gel_core = take(0, 210, 211)
    gel_lt = take(0, 213, 215)
    gel_bright = take(0, 229, 227)
    gel_hi = take(0, 251, 250)
    gel_hot = take(166, 253, 254)
    sage_shadow = take(7, 35, 33)
    sage_mid = take(28, 103, 99)
    sage_dk = take(67, 94, 77)
    sage = take(80, 127, 100)
    sage_lt = take(84, 122, 95)
    purple_dk = take(108, 21, 120)
    purple = take(184, 118, 200)
    purple_lt = take(244, 136, 254)
    steel_deep = take(13, 13, 13)
    steel_void = take(16, 16, 16)
    steel_shadow = take(22, 22, 26)
    steel_dk = take(38, 38, 43)
    steel = take(48, 48, 53)
    steel_cloth = take(37, 36, 37)
    steel_mid = take(135, 133, 137)
    steel_lt = take(168, 168, 169)
    bone_shadow = take(126, 126, 116)
    bone_dust = take(148, 154, 143)
    bone_dk = take(150, 149, 144)
    bone_md = take(174, 172, 175)
    bone_mid = take(179, 178, 182)
    bone = take(245, 244, 243)
    bone_lt = take(255, 255, 255)
    dead_dk = take(52, 36, 31)
    dead = take(54, 37, 32)
    white = take(255, 255, 255)
    gel_ramp = (
        gel_shadow,
        gel_deep,
        gel_well,
        gel_pool,
        gel_tide,
        gel_sea,
        gel_foam,
        gel_mid,
        gel_kelp,
        gel_lagoon,
        gel_dk,
        gel,
        gel_core,
        gel_lt,
        gel_bright,
        gel_hi,
    )
    gel_under = gel_ramp[:8]
    gel_body = gel_ramp[4:13]
    gel_shine = gel_ramp[10:]
    sage_ramp = (
        sage_shadow,
        sage_dk,
        sage_mid,
        sage,
        sage_lt,
    )
    leather_ramp = (
        leather_shadow,
        leather_deep,
        leather_soot,
        leather_dk,
        leather_md,
        leather_above_mid,
        leather,
        leather_fold,
        leather_lt,
        leather_above_light,
        leather_hi,
    )
    brass_ramp = (
        brass_shadow,
        brass_md,
        brass_dk,
        brass,
        brass_lt,
        brass_hi,
    )
    bone_ramp = (
        bone_shadow,
        bone_dk,
        bone_dust,
        bone_md,
        bone_mid,
        bone,
        bone_lt,
    )
    bone_form = (
        bone_shadow,
        bone_dk,
        bone_dust,
        bone_md,
        bone_mid,
    )
    steel_ramp = (
        steel_deep,
        steel_void,
        steel_shadow,
        steel_cloth,
        steel_dk,
        steel,
    )
    wing_ramp = steel_ramp
    dead_ramp = (
        dead_dk,
        dead,
        steel_mid,
        bone_dust,
        steel_lt,
        bone_md,
    )
    skin_ramp = (
        skin_dk,
        skin_md,
        skin_warm,
        skin,
        skin_lt,
    )
    cloth_ramp = (
        steel_deep,
        steel_void,
        steel_shadow,
        steel_cloth,
        steel_dk,
        steel,
        steel_mid,
    )
    return Palette(
        ink=ink,
        ink_warm=ink_warm,
        leather_shadow=leather_shadow,
        leather_deep=leather_deep,
        leather_soot=leather_soot,
        leather_dk=leather_dk,
        leather_md=leather_md,
        leather_above_mid=leather_above_mid,
        leather=leather,
        leather_fold=leather_fold,
        leather_lt=leather_lt,
        leather_above_light=leather_above_light,
        leather_hi=leather_hi,
        skin_dk=skin_dk,
        skin_md=skin_md,
        skin=skin,
        skin_warm=skin_warm,
        skin_lt=skin_lt,
        brass_shadow=brass_shadow,
        brass_md=brass_md,
        brass_dk=brass_dk,
        brass=brass,
        brass_lt=brass_lt,
        brass_hi=brass_hi,
        gel_shadow=gel_shadow,
        gel_deep=gel_deep,
        gel_well=gel_well,
        gel_pool=gel_pool,
        gel_tide=gel_tide,
        gel_sea=gel_sea,
        gel_foam=gel_foam,
        gel_mid=gel_mid,
        gel_kelp=gel_kelp,
        gel_lagoon=gel_lagoon,
        gel_dk=gel_dk,
        gel=gel,
        gel_core=gel_core,
        gel_lt=gel_lt,
        gel_bright=gel_bright,
        gel_hi=gel_hi,
        gel_hot=gel_hot,
        sage_shadow=sage_shadow,
        sage_mid=sage_mid,
        sage_dk=sage_dk,
        sage=sage,
        sage_lt=sage_lt,
        purple_dk=purple_dk,
        purple=purple,
        purple_lt=purple_lt,
        steel_deep=steel_deep,
        steel_void=steel_void,
        steel_shadow=steel_shadow,
        steel_dk=steel_dk,
        steel=steel,
        steel_cloth=steel_cloth,
        steel_mid=steel_mid,
        steel_lt=steel_lt,
        bone_shadow=bone_shadow,
        bone_dust=bone_dust,
        bone_dk=bone_dk,
        bone_md=bone_md,
        bone_mid=bone_mid,
        bone=bone,
        bone_lt=bone_lt,
        dead_dk=dead_dk,
        dead=dead,
        white=white,
        gel_ramp=gel_ramp,
        gel_under=gel_under,
        gel_body=gel_body,
        gel_shine=gel_shine,
        sage_ramp=sage_ramp,
        leather_ramp=leather_ramp,
        brass_ramp=brass_ramp,
        bone_ramp=bone_ramp,
        bone_form=bone_form,
        steel_ramp=steel_ramp,
        wing_ramp=wing_ramp,
        dead_ramp=dead_ramp,
        skin_ramp=skin_ramp,
        cloth_ramp=cloth_ramp,
    )


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
    ) -> None:
        self.width = width
        self.height = height
        self.pixels = [fill] * (width * height)

    def store_opaque(self, x: int, y: int, color: Color) -> None:
        if color[3] not in (0, 255):
            raise SystemExit("partial alpha")
        self.pixels[y * self.width + x] = color

    def put(self, x: int, y: int, color: Color) -> None:
        if not (0 <= x < self.width and 0 <= y < self.height):
            return
        self.store_opaque(x, y, color)

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


class RejectingCanvas(Canvas):
    def put(self, x: int, y: int, color: Color) -> None:
        if not (0 <= x < self.width and 0 <= y < self.height):
            raise ValueError(f"pixel outside the icon: {(x, y)}")
        self.store_opaque(x, y, color)


def clipping_canvas(width: int, height: int, fill: Color = CLEAR) -> Canvas:
    return Canvas(width, height, fill)


def rejecting_canvas(width: int, height: int, fill: Color = CLEAR) -> Canvas:
    return RejectingCanvas(width, height, fill)


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


def _frame_pixels(frame: dict[Point, Color]) -> dict[Point, Color]:
    if isinstance(frame, dict):
        return frame
    return frame.pixels


def pack_strip(frames: list, cell_width: int, cell_height: int) -> Image.Image:
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
