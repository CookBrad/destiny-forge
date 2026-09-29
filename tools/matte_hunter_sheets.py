#!/usr/bin/env python3
"""Clear the opaque black matte on the hunter side sheets.

Taste-signed character pixels stay byte-identical. Per 343×160 cell the key is:

1. Flood-fill opaque pure black inward from the cell border (4-connected).
2. Clear specks that never touch the painted figure (every channel <= 1).
3. Clear one more layer of that fringe where it touches the matte.
4. Clear specks the peel left detached. Those specks never reach channel 2.

A tolerance flood is not used. The duster, boots, and outlines are near-black,
and pure black inside the coat is part of the figure. Walking through channel
<= 1 from the border tunnels into that interior. The fringe removed here is
only the layer that actually touches the matte.

Cleared pixels are transparent black. The fill walks opaque black only, so a
second run cannot cross those pixels into the figure.
"""

from __future__ import annotations

import subprocess
import sys
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path

from PIL import Image

CELL_WIDTH = 343
CELL_HEIGHT = 160
FRAME_COUNT = 4
SHEET_WIDTH = CELL_WIDTH * FRAME_COUNT
SHEET_HEIGHT = CELL_HEIGHT
# (1, 0, 0) and (1, 1, 1) sit on the matte. Painted outlines start at channel 2.
FRINGE_CHANNEL_MAX = 1

ROOT = Path(__file__).resolve().parents[1]
SHEETS = (
    Path("assets/player/combat/knight_attack_side.png"),
    Path("assets/player/combat/knight_idle_side.png"),
    Path("assets/player/combat/knight_run_side.png"),
)

TRANSPARENT = (0, 0, 0, 0)
Pixel = tuple[int, int, int, int]


@dataclass(frozen=True)
class CellBounds:
    frame: int
    width: int
    height: int
    x: int
    y: int


@dataclass(frozen=True)
class SheetReport:
    relative: Path
    border_black: int
    speckle: int
    fringe: int
    character: int
    changed_character: int
    cells: tuple[CellBounds, ...]

    @property
    def background(self) -> int:
        return self.border_black + self.speckle + self.fringe


def max_channel(pixel: Pixel) -> int:
    return max(pixel[0], pixel[1], pixel[2])


def opaque_black(pixel: Pixel) -> bool:
    return pixel[3] == 255 and pixel[0] == 0 and pixel[1] == 0 and pixel[2] == 0


def load_rgba(path: Path) -> list[Pixel]:
    image = Image.open(path).convert("RGBA")
    if image.size != (SHEET_WIDTH, SHEET_HEIGHT):
        raise SystemExit(f"{path} is {image.size}, expected {(SHEET_WIDTH, SHEET_HEIGHT)}")
    return list(image.getdata())


def neighbors(x: int, y: int) -> tuple[tuple[int, int], ...]:
    found = []
    if x > 0:
        found.append((x - 1, y))
    if x + 1 < CELL_WIDTH:
        found.append((x + 1, y))
    if y > 0:
        found.append((x, y - 1))
    if y + 1 < CELL_HEIGHT:
        found.append((x, y + 1))
    return tuple(found)


def flood_opaque_black(pixels: list[Pixel], origin_x: int) -> bytearray:
    """Opaque pure black reachable from this cell's border. Interior black stays out."""
    background = bytearray(CELL_WIDTH * CELL_HEIGHT)
    stack: list[tuple[int, int]] = []

    def consider(x: int, y: int) -> None:
        index = y * CELL_WIDTH + x
        if background[index]:
            return
        if not opaque_black(pixels[y * SHEET_WIDTH + origin_x + x]):
            return
        background[index] = 1
        stack.append((x, y))

    for x in range(CELL_WIDTH):
        consider(x, 0)
        consider(x, CELL_HEIGHT - 1)
    for y in range(CELL_HEIGHT):
        consider(0, y)
        consider(CELL_WIDTH - 1, y)

    while stack:
        x, y = stack.pop()
        for nx, ny in neighbors(x, y):
            consider(nx, ny)
    return background


def connected_group(
    pixels: list[Pixel],
    origin_x: int,
    blocked: bytearray,
    seen: bytearray,
    x: int,
    y: int,
) -> tuple[list[tuple[int, int]], int]:
    group: list[tuple[int, int]] = []
    brightest = 0
    stack = [(x, y)]
    seen[y * CELL_WIDTH + x] = 1
    while stack:
        cx, cy = stack.pop()
        group.append((cx, cy))
        pixel = pixels[cy * SHEET_WIDTH + origin_x + cx]
        brightest = max(brightest, max_channel(pixel))
        for nx, ny in neighbors(cx, cy):
            index = ny * CELL_WIDTH + nx
            if blocked[index] or seen[index]:
                continue
            seen[index] = 1
            stack.append((nx, ny))
    return group, brightest


