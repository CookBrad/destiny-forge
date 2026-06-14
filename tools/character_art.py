"""Build the 16x32 Stardew-style player sheet from AI-generated art."""

from __future__ import annotations

from PIL import Image

from ai_art_common import cleanup_pixels, load_clean, lowest_opaque_row

FRAME_W = 16
FRAME_H = 32
WALK_FRAMES = 4
GRID_COLS = 4
GRID_ROWS = 5
GROUND_Y = 31
PAD_TOP = 6
PAD_BOTTOM = 18
PAD_X = 8

# AI sheet is 5 rows x 4 cols (not 6). Each row is a full walk cycle.
ROW_DOWN = 4
ROW_RIGHT = 1
ROW_UP = 2
ROW_LEFT = 0


def cell_size(img: Image.Image) -> tuple[int, int]:
    return img.width // GRID_COLS, img.height // GRID_ROWS


def crop_cell(img: Image.Image, row: int, col: int) -> Image.Image:
    cell_w, cell_h = cell_size(img)
    return img.crop((col * cell_w, row * cell_h, (col + 1) * cell_w, (row + 1) * cell_h))


def padded_crop(cell: Image.Image) -> Image.Image:
    bbox = cell.getbbox()
    if not bbox:
        return Image.new("RGBA", (1, 1), (0, 0, 0, 0))

    x0, y0, x1, y1 = bbox
    x0 = max(0, x0 - PAD_X)
    y0 = max(0, y0 - PAD_TOP)
    x1 = min(cell.width - 1, x1 + PAD_X)
    y1 = min(cell.height - 1, y1 + PAD_BOTTOM)
    return cell.crop((x0, y0, x1 + 1, y1 + 1))


def fit_frame_to_ground(crop: Image.Image, ground_y: int = GROUND_Y) -> Image.Image:
    frame = Image.new("RGBA", (FRAME_W, FRAME_H), (0, 0, 0, 0))
    bbox = crop.getbbox()
    if not bbox:
        return frame

    cropped = crop.crop(bbox)

    scale = min(FRAME_W / cropped.width, FRAME_H / cropped.height)
    new_w = max(1, int(cropped.width * scale))
    new_h = max(1, int(cropped.height * scale))
    resized = cropped.resize((new_w, new_h), Image.Resampling.NEAREST)

    foot = lowest_opaque_row(resized)
    oy = ground_y - foot
    if oy < 0:
        scale = min(scale, (ground_y + 1) / cropped.height)
        new_w = max(1, int(cropped.width * scale))
        new_h = max(1, int(cropped.height * scale))
        resized = cropped.resize((new_w, new_h), Image.Resampling.NEAREST)
        foot = lowest_opaque_row(resized)
        oy = ground_y - foot

    ox = (FRAME_W - new_w) // 2
    frame.paste(resized, (ox, oy))
    return cleanup_pixels(frame)


def mirror_frame(frame: Image.Image) -> Image.Image:
    return frame.transpose(Image.Transpose.FLIP_LEFT_RIGHT)


def extract_direction_row(img: Image.Image, row: int) -> list[Image.Image]:
    """Extract 4 frames from one sheet row, anchoring each frame's feet to GROUND_Y."""
    crops = [padded_crop(crop_cell(img, row, col)) for col in range(WALK_FRAMES)]
    return [fit_frame_to_ground(crop) for crop in crops]


def build_from_ai_grid(img: Image.Image) -> Image.Image:
    sheet = Image.new("RGBA", (64, 128), (0, 0, 0, 0))
    direction_rows = [ROW_DOWN, ROW_RIGHT, ROW_UP, ROW_LEFT]

    for dest_row, src_row in enumerate(direction_rows):
        frames = extract_direction_row(img, src_row)
        for col, frame in enumerate(frames):
            sheet.paste(frame, (col * FRAME_W, dest_row * FRAME_H))

    return sheet


def build_original_player_sheet() -> Image.Image:
    img = load_clean("player")
    return build_from_ai_grid(img)