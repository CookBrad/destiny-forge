from __future__ import annotations

import hashlib

import pytest
from PIL import Image

import draw_action_bar as action_bar
from art_common import rejecting_canvas

FRAME_PIXEL_DIGEST = "1fa1f8ca61803de010d06dcb05c197588b67500ec3d9d08ee59b44aba6cbdf9e"
HIGHLIGHT_PIXEL_DIGEST = "3fff2e88ecbcc29c3ab880ee2d56c505730ad1bb940f0405decd459853fb15ad"
SLOT_PIXEL_DIGESTS = (
    "22953aa4f257320a3d974186fe4d4bbba595973051057741effdf75bdccbc5cb",
    "00afd48e82f6619a063a57b8998ea71b607cdd920103f0d0d7a71a26df92f9fb",
    "9795deaa5eee390117da5227218246b48a4681746eb46ea80f7cd9accab230d5",
    "804b8c8db8a65916c3a45de074f9dd258a7d9886c8be302d1d56dcfd1aadb110",
    "bcfeb6806390d50fdfa853a4d28df7ac6b1cfe068090eb72f1b64b57b7ecaebb",
    "bc17f8faf33880584b3585865341e612a45cb479422247c95b94781a265ad473",
    "900e0f6df080ddc0b6efa0df8e4a6483923e0487fccc7ea91f5c2f2b524484c0",
    "58257ba3323ac90797dae9ad55270c63d454ec4940954c2956d6851865d655ca",
)
DESIGN_FRAME_PIXEL_DIGEST = "c187816c8178d422e6acf44f32fed1c7833d492547acd1e92eefcdbcaf659ad6"
DESIGN_HIGHLIGHT_PIXEL_DIGEST = "eef548ed001eb649e2c365d3bc7c5ab397acd5efdb97ce55bacaf4af9d8cff9a"
DESIGN_SLOT_EIGHT_PIXEL_DIGEST = "06891eb0de2752ab23d729dd40f6ea8a3429a3e31a589e25788c3696309efc7a"
PARCHMENT_PIXEL_DIGEST = "b60630afa139a05f77f1f1a93653b79011dd86cfc035c514f5359a93ab82feb5"
SLOT_ONE_INK_PIXEL_DIGEST = "7bcedf36cb9c2aee5f1d44e7afc050e2d26bbcbf0bfc2cbd0abdb31545bb7956"
SLOT_EIGHT_INK_PIXEL_DIGEST = "0fb2f5c45523ee25822d435b976f8164106c10c0d49667c5f104b8d5a2ae1d54"
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
SLOT_ONE_INK = (
    (1, 0),
    (0, 1), (1, 1),
    (1, 2),
    (1, 3),
    (1, 4),
    (1, 5),
    (1, 6), (2, 6), (3, 6),
)
SLOT_EIGHT_INK = (
    (1, 0), (2, 0), (3, 0),
    (0, 1), (4, 1),
    (0, 2), (4, 2),
    (1, 3), (2, 3), (3, 3),
    (0, 4), (4, 4),
    (0, 5), (4, 5),
    (1, 6), (2, 6), (3, 6),
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


def _opaque(canvas) -> dict:
    found = {}
    for row in range(canvas.height):
        for column in range(canvas.width):
            color = canvas.read(column, row)
            if color is not None:
                found[(column, row)] = color
    return found


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


def _placed_ink(ink_pixels) -> dict:
    return {
        (
            column + action_bar.SLOT_NUMBER_LEFT,
            row + action_bar.SLOT_NUMBER_TOP,
        ): INK
        for column, row in ink_pixels
    }


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
    assert action_bar.FRAME_WIDTH == 166
    assert action_bar.FRAME_HEIGHT == 26
    assert frame.size == (498, 78)
    assert frame.mode == "RGBA"
    assert frame.getpixel((0 * 3, 0 * 3)) == WOOD_LIGHT
    assert frame.getpixel((2 * 3, 2 * 3)) == WOOD
    assert frame.getpixel((82 * 3, 12 * 3)) == WOOD
    assert frame.getpixel((2 * 3, 4 * 3)) == GRAIN_DARK
    assert frame.getpixel((150 * 3, 2 * 3)) == KNOT
    assert frame.getpixel((164 * 3, 1 * 3)) == WOOD_DARK
    assert frame.getpixel((165 * 3, 25 * 3)) == WOOD_DARK
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
    assert frame.getpixel((165 * 3, 0 * 3)) == WOOD
    assert frame.getpixel((0 * 3, 25 * 3)) == WOOD
    assert frame.getpixel((164 * 3, 0 * 3)) == WOOD_LIGHT
    assert frame.getpixel((0 * 3, 24 * 3)) == WOOD_LIGHT
    assert frame.getpixel((1 * 3, 24 * 3)) == WOOD_DARK
    assert frame.getpixel((1 * 3, 25 * 3)) == WOOD_DARK
    assert frame.getpixel((163 * 3, 23 * 3)) == WOOD
    assert _image_digest(frame) == FRAME_PIXEL_DIGEST


def test_draw_frame_rejects_a_canvas_that_is_too_small(monkeypatch) -> None:
    monkeypatch.setattr(action_bar, "FRAME_WIDTH", 3)
    with pytest.raises(ValueError, match="canvas is too small to draw"):
        action_bar.draw_frame()
    monkeypatch.setattr(action_bar, "FRAME_WIDTH", 166)
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
    assert canvas.read(1, 0) == WOOD_LIGHT
    assert canvas.read(2, 2) == WOOD
    assert canvas.read(82, 12) == WOOD
    assert canvas.read(2, 4) == GRAIN_DARK
    assert canvas.read(150, 2) == KNOT
    assert canvas.read(164, 12) == WOOD_DARK
    assert canvas.read(165, 25) == WOOD_DARK
    assert _colors(image) == FRAME_COLORS
    assert _image_digest(image) == DESIGN_FRAME_PIXEL_DIGEST


def test_paint_wooden_bevel_on_the_minimum_frame_keeps_two_wood_corners() -> None:
    canvas = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    action_bar.paint_wooden_bevel(canvas)
    assert _opaque(canvas) == {
        (0, 0): WOOD_LIGHT,
        (1, 0): WOOD_LIGHT,
        (2, 0): WOOD_LIGHT,
        (3, 0): WOOD,
        (0, 1): WOOD_LIGHT,
        (1, 1): WOOD_LIGHT,
        (2, 1): WOOD_DARK,
        (3, 1): WOOD_DARK,
        (0, 2): WOOD_LIGHT,
        (1, 2): WOOD_DARK,
        (2, 2): WOOD_DARK,
        (3, 2): WOOD_DARK,
        (0, 3): WOOD,
        (1, 3): WOOD_DARK,
        (2, 3): WOOD_DARK,
        (3, 3): WOOD_DARK,
    }


def test_paint_wooden_bevel_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_wooden_bevel(None)


def test_paint_wood_grain_paints_a_grain_pixel_and_the_knot() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_wood_grain(canvas)
    assert canvas.read(2, 4) == GRAIN_DARK
    assert canvas.read(150, 2) == KNOT
    assert canvas.read(149, 2) == GRAIN_DARK
    assert canvas.read(151, 2) == GRAIN_DARK
    assert canvas.read(150, 3) == GRAIN_DARK
    assert canvas.read(2, 2) == WOOD
    assert canvas.read(82, 12) == WOOD


def test_paint_wood_grain_leaves_bevel_pixels_unpainted() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_wood_grain(canvas)
    assert canvas.read(150, 1) == WOOD
    assert canvas.read(30, 1) == WOOD
    assert canvas.read(38, 1) == WOOD
    assert canvas.read(0, 0) == WOOD
    canvas_four = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    action_bar.paint_wooden_bevel(canvas_four)
    assert _opaque(canvas_four) == {
        (0, 0): WOOD_LIGHT,
        (1, 0): WOOD_LIGHT,
        (2, 0): WOOD_LIGHT,
        (3, 0): WOOD,
        (0, 1): WOOD_LIGHT,
        (1, 1): WOOD_LIGHT,
        (2, 1): WOOD_DARK,
        (3, 1): WOOD_DARK,
        (0, 2): WOOD_LIGHT,
        (1, 2): WOOD_DARK,
        (2, 2): WOOD_DARK,
        (3, 2): WOOD_DARK,
        (0, 3): WOOD,
        (1, 3): WOOD_DARK,
        (2, 3): WOOD_DARK,
        (3, 3): WOOD_DARK,
    }


def test_paint_wood_grain_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_wood_grain(None)


def test_paint_grain_line_paints_the_dark_line_on_wood() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_grain_line(canvas, 3, 0.0, GRAIN_DARK)
    assert canvas.read(2, 4) == GRAIN_DARK
    assert canvas.read(4, 4) == GRAIN_DARK
    assert canvas.read(2, 2) == WOOD


def test_paint_grain_line_skips_a_bevel_row_and_a_row_off_the_canvas() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    action_bar.paint_grain_line(canvas, 3, 0.0, GRAIN_DARK)
    assert canvas.read(3, 4) == GRAIN_DARK
    assert canvas.read(38, 1) == WOOD
    assert canvas.read(0, 3) == WOOD
    action_bar.paint_grain_line(canvas, 24, 2.78, GRAIN_DARK)
    assert canvas.read(12, 23) == GRAIN_DARK
    assert canvas.read(99, 25) == WOOD
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    small.read = _read_that_rejects_pixels_off_the_canvas(small)
    action_bar.paint_grain_line(small, 3, 0.0, GRAIN_DARK)
    action_bar.paint_grain_line(small, 24, 2.78, GRAIN_DARK)
    assert _opaque(small) == {
        (column, row): WOOD
        for row in range(action_bar.MINIMUM_FRAME_SPAN)
        for column in range(action_bar.MINIMUM_FRAME_SPAN)
    }


def test_paint_grain_line_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_grain_line(None, 3, 0.0, GRAIN_DARK)


def test_grain_row_follows_the_dark_line_near_the_left_edge() -> None:
    assert action_bar.grain_row(2, 3, 0.0) == 4
    assert action_bar.grain_row(4, 3, 0.0) == 4
    assert action_bar.grain_row(18, 3, 0.0) == 3


def test_grain_row_reaches_the_bevel_row_on_the_dark_line() -> None:
    assert action_bar.grain_row(38, 3, 0.0) == 1
    assert action_bar.grain_row(0, 3, 0.0) == 3
    assert action_bar.grain_row(150, 3, 0.0) == 2


def test_grain_row_rejects_a_missing_column() -> None:
    with pytest.raises(TypeError):
        action_bar.grain_row(None, 3, 0.0)


def test_paint_on_wood_body_paints_an_interior_pixel() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_on_wood_body(canvas, 2, 4, GRAIN_DARK)
    assert canvas.read(2, 4) == GRAIN_DARK
    assert canvas.read(2, 2) == WOOD


def test_paint_on_wood_body_leaves_bevel_pixels_unpainted() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.put(4, 4, GRAIN_LIGHT)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    action_bar.paint_on_wood_body(canvas, 0, 0, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 150, 1, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 165, 0, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, -1, 2, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 4, canvas.height, GRAIN_DARK)
    action_bar.paint_on_wood_body(canvas, 4, 4, GRAIN_DARK)
    assert canvas.read(0, 0) == WOOD
    assert canvas.read(150, 1) == WOOD
    assert canvas.read(165, 0) == WOOD
    assert canvas.read(4, 4) == GRAIN_LIGHT
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    small.read = _read_that_rejects_pixels_off_the_canvas(small)
    action_bar.paint_on_wood_body(small, 1, 1, GRAIN_DARK)
    assert small.read(1, 1) == WOOD
    assert _opaque(small) == {
        (column, row): WOOD
        for row in range(action_bar.MINIMUM_FRAME_SPAN)
        for column in range(action_bar.MINIMUM_FRAME_SPAN)
    }


def test_pixel_is_still_on_the_wood_body_accepts_an_interior_wood_pixel() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    assert action_bar.pixel_is_still_on_the_wood_body(canvas, 2, 4)
    assert action_bar.pixel_is_still_on_the_wood_body(canvas, 82, 12)
    assert action_bar.pixel_is_still_on_the_wood_body(canvas, 163, 23)


def test_pixel_is_still_on_the_wood_body_rejects_bevel_grain_and_off_canvas_pixels() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.put(4, 4, GRAIN_LIGHT)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 0, 0)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 150, 1)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 165, 25)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 4, 4)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, -1, 2)
    assert not action_bar.pixel_is_still_on_the_wood_body(canvas, 99, canvas.height)
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    small.read = _read_that_rejects_pixels_off_the_canvas(small)
    assert not action_bar.pixel_is_still_on_the_wood_body(small, 1, 1)