def mark_speckle(pixels: list[Pixel], origin_x: int, background: bytearray) -> int:
    """Components that never reach a painted pixel are matte dust, not the figure."""
    seen = bytearray(CELL_WIDTH * CELL_HEIGHT)
    marked = 0
    for y in range(CELL_HEIGHT):
        for x in range(CELL_WIDTH):
            start = y * CELL_WIDTH + x
            if background[start] or seen[start]:
                continue
            group, brightest = connected_group(pixels, origin_x, background, seen, x, y)
            if brightest > FRINGE_CHANNEL_MAX:
                continue
            for cx, cy in group:
                background[cy * CELL_WIDTH + cx] = 1
            marked += len(group)
    return marked


def mark_fringe(pixels: list[Pixel], origin_x: int, background: bytearray) -> int:
    """One layer of channel-<=1 pixels touching the matte. Do not walk further in."""
    fringe: list[int] = []
    for y in range(CELL_HEIGHT):
        for x in range(CELL_WIDTH):
            index = y * CELL_WIDTH + x
            if background[index]:
                continue
            pixel = pixels[y * SHEET_WIDTH + origin_x + x]
            if max_channel(pixel) > FRINGE_CHANNEL_MAX:
                continue
            if any(background[ny * CELL_WIDTH + nx] for nx, ny in neighbors(x, y)):
                fringe.append(index)
    for index in fringe:
        background[index] = 1
    return len(fringe)


def assert_figure_kept(
    pixels: list[Pixel],
    origin_x: int,
    blocked: bytearray,
    mask: bytearray,
) -> None:
    """Paint stays. Pure black that still reaches that paint stays with it."""
    seen = bytearray(CELL_WIDTH * CELL_HEIGHT)
    for y in range(CELL_HEIGHT):
        for x in range(CELL_WIDTH):
            index = y * CELL_WIDTH + x
            pixel = pixels[y * SHEET_WIDTH + origin_x + x]
            if mask[index] and max_channel(pixel) > FRINGE_CHANNEL_MAX:
                raise SystemExit("matte mask includes a painted pixel")
            if blocked[index] or seen[index]:
                continue
            group, brightest = connected_group(pixels, origin_x, blocked, seen, x, y)
            if brightest <= FRINGE_CHANNEL_MAX:
                continue
            for cx, cy in group:
                kept = pixels[cy * SHEET_WIDTH + origin_x + cx]
                if opaque_black(kept) and mask[cy * CELL_WIDTH + cx]:
                    raise SystemExit("matte mask includes pure black inside the figure")


def cell_mask(pixels: list[Pixel], origin_x: int) -> tuple[bytearray, int, int, int]:
    border = flood_opaque_black(pixels, origin_x)
    border_count = sum(border)
    mask = bytearray(border)
    speckle = mark_speckle(pixels, origin_x, mask)
    fringe = mark_fringe(pixels, origin_x, mask)
    # Border, matte dust, and the one fringe layer. Black that still reaches
    # paint through what remains is inside the figure, not a detached speck.
    blocked = bytearray(mask)
    speckle += mark_speckle(pixels, origin_x, mask)
    assert_figure_kept(pixels, origin_x, blocked, mask)
    return mask, border_count, speckle, fringe


def apply_mask(
    source: list[Pixel],
    cleared: list[Pixel],
    origin_x: int,
    mask: bytearray,
) -> tuple[int, int]:
    """Copy the cell. Return unchanged character pixels and changed non-background pixels."""
    unchanged = 0
    changed = 0
    for y in range(CELL_HEIGHT):
        for x in range(CELL_WIDTH):
            index = y * SHEET_WIDTH + origin_x + x
            original = source[index]
            if mask[y * CELL_WIDTH + x]:
                cleared[index] = TRANSPARENT
                continue
            cleared[index] = original
            if cleared[index] == original:
                unchanged += 1
            else:
                changed += 1
    return unchanged, changed


