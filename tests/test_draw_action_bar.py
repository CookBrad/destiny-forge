from __future__ import annotations

import hashlib

import pytest
from PIL import Image

import draw_action_bar as action_bar
from art_common import rejecting_canvas

FRAME_PIXEL_DIGEST = "1484f7b01e1e6b3f3de5bf9e50f908821308d52ac0522fdc98a478cdb65b62bd"
HIGHLIGHT_PIXEL_DIGEST = "3fff2e88ecbcc29c3ab880ee2d56c505730ad1bb940f0405decd459853fb15ad"
SLOT_PIXEL_DIGESTS = (
    "c660cd8a641e16966f872c203e860a222f0b05d1a950116061820ca027e541d6",
    "4bd5b609d7c183008b037d070560f06aeac7a26c7b98a048fc276f2c6f22bf2c",
    "396f17cf976c864e2d36680caba2a24de1d9ffa26e7d2e4dc024251f665fc771",
    "901a90e90eea838ea4c7421e4fb9f7e5770bface640b0246ef5f614e69a6ba7e",
    "2e4af19186cd5ac9bb415dd0795334935193144e93c8b0a46b3c1d640d28d602",
    "281689333084e4707dab98edb01469ce6ae2bef72fcc195e71928774353bf7a0",
    "88c308deaede69aad40d1b0123f203306c157cf70df220ddaac0d5a64c864e96",
    "c31f9f61695d4aa3005f1c29b005ae461efd4383280d9675c95cefb242c3df4f",
)
PARCHMENT_PIXEL_DIGEST = "a055e08e740e9d182fdcf22e0612e7107814042bc74a16943405bc416353b14b"
SLOT_ONE_INK_PIXEL_DIGEST = "79d1f0aa2a53589498581c114d09134f66b6b92b002784785069f02b21b5b5ba"
ACTION_BAR_FILE_NAMES = (
    "frame.png",
    "highlight.png",
    "slot_1.png",
    "slot_2.png",
    "slot_3.png",
    "slot_4.png",
    "slot_5.png",
    "slot_6.png",
    "slot_7.png",
    "slot_8.png",
)
WOOD = (139, 90, 48, 255)
WOOD_LIGHT = (186, 132, 78, 255)
WOOD_DARK = (86, 52, 28, 255)
GRAIN_DARK = (112, 70, 36, 255)
GRAIN_MID = (124, 80, 42, 255)
GRAIN_LIGHT = (154, 102, 56, 255)
KNOT = (96, 58, 30, 255)
FRAME_COLORS = {WOOD, WOOD_LIGHT, WOOD_DARK, GRAIN_DARK, GRAIN_MID, GRAIN_LIGHT, KNOT}
PARCHMENT = (244, 232, 206, 255)
PARCHMENT_SHADOW = (214, 196, 164, 255)
HIGHLIGHT = (220, 78, 36, 255)
INK = (62, 38, 24, 255)
CLEAR = (0, 0, 0, 0)


def _image_digest(image: Image.Image) -> str:
    return hashlib.sha256(image.tobytes()).hexdigest()


def _colors(image: Image.Image) -> set:
    width, height = image.size
    return {
        image.getpixel((column, row))
        for row in range(height)
        for column in range(width)
    }


def _read_that_rejects_pixels_off_the_canvas(canvas):
    original_read = canvas.read

    def read(column, row):
        if column < 0 or row < 0 or column >= canvas.width or row >= canvas.height:
            raise AssertionError("read off the canvas")
        return original_read(column, row)

    return read


def test_main_writes_the_action_bar_images_with_the_locked_pixel_digests(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "FOLDER", tmp_path)
    action_bar.main()
    written_names = sorted(path.name for path in tmp_path.iterdir())
    assert written_names == sorted(ACTION_BAR_FILE_NAMES)
    frame = Image.open(tmp_path / "frame.png").convert("RGBA")
    highlight = Image.open(tmp_path / "highlight.png").convert("RGBA")
    slots = [
        Image.open(tmp_path / f"slot_{slot_number}.png").convert("RGBA")
        for slot_number in action_bar.SLOT_NUMBERS
    ]
    assert frame.size == (498, 78)
    assert frame.mode == "RGBA"
    assert highlight.size == (54, 54)
    assert highlight.mode == "RGBA"
    assert _image_digest(frame) == FRAME_PIXEL_DIGEST
    assert _image_digest(highlight) == HIGHLIGHT_PIXEL_DIGEST
    assert [_image_digest(slot) for slot in slots] == list(SLOT_PIXEL_DIGESTS)
    for slot in slots:
        assert slot.size == (54, 54)
        assert slot.mode == "RGBA"
    pixel_digests = [
        _image_digest(frame),
        _image_digest(highlight),
        *(_image_digest(slot) for slot in slots),
    ]
    assert len(set(pixel_digests)) == 10