def test_pixel_is_still_on_the_wood_body_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.pixel_is_still_on_the_wood_body(None, 2, 4)


def test_paint_on_wood_body_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_on_wood_body(None, 2, 4, GRAIN_DARK)


def test_paint_knot_paints_the_center_and_the_plus() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    action_bar.paint_knot(canvas, 150, 2)
    assert canvas.read(150, 2) == KNOT
    assert canvas.read(149, 2) == GRAIN_DARK
    assert canvas.read(151, 2) == GRAIN_DARK
    assert canvas.read(150, 3) == GRAIN_DARK
    assert canvas.read(148, 2) == WOOD
    assert canvas.read(152, 2) == WOOD


def test_paint_knot_skips_bevel_neighbors() -> None:
    canvas = rejecting_canvas(action_bar.FRAME_WIDTH, action_bar.FRAME_HEIGHT)
    canvas.fill_rect(0, 0, canvas.width, canvas.height, WOOD)
    canvas.read = _read_that_rejects_pixels_off_the_canvas(canvas)
    action_bar.paint_knot(canvas, 150, 2)
    assert canvas.read(150, 1) == WOOD
    action_bar.paint_knot(canvas, 30, 1)
    assert canvas.read(30, 1) == WOOD
    assert canvas.read(30, 2) == GRAIN_DARK
    action_bar.paint_knot(canvas, 0, 0)
    assert canvas.read(0, 0) == WOOD
    assert canvas.read(1, 0) == WOOD
    small = rejecting_canvas(action_bar.MINIMUM_FRAME_SPAN, action_bar.MINIMUM_FRAME_SPAN)
    small.fill_rect(0, 0, small.width, small.height, WOOD)
    action_bar.paint_knot(small, 1, 1)
    assert _opaque(small) == {
        (column, row): WOOD
        for row in range(action_bar.MINIMUM_FRAME_SPAN)
        for column in range(action_bar.MINIMUM_FRAME_SPAN)
    }


