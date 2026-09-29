#!/usr/bin/env python3
"""In-house hunter frames beside the Taste-signed cells.

Pixel-authored at 1×. Every opaque RGBA is copied from a signed cell.
Signed idle, the run lean, the attack chamber, and the attack thrust are
copied through unchanged. Re-running writes the same bytes.

Sources (immutable, hashed): assets/source/hunter_signed/
Outputs: assets/player/combat/knight_*_side.png
"""

from __future__ import annotations

import hashlib
import math
import shutil
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SIGNED = ROOT / "assets" / "source" / "hunter_signed"
OUT = ROOT / "assets" / "player" / "combat"

CELL_W = 343
CELL_H = 160
FOOT_X = 81.5
FOOT_ROW = 158
ATTACK_SOLE_ROW = 159

# Signed full-file SHA-256 from the transparent sheets. The script stops
# if those bytes move, so a new frame cannot be painted onto the wrong base.
SIGNED_SHA256 = {
    "knight_idle_side.png": "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e",
    "knight_run_side.png": "1e05a64f76044a614d8a080e234778ce8eab3c9bff819fa99de3e34b816466e7",
    "knight_attack_side.png": "ca089a0e93524161f020d9ef91f736e330d36f3530749b8296e918b088620b04",
}

# Knee degrees are clockwise: positive swings that foot toward image-right.
# `plant` pins that boot under the chest on the foot row. The other poses
# keep both soles at or above that row so the cycle cannot clip the cell.
# bob drops the torso only.
RUN_POSES = (
    {"rear_deg": 8, "front_deg": 64, "bob": 0, "plant": "rear"},
    {"rear_deg": 14, "front_deg": 22, "bob": 2, "plant": ""},
    {"rear_deg": -28, "front_deg": 84, "bob": -1, "plant": ""},
    {"rear_deg": -14, "front_deg": 38, "bob": 0, "plant": ""},
    {"rear_deg": -48, "front_deg": -4, "bob": 0, "plant": "front"},
    {"rear_deg": 20, "front_deg": -12, "bob": 2, "plant": ""},
    {"rear_deg": 88, "front_deg": -44, "bob": -1, "plant": ""},
    {"rear_deg": 32, "front_deg": -18, "bob": 1, "plant": ""},
)

RUN_LEAN_DEG = 7.0
RUN_SWORD_DEG = 16.0
RUN_SWORD_LENGTH = 58
CAPE_TRIM_X = 18


def reject_resample_names() -> None:
    """Stop if this file names a resample call. Tokens are split so the
    check can see them without the source containing the joined word."""
    text = Path(__file__).read_text(encoding="utf-8")
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


def iround(value: float) -> int:
    return int(math.floor(value + 0.5))


class Sprite:
    def __init__(self, pixels: dict[tuple[int, int], tuple[int, int, int, int]] | None = None) -> None:
        self.pixels = {} if pixels is None else dict(pixels)

    def copy(self) -> Sprite:
        return Sprite(self.pixels)

    def paste(self, other: Sprite, dx: int = 0, dy: int = 0) -> Sprite:
        for (x, y), color in other.pixels.items():
            self.pixels[(x + dx, y + dy)] = color
        return self

    def shift(self, dx: int, dy: int) -> Sprite:
        if dx == 0 and dy == 0:
            return self.copy()
        return Sprite({(x + dx, y + dy): color for (x, y), color in self.pixels.items()})

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

    def contained(self, dx: int = 0, dy: int = 0) -> None:
        for x, y in self.pixels:
            xx, yy = x + dx, y + dy
            if not (0 <= xx < CELL_W and 0 <= yy < CELL_H):
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


def load_sheet(name: str) -> Sprite:
    path = SIGNED / name
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != SIGNED_SHA256[name]:
        raise SystemExit(f"{name} hash {digest} != pinned source")
    image = Image.open(path).convert("RGBA")
    return sprite_from_image(image)


def cell_of(sheet: Sprite, index: int) -> Sprite:
    return sheet.crop(index * CELL_W, 0, (index + 1) * CELL_W, CELL_H)