def test_main_with_no_slot_numbers_writes_the_frame_and_the_highlight(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "FOLDER", tmp_path)
    monkeypatch.setattr(action_bar, "SLOT_NUMBERS", ())
    action_bar.main()
    assert sorted(path.name for path in tmp_path.iterdir()) == ["frame.png", "highlight.png"]
    frame = Image.open(tmp_path / "frame.png").convert("RGBA")
    highlight = Image.open(tmp_path / "highlight.png").convert("RGBA")
    assert _image_digest(frame) == FRAME_PIXEL_DIGEST
    assert _image_digest(highlight) == HIGHLIGHT_PIXEL_DIGEST


def test_main_rejects_a_folder_path_that_is_a_file(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "action_bar"
    folder.write_text("not a folder")
    monkeypatch.setattr(action_bar, "FOLDER", folder)
    with pytest.raises(FileExistsError):
        action_bar.main()


def test_main_rejects_duplicate_images_before_writing(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "FOLDER", tmp_path)
    highlight = action_bar.draw_highlight()
    monkeypatch.setattr(action_bar, "slot_images", lambda: [("slot_1.png", highlight)])
    with pytest.raises(ValueError, match="action bar images are not unique"):
        action_bar.main()
    assert list(tmp_path.iterdir()) == []


def test_action_bar_images_returns_the_frame_the_highlight_and_eight_slots() -> None:
    named_images = action_bar.action_bar_images()
    file_names = [file_name for file_name, _image in named_images]
    images = [image for _file_name, image in named_images]
    pixel_digests = [_image_digest(image) for image in images]
    assert file_names == list(ACTION_BAR_FILE_NAMES)
    assert pixel_digests == [FRAME_PIXEL_DIGEST, HIGHLIGHT_PIXEL_DIGEST, *SLOT_PIXEL_DIGESTS]
    assert len(set(pixel_digests)) == 10
    assert images[0].size == (498, 78)
    assert images[1].size == (54, 54)
    for image in images[2:]:
        assert image.size == (54, 54)
        assert image.mode == "RGBA"


def test_action_bar_images_with_no_slot_numbers_returns_the_frame_and_the_highlight(monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "SLOT_NUMBERS", ())
    named_images = action_bar.action_bar_images()
    assert [file_name for file_name, _image in named_images] == ["frame.png", "highlight.png"]
    assert _image_digest(named_images[0][1]) == FRAME_PIXEL_DIGEST
    assert _image_digest(named_images[1][1]) == HIGHLIGHT_PIXEL_DIGEST
    assert named_images[0][1].tobytes() != named_images[1][1].tobytes()


def test_action_bar_images_rejects_a_duplicate_slot(monkeypatch) -> None:
    highlight = action_bar.draw_highlight()
    monkeypatch.setattr(action_bar, "slot_images", lambda: [("slot_1.png", highlight)])
    with pytest.raises(ValueError, match="action bar images are not unique"):
        action_bar.action_bar_images()


def test_the_action_bar_images_differ_from_each_other() -> None:
    named_images = action_bar.action_bar_images()
    images = [image for _file_name, image in named_images]
    pixel_digests = [_image_digest(image) for image in images]
    assert pixel_digests == [FRAME_PIXEL_DIGEST, HIGHLIGHT_PIXEL_DIGEST, *SLOT_PIXEL_DIGESTS]
    assert len(set(pixel_digests)) == 10
    for left in range(len(images)):
        for right in range(left + 1, len(images)):
            assert images[left].tobytes() != images[right].tobytes()
    assert images[0].size == (498, 78)
    for image in images:
        assert image.mode == "RGBA"
        raw_pixels = image.tobytes()
        assert len(raw_pixels) == image.size[0] * image.size[1] * 4
        for index in range(0, len(raw_pixels), 4):
            pixel = raw_pixels[index:index + 4]
            assert pixel[3] in (0, 255)
            if pixel[3] == 0:
                assert pixel == b"\x00\x00\x00\x00"


def test_slot_images_returns_eight_slots_with_the_locked_pixel_digests() -> None:
    named_images = action_bar.slot_images()
    assert [file_name for file_name, _image in named_images] == [
        "slot_1.png",
        "slot_2.png",
        "slot_3.png",
        "slot_4.png",
        "slot_5.png",
        "slot_6.png",
        "slot_7.png",
        "slot_8.png",
    ]
    pixel_digests = [_image_digest(image) for _file_name, image in named_images]
    assert pixel_digests == list(SLOT_PIXEL_DIGESTS)
    assert len(set(pixel_digests)) == 8
    for _file_name, image in named_images:
        assert image.size == (54, 54)
        assert image.mode == "RGBA"
        assert _colors(image) == {PARCHMENT, PARCHMENT_SHADOW, INK}


def test_slot_images_with_no_slot_numbers_returns_nothing(monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "SLOT_NUMBERS", ())
    assert action_bar.slot_images() == []


def test_slot_images_rejects_a_slot_number_of_nine(monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "SLOT_NUMBERS", (1, 9))
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_images()


def test_slot_file_name_uses_the_slot_number() -> None:
    assert action_bar.slot_file_name(2) == "slot_2.png"
    assert action_bar.slot_file_name(4) == "slot_4.png"
    assert action_bar.slot_file_name(7) == "slot_7.png"


def test_slot_file_name_spans_the_first_and_last_slots() -> None:
    assert action_bar.slot_file_name(1) == "slot_1.png"
    assert action_bar.slot_file_name(8) == "slot_8.png"


def test_slot_file_name_rejects_zero_nine_and_bools() -> None:
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_file_name(0)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_file_name(9)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_file_name(True)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_file_name(False)


def test_require_unique_action_bar_images_accepts_the_ten_drawn_images() -> None:
    action_bar.require_unique_action_bar_images(action_bar.action_bar_images())


def test_require_unique_action_bar_images_accepts_an_empty_list_and_one_image() -> None:
    action_bar.require_unique_action_bar_images([])
    frame = action_bar.draw_frame()
    action_bar.require_unique_action_bar_images([("frame.png", frame)])


def test_require_unique_action_bar_images_rejects_two_copies_of_one_image() -> None:
    frame = action_bar.draw_frame()
    with pytest.raises(ValueError, match="action bar images are not unique"):
        action_bar.require_unique_action_bar_images([
            ("frame.png", frame),
            ("copy.png", frame),
        ])
    with pytest.raises(ValueError, match="not enough values to unpack"):
        action_bar.require_unique_action_bar_images([("frame.png",)])
    with pytest.raises(AttributeError):
        action_bar.require_unique_action_bar_images([("frame.png", None)])


def test_draw_frame_paints_the_locked_wooden_bevel() -> None:
    frame = action_bar.draw_frame()
    assert action_bar.FRAME_WIDTH == 498
    assert action_bar.FRAME_HEIGHT == 78
    assert action_bar.BEVEL_THICKNESS == 6
    assert frame.size == (498, 78)
    assert frame.mode == "RGBA"
    assert frame.getpixel((0, 0)) == WOOD_LIGHT
    assert frame.getpixel((6, 6)) == WOOD
    assert frame.getpixel((246, 36)) == WOOD
    assert frame.getpixel((6, 12)) == GRAIN_DARK
    assert frame.getpixel((100, 30)) == GRAIN_MID
    assert frame.getpixel((450, 8)) == KNOT
    assert frame.getpixel((492, 40)) == WOOD_DARK
    assert frame.getpixel((497, 77)) == WOOD_DARK
    assert action_bar.WOOD == WOOD
    assert action_bar.WOOD_LIGHT == WOOD_LIGHT
    assert action_bar.WOOD_DARK == WOOD_DARK
    assert action_bar.GRAIN_DARK == GRAIN_DARK
    assert action_bar.GRAIN_MID == GRAIN_MID
    assert action_bar.GRAIN_LIGHT == GRAIN_LIGHT
    assert action_bar.KNOT == KNOT
    assert _colors(frame) == FRAME_COLORS
    assert _image_digest(frame) == FRAME_PIXEL_DIGEST


def test_draw_frame_leaves_the_outer_wood_corners_outside_the_bevel() -> None:
    frame = action_bar.draw_frame()
    assert frame.getpixel((497, 0)) == WOOD
    assert frame.getpixel((0, 77)) == WOOD
    assert frame.getpixel((496, 0)) == WOOD_LIGHT
    assert frame.getpixel((0, 76)) == WOOD_LIGHT
    assert frame.getpixel((1, 72)) == WOOD_DARK
    assert frame.getpixel((1, 77)) == WOOD_DARK
    assert frame.getpixel((491, 71)) == WOOD
    assert _image_digest(frame) == FRAME_PIXEL_DIGEST


def test_draw_frame_rejects_a_canvas_that_is_too_small(monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "FRAME_WIDTH", 3)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.draw_frame()
    monkeypatch.setattr(action_bar, "FRAME_WIDTH", 498)
    monkeypatch.setattr(action_bar, "FRAME_HEIGHT", 3)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.draw_frame()


def test_require_canvas_can_hold_accepts_the_frame_and_the_highlight() -> None:
    action_bar.require_canvas_can_hold(
        action_bar.FRAME_WIDTH,
        action_bar.FRAME_HEIGHT,
        action_bar.MINIMUM_FRAME_SPAN,
    )
    action_bar.require_canvas_can_hold(
        action_bar.SLOT_SIZE,
        action_bar.SLOT_SIZE,
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
    )


def test_require_canvas_can_hold_accepts_a_canvas_at_the_minimum_span() -> None:
    action_bar.require_canvas_can_hold(
        action_bar.MINIMUM_FRAME_SPAN,
        action_bar.FRAME_HEIGHT,
        action_bar.MINIMUM_FRAME_SPAN,
    )
    action_bar.require_canvas_can_hold(
        action_bar.FRAME_WIDTH,
        action_bar.MINIMUM_FRAME_SPAN,
        action_bar.MINIMUM_FRAME_SPAN,
    )
    action_bar.require_canvas_can_hold(
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
        action_bar.SLOT_SIZE,
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
    )


def test_require_canvas_can_hold_rejects_a_canvas_smaller_than_the_span() -> None:
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.require_canvas_can_hold(3, action_bar.FRAME_HEIGHT, action_bar.MINIMUM_FRAME_SPAN)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.require_canvas_can_hold(action_bar.FRAME_WIDTH, 3, action_bar.MINIMUM_FRAME_SPAN)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.require_canvas_can_hold(1, action_bar.SLOT_SIZE, action_bar.MINIMUM_HIGHLIGHT_SPAN)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.require_canvas_can_hold(0, 0, action_bar.MINIMUM_HIGHLIGHT_SPAN)


def test_paint_wooden_bevel_paints_the_light_top_and_the_dark_bottom() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    action_bar.paint_wooden_bevel(canvas)
    image = canvas.image()
    assert canvas.read(0, 0) == WOOD_LIGHT
    assert canvas.read(5, 0) == WOOD_LIGHT
    assert canvas.read(6, 6) == WOOD
    assert canvas.read(246, 36) == WOOD
    assert canvas.read(6, 12) == GRAIN_DARK
    assert canvas.read(450, 8) == KNOT
    assert canvas.read(492, 40) == WOOD_DARK
    assert canvas.read(200, 72) == WOOD_DARK
    assert _colors(image) == FRAME_COLORS
    assert _image_digest(image) == FRAME_PIXEL_DIGEST


def test_paint_wooden_bevel_on_the_minimum_frame_keeps_the_outer_corners() -> None:
    canvas = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    action_bar.paint_wooden_bevel(canvas)
    assert canvas.read(0, 0) == WOOD_LIGHT
    assert canvas.read(5, 0) == WOOD_LIGHT
    assert canvas.read(0, 5) == WOOD_LIGHT
    assert canvas.read(11, 0) == WOOD
    assert canvas.read(0, 11) == WOOD
    assert canvas.read(11, 6) == WOOD_DARK
    assert canvas.read(6, 11) == WOOD_DARK
    assert canvas.read(11, 11) == WOOD_DARK
    assert canvas.read(6, 6) == WOOD_DARK


def test_paint_wooden_bevel_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_wooden_bevel(None)


def test_paint_wood_grain_paints_a_grain_pixel_and_the_knot() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_wood_grain(canvas)
    assert canvas.read(6, 11) == GRAIN_DARK
    assert canvas.read(6, 12) == GRAIN_DARK
    assert canvas.read(450, 8) == KNOT
    assert canvas.read(448, 8) == GRAIN_DARK
    assert canvas.read(452, 8) == GRAIN_DARK
    assert canvas.read(450, 6) == GRAIN_DARK
    assert canvas.read(450, 10) == GRAIN_DARK
    assert canvas.read(6, 6) == WOOD
    assert canvas.read(246, 36) == WOOD


def test_paint_wood_grain_leaves_bevel_pixels_unpainted() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_wood_grain(canvas)
    assert canvas.read(450, 5) == WOOD
    assert canvas.read(90, 5) == WOOD
    assert canvas.read(0, 0) == WOOD
    assert canvas.read(497, 0) == WOOD
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    action_bar.paint_wood_grain(small)
    assert small.read(0, 0) == WOOD
    assert small.read(6, 6) == WOOD
    assert small.read(11, 11) == WOOD


def test_paint_wood_grain_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_wood_grain(None)


def test_paint_grain_line_paints_the_dark_line_on_wood() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_grain_line(canvas, 9, 0.0, GRAIN_DARK)
    assert canvas.read(6, 11) == GRAIN_DARK
    assert canvas.read(6, 12) == GRAIN_DARK
    assert canvas.read(12, 12) == GRAIN_DARK
    assert canvas.read(12, 13) == GRAIN_DARK
    assert canvas.read(6, 10) == WOOD
    assert canvas.read(6, 6) == WOOD


def test_paint_grain_line_skips_a_bevel_row_and_a_row_off_the_canvas() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    action_bar.paint_grain_line(canvas, 9, 0.0, GRAIN_DARK)
    assert canvas.read(6, 11) == GRAIN_DARK
    assert canvas.read(90, 5) == WOOD
    assert canvas.read(90, 6) == GRAIN_DARK
    assert canvas.read(0, 9) == WOOD
    action_bar.paint_grain_line(canvas, 72, 2.78, GRAIN_DARK)
    assert canvas.read(40, 68) == GRAIN_DARK
    assert canvas.read(40, 69) == GRAIN_DARK
    assert canvas.read(100, 74) == WOOD
    assert canvas.read(0, 72) == WOOD
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    small.read = _read_that_rejects_pixels_off_the_canvas(small)
    action_bar.paint_grain_line(small, 9, 0.0, GRAIN_DARK)
    action_bar.paint_grain_line(small, 72, 2.78, GRAIN_DARK)
    assert small.read(0, 0) == WOOD
    assert small.read(6, 6) == WOOD
    assert small.read(11, 11) == WOOD


def test_paint_grain_line_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_grain_line(None, 9, 0.0, GRAIN_DARK)


def test_grain_row_follows_the_dark_line_near_the_left_edge() -> None:
    assert action_bar.grain_row(0, 9, 0.0) == 9
    assert action_bar.grain_row(6, 9, 0.0) == 11
    assert action_bar.grain_row(18, 9, 0.0) == 13


def test_grain_row_reaches_the_bevel_row_on_the_dark_line() -> None:
    assert action_bar.grain_row(90, 9, 0.0) == 5
    assert action_bar.grain_row(0, 72, 2.78) == 72
    assert action_bar.grain_row(100, 72, 2.78) == 74


def test_grain_row_rejects_a_missing_column() -> None:
    with pytest.raises(TypeError):
        action_bar.grain_row(None, 9, 0.0)


def test_paint_on_wood_body_paints_an_interior_pixel() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_on_wood_body(canvas, 20, 20, GRAIN_DARK)
    assert canvas.read(20, 20) == GRAIN_DARK
    assert canvas.read(20, 22) == WOOD


def test_paint_on_wood_body_leaves_bevel_pixels_unpainted() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.put(20, 20, GRAIN_LIGHT)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    action_bar.paint_on_wood_body(canvas, 0, 0, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 450, 2, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 497, 0, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, -1, 20, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 20, canvas.height, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 20, 20, GRAIN_DARK)
    assert canvas.read(0, 0) == WOOD
    assert canvas.read(450, 2) == WOOD
    assert canvas.read(497, 0) == WOOD
    assert canvas.read(20, 20) == GRAIN_LIGHT
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    small.read = _read_that_rejects_pixels_off_the_canvas(small)
    action_bar.paint_on_wood_body(small, 6, 6, GRAIN_DARK)
    assert small.read(0, 0) == WOOD
    assert small.read(6, 6) == WOOD
    assert small.read(11, 11) == WOOD


def test_pixel_is_still_on_the_wood_body_accepts_an_interior_wood_pixel() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    assert action_bar.pixel_is_still_on_the_wood_body(canvas, 20, 20)
    assert action_bar.pixel_is_still_on_the_wood_body(canvas, 246, 36)
    assert action_bar.pixel_is_still_on_the_wood_body(canvas, 491, 71)


def test_pixel_is_still_on_the_wood_body_rejects_bevel_grain_and_off_canvas_pixels() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.put(20, 20, GRAIN_LIGHT)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 0, 0)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 450, 2)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 497, 77)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 20, 20)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, -1, 20)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 99, canvas.height)
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    small.read = _read_that_rejects_pixels_off_the_canvas(small)
    assert not action_bar.pixel_is_still_on_the_wood_body(small, 6, 6)