def test_paint_knot_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_knot(None, 150, 2)


def test_draw_highlight_paints_the_locked_red_orange_border() -> None:
    highlight = action_bar.draw_highlight()
    assert action_bar.SLOT_SIZE == 18
    assert highlight.size == (54, 54)
    assert highlight.mode == "RGBA"
    assert highlight.getpixel((0 * 3, 0 * 3)) == HIGHLIGHT
    assert highlight.getpixel((17 * 3, 0 * 3)) == HIGHLIGHT
    assert highlight.getpixel((0 * 3, 17 * 3)) == HIGHLIGHT
    assert highlight.getpixel((17 * 3, 17 * 3)) == HIGHLIGHT
    assert highlight.getpixel((8 * 3, 0 * 3)) == HIGHLIGHT
    assert highlight.getpixel((1 * 3, 1 * 3)) == CLEAR
    assert highlight.getpixel((8 * 3, 8 * 3)) == CLEAR
    assert highlight.getpixel((16 * 3, 16 * 3)) == CLEAR
    assert action_bar.HIGHLIGHT == HIGHLIGHT
    assert _colors(highlight) == {CLEAR, HIGHLIGHT}
    assert _image_digest(highlight) == HIGHLIGHT_PIXEL_DIGEST


def test_draw_highlight_leaves_the_inner_square_clear() -> None:
    highlight = action_bar.draw_highlight()
    block_size = action_bar.DESIGN_PIXEL_BLOCK_SIZE
    border = action_bar.HIGHLIGHT_THICKNESS * block_size
    size = action_bar.SLOT_SIZE * block_size
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
    assert canvas.read(17, 8) == HIGHLIGHT
    assert canvas.read(8, 17) == HIGHLIGHT
    assert canvas.read(1, 1) is None
    assert canvas.read(16, 16) is None
    assert action_bar.HIGHLIGHT == HIGHLIGHT
    assert _image_digest(canvas.image()) == DESIGN_HIGHLIGHT_PIXEL_DIGEST