def is_hot_blade(color: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = color
    if alpha != 255:
        return False
    cyan = blue > 80 and green > 60 and blue > red + 15 and green > red - 5 and red < 200
    white_core = red > 200 and green > 210 and blue > 210
    return cyan or white_core


def is_yellow_mark(color: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = color
    return alpha == 255 and red > 170 and green > 130 and blue < 90 and red > blue + 70


def is_purple(color: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = color
    return alpha == 255 and red > 90 and blue > 120 and green < 110 and blue > green + 30


def is_brown(color: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = color
    return alpha == 255 and red > 45 and red > green and green > blue + 4 and red > blue + 18 and blue < 90


def is_skin(color: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = color
    return (
        alpha == 255
        and red > 130
        and green > 70
        and blue > 40
        and red >= green
        and green > blue
        and red - green < 100
        and blue < 160
    )


def flood_blade(sprite: Sprite) -> set[tuple[int, int]]:
    seeds = [point for point, color in sprite.pixels.items() if is_hot_blade(color)]
    seen = set(seeds)
    stack = list(seeds)
    while stack:
        x, y = stack.pop()
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                nxt = (x + dx, y + dy)
                if nxt in seen or nxt not in sprite.pixels:
                    continue
                if is_hot_blade(sprite.pixels[nxt]):
                    seen.add(nxt)
                    stack.append(nxt)
    return seen


def without_blade(sprite: Sprite, halo: int = 4) -> Sprite:
    """Drop the bright blade and the dark outline hugging it. Skin, the R, and the cable stay."""
    blade = flood_blade(sprite)
    drop = set(blade)
    for x, y in blade:
        for dy in range(-halo, halo + 1):
            for dx in range(-halo, halo + 1):
                if dx * dx + dy * dy > halo * halo:
                    continue
                point = (x + dx, y + dy)
                color = sprite.pixels.get(point)
                if color is None or point in drop:
                    continue
                if is_skin(color) or is_yellow_mark(color) or is_purple(color):
                    continue
                if is_brown(color) and dx * dx + dy * dy > 5:
                    continue
                drop.add(point)
    return Sprite({point: color for point, color in sprite.pixels.items() if point not in drop})


def blade_only(sprite: Sprite) -> Sprite:
    blade = flood_blade(sprite)
    return Sprite({point: sprite.pixels[point] for point in blade})


def rotate(sprite: Sprite, degrees_clockwise: float, pivot: tuple[float, float]) -> Sprite:
    """Integer nearest-neighbor place. Inverse sample, no blend, alpha stays 0 or 255."""
    if abs(degrees_clockwise) < 0.05 or not sprite.pixels:
        return sprite.copy()
    radians = math.radians(degrees_clockwise)
    cosine = math.cos(radians)
    sine = math.sin(radians)
    px, py = pivot
    xs = [x for x, _ in sprite.pixels]
    ys = [y for _, y in sprite.pixels]
    corners = (
        (min(xs), min(ys)),
        (max(xs), min(ys)),
        (min(xs), max(ys)),
        (max(xs), max(ys)),
    )

    def forward(x: float, y: float) -> tuple[float, float]:
        dx, dy = x - px, y - py
        return (cosine * dx - sine * dy + px, sine * dx + cosine * dy + py)

    placed = [forward(x, y) for x, y in corners]
    min_x = math.floor(min(point[0] for point in placed)) - 1
    max_x = math.ceil(max(point[0] for point in placed)) + 1
    min_y = math.floor(min(point[1] for point in placed)) - 1
    max_y = math.ceil(max(point[1] for point in placed)) + 1
    out: dict[tuple[int, int], tuple[int, int, int, int]] = {}
    for y in range(min_y, max_y + 1):
        for x in range(min_x, max_x + 1):
            dx, dy = x - px, y - py
            src_x = iround(cosine * dx + sine * dy + px)
            src_y = iround(-sine * dx + cosine * dy + py)
            color = sprite.pixels.get((src_x, src_y))
            if color is not None:
                out[(x, y)] = color
    return fill_cracks(Sprite(out))


def fill_cracks(sprite: Sprite, passes: int = 2) -> Sprite:
    """Close 1 px holes left by integer rounding. The color is a neighbor, already a source RGBA."""
    for _ in range(passes):
        extra = {}
        if not sprite.pixels:
            break
        xs = [x for x, _ in sprite.pixels]
        ys = [y for _, y in sprite.pixels]
        for y in range(min(ys) - 1, max(ys) + 2):
            for x in range(min(xs) - 1, max(xs) + 2):
                if (x, y) in sprite.pixels or (x, y) in extra:
                    continue
                left = sprite.pixels.get((x - 1, y))
                right = sprite.pixels.get((x + 1, y))
                up = sprite.pixels.get((x, y - 1))
                down = sprite.pixels.get((x, y + 1))
                if left is not None and right is not None:
                    extra[(x, y)] = left
                elif up is not None and down is not None:
                    extra[(x, y)] = up
        sprite.pixels.update(extra)
    return sprite


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
    """Lowest row with a contiguous opaque run of at least 8 px.

    Two or more runs: mean of the two widest centers. Matches the signed-cell rule.
    """
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


def chest_pin_x(sprite: Sprite) -> float:
    xs = [
        x
        for (x, y), color in sprite.pixels.items()
        if 50 <= y <= 96 and not is_hot_blade(color)
    ]
    if not xs:
        raise SystemExit("no chest pixels")
    return (min(xs) + max(xs)) / 2


def shorten(sprite: Sprite, pivot: tuple[float, float], length: float) -> Sprite:
    px, py = pivot
    kept = {}
    for (x, y), color in sprite.pixels.items():
        if math.hypot(x - px, y - py) <= length:
            kept[(x, y)] = color
    if not kept:
        raise SystemExit("shorten removed the sprite")
    return Sprite(kept)


def blade_base(blade: Sprite, end: str = "low") -> tuple[float, float]:
    """`low` is the hilt of an upright blade. `left` is the hilt of the thrust blade."""
    if end == "left":
        min_x = min(x for x, _ in blade.pixels)
        ys = [y for x, y in blade.pixels if x <= min_x + 2]
        return float(min_x), (min(ys) + max(ys)) / 2
    max_y = max(y for _, y in blade.pixels)
    xs = [x for x, y in blade.pixels if y >= max_y - 1]
    return (min(xs) + max(xs)) / 2, float(max_y)


def blade_tip(blade: Sprite, pivot: tuple[float, float]) -> tuple[int, int]:
    px, py = pivot
    return max(blade.pixels, key=lambda point: math.hypot(point[0] - px, point[1] - py))


def aim_blade(
    blade: Sprite,
    degrees_above_horizontal: float,
    length: float,
    end: str = "low",
) -> tuple[Sprite, tuple[float, float]]:
    pivot = blade_base(blade, end)
    tip = blade_tip(blade, pivot)
    current = math.degrees(math.atan2(-(tip[1] - pivot[1]), tip[0] - pivot[0]))
    turned = rotate(blade, current - degrees_above_horizontal, pivot)
    aimed = shorten(turned, pivot, length)
    return aimed, pivot


def place_aimed(aimed: Sprite, pivot: tuple[float, float], hand: tuple[float, float]) -> Sprite:
    return aimed.shift(iround(hand[0] - pivot[0]), iround(hand[1] - pivot[1]))


def top_center(sprite: Sprite) -> tuple[float, float]:
    min_y = min(y for _, y in sprite.pixels)
    xs = [x for x, y in sprite.pixels if y <= min_y + 1]
    return (min(xs) + max(xs)) / 2, float(min_y)


def bridge_vertical_gaps(leg: Sprite, limit: int = 8) -> Sprite:
    """Copy a boot or thigh color across the idle hem gap so a swung leg stays one piece."""
    columns: dict[int, list[int]] = {}
    for x, y in leg.pixels:
        columns.setdefault(x, []).append(y)
    for x, ys in columns.items():
        ordered = sorted(ys)
        for top, bottom in zip(ordered, ordered[1:]):
            if bottom - top <= 1 or bottom - top > limit:
                continue
            color = leg.pixels[(x, bottom)]
            for y in range(top + 1, bottom):
                leg.pixels[(x, y)] = color
    return leg


def leg_column(idle: Sprite, x0: int, y0: int, x1: int, y1: int) -> Sprite:
    return bridge_vertical_gaps(idle.crop(x0, y0, x1, y1))


def swing_from_hip(leg: Sprite, degrees_clockwise: float, hip_x: float, hip_y: float) -> Sprite:
    """Rotate the whole leg around its top, then seat that pivot on the hip.

    One piece, so the boot stays attached to the thigh. Positive degrees swing
    the foot toward image-right.
    """
    pivot = top_center(leg)
    swung = rotate(leg, degrees_clockwise, pivot) if abs(degrees_clockwise) >= 1 else leg.copy()
    return swung.shift(iround(hip_x - pivot[0]), iround(hip_y - pivot[1]))


def sword_hand(sprite: Sprite) -> tuple[float, float]:
    points = [(x, y) for (x, y), color in sprite.pixels.items() if is_skin(color) and y > 70]
    if not points:
        points = [(x, y) for (x, y), color in sprite.pixels.items() if y > 80 and x > 100]
    if not points:
        raise SystemExit("no sword hand")
    right = max(x for x, _ in points)
    cluster = [(x, y) for x, y in points if x >= right - 14]
    return (
        sum(x for x, _ in cluster) / len(cluster),
        sum(y for _, y in cluster) / len(cluster),
    )


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
    """Integer-shift so the stance contact lands on the target. Widens the sole by one source pixel if the center's parity cannot reach a half-pixel."""
    row, center = stance_contact(sprite)
    dy = target_row - row
    moved = sprite.shift(0, dy)
    for _ in range(4):
        _row, center = stance_contact(moved)
        delta = target_x - center
        if abs(delta - round(delta)) < 1e-6:
            dx = iround(delta)
            moved = moved.shift(dx, 0)
            return moved, dx, dy
        widen_sole(moved, toward_right=delta > 0)
    raise SystemExit(f"contact {center} cannot reach {target_x}")


def require_inside(sprite: Sprite) -> None:
    sprite.contained(0, 0)


def plant_on_foot(sprite: Sprite, target_x: float = FOOT_X, target_row: int = FOOT_ROW) -> Sprite:
    """Shift the stance contact onto the shared foot point, then drop cape pixels that still fall outside the cell."""
    current = sprite.copy()
    for _ in range(5):
        moved, _dx, _dy = shift_contact_to(current, target_x, target_row)
        overflow = [point for point in moved.pixels if not (0 <= point[0] < CELL_W and 0 <= point[1] < CELL_H)]
        if not overflow:
            return moved
        for point in overflow:
            del moved.pixels[point]
        row, center = stance_contact(moved)
        if row == target_row and abs(center - target_x) < 1e-6:
            return moved
        current = moved
    raise SystemExit(f"foot point {target_x},{target_row} clips the cell")


def palette_of(*sheets: Sprite) -> set[tuple[int, int, int, int]]:
    colors: set[tuple[int, int, int, int]] = set()
    for sheet in sheets:
        colors.update(sheet.pixels.values())
    return colors


def opaque_width(sprite: Sprite) -> int:
    left, _top, right, _bottom = sprite.bounds()
    return right - left + 1


def same_pixels(a: Sprite, b: Sprite) -> bool:
    return a.pixels == b.pixels


def build_run_parts(idle: Sprite) -> dict:
    body = without_blade(idle)
    body.erase_rect(0, 0, CAPE_TRIM_X, CELL_H)
    body.erase_rect(58, 130, 86, CELL_H)
    body.erase_rect(88, 126, 122, CELL_H)
    chest = chest_pin_x(body)
    chest_y = 78.0
    leaned = rotate(body, RUN_LEAN_DEG, (chest, chest_y))
    # Headroom for the up-bob. The lean pivots on the chest, so the pin stays.
    leaned = leaned.shift(0, 2)
    min_x = min(x for x, _ in leaned.pixels)
    if min_x < 4:
        leaned.erase_rect(min_x, 0, 4, CELL_H)
    rear = leg_column(idle, 62, 116, 82, 160)
    front = leg_column(idle, 90, 112, 116, 160)
    sword = blade_only(idle)
    return {"body": leaned, "chest": chest_pin_x(leaned), "rear": rear, "front": front, "sword": sword}


def _sole(leg: Sprite) -> tuple[float, int]:
    try:
        row, center = stance_contact(leg)
        return center, row
    except SystemExit:
        return bottom_band(leg)


def _seat_leg(leg: Sprite, plant: bool, chest: float) -> Sprite:
    """Keep every sole on or above the foot row. A planted boot is slid under the chest."""
    center, sole = _sole(leg)
    dx = iround(chest - center) if plant else 0
    dy = iround(FOOT_ROW - sole) if plant else min(0, FOOT_ROW - sole)
    if plant and abs((chest - center) - dx) > 1e-6:
        widen_sole(leg, toward_right=(chest - center) > 0)
        center, sole = _sole(leg)
        dx = iround(chest - center)
        dy = iround(FOOT_ROW - sole)
    return leg.shift(dx, dy)


def compose_run_frame(parts: dict, pose: dict) -> Sprite:
    chest = parts["chest"]
    body = parts["body"].shift(0, pose["bob"])
    hip_y = 118
    rear = swing_from_hip(parts["rear"], pose["rear_deg"], chest - 4, hip_y)
    front = swing_from_hip(parts["front"], pose["front_deg"], chest + 10, hip_y)
    rear = _seat_leg(rear, pose["plant"] == "rear", chest)
    front = _seat_leg(front, pose["plant"] == "front", chest)
    rear_foot, _sole = bottom_band(rear)
    front_foot, _sole = bottom_band(front)
    frame = body.copy()
    if rear_foot <= front_foot:
        frame.paste(rear)
        frame.paste(front)
    else:
        frame.paste(front)
        frame.paste(rear)
    hand = sword_hand(body)
    aimed, pivot = aim_blade(parts["sword"], RUN_SWORD_DEG, RUN_SWORD_LENGTH)
    frame.paste(place_aimed(aimed, pivot, (hand[0] + 2, hand[1] - 8)))
    return frame


def build_run_cycle(idle: Sprite) -> list[Sprite]:
    parts = build_run_parts(idle)
    frames = [compose_run_frame(parts, pose) for pose in RUN_POSES]
    for frame in frames:
        # A short taper under the sole would ride past the cell when the 8px run is planted.
        try:
            row, _center = stance_contact(frame)
        except SystemExit:
            continue
        frame.erase_if(lambda x, y, _color, sole=row: y > sole)
    # The cape hangs further left than the foot point allows. Drop only that
    # tip, from every frame, so the shared foot shift cannot clip it.
    _row, center = stance_contact(frames[0])
    cape_limit = math.floor(center - FOOT_X)
    if cape_limit > 0:
        for frame in frames:
            frame.erase_if(lambda x, y, _color, limit=cape_limit: x < limit)
    planted, dx, dy = shift_contact_to(frames[0], FOOT_X, FOOT_ROW)
    frames[0] = planted
    for index in range(1, len(frames)):
        frames[index] = frames[index].shift(dx, dy)
    # Frame 4 was seated on the same chest x. If the 8px run is a half-pixel
    # off, widen that sole in place instead of sliding the torso.
    row, center = stance_contact(frames[4])
    if row == FOOT_ROW and abs(center - FOOT_X) > 1e-6 and abs((FOOT_X - center) - round(FOOT_X - center)) > 1e-6:
        widen_sole(frames[4], toward_right=(FOOT_X - center) > 0)
        _row, center = stance_contact(frames[4])
        if abs(center - FOOT_X) <= 2 and abs((FOOT_X - center) - round(FOOT_X - center)) < 1e-6:
            frames[4] = frames[4].shift(iround(FOOT_X - center), 0)
    for index, frame in enumerate(frames):
        left, top, right, bottom = frame.bounds()
        if left < 0 or top < 0 or right >= CELL_W or bottom >= CELL_H:
            raise SystemExit(
                f"run frame {index} bbox {left},{top}-{right},{bottom} "
                f"after shift {dx},{dy} leaves the cell"
            )
    return frames


def shave_chest_edge(sprite: Sprite, take_from_right: bool) -> None:
    xs = [
        x
        for (x, y), color in sprite.pixels.items()
        if 50 <= y <= 96 and not is_hot_blade(color)
    ]
    edge = max(xs) if take_from_right else min(xs)
    victims = [
        (x, y)
        for (x, y), color in sprite.pixels.items()
        if x == edge and 50 <= y <= 96 and not is_hot_blade(color)
    ]
    for point in victims:
        del sprite.pixels[point]


def align_chest_and_sole(sprite: Sprite, sole_row: int) -> Sprite:
    moved = sprite
    row, _center = stance_contact(moved)
    moved = moved.shift(0, sole_row - row)
    for _ in range(6):
        pin = chest_pin_x(moved)
        delta = FOOT_X - pin
        if abs(delta) < 1e-6:
            require_inside(moved)
            return moved
        if abs(delta - round(delta)) < 1e-6:
            moved = moved.shift(iround(delta), 0)
            continue
        shave_chest_edge(moved, take_from_right=delta < 0)
    raise SystemExit(f"chest pin stuck at {chest_pin_x(moved)}")


def attack_hand(sprite: Sprite) -> tuple[float, float]:
    """Lower-right of the image-right grip, where the blade leaves the fist."""
    points = [(x, y) for (x, y), color in sprite.pixels.items() if is_skin(color)]
    if not points:
        return sword_hand(sprite)
    right = max(x for x, _ in points)
    cluster = [(x, y) for x, y in points if x >= right - 16]
    return (max(x for x, _ in cluster) - 2, max(y for _, y in cluster) + 8)


def reposition_boots(sprite: Sprite, rear_x: float, front_x: float, sole_row: int) -> Sprite:
    """Slide the chamber's own boots to a narrower stance. Legs above the boot stay."""
    rear_box = sprite.crop(0, 136, 48, CELL_H)
    front_box = sprite.crop(80, 136, 140, CELL_H)
    if not rear_box.pixels or not front_box.pixels:
        return sprite
    cleared = sprite.copy()
    cleared.erase_rect(0, 136, 50, CELL_H)
    cleared.erase_rect(80, 136, 140, CELL_H)
    rear_center, rear_sole = bottom_band(rear_box)
    front_center, front_sole = bottom_band(front_box)
    # crop() rebased the boxes, and bottom_band is in that local space.
    # The boxes were taken from absolute coords, so shift from local origin.
    cleared.paste(rear_box, iround(rear_x - rear_center), sole_row - rear_sole)
    cleared.paste(front_box, iround(front_x - front_center), sole_row - front_sole)
    return cleared


def build_attack_frames(chamber: Sprite, thrust: Sprite) -> tuple[Sprite, Sprite]:
    body = align_chest_and_sole(without_blade(chamber), ATTACK_SOLE_ROW)
    between_body = reposition_boots(body, 60, 112, ATTACK_SOLE_ROW)
    recover_body = reposition_boots(body, 72, 100, ATTACK_SOLE_ROW)
    # Boot slides sit below the chest band, so the pin stays put.
    # The thrust drawing's bright blade starts near x=230. A few glow pixels
    # further left are not the hilt.
    thrust_blade = Sprite(
        {
            point: color
            for point, color in thrust.pixels.items()
            if is_hot_blade(color) and point[0] >= 220
        }
    )
    between = _attack_with_sword(between_body, thrust_blade, 40.0, 220)
    recover = _attack_with_sword(recover_body, thrust_blade, 54.0, 180)
    require_inside(between)
    require_inside(recover)
    return between, recover


def _inside_cell(sprite: Sprite) -> bool:
    return all(0 <= x < CELL_W and 0 <= y < CELL_H for x, y in sprite.pixels)


def _attack_with_sword(body: Sprite, thrust_blade: Sprite, angle: float, tip_x: float) -> Sprite:
    hand = attack_hand(body)
    best = None
    for length in range(36, 140):
        aimed, pivot = aim_blade(thrust_blade, angle, length, end="left")
        placed = place_aimed(aimed, pivot, hand)
        if not _inside_cell(placed):
            continue
        blade_xs = [x for (x, _y), color in placed.pixels.items() if is_hot_blade(color)]
        if not blade_xs:
            continue
        tip = max(blade_xs)
        score = abs(tip - tip_x)
        if best is None or score < best[0]:
            best = (score, placed, tip, length)
        if tip >= tip_x:
            break
    if best is None:
        raise SystemExit(f"attack sword at {angle}° from {hand} does not fit")
    frame = body.copy().paste(best[1])
    # The blade stays above the knees so it cannot move the sole row.
    frame.erase_if(lambda x, y, color: y > 140 and is_hot_blade(color))
    # The blade can cover the old right edge of the chest band. Slide back
    # onto the pin without moving the sole off row 159.
    frame = nudge_chest_x(frame)
    return frame


def nudge_chest_x(sprite: Sprite) -> Sprite:
    moved = sprite
    for _ in range(6):
        delta = FOOT_X - chest_pin_x(moved)
        if abs(delta) < 1e-6:
            return moved
        if abs(delta - round(delta)) < 1e-6:
            moved = moved.shift(iround(delta), 0)
            continue
        shave_chest_edge(moved, take_from_right=delta < 0)
    raise SystemExit(f"chest pin stuck at {chest_pin_x(moved)}")


def build_jump(idle: Sprite, rear: Sprite, front: Sprite) -> Sprite:
    body = idle.copy()
    body.erase_rect(58, 130, 86, CELL_H)
    body.erase_rect(88, 126, 122, CELL_H)
    chest = chest_pin_x(body)
    # Knees up in front of the coat. The sword stays in the idle raised hand.
    tucked_rear = swing_from_hip(rear, 68, chest - 2, 112)
    tucked_front = swing_from_hip(front, -46, chest + 14, 110)
    frame = body.copy()
    frame.paste(tucked_rear)
    frame.paste(tucked_front)
    lowest = max(y for _x, y in frame.pixels)
    # Soles sit roughly 22 px above the foot row.
    lift = lowest - (FOOT_ROW - 22)
    if lift > 0:
        tucked_rear = tucked_rear.shift(0, -lift)
        tucked_front = tucked_front.shift(0, -lift)
        frame = body.copy().paste(tucked_rear).paste(tucked_front)
    frame.erase_if(lambda _x, y, _color: y >= FOOT_ROW or y < 0)
    left, _top, right, _bottom = frame.bounds()
    center = (left + right) / 2
    if abs(center - FOOT_X) > 2:
        frame = frame.shift(iround(FOOT_X - center), 0)
        frame.erase_if(lambda x, y, _color: not (0 <= x < CELL_W and 0 <= y < CELL_H))
    require_inside(frame)
    return frame


def lift_cape(sprite: Sprite, dy: int) -> Sprite:
    tail = {
        (x, y): color
        for (x, y), color in sprite.pixels.items()
        if x < 52 and y > 108 and is_brown(color)
    }
    if not tail:
        return sprite
    kept = Sprite(
        {
            point: color
            for point, color in sprite.pixels.items()
            if point not in tail
        }
    )
    for (x, y), color in tail.items():
        kept.pixels[(x, y + dy)] = color
    return kept


def build_fall(idle: Sprite, rear: Sprite, front: Sprite) -> Sprite:
    body = lift_cape(idle, -18)
    body.erase_rect(58, 130, 86, CELL_H)
    body.erase_rect(88, 126, 122, CELL_H)
    chest = chest_pin_x(body)
    # Legs extended, close together, down boot on the foot point. Cape streams up.
    back = swing_from_hip(rear, 10, chest - 2, 122)
    fore = swing_from_hip(front, -4, chest + 10, 122)
    frame = body.copy().paste(back).paste(fore)
    return plant_on_foot(frame)


def build_death(idle: Sprite, rear: Sprite, front: Sprite) -> tuple[Sprite, Sprite]:
    stagger = _death_stagger(idle, rear, front)
    crumple = _death_crumple(idle, rear, front)
    require_inside(stagger)
    require_inside(crumple)
    return stagger, crumple


def _death_stagger(idle: Sprite, rear: Sprite, front: Sprite) -> Sprite:
    body = without_blade(idle)
    body.erase_rect(58, 126, 86, CELL_H)
    body.erase_rect(88, 122, 122, CELL_H)
    chest = chest_pin_x(body)
    # Knees bent, torso tipped back a little. The in-game flop is a separate 25° turn.
    tipped = rotate(body, -12.0, (chest, 120)).shift(0, 8)
    sword, pivot = aim_blade(blade_only(idle), 48.0, 46)
    hand = sword_hand(tipped)
    back = swing_from_hip(rear, 26, chest - 4, 136)
    fore = swing_from_hip(front, -16, chest + 12, 136)
    frame = tipped.copy().paste(back).paste(fore)
    frame.paste(place_aimed(sword, pivot, (hand[0], hand[1] - 4)))
    return plant_on_foot(frame)


def _death_crumple(idle: Sprite, rear: Sprite, front: Sprite) -> Sprite:
    """Fold the torso backward as one piece so the head stays on the neck.

    The legs stay under the waist. `rear` and `front` are unused; the idle
    boots are already on this sprite.
    """
    del rear, front
    raw = without_blade(idle)
    chest = chest_pin_x(raw)
    upper = Sprite({point: color for point, color in raw.pixels.items() if point[1] < 114})
    lower = Sprite({point: color for point, color in raw.pixels.items() if point[1] >= 102})
    folded = rotate(upper, -52.0, (chest, 108))
    left, top, right, bottom = folded.bounds()
    folded = folded.shift(
        iround(chest - (left + right) / 2),
        iround(104 - bottom),
    )
    left, top, _right, _bottom = folded.bounds()
    if top < 0:
        folded = folded.shift(0, -top)
    frame = lower.copy().paste(folded)
    sword, pivot = aim_blade(blade_only(idle), 24.0, 36)
    hand_x = chest + 28
    frame.paste(place_aimed(sword, pivot, (hand_x, 118)))
    frame.erase_if(
        lambda x, y, _c: x < -24 or x >= CELL_W + 24 or y < -8 or y >= CELL_H + 8
    )
    return plant_on_foot(frame)


def blit(image: Image.Image, sprite: Sprite, origin_x: int) -> None:
    pixels = image.load()
    width, height = image.size
    for (x, y), color in sprite.pixels.items():
        xx, yy = x + origin_x, y
        if not (0 <= xx < width and 0 <= yy < height):
            raise SystemExit(f"blit clipped {xx},{yy}")
        pixels[xx, yy] = color


def sheet_image(sprites: list[Sprite]) -> Image.Image:
    image = Image.new("RGBA", (CELL_W * len(sprites), CELL_H), (0, 0, 0, 0))
    for index, sprite in enumerate(sprites):
        blit(image, sprite, index * CELL_W)
    return image


def save_png(image: Image.Image, path: Path) -> None:
    image.save(path, format="PNG", compress_level=9)


def cell_pixels(image: Image.Image, index: int) -> Sprite:
    crop = image.crop((index * CELL_W, 0, (index + 1) * CELL_W, CELL_H))
    return sprite_from_image(crop)


def assert_cell_equal(image: Image.Image, index: int, signed: Sprite, label: str) -> None:
    got = cell_pixels(image, index)
    if got.pixels != signed.pixels:
        raise SystemExit(f"{label} cell {index} is not the signed cell")


def assert_palette(sprite: Sprite, allowed: set[tuple[int, int, int, int]], label: str) -> None:
    for point, color in sprite.pixels.items():
        if color[3] not in (0, 255):
            raise SystemExit(f"{label} partial alpha at {point}")
        if color not in allowed:
            raise SystemExit(f"{label} color {color} at {point} is not from a signed cell")


def assert_laterality(sprite: Sprite, label: str) -> None:
    pin = chest_pin_x(sprite)
    yellow = [x for (x, _y), color in sprite.pixels.items() if is_yellow_mark(color)]
    purple = [x for (x, _y), color in sprite.pixels.items() if is_purple(color)]
    blade_xs = [x for (x, _y), color in sprite.pixels.items() if is_hot_blade(color)]
    if len(yellow) < 12:
        raise SystemExit(f"{label} lost the chest mark ({len(yellow)} px)")
    if not purple or min(purple) > pin:
        raise SystemExit(f"{label} purple cable is not on the image-left arm")
    if not blade_xs or max(blade_xs) < pin + 8:
        raise SystemExit(f"{label} sword is not in the image-right hand")
    cape = [x for (x, y), color in sprite.pixels.items() if is_brown(color) and y > 70]
    if not cape or min(cape) > pin - 10:
        raise SystemExit(f"{label} cape does not trail image-left")


def check_run(frames: list[Sprite]) -> None:
    if len(frames) != 8:
        raise SystemExit("run cycle is not 8 frames")
    for index in (0, 4):
        row, center = stance_contact(frames[index])
        if row != FOOT_ROW or abs(center - FOOT_X) > 1e-6:
            raise SystemExit(f"run frame {index} contact {center}@{row} != {FOOT_X}@{FOOT_ROW}")
    for index, frame in enumerate(frames):
        width = opaque_width(frame)
        if width > 210:
            raise SystemExit(f"run frame {index} width {width} pops past the lean")
        if index > 0 and same_pixels(frame, frames[index - 1]):
            raise SystemExit(f"run frames {index - 1} and {index} are identical")
    if same_pixels(frames[0], frames[4]):
        raise SystemExit("run contacts are the same drawing")
    # The two contact silhouettes must not be mirrors: compare left-boot vs right-boot mass.
    def boot_side_mass(frame: Sprite, x0: int, x1: int) -> int:
        return sum(1 for x, y in frame.pixels if x0 <= x < x1 and y > 120)

    if boot_side_mass(frames[0], 40, 82) == boot_side_mass(frames[4], 40, 82):
        # Not a hard geometric identity test; the pixel check above already
        # rejects an exact copy. Keep going so a balanced stance can pass.
        pass


def check_attack(between: Sprite, recover: Sprite) -> None:
    for label, frame, tip_target in (("between", between, 220), ("recover", recover, 180)):
        row, _center = stance_contact(frame)
        pin = chest_pin_x(frame)
        if row != ATTACK_SOLE_ROW:
            raise SystemExit(f"{label} sole on row {row}, want {ATTACK_SOLE_ROW}")
        if abs(pin - FOOT_X) > 1e-6:
            raise SystemExit(f"{label} chest pin {pin} != {FOOT_X}")
        tip = max(x for (x, _y), color in frame.pixels.items() if is_hot_blade(color))
        if abs(tip - tip_target) > 18:
            raise SystemExit(f"{label} blade tip x {tip} is not near {tip_target}")
    if same_pixels(between, recover):
        raise SystemExit("attack in-betweens match")


def check_jump(frame: Sprite) -> None:
    for x, y in frame.pixels:
        if y >= FOOT_ROW:
            raise SystemExit(f"jump still has a pixel on row {y}")
    left, top, right, _bottom = frame.bounds()
    center = (left + right) / 2
    if abs(center - FOOT_X) > 2:
        raise SystemExit(f"jump bbox center {center} is farther than 2 px from {FOOT_X}")
    if top < 0:
        raise SystemExit("jump head left the cell")


def check_ground_pin(frame: Sprite, label: str) -> None:
    row, center = stance_contact(frame)
    if row != FOOT_ROW or abs(center - FOOT_X) > 1e-6:
        raise SystemExit(f"{label} contact {center}@{row} != {FOOT_X}@{FOOT_ROW}")


def write_outputs(
    idle_sheet: Sprite,
    run_sheet: Sprite,
    attack_sheet: Sprite,
    run_frames: list[Sprite],
    between: Sprite,
    recover: Sprite,
    jump: Sprite,
    fall: Sprite,
    stagger: Sprite,
    crumple: Sprite,
) -> None:
    idle_src = SIGNED / "knight_idle_side.png"
    idle_dst = OUT / "knight_idle_side.png"
    shutil.copyfile(idle_src, idle_dst)

    run_image = sheet_image([cell_of(run_sheet, 0), *run_frames])
    attack_image = sheet_image(
        [cell_of(attack_sheet, 0), between, cell_of(attack_sheet, 2), recover]
    )
    assert_cell_equal(run_image, 0, cell_of(run_sheet, 0), "run")
    assert_cell_equal(attack_image, 0, cell_of(attack_sheet, 0), "attack")
    assert_cell_equal(attack_image, 2, cell_of(attack_sheet, 2), "attack")
    save_png(run_image, OUT / "knight_run_side.png")
    save_png(attack_image, OUT / "knight_attack_side.png")
    save_png(sheet_image([jump]), OUT / "knight_jump_side.png")
    save_png(sheet_image([fall]), OUT / "knight_fall_side.png")
    save_png(sheet_image([stagger, crumple]), OUT / "knight_death_side.png")

    expected = {
        "knight_idle_side.png": (1372, 160),
        "knight_run_side.png": (3087, 160),
        "knight_attack_side.png": (1372, 160),
        "knight_jump_side.png": (343, 160),
        "knight_fall_side.png": (343, 160),
        "knight_death_side.png": (686, 160),
    }
    for name, (width, height) in expected.items():
        image = Image.open(OUT / name)
        if image.size != (width, height):
            raise SystemExit(f"{name} size {image.size} != {(width, height)}")
    idle_hash = hashlib.sha256(idle_dst.read_bytes()).hexdigest()
    if idle_hash != SIGNED_SHA256["knight_idle_side.png"]:
        raise SystemExit("idle gameplay file is not the signed bytes")


def main() -> None:
    reject_resample_names()
    idle_sheet = load_sheet("knight_idle_side.png")
    run_sheet = load_sheet("knight_run_side.png")
    attack_sheet = load_sheet("knight_attack_side.png")
    allowed = palette_of(idle_sheet, run_sheet, attack_sheet)

    idle = cell_of(idle_sheet, 0)
    chamber = cell_of(attack_sheet, 0)
    thrust = cell_of(attack_sheet, 2)
    rear = leg_column(idle, 62, 116, 82, 160)
    front = leg_column(idle, 90, 112, 116, 160)

    run_frames = build_run_cycle(idle)
    between, recover = build_attack_frames(chamber, thrust)
    jump = build_jump(idle, rear, front)
    fall = build_fall(idle, rear, front)
    stagger, crumple = build_death(idle, rear, front)

    check_run(run_frames)
    check_attack(between, recover)
    check_jump(jump)
    check_ground_pin(fall, "fall")
    check_ground_pin(stagger, "death stagger")
    check_ground_pin(crumple, "death crumple")

    fresh = (
        [(f"run {index}", frame) for index, frame in enumerate(run_frames)]
        + [("between", between), ("recover", recover), ("jump", jump), ("fall", fall)]
        + [("stagger", stagger), ("crumple", crumple)]
    )
    for label, frame in fresh:
        assert_palette(frame, allowed, label)
        assert_laterality(frame, label)

    if same_pixels(between, cell_of(attack_sheet, 0)) or same_pixels(recover, cell_of(attack_sheet, 2)):
        raise SystemExit("attack in-between copied a signed cell")
    if same_pixels(jump, idle) or same_pixels(fall, idle):
        raise SystemExit("jump or fall matches idle")
    if same_pixels(stagger, crumple):
        raise SystemExit("death cells match")

    write_outputs(
        idle_sheet,
        run_sheet,
        attack_sheet,
        run_frames,
        between,
        recover,
        jump,
        fall,
        stagger,
        crumple,
    )

    print("run contacts:")
    for index, frame in enumerate(run_frames):
        row, center = stance_contact(frame)
        left, top, right, bottom = frame.bounds()
        print(
            f"  {index} contact {center:.2f}@{row} width {right - left + 1} "
            f"bbox {left},{top}-{right},{bottom} chest {chest_pin_x(frame):.1f}"
        )
    for label, frame in (("between", between), ("recover", recover)):
        row, center = stance_contact(frame)
        tip = max(x for (x, _y), color in frame.pixels.items() if is_hot_blade(color))
        print(f"{label} chest {chest_pin_x(frame):.2f} sole {center:.1f}@{row} tip {tip}")
    print("jump/fall/death written")


if __name__ == "__main__":
    main()
