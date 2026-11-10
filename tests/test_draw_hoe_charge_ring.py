from __future__ import annotations

import hashlib
import math
from pathlib import Path

import pytest
from PIL import Image

import draw_homestead_hoe_charge_ring as charge_ring
from art_common import rejecting_canvas

ROOT = Path(__file__).resolve().parents[1]
FRAME_PIXEL_DIGESTS = (
    "2d40fa1cd8d8835a35d3a3ed378867e5f8dc7a5241e1bd7153846e808f78c9c5",
    "d82460ca1b96554bd2e407e4119feae58170e3687bbbcef0a8a1442b60d1dae9",
    "255315cf654c02aa8c4a801c6fcd2d8e309a7ebe94ed517ad3e28e285cce74fd",
    "0727c1c82eede89d1a9621d11f021886e04c1198e020a91ffb35ee25ac8b6e2c",
)
RING_PIXEL_DIGEST = "1f8d2ffcef044e853f630ab16e75ddaaa4baf474a16f690d728ec48fbb08e655"
LEVEL_TWO_PIPS_DIGEST = "5c4cd23ed6fac97966ed421fa467e70e863ceb430268821716c58840ebfcc14b"
LEVEL_ONE_PIPS_DIGEST = "8db665eb6ac2a7fb85a34137d23f953b29d8f9ac5414e264283112157b2c5fa2"
LEVEL_FOUR_PIPS_DIGEST = "023334d446cadf40af371c9bda24cfaa6cb9218942b14b80ed1868058cb102a9"
BAKED_COLORS = {
    (0, 0, 0, 0),
    charge_ring.OUTLINE,
    charge_ring.RING_LIGHT,
    charge_ring.RING,
    charge_ring.RING_DARK,
    charge_ring.PIP_LIT,
    charge_ring.PIP_LIT_CENTER,
    charge_ring.PIP_DARK,
    charge_ring.PIP_DARK_CENTER,
}


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
    return {image.getpixel((column, row)) for row in range(height) for column in range(width)}


def _color_count(image: Image.Image, color) -> int:
    width, height = image.size
    return sum(
        image.getpixel((column, row)) == color
        for row in range(height)
        for column in range(width)
    )


def test_main_writes_four_charge_ring_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "FOLDER", tmp_path)
    charge_ring.main()
    digests = [
        _image_digest(Image.open(tmp_path / f"hoe_charge_ring_f{index}.png").convert("RGBA"))
        for index in range(4)
    ]
    assert digests == list(FRAME_PIXEL_DIGESTS)
    assert len(set(digests)) == 4


def test_main_with_no_charge_levels_writes_nothing(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "FOLDER", tmp_path)
    monkeypatch.setattr(charge_ring, "CHARGE_LEVELS", ())
    charge_ring.main()
    assert list(tmp_path.iterdir()) == []