def test_pixel_is_still_on_the_wood_body_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.pixel_is_still_on_the_wood_body(None, 20, 20)


def test_paint_on_wood_body_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_on_wood_body(None, 20, 20, GRAIN_DARK)


def test_paint_knot_paints_the_center_and_the_plus() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_knot(canvas, 450, 8)
    assert canvas.read(450, 8) == KNOT
    assert canvas.read(448, 8) == GRAIN_DARK
    assert canvas.read(449, 8) == GRAIN_DARK
    assert canvas.read(451, 8) == GRAIN_DARK
    assert canvas.read(452, 8) == GRAIN_DARK
    assert canvas.read(450, 6) == GRAIN_DARK
    assert canvas.read(450, 7) == GRAIN_DARK
    assert canvas.read(450, 9) == GRAIN_DARK
    assert canvas.read(450, 10) == GRAIN_DARK
    assert canvas.read(447, 8) == WOOD
    assert canvas.read(453, 8) == WOOD
    assert canvas.read(450, 5) == WOOD
    assert canvas.read(450, 11) == WOOD


def test_paint_knot_skips_bevel_neighbors() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    action_bar.paint_knot(canvas, 450, 6)
    assert canvas.read(450, 6) == KNOT
    assert canvas.read(450, 5) == WOOD
    assert canvas.read(450, 4) == WOOD
    assert canvas.read(450, 7) == GRAIN_DARK
    assert canvas.read(450, 8) == GRAIN_DARK
    action_bar.paint_knot(canvas, 90, 4)
    assert canvas.read(90, 4) == WOOD
    assert canvas.read(90, 6) == GRAIN_DARK
    action_bar.paint_knot(canvas, 0, 0)
    assert canvas.read(0, 0) == WOOD
    assert canvas.read(1, 0) == WOOD
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    action_bar.paint_knot(small, 6, 6)
    assert small.read(0, 0) == WOOD
    assert small.read(6, 6) == WOOD
    assert small.read(11, 11) == WOOD