def test_paint_highlight_border_covers_a_two_by_two_canvas() -> None:
    canvas = rejecting_canvas(
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
        action_bar.MINIMUM_HIGHLIGHT_SPAN,
    )
    action_bar.paint_highlight_border(canvas)
    assert _opaque(canvas) == {
        (0, 0): HIGHLIGHT,
        (1, 0): HIGHLIGHT,
        (0, 1): HIGHLIGHT,
        (1, 1): HIGHLIGHT,
    }


def test_paint_highlight_border_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_highlight_border(None)


def test_draw_slot_paints_the_locked_digit_one() -> None:
    slot = action_bar.draw_slot(1)
    assert slot.size == (54, 54)
    assert slot.mode == "RGBA"
    assert slot.getpixel((0 * 3, 0 * 3)) == PARCHMENT
    assert slot.getpixel((16 * 3, 16 * 3)) == PARCHMENT
    assert slot.getpixel((17 * 3, 0 * 3)) == PARCHMENT_SHADOW
    assert slot.getpixel((0 * 3, 17 * 3)) == PARCHMENT_SHADOW
    assert slot.getpixel((17 * 3, 17 * 3)) == PARCHMENT_SHADOW
    assert slot.getpixel((3 * 3, 2 * 3)) == INK
    assert slot.getpixel((2 * 3, 2 * 3)) == PARCHMENT
    assert slot.getpixel((5 * 3, 8 * 3)) == INK
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
    assert first.getpixel((4 * 3, 2 * 3)) == PARCHMENT
    assert last.getpixel((4 * 3, 2 * 3)) == INK
    assert first.getpixel((2 * 3, 4 * 3)) == PARCHMENT
    assert last.getpixel((2 * 3, 4 * 3)) == INK
    assert first.getpixel((0 * 3, 0 * 3)) == PARCHMENT
    assert last.getpixel((17 * 3, 17 * 3)) == PARCHMENT_SHADOW
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
    assert canvas.read(16, 16) == PARCHMENT
    assert canvas.read(9, 9) == PARCHMENT
    assert canvas.read(17, 0) == PARCHMENT_SHADOW
    assert canvas.read(0, 17) == PARCHMENT_SHADOW
    assert canvas.read(17, 8) == PARCHMENT_SHADOW
    assert canvas.read(8, 17) == PARCHMENT_SHADOW
    assert canvas.read(17, 17) == PARCHMENT_SHADOW
    assert action_bar.PARCHMENT == PARCHMENT
    assert action_bar.PARCHMENT_SHADOW == PARCHMENT_SHADOW
    assert _colors(image) == {PARCHMENT, PARCHMENT_SHADOW}
    assert _image_digest(image) == PARCHMENT_PIXEL_DIGEST


