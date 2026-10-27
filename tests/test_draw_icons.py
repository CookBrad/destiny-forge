from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_skill_icons as icons
from art_common import rejecting_canvas

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _image_digest(image: Image.Image) -> str:
    return hashlib.sha256(image.tobytes()).hexdigest()


def _file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_new_canvas_is_a_rejecting_32_pixel_icon() -> None:
    canvas = icons.new_canvas()
    assert (canvas.width, canvas.height) == (32, 32)
    with pytest.raises(ValueError, match=r"pixel outside the icon"):
        canvas.put(32, 0, icons.STEEL)


def test_put_stores_a_pixel_and_rejects_one_outside_the_icon() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 1, 2, icons.STEEL)
    assert canvas.read(1, 2) == icons.STEEL
    with pytest.raises(ValueError, match=r"pixel outside the icon: \(40, 0\)"):
        icons.put(canvas, 40, 0, icons.STEEL)


def test_fills_span_paints_a_closed_horizontal_span() -> None:
    canvas = icons.new_canvas()
    icons.fills_span(canvas, 1, 0, 2, icons.STEEL)
    assert canvas.read(0, 1) == icons.STEEL
    assert canvas.read(2, 1) == icons.STEEL
    assert canvas.read(3, 1) is None


def test_fills_span_rejects_a_span_that_leaves_the_icon() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.fills_span(icons.new_canvas(), 0, 30, 32, icons.STEEL)


def test_neighbor_returns_none_outside_the_icon_and_for_a_clear_pixel() -> None:
    canvas = icons.new_canvas()
    assert icons.neighbor(canvas, 0, 0, -1, 0) is None
    assert icons.neighbor(canvas, 1, 1, 0, 0) is None


def test_neighbor_returns_the_stored_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 2, 2, icons.STEEL)
    assert icons.neighbor(canvas, 2, 3, 0, -1) == icons.STEEL


def test_touches_empty_is_true_for_a_pixel_with_a_clear_side() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 1, 1, icons.STEEL)
    assert icons.touches_empty(canvas, 1, 1) is True


def test_touches_empty_is_false_when_all_four_sides_are_painted() -> None:
    canvas = icons.new_canvas()
    for x, y in ((1, 1), (0, 1), (2, 1), (1, 0), (1, 2)):
        icons.put(canvas, x, y, icons.STEEL)
    assert icons.touches_empty(canvas, 1, 1) is False


def test_outline_icon_paints_the_shared_outline_around_one_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.outline_icon(canvas)
    assert canvas.read(4, 3) == icons.OUTLINE
    assert canvas.read(4, 4) == icons.STEEL


def test_shade_outline_recolors_the_upper_outline() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.outline_icon(canvas)
    icons.shade_outline(canvas)
    assert canvas.read(4, 3) == icons.RIM_LIT


def test_finish_body_outlines_and_lights_the_upper_edge() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.finish_body(canvas)
    assert canvas.read(4, 4) == icons.STEEL
    assert canvas.read(4, 3) == icons.RIM_LIT


def test_finish_body_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(AttributeError):
        icons.finish_body(object())


def test_to_image_copies_the_canvas_pixels() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 0, 0, icons.STEEL)
    image = icons.to_image(canvas)
    assert image.size == (32, 32)
    assert image.getpixel((0, 0)) == icons.STEEL


def test_paint_sword_stores_the_bright_steel_at_the_tip() -> None:
    canvas = icons.new_canvas()
    icons.paint_sword(canvas)
    assert canvas.read(15, 3) == icons.STEEL_LIGHT


def test_draw_sword_matches_the_locked_icon_pixels() -> None:
    image = icons.draw_sword()
    assert image.getpixel((0, 0)) == (0, 0, 0, 0)
    assert image.getpixel((15, 3)) == (232, 240, 248, 255)
    assert _image_digest(image) == "680106f437068abbfd8a6ba3110bb04ebb97bb2a719810094c0f6940fbff5c9b"


def test_paint_sword_rejects_a_pixel_if_the_canvas_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_sword(rejecting_canvas(4, 4))


def test_shield_spans_cover_the_shield_from_top_to_bottom() -> None:
    spans = icons.shield_spans()
    assert len(spans) == 21
    assert spans[0] == (6, 10, 21)
    assert spans[-1] == (26, 15, 16)


def test_paint_shield_fills_the_shield_body() -> None:
    canvas = icons.new_canvas()
    icons.paint_shield(canvas)
    assert sum(canvas.opaque_at(x, y) for y in range(32) for x in range(32)) == 316


def test_paint_shield_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_shield(rejecting_canvas(4, 4))


def test_recolor_rim_leaves_an_empty_canvas_unchanged() -> None:
    canvas = icons.new_canvas()
    icons.recolor_rim(canvas, icons.BRASS_LIGHT, icons.BRASS, icons.BRASS_DARK)
    assert all(pixel == (0, 0, 0, 0) for pixel in canvas.pixels)