def test_paint_knot_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_knot(None, 450, 8)


def test_draw_highlight_paints_the_locked_red_orange_border() -> None:
    highlight = action_bar.draw_highlight()
    assert action_bar.SLOT_SIZE == 54
    assert action_bar.HIGHLIGHT_THICKNESS == 3
    assert highlight.size == (54, 54)
    assert highlight.mode == "RGBA"
    assert highlight.getpixel((0, 0)) == HIGHLIGHT
    assert highlight.getpixel((53, 0)) == HIGHLIGHT
    assert highlight.getpixel((0, 53)) == HIGHLIGHT
    assert highlight.getpixel((53, 53)) == HIGHLIGHT
    assert highlight.getpixel((27, 0)) == HIGHLIGHT
    assert highlight.getpixel((3, 3)) == CLEAR
    assert highlight.getpixel((27, 27)) == CLEAR
    assert highlight.getpixel((50, 50)) == CLEAR
    assert action_bar.HIGHLIGHT == HIGHLIGHT
    assert _colors(highlight) == {CLEAR, HIGHLIGHT}
    assert _image_digest(highlight) == HIGHLIGHT_PIXEL_DIGEST


def test_draw_highlight_leaves_the_inner_square_clear() -> None:
    highlight = action_bar.draw_highlight()
    border = action_bar.HIGHLIGHT_THICKNESS
    size = action_bar.SLOT_SIZE
    inner_end = size - border
    for row in range(border, inner_end):
        for column in range(border, inner_end):
            assert highlight.getpixel((column, row)) == CLEAR
    for column in range(size):
        for edge in range(border):
            assert highlight.getpixel((column, edge)) == HIGHLIGHT
            assert highlight.getpixel((column, inner_end + edge)) == HIGHLIGHT
    for row in range(size):
        for edge in range(border):
            assert highlight.getpixel((edge, row)) == HIGHLIGHT
            assert highlight.getpixel((inner_end + edge, row)) == HIGHLIGHT