def opaque_bounds(pixels: list[Pixel], origin_x: int, frame: int) -> CellBounds:
    min_x, min_y = CELL_WIDTH, CELL_HEIGHT
    max_x, max_y = -1, -1
    for y in range(CELL_HEIGHT):
        row = y * SHEET_WIDTH + origin_x
        for x in range(CELL_WIDTH):
            if pixels[row + x][3] == 0:
                continue
            min_x = min(min_x, x)
            min_y = min(min_y, y)
            max_x = max(max_x, x)
            max_y = max(max_y, y)
    if max_x < 0:
        raise SystemExit(f"frame {frame} has no opaque pixels")
    return CellBounds(frame, max_x - min_x + 1, max_y - min_y + 1, min_x, min_y)


def clear_pixels(source: list[Pixel]) -> tuple[list[Pixel], SheetReport]:
    cleared = list(source)
    border_black = 0
    speckle = 0
    fringe = 0
    character = 0
    changed = 0
    cells: list[CellBounds] = []
    for frame in range(FRAME_COUNT):
        origin_x = frame * CELL_WIDTH
        mask, frame_border, frame_speckle, frame_fringe = cell_mask(source, origin_x)
        frame_character, frame_changed = apply_mask(source, cleared, origin_x, mask)
        border_black += frame_border
        speckle += frame_speckle
        fringe += frame_fringe
        character += frame_character
        changed += frame_changed
        cells.append(opaque_bounds(cleared, origin_x, frame))
    report = SheetReport(
        Path(""),
        border_black,
        speckle,
        fringe,
        character,
        changed,
        tuple(cells),
    )
    return cleared, report


def save_rgba(path: Path, pixels: list[Pixel]) -> None:
    image = Image.new("RGBA", (SHEET_WIDTH, SHEET_HEIGHT))
    image.putdata(pixels)
    image.save(path, format="PNG")


def clear_sheet(relative: Path) -> SheetReport:
    path = ROOT / relative
    source = load_rgba(path)
    cleared, report = clear_pixels(source)
    again, _ = clear_pixels(cleared)
    if again != cleared:
        raise SystemExit(f"{relative} is not idempotent")
    if cleared != source:
        save_rgba(path, cleared)
        if load_rgba(path) != cleared:
            raise SystemExit(f"{relative} did not round-trip")
    return SheetReport(
        relative,
        report.border_black,
        report.speckle,
        report.fringe,
        report.character,
        report.changed_character,
        report.cells,
    )


def original_from_head(relative: Path) -> list[Pixel]:
    shown = subprocess.run(
        ["git", "show", f"HEAD:{relative.as_posix()}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )
    image = Image.open(BytesIO(shown.stdout)).convert("RGBA")
    if image.size != (SHEET_WIDTH, SHEET_HEIGHT):
        raise SystemExit(f"HEAD:{relative} is {image.size}")
    return list(image.getdata())


def changed_character_pixels(
    original: list[Pixel],
    on_disk: list[Pixel],
    expected: list[Pixel],
) -> int:
    changed = 0
    for src, disk, want in zip(original, on_disk, expected):
        if want == TRANSPARENT:
            continue
        if disk != src:
            changed += 1
    return changed


def verify_sheet(relative: Path) -> SheetReport:
    """Diff the working tree against the matte of HEAD. Non-background pixels must match HEAD."""
    original = original_from_head(relative)
    on_disk = load_rgba(ROOT / relative)
    expected, report = clear_pixels(original)
    changed = changed_character_pixels(original, on_disk, expected)
    if on_disk != expected or changed != 0:
        raise SystemExit(f"{relative} verify failed: changed non-background {changed}")
    cells = tuple(opaque_bounds(on_disk, frame * CELL_WIDTH, frame) for frame in range(FRAME_COUNT))
    return SheetReport(
        relative,
        report.border_black,
        report.speckle,
        report.fringe,
        report.character,
        changed,
        cells,
    )


def print_report(report: SheetReport) -> None:
    print(report.relative.as_posix())
    print(f"  background px made transparent: {report.background}")
    print(f"    border-connected pure black: {report.border_black}")
    print(f"    disconnected matte speckle: {report.speckle}")
    print(f"    fringe touching the matte: {report.fringe}")
    print(f"  character px unchanged: {report.character}")
    print(f"  changed non-background px: {report.changed_character}")
    for cell in report.cells:
        print(
            f"  cell {cell.frame}: {cell.width}x{cell.height} at ({cell.x}, {cell.y})"
        )


def main(argv: list[str]) -> int:
    verify = argv[1:] == ["--verify"]
    if argv[1:] and not verify:
        raise SystemExit("usage: matte_hunter_sheets.py [--verify]")
    for relative in SHEETS:
        report = verify_sheet(relative) if verify else clear_sheet(relative)
        print_report(report)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