def test_recolor_rim_lights_the_top_of_a_single_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.recolor_rim(canvas, icons.BRASS_LIGHT, icons.BRASS, icons.BRASS_DARK)
    assert canvas.read(4, 4) == icons.BRASS_LIGHT


def test_recolor_rim_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(TypeError):
        icons.recolor_rim(None, icons.BRASS_LIGHT, icons.BRASS, icons.BRASS_DARK)


def test_stamp_boss_paints_one_pixel() -> None:
    canvas = icons.new_canvas()
    icons.stamp_boss(canvas)
    assert sum(canvas.opaque_at(x, y) for y in range(32) for x in range(32)) == 1


def test_stamp_boss_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.stamp_boss(rejecting_canvas(2, 2))


def test_paint_boot_paints_the_locked_opaque_count() -> None:
    canvas = icons.new_canvas()
    icons.paint_boot(canvas)
    assert sum(canvas.opaque_at(x, y) for y in range(32) for x in range(32)) == 196


def test_paint_boot_leaves_the_early_rows_clear() -> None:
    canvas = icons.new_canvas()
    icons.paint_boot(canvas)
    assert canvas.read(8, 8) is None


def test_paint_boot_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_boot(rejecting_canvas(4, 4))


def test_paint_boot_trail_keeps_a_pixel_that_was_already_there() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 1, 1, icons.STEEL)
    icons.paint_boot_trail(canvas)
    assert canvas.read(1, 1) == icons.STEEL


def test_paint_boot_trail_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_boot_trail(rejecting_canvas(2, 2))


def test_paint_wand_stores_the_purple_pixel() -> None:
    canvas = icons.new_canvas()
    icons.paint_wand(canvas)
    assert canvas.read(16, 10) == (214, 75, 255, 255)


def test_paint_wand_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_wand(rejecting_canvas(4, 4))


def test_paint_wand_orbit_runs_on_a_full_icon() -> None:
    canvas = icons.new_canvas()
    icons.paint_wand_orbit(canvas)
    assert canvas.width == 32


def test_paint_wand_orbit_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_wand_orbit(rejecting_canvas(4, 4))


def test_ascii_preview_draws_the_sword_glyph() -> None:
    preview = icons.ascii_preview(icons.draw_sword())
    assert hashlib.sha256(preview.encode()).hexdigest() == "0cd91b14f39d92524413c774fedfa09aa0ecf72b740a95483f629b9102732d47"
    assert next(line for line in preview.splitlines() if line.strip()) == "              ++++"


def test_ascii_preview_of_a_clear_icon_is_blank_lines() -> None:
    preview = icons.ascii_preview(icons.to_image(icons.new_canvas()))
    assert preview == "\n" * 31


def test_ascii_preview_rejects_an_image_with_no_pixels() -> None:
    with pytest.raises(AttributeError):
        icons.ascii_preview(object())


def test_skill_main_writes_the_four_icons_with_the_locked_hashes(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "skills"
    folder.mkdir()
    monkeypatch.setattr(icons, "OUT_DIR", folder)
    monkeypatch.setattr(icons.sys, "argv", ["draw_skill_icons.py"])
    icons.main()
    expected = {
        "short_wep.png": "ff5a3b96d7eb7149a93573a43cd7b768c42c8876df84a386196adc1e43c47c6d",
        "shield.png": "d09c9853fd5bac9946cddc1b878b65b76a68b04150811ec6e0aae9f4c31eafb9",
        "boot.png": "496fe8e3de666315615f721b20ce8f6a961ad4f3841db036a5c467079e7068a0",
        "wand.png": "f0f78fdf6267a855090e52364c30229002fb2f2d2eb45a68b9bb4b5ac62eac82",
    }
    for name, digest in expected.items():
        assert _file_digest(folder / name) == digest


def test_outline_icon_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(AttributeError):
        icons.outline_icon(object())


def test_shade_outline_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(AttributeError):
        icons.shade_outline(object())


def test_neighbor_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(TypeError):
        icons.neighbor(None, 0, 0, 1, 0)


def test_skill_main_check_rejects_a_missing_icon(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "skills"
    folder.mkdir()
    monkeypatch.setattr(icons, "OUT_DIR", folder)
    monkeypatch.setattr(icons.sys, "argv", ["draw_skill_icons.py", "--check"])
    with pytest.raises(SystemExit, match="missing"):
        icons.main()

def test_circle_pixels_includes_the_rightmost_point_of_the_radius() -> None:
    assert (18, 10) in icons.circle_pixels(10, 10, 8)


def test_circle_pixels_of_radius_zero_is_the_center() -> None:
    assert icons.circle_pixels(4, 5, 0) == [(4, 5)]


def test_circle_pixels_rejects_a_radius_that_is_not_a_number() -> None:
    with pytest.raises(TypeError):
        icons.circle_pixels(1, 1, None)