def test_draw_highlight_rejects_a_canvas_that_is_too_small(monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "SLOT_SIZE", 1)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.draw_highlight()


def test_paint_highlight_border_paints_the_locked_red_orange_edge() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    action_bar.paint_highlight_border(canvas)
    assert canvas.read(0, 0) == HIGHLIGHT
    assert canvas.read(53, 27) == HIGHLIGHT
    assert canvas.read(27, 53) == HIGHLIGHT
    assert canvas.read(3, 3) is None
    assert canvas.read(50, 50) is None
    assert action_bar.HIGHLIGHT == HIGHLIGHT
    assert _image_digest(canvas.image()) == HIGHLIGHT_PIXEL_DIGEST


def test_paint_highlight_border_covers_the_minimum_span() -> None:
    canvas = rejecting_canvas(
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
    )
    action_bar.paint_highlight_border(canvas)
    assert canvas.read(0, 0) == HIGHLIGHT
    assert canvas.read(5, 0) == HIGHLIGHT
    assert canvas.read(0, 5) == HIGHLIGHT
    assert canvas.read(5, 5) == HIGHLIGHT
    assert canvas.read(2, 2) == HIGHLIGHT


def test_paint_highlight_border_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_highlight_border(None)


def test_draw_slot_paints_the_locked_digit_one() -> None:
    slot = action_bar.draw_slot(1)
    assert slot.size == (54, 54)
    assert slot.mode == "RGBA"
    assert action_bar.SLOT_NUMBER_LEFT == 6
    assert action_bar.SLOT_NUMBER_TOP == 6
    assert action_bar.STROKE_THICKNESS == 3
    assert slot.getpixel((0, 0)) == PARCHMENT
    assert slot.getpixel((50, 50)) == PARCHMENT
    assert slot.getpixel((53, 0)) == PARCHMENT_SHADOW
    assert slot.getpixel((0, 53)) == PARCHMENT_SHADOW
    assert slot.getpixel((53, 53)) == PARCHMENT_SHADOW
    assert slot.getpixel((12, 6)) == INK
    assert slot.getpixel((13, 6)) == INK
    assert slot.getpixel((14, 6)) == INK
    assert slot.getpixel((15, 6)) == PARCHMENT
    assert slot.getpixel((9, 6)) == INK
    assert slot.getpixel((8, 6)) == PARCHMENT
    assert action_bar.PARCHMENT == PARCHMENT
    assert action_bar.PARCHMENT_SHADOW == PARCHMENT_SHADOW
    assert action_bar.INK == INK
    assert _colors(slot) == {PARCHMENT, PARCHMENT_SHADOW, INK}
    assert _image_digest(slot) == SLOT_PIXEL_DIGESTS[0]