def test_main_rejects_a_folder_path_that_is_a_file(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "vfx"
    folder.write_text("not a folder")
    monkeypatch.setattr(charge_ring, "FOLDER", folder)
    with pytest.raises(FileExistsError):
        charge_ring.main()


def test_the_four_charge_ring_frames_differ_from_each_other() -> None:
    frames = [charge_ring.draw_charge_ring(level) for level in charge_ring.CHARGE_LEVELS]
    digests = [_image_digest(frame) for frame in frames]
    assert digests == list(FRAME_PIXEL_DIGESTS)
    assert digests[0] != digests[1]
    assert digests[0] != digests[2]
    assert digests[0] != digests[3]
    assert digests[1] != digests[2]
    assert digests[1] != digests[3]
    assert digests[2] != digests[3]
    assert [_color_count(frame, charge_ring.PIP_LIT_CENTER) for frame in frames] == [1, 2, 3, 4]
    assert [_color_count(frame, charge_ring.PIP_DARK_CENTER) for frame in frames] == [3, 2, 1, 0]


def test_committed_charge_ring_frames_match_the_locked_pixel_digests() -> None:
    folder = ROOT / "assets" / "homestead" / "vfx"
    digests = [
        _image_digest(Image.open(folder / f"hoe_charge_ring_f{index}.png").convert("RGBA"))
        for index in range(4)
    ]
    assert digests == list(FRAME_PIXEL_DIGESTS)
    assert len(set(digests)) == 4


def test_charge_ring_records_returns_four_levels_with_their_seconds() -> None:
    records = charge_ring.charge_ring_records()
    assert [(level, seconds) for level, seconds, _frame in records] == [
        (1, 0.35),
        (2, 0.7),
        (3, 1.05),
        (4, 1.4),
    ]
    assert [_image_digest(frame) for _level, _seconds, frame in records] == list(FRAME_PIXEL_DIGESTS)


def test_charge_ring_records_with_no_levels_returns_nothing(monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "CHARGE_LEVELS", ())
    assert charge_ring.charge_ring_records() == []


def test_charge_ring_records_rejects_a_level_whose_timing_is_missing(monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "CHARGE_LEVEL_MILLISECONDS", {})
    with pytest.raises(KeyError):
        charge_ring.charge_ring_records()


def test_charge_ring_record_returns_the_level_seconds_and_frame() -> None:
    level, seconds, frame = charge_ring.charge_ring_record(2)
    assert (level, seconds) == (2, 0.7)
    assert frame.size == (32, 32)
    assert _image_digest(frame) == FRAME_PIXEL_DIGESTS[1]


def test_charge_ring_record_for_the_first_level_is_shorter_than_the_last() -> None:
    first_level, first_seconds, first_frame = charge_ring.charge_ring_record(1)
    last_level, last_seconds, last_frame = charge_ring.charge_ring_record(4)
    assert (first_level, first_seconds) == (1, 0.35)
    assert (last_level, last_seconds) == (4, 1.4)
    assert first_seconds < last_seconds
    assert _image_digest(first_frame) != _image_digest(last_frame)


def test_charge_ring_record_rejects_a_missing_level() -> None:
    with pytest.raises(ValueError, match="charge level must be 1, 2, 3, or 4"):
        charge_ring.charge_ring_record(None)


def test_require_charge_level_returns_a_middle_level() -> None:
    assert charge_ring.require_charge_level(2) == 2
    assert charge_ring.require_charge_level(3) == 3


def test_require_charge_level_accepts_the_first_and_last_levels() -> None:
    assert charge_ring.require_charge_level(1) == 1
    assert charge_ring.require_charge_level(4) == 4


def test_require_charge_level_rejects_a_missing_level() -> None:
    with pytest.raises(ValueError, match="charge level must be 1, 2, 3, or 4"):
        charge_ring.require_charge_level(None)


def test_seconds_for_charge_level_is_thirty_five_hundredths_times_the_level() -> None:
    assert charge_ring.seconds_for_charge_level(2) == 0.7
    assert charge_ring.seconds_for_charge_level(3) == 1.05


def test_seconds_for_charge_level_spans_the_first_and_last_levels() -> None:
    assert charge_ring.seconds_for_charge_level(1) == 0.35
    assert charge_ring.seconds_for_charge_level(4) == 1.4


def test_seconds_for_charge_level_rejects_a_missing_level() -> None:
    with pytest.raises(ValueError, match="charge level must be 1, 2, 3, or 4"):
        charge_ring.seconds_for_charge_level(None)


def test_draw_charge_ring_paints_the_locked_level_two_frame() -> None:
    frame = charge_ring.draw_charge_ring(2)
    assert frame.size == (32, 32)
    assert frame.mode == "RGBA"
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert frame.getpixel((15, 15)) == (0, 0, 0, 0)
    assert frame.getpixel((16, 16)) == (0, 0, 0, 0)
    assert _colors(frame) <= BAKED_COLORS
    assert _color_count(frame, charge_ring.PIP_LIT_CENTER) == 2
    assert _image_digest(frame) == FRAME_PIXEL_DIGESTS[1]


def test_draw_charge_ring_keeps_the_open_middle_at_the_first_and_last_levels() -> None:
    first = charge_ring.draw_charge_ring(1)
    last = charge_ring.draw_charge_ring(4)
    assert first.size == (32, 32)
    assert last.size == (32, 32)
    assert first.getpixel((0, 0)) == (0, 0, 0, 0)
    assert last.getpixel((31, 31)) == (0, 0, 0, 0)
    assert first.getpixel((15, 15)) == (0, 0, 0, 0)
    assert last.getpixel((16, 16)) == (0, 0, 0, 0)
    assert _color_count(first, charge_ring.PIP_LIT_CENTER) == 1
    assert _color_count(last, charge_ring.PIP_LIT_CENTER) == 4
    assert _image_digest(first) == FRAME_PIXEL_DIGESTS[0]
    assert _image_digest(last) == FRAME_PIXEL_DIGESTS[3]


def test_draw_charge_ring_rejects_a_level_past_four() -> None:
    with pytest.raises(ValueError, match="charge level must be 1, 2, 3, or 4"):
        charge_ring.draw_charge_ring(5)


def test_paint_ring_paints_the_locked_brass_ring() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    charge_ring.paint_ring(canvas)
    assert canvas.read(15, 4) == charge_ring.RING_LIGHT
    assert canvas.read(27, 15) == charge_ring.RING_DARK
    assert canvas.read(23, 5) == charge_ring.RING
    assert _image_digest(canvas.image()) == RING_PIXEL_DIGEST


def test_paint_ring_leaves_the_middle_and_the_corner_clear() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    charge_ring.paint_ring(canvas)
    assert canvas.read(15, 15) is None
    assert canvas.read(0, 0) is None
    assert canvas.read(4, 9) is None
    assert canvas.read(5, 15) == charge_ring.RING_LIGHT


def test_paint_ring_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        charge_ring.paint_ring(None)


def test_ring_contains_the_north_band() -> None:
    assert charge_ring.ring_contains(15, 4) is True
    assert charge_ring.ring_contains(16, 4) is True


def test_ring_contains_stops_at_the_inner_and_outer_edges() -> None:
    assert charge_ring.ring_contains(5, 15) is True
    assert charge_ring.ring_contains(25, 19) is False
    assert charge_ring.ring_contains(4, 9) is False
    assert charge_ring.ring_contains(15, 15) is False


def test_ring_contains_rejects_a_missing_column() -> None:
    with pytest.raises(TypeError):
        charge_ring.ring_contains(None, 0)


def test_distance_from_center_measures_the_north_band() -> None:
    assert charge_ring.distance_from_center(15, 4) == math.hypot(-0.5, -11.5)


def test_distance_from_center_measures_the_middle_pixel_and_the_corner() -> None:
    assert charge_ring.distance_from_center(15, 15) == math.hypot(-0.5, -0.5)
    assert charge_ring.distance_from_center(0, 0) == math.hypot(-15.5, -15.5)


def test_distance_from_center_rejects_a_missing_column() -> None:
    with pytest.raises(TypeError):
        charge_ring.distance_from_center(None, 0)


def test_ring_color_lights_the_upper_left_and_darkens_the_lower_right() -> None:
    assert charge_ring.ring_color(4, 4) == charge_ring.RING_LIGHT
    assert charge_ring.ring_color(24, 24) == charge_ring.RING_DARK


def test_ring_color_uses_middle_brass_on_the_diagonal_boundaries() -> None:
    assert charge_ring.ring_color(14, 14) == charge_ring.RING
    assert charge_ring.ring_color(17, 17) == charge_ring.RING


def test_ring_color_rejects_a_missing_column() -> None:
    with pytest.raises(TypeError):
        charge_ring.ring_color(None, 0)


def test_paint_pips_lights_the_first_two_pips_for_level_two() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    charge_ring.paint_pips(canvas, 2)
    assert canvas.read(15, 4) == charge_ring.PIP_LIT_CENTER
    assert canvas.read(27, 15) == charge_ring.PIP_LIT_CENTER
    assert canvas.read(15, 27) == charge_ring.PIP_DARK_CENTER
    assert canvas.read(4, 15) == charge_ring.PIP_DARK_CENTER
    assert _image_digest(canvas.image()) == LEVEL_TWO_PIPS_DIGEST


def test_paint_pips_lights_only_the_first_pip_at_level_one_and_all_four_at_level_four() -> None:
    first = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    last = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    charge_ring.paint_pips(first, 1)
    charge_ring.paint_pips(last, 4)
    assert first.read(15, 4) == charge_ring.PIP_LIT_CENTER
    assert first.read(27, 15) == charge_ring.PIP_DARK_CENTER
    assert last.read(4, 15) == charge_ring.PIP_LIT_CENTER
    assert _image_digest(first.image()) == LEVEL_ONE_PIPS_DIGEST
    assert _image_digest(last.image()) == LEVEL_FOUR_PIPS_DIGEST


def test_paint_pips_rejects_a_missing_level() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    with pytest.raises(ValueError, match="charge level must be 1, 2, 3, or 4"):
        charge_ring.paint_pips(canvas, None)


def test_pip_color_lights_only_the_sockets_below_the_level() -> None:
    assert charge_ring.pip_color(1, 2) == charge_ring.PIP_LIT
    assert charge_ring.pip_color(2, 2) == charge_ring.PIP_DARK


def test_pip_color_keeps_the_last_socket_dark_until_level_four() -> None:
    assert charge_ring.pip_color(0, 1) == charge_ring.PIP_LIT
    assert charge_ring.pip_color(3, 1) == charge_ring.PIP_DARK
    assert charge_ring.pip_color(3, 4) == charge_ring.PIP_LIT


def test_pip_color_rejects_a_pip_index_that_is_not_a_socket() -> None:
    with pytest.raises(ValueError, match="pip index must be 0, 1, 2, or 3"):
        charge_ring.pip_color(4, 1)


def test_paint_pip_stamps_a_lit_three_by_three_gem() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    charge_ring.paint_pip(canvas, (15, 4), charge_ring.PIP_LIT)
    assert _opaque(canvas) == {
        (14, 3): charge_ring.PIP_LIT,
        (15, 3): charge_ring.PIP_LIT,
        (16, 3): charge_ring.PIP_LIT,
        (14, 4): charge_ring.PIP_LIT,
        (15, 4): charge_ring.PIP_LIT_CENTER,
        (16, 4): charge_ring.PIP_LIT,
        (14, 5): charge_ring.PIP_LIT,
        (15, 5): charge_ring.PIP_LIT,
        (16, 5): charge_ring.PIP_LIT,
    }


def test_paint_pip_stamps_a_dark_gem_in_the_corner_of_the_canvas() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    charge_ring.paint_pip(canvas, (1, 1), charge_ring.PIP_DARK)
    assert _opaque(canvas) == {
        (0, 0): charge_ring.PIP_DARK,
        (1, 0): charge_ring.PIP_DARK,
        (2, 0): charge_ring.PIP_DARK,
        (0, 1): charge_ring.PIP_DARK,
        (1, 1): charge_ring.PIP_DARK_CENTER,
        (2, 1): charge_ring.PIP_DARK,
        (0, 2): charge_ring.PIP_DARK,
        (1, 2): charge_ring.PIP_DARK,
        (2, 2): charge_ring.PIP_DARK,
    }


def test_paint_pip_rejects_a_gem_that_leaves_the_canvas() -> None:
    canvas = rejecting_canvas(charge_ring.CELL_SIZE, charge_ring.CELL_SIZE)
    with pytest.raises(ValueError, match="pixel outside the icon"):
        charge_ring.paint_pip(canvas, (0, 0), charge_ring.PIP_LIT)


def test_pip_pixel_color_puts_the_bright_center_on_a_lit_pip() -> None:
    assert charge_ring.pip_pixel_color(charge_ring.PIP_LIT, 0, 0) == charge_ring.PIP_LIT_CENTER
    assert charge_ring.pip_pixel_color(charge_ring.PIP_DARK, 0, 0) == charge_ring.PIP_DARK_CENTER


def test_pip_pixel_color_keeps_the_body_color_off_the_center() -> None:
    assert charge_ring.pip_pixel_color(charge_ring.PIP_LIT, 1, 0) == charge_ring.PIP_LIT
    assert charge_ring.pip_pixel_color(charge_ring.PIP_DARK, -1, 1) == charge_ring.PIP_DARK


def test_pip_pixel_color_rejects_an_unknown_color() -> None:
    with pytest.raises(ValueError, match="pip color must be lit or dark"):
        charge_ring.pip_pixel_color((0, 0, 0, 255), 1, 0)


def test_save_charge_ring_writes_the_level_one_frame(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "FOLDER", tmp_path)
    charge_ring.save_charge_ring(charge_ring.charge_ring_record(1))
    saved = Image.open(tmp_path / "hoe_charge_ring_f0.png").convert("RGBA")
    assert saved.size == (32, 32)
    assert _image_digest(saved) == FRAME_PIXEL_DIGESTS[0]


def test_save_charge_ring_creates_a_missing_homestead_folder(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "homestead" / "vfx"
    monkeypatch.setattr(charge_ring, "FOLDER", folder)
    charge_ring.save_charge_ring(charge_ring.charge_ring_record(4))
    saved = folder / "hoe_charge_ring_f3.png"
    assert saved.is_file()
    assert _image_digest(Image.open(saved).convert("RGBA")) == FRAME_PIXEL_DIGESTS[3]


def test_save_charge_ring_rejects_a_record_missing_the_frame(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "FOLDER", tmp_path)
    with pytest.raises(ValueError, match="not enough values to unpack"):
        charge_ring.save_charge_ring((1, 0.35))
    assert list(tmp_path.iterdir()) == []


def test_save_charge_ring_rejects_seconds_that_do_not_match_the_level(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(charge_ring, "FOLDER", tmp_path)
    frame = charge_ring.draw_charge_ring(1)
    with pytest.raises(ValueError, match="charge ring seconds do not match the level"):
        charge_ring.save_charge_ring((1, 0.0, frame))
    assert list(tmp_path.iterdir()) == []


def test_charge_ring_file_name_uses_the_frame_index_before_the_level() -> None:
    assert charge_ring.charge_ring_file_name(2) == "hoe_charge_ring_f1.png"
    assert charge_ring.charge_ring_file_name(3) == "hoe_charge_ring_f2.png"


def test_charge_ring_file_name_spans_the_first_and_last_frames() -> None:
    assert charge_ring.charge_ring_file_name(1) == "hoe_charge_ring_f0.png"
    assert charge_ring.charge_ring_file_name(4) == "hoe_charge_ring_f3.png"


def test_charge_ring_file_name_rejects_a_level_of_zero() -> None:
    with pytest.raises(ValueError, match="charge level must be 1, 2, 3, or 4"):
        charge_ring.charge_ring_file_name(0)
