"""Shared helpers for processing AI-generated pixel art into game assets."""

from __future__ import annotations

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "source"


def raw_path(name: str) -> Path:
    return SOURCE / f"ai_{name}_raw.jpg"


def clean_path(name: str) -> Path:
    return SOURCE / f"ai_{name}_clean.png"


def strip_background(img: Image.Image, *, opaque: bool = False) -> Image.Image:
    px = img.load()
    width, height = img.size
    for y in range(height):
        for x in range(width):
            r, g, b, a = px[x, y]
            if a < 20:
                continue
            if opaque:
                if abs(r - g) < 18 and abs(g - b) < 18 and 110 < r < 210:
                    px[x, y] = (0, 0, 0, 0)
                continue
            if r > 210 and g > 210 and b > 210:
                px[x, y] = (0, 0, 0, 0)
            elif abs(r - g) < 18 and abs(g - b) < 18 and r > 110:
                px[x, y] = (0, 0, 0, 0)
    return img


def cleanup_pixels(frame: Image.Image, alpha_cutoff: int = 40) -> Image.Image:
    px = frame.load()
    width, height = frame.size
    for y in range(height):
        for x in range(width):
            r, g, b, a = px[x, y]
            if a < alpha_cutoff or r + g + b > 720:
                px[x, y] = (0, 0, 0, 0)
    return frame


def lowest_opaque_row(image: Image.Image) -> int:
    px = image.load()
    width, height = image.size
    for y in range(height - 1, -1, -1):
        for x in range(width):
            if px[x, y][3] > 30:
                return y
    return height - 1


def prepare_raw(name: str, *, opaque: bool = False) -> Image.Image:
    path = raw_path(name)
    if not path.exists():
        raise FileNotFoundError(f"Missing AI source: {path}")

    img = strip_background(Image.open(path).convert("RGBA"), opaque=opaque)
    if not opaque:
        bbox = img.getbbox()
        if bbox:
            img = img.crop(bbox)

    SOURCE.mkdir(parents=True, exist_ok=True)
    img.save(clean_path(name))
    return img


def load_clean(name: str, *, opaque: bool = False) -> Image.Image:
    clean = clean_path(name)
    if clean.exists():
        return Image.open(clean).convert("RGBA")
    return prepare_raw(name, opaque=opaque)


def fit_to_canvas(
    crop: Image.Image,
    width: int,
    height: int,
    *,
    ground_y: int | None = None,
    pad: tuple[int, int, int, int] = (0, 0, 0, 0),
) -> Image.Image:
    frame = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    bbox = crop.getbbox()
    if not bbox:
        return frame

    x0, y0, x1, y1 = bbox
    pl, pt, pr, pb = pad
    x0 = max(0, x0 - pl)
    y0 = max(0, y0 - pt)
    x1 = min(crop.width - 1, x1 + pr)
    y1 = min(crop.height - 1, y1 + pb)
    cropped = crop.crop((x0, y0, x1 + 1, y1 + 1))

    scale = min(width / cropped.width, height / cropped.height)
    new_w = max(1, int(cropped.width * scale))
    new_h = max(1, int(cropped.height * scale))
    resized = cropped.resize((new_w, new_h), Image.Resampling.NEAREST)

    if ground_y is None:
        ox = (width - new_w) // 2
        oy = (height - new_h) // 2
    else:
        foot = lowest_opaque_row(resized)
        oy = ground_y - foot
        if oy < 0:
            scale = min(scale, (ground_y + 1) / cropped.height)
            new_w = max(1, int(cropped.width * scale))
            new_h = max(1, int(cropped.height * scale))
            resized = cropped.resize((new_w, new_h), Image.Resampling.NEAREST)
            foot = lowest_opaque_row(resized)
            oy = ground_y - foot
        ox = (width - new_w) // 2

    frame.paste(resized, (ox, oy))
    return cleanup_pixels(frame)


def resize_exact(img: Image.Image, width: int, height: int) -> Image.Image:
    if img.mode != "RGBA":
        img = img.convert("RGBA")
    return img.resize((width, height), Image.Resampling.NEAREST)


def find_tile_band(img: Image.Image, cols: int) -> Image.Image:
    width, height = img.size
    target_h = max(16, width // cols)
    best: tuple[int, int, int, int] | None = None
    best_score = 0

    for y0 in range(0, max(1, height - target_h), 8):
        y1 = min(height, y0 + int(target_h * 1.6))
        band = img.crop((0, y0, width, y1))
        bbox = band.getbbox()
        if not bbox:
            continue
        score = (bbox[2] - bbox[0]) * (bbox[3] - bbox[1])
        if score > best_score:
            best_score = score
            best = (bbox[0], y0 + bbox[1], bbox[2], y0 + bbox[3])

    if best is None:
        return img

    return img.crop(best)


def slice_tile_sheet(
    img: Image.Image,
    cols: int,
    *,
    tile_size: int = 16,
    sheet_width: int,
    sheet_height: int = 64,
) -> Image.Image:
    band = find_tile_band(img, cols)
    sheet = Image.new("RGBA", (sheet_width, sheet_height), (0, 0, 0, 0))
    cell_w = band.width // cols
    cell_h = band.height

    for col in range(cols):
        cell = band.crop((col * cell_w, 0, (col + 1) * cell_w, cell_h))
        tile = fit_to_canvas(cell, tile_size, tile_size, ground_y=tile_size - 1)
        sheet.paste(tile, (col * tile_size, 0))

    return sheet