def test_draw_slot_keeps_the_first_digit_different_from_the_last() -> None:
    first = action_bar.draw_slot(1)
    last = action_bar.draw_slot(8)
    assert first.size == (54, 54)
    assert last.size == (54, 54)
    assert first.getpixel((6, 6)) == PARCHMENT
    assert last.getpixel((6, 6)) == INK
    assert first.getpixel((12, 10)) == INK
    assert last.getpixel((12, 10)) == PARCHMENT
    assert first.getpixel((20, 26)) == PARCHMENT
    assert last.getpixel((20, 26)) == INK
    assert first.getpixel((0, 0)) == PARCHMENT
    assert last.getpixel((53, 53)) == PARCHMENT_SHADOW
    assert _image_digest(first) == SLOT_PIXEL_DIGESTS[0]
    assert _image_digest(last) == SLOT_PIXEL_DIGESTS[7]
    assert first.tobytes() != last.tobytes()


def test_draw_slot_rejects_zero_nine_and_bools() -> None:
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.draw_slot(0)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.draw_slot(9)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.draw_slot(True)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.draw_slot(False)


def test_require_slot_number_returns_a_middle_slot() -> None:
    assert action_bar.require_slot_number(2) == 2
    assert action_bar.require_slot_number(4) == 4
    assert action_bar.require_slot_number(7) == 7