def test_paint_parchment_slot_on_a_one_by_one_canvas_is_only_shadow() -> None:
    canvas = rejecting_canvas(1, 1)
    action_bar.paint_parchment_slot(canvas)
    assert _opaque(canvas) == {(0, 0): PARCHMENT_SHADOW}


def test_paint_parchment_slot_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        action_bar.paint_parchment_slot(None)


def test_paint_slot_number_stamps_the_digit_one_in_the_top_left() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    action_bar.paint_slot_number(canvas, 1)
    assert canvas.read(3, 2) == INK
    assert canvas.read(2, 3) == INK
    assert canvas.read(5, 8) == INK
    assert canvas.read(2, 2) is None
    assert canvas.read(0, 0) is None
    assert canvas.read(17, 17) is None
    assert action_bar.INK == INK
    assert _opaque(canvas) == _placed_ink(SLOT_ONE_INK)
    assert _image_digest(canvas.image()) == SLOT_ONE_INK_PIXEL_DIGEST


def test_paint_slot_number_stamps_the_digit_eight_without_covering_the_shadow() -> None:
    canvas = rejecting_canvas(action_bar.SLOT_SIZE, action_bar.SLOT_SIZE)
    action_bar.paint_parchment_slot(canvas)
    action_bar.paint_slot_number(canvas, 8)
    for (column, row), color in _placed_ink(SLOT_EIGHT_INK).items():
        assert canvas.read(column, row) == color
    assert canvas.read(2, 2) == PARCHMENT
    assert canvas.read(17, 0) == PARCHMENT_SHADOW
    assert canvas.read(0, 17) == PARCHMENT_SHADOW
    assert canvas.read(9, 9) == PARCHMENT
    assert _image_digest(canvas.image()) == DESIGN_SLOT_EIGHT_PIXEL_DIGEST


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