def test_require_slot_number_accepts_the_first_and_last_slots() -> None:
    assert action_bar.require_slot_number(1) == 1
    assert action_bar.require_slot_number(8) == 8


def test_require_slot_number_rejects_zero_nine_and_bools() -> None:
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.require_slot_number(0)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.require_slot_number(9)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.require_slot_number(True)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.require_slot_number(False)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.require_slot_number(1.0)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.require_slot_number(None)


def test_paint_parchment_slot_fills_the_slot_and_shadows_the_bottom_right() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    action_bar.paint_parchment_slot(canvas)
    image = canvas.image()
    assert canvas.read(0, 0) == PARCHMENT
    assert canvas.read(50, 50) == PARCHMENT
    assert canvas.read(27, 27) == PARCHMENT
    assert canvas.read(53, 0) == PARCHMENT_SHADOW
    assert canvas.read(0, 53) == PARCHMENT_SHADOW
    assert canvas.read(53, 27) == PARCHMENT_SHADOW
    assert canvas.read(27, 53) == PARCHMENT_SHADOW
    assert canvas.read(53, 53) == PARCHMENT_SHADOW
    assert action_bar.PARCHMENT == PARCHMENT
    assert action_bar.PARCHMENT_SHADOW == PARCHMENT_SHADOW
    assert action_bar.PARCHMENT_SHADOW_THICKNESS == 3
    assert _colors(image) == {PARCHMENT, PARCHMENT_SHADOW}
    assert _image_digest(image) == PARCHMENT_PIXEL_DIGEST


def test_paint_parchment_slot_on_a_shadow_sized_canvas_is_only_shadow() -> None:
    span = action_bar.PARCHMENT_SHADOW_THICKNESS
    canvas = rejecting_canvas(span, span)
    action_bar.paint_parchment_slot(canvas)
    assert canvas.read(0, 0) == PARCHMENT_SHADOW
    assert canvas.read(2, 0) == PARCHMENT_SHADOW
    assert canvas.read(0, 2) == PARCHMENT_SHADOW
    assert canvas.read(2, 2) == PARCHMENT_SHADOW


def test_paint_parchment_slot_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_parchment_slot(None)


def test_paint_slot_number_stamps_the_digit_one_in_the_top_left() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    action_bar.paint_slot_number(canvas, 1)
    assert canvas.read(12, 6) == INK
    assert canvas.read(13, 6) == INK
    assert canvas.read(14, 6) == INK
    assert canvas.read(15, 6) is None
    assert canvas.read(9, 6) == INK
    assert canvas.read(8, 6) is None
    assert canvas.read(12, 24) == INK
    assert canvas.read(0, 0) is None
    assert canvas.read(53, 53) is None
    assert action_bar.INK == INK
    assert _image_digest(canvas.image()) == SLOT_ONE_INK_PIXEL_DIGEST


def test_paint_slot_number_stamps_the_digit_eight_without_covering_the_shadow() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    action_bar.paint_parchment_slot(canvas)
    action_bar.paint_slot_number(canvas, 8)
    assert canvas.read(6, 6) == INK
    assert canvas.read(20, 6) == INK
    assert canvas.read(6, 26) == INK
    assert canvas.read(20, 26) == INK
    assert canvas.read(10, 6) == INK
    assert canvas.read(10, 7) == INK
    assert canvas.read(10, 8) == INK
    assert canvas.read(10, 9) == PARCHMENT
    assert canvas.read(12, 10) == PARCHMENT
    assert canvas.read(5, 6) == PARCHMENT
    assert canvas.read(21, 6) == PARCHMENT
    assert canvas.read(6, 5) == PARCHMENT
    assert canvas.read(6, 27) == PARCHMENT
    assert canvas.read(53, 0) == PARCHMENT_SHADOW
    assert canvas.read(0, 53) == PARCHMENT_SHADOW
    assert canvas.read(27, 27) == PARCHMENT
    assert _image_digest(canvas.image()) == SLOT_PIXEL_DIGESTS[7]


def test_paint_slot_number_rejects_zero_nine_bools_and_a_missing_canvas() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.paint_slot_number(canvas, 0)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.paint_slot_number(canvas, 9)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.paint_slot_number(canvas, True)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.paint_slot_number(canvas, False)
    with pytest.raises(AttributeError):
        action_bar.paint_slot_number(None, 1)