def test_slot_number_ink_returns_the_digit_one() -> None:
    assert action_bar.slot_number_ink(1) == list(SLOT_ONE_INK)


def test_slot_number_ink_draws_a_closed_eight() -> None:
    eight = action_bar.slot_number_ink(8)
    seven = action_bar.slot_number_ink(7)
    assert eight == list(SLOT_EIGHT_INK)
    assert len(eight) == 17
    assert len(seven) == 10
    assert eight != seven
    assert (0, 0) not in eight
    assert (4, 0) not in eight
    assert (4, 6) not in eight
    assert (1, 6) in eight
    assert (4, 1) in seven
    assert (0, 5) not in seven


def test_slot_number_ink_rejects_zero_nine_and_bools() -> None:
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_number_ink(0)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_number_ink(9)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_number_ink(True)
    with pytest.raises(ValueError, match="action bar slot number must be 1 through 8"):
        action_bar.slot_number_ink(False)


def test_ink_pixels_in_row_returns_the_marks_in_a_digit_row() -> None:
    assert action_bar.ink_pixels_in_row(0, ".#...") == [(1, 0)]
    assert action_bar.ink_pixels_in_row(1, "##...") == [(0, 1), (1, 1)]
    assert action_bar.ink_pixels_in_row(6, ".###.") == [(1, 6), (2, 6), (3, 6)]


def test_ink_pixels_in_row_returns_nothing_when_the_row_has_no_ink() -> None:
    assert action_bar.ink_pixels_in_row(0, ".....") == []
    assert action_bar.ink_pixels_in_row(3, "") == []
    assert action_bar.ink_pixels_in_row(4, ".....") == []


def test_ink_pixels_in_row_rejects_a_missing_pattern() -> None:
    with pytest.raises(TypeError):
        action_bar.ink_pixels_in_row(0, None)
