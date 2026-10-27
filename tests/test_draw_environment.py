from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_environment as environment
from art_common import clipping_canvas, rejecting_canvas

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _digest(pixels) -> str:
    return hashlib.sha256(b"".join(bytes(pixel) for pixel in pixels)).hexdigest()


def test_draw_wall_paints_the_locked_wall_canvas():
    canvas = environment.draw_wall()
    assert (canvas.width, canvas.height) == (256, 256)
    assert _digest(canvas.pixels) == '31b21d8a6b151313ba959aa0401b1e2ce78b12e2e9cee0f21e4861318f60b000'


def test_draw_wall_keeps_the_locked_corner_pixel():
    assert environment.draw_wall().read(0, 0) == (32, 26, 28, 255)


def test_draw_wall_rejects_a_gray_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "GRAY", 5)
    with pytest.raises(TypeError):
        environment.draw_wall()


def test_draw_floor_paints_the_locked_floor_canvas():
    canvas = environment.draw_floor()
    assert (canvas.width, canvas.height) == (128, 32)
    assert _digest(canvas.pixels) == '38a5ba5af0b8dcf89744390e441723b703047988a56139916f1ca9fb0f659815'


def test_draw_floor_keeps_the_locked_corner_pixel():
    assert environment.draw_floor().read(0, 0) == (214, 164, 70, 255)


def test_draw_floor_rejects_a_brown_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "BROWN", 5)
    with pytest.raises(TypeError):
        environment.draw_floor()


def test_draw_ground_fill_paints_the_locked_ground_fill_canvas():
    canvas = environment.draw_ground_fill()
    assert (canvas.width, canvas.height) == (128, 128)
    assert _digest(canvas.pixels) == '5ffaca6fb8d0286f8ce22b855d6fbc7e4c39c6d89047a312fe80d1d080efe571'


def test_draw_ground_fill_keeps_the_locked_corner_pixel():
    assert environment.draw_ground_fill().read(0, 0) == (16, 14, 20, 255)


def test_draw_ground_fill_rejects_a_purple_dark_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "PURPLE_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_ground_fill()


def test_draw_ceiling_beam_paints_the_locked_ceiling_beam_canvas():
    canvas = environment.draw_ceiling_beam()
    assert (canvas.width, canvas.height) == (128, 32)
    assert _digest(canvas.pixels) == '258454173e005ad5d90158b8ea3722e03b9004af8dafe97f0e881542223671f6'


def test_draw_ceiling_beam_keeps_the_locked_corner_pixel():
    assert environment.draw_ceiling_beam().read(0, 0) == (50, 48, 58, 255)


def test_draw_ceiling_beam_rejects_a_iron_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "IRON", 5)
    with pytest.raises(TypeError):
        environment.draw_ceiling_beam()


def test_draw_platform_paints_the_locked_platform_canvas():
    canvas = environment.draw_platform()
    assert (canvas.width, canvas.height) == (128, 32)
    assert _digest(canvas.pixels) == '77af776cfb64be50b9d2735f916f45e594fea6b1c89e4ed6079a784855dc89c1'


def test_draw_platform_keeps_the_locked_corner_pixel():
    assert environment.draw_platform().read(0, 0) == (230, 170, 255, 255)


def test_draw_platform_rejects_a_iron_dark_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "IRON_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_platform()


def test_draw_pit_paints_the_locked_pit_canvas():
    canvas = environment.draw_pit()
    assert (canvas.width, canvas.height) == (128, 128)
    assert _digest(canvas.pixels) == '5d7b769bffa2b77ff790fef64a7d1a9afb472b110c283144651abfaa9a86dda4'


def test_draw_pit_keeps_the_locked_corner_pixel():
    assert environment.draw_pit().read(0, 0) == (2, 0, 8, 255)


def test_draw_pit_rejects_a_void_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "VOID", 5)
    with pytest.raises(TypeError):
        environment.draw_pit()


def test_draw_ladder_paints_the_locked_ladder_canvas():
    canvas = environment.draw_ladder()
    assert (canvas.width, canvas.height) == (64, 256)
    assert _digest(canvas.pixels) == '5de2d5267d5be02c235dff5afb32baaf830c40344e0f44b3b6101d7d497f7b43'


def test_draw_ladder_keeps_the_locked_corner_pixel():
    assert environment.draw_ladder().read(0, 0) == None


def test_draw_ladder_rejects_a_iron_dark_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "IRON_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_ladder()


def test_draw_stake_paints_the_locked_stake_canvas():
    canvas = environment.draw_stake()
    assert (canvas.width, canvas.height) == (24, 112)
    assert _digest(canvas.pixels) == '942c14b4862bddeae0e1871995757436e1e6b5193d05f3b154e0550d60fd58ff'


def test_draw_stake_keeps_the_locked_corner_pixel():
    assert environment.draw_stake().read(0, 0) == None


def test_draw_stake_rejects_a_brass_dark_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "BRASS_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_stake()


def test_draw_lip_paints_the_locked_lip_canvas():
    canvas = environment.draw_lip()
    assert (canvas.width, canvas.height) == (24, 96)
    assert _digest(canvas.pixels) == '00c04ef76b480b3674f0ac436c1c8575625ec0325bf3b1e1e19a0a81bbb8e05b'


def test_draw_lip_keeps_the_locked_corner_pixel():
    assert environment.draw_lip().read(0, 0) == (214, 164, 70, 255)


def test_draw_lip_rejects_a_brass_dark_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "BRASS_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_lip()


def test_draw_pillar_paints_the_locked_pillar_canvas():
    canvas = environment.draw_pillar()
    assert (canvas.width, canvas.height) == (64, 512)
    assert _digest(canvas.pixels) == '3cd652fca2c3ccebec9791d5c50ca36e18d506738068f7ccbfdf2497561cdd1a'


def test_draw_pillar_keeps_the_locked_corner_pixel():
    assert environment.draw_pillar().read(0, 0) == (48, 46, 56, 255)


def test_draw_pillar_rejects_a_gray_dark_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "GRAY_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_pillar()


def test_draw_torch_paints_the_locked_torch_canvas():
    canvas = environment.draw_torch()
    assert (canvas.width, canvas.height) == (32, 64)
    assert _digest(canvas.pixels) == 'a09fdee93972c6f1117599f7b850737e3250d4f8863d82725ddb3277fb44f82b'


def test_draw_torch_keeps_the_locked_corner_pixel():
    assert environment.draw_torch().read(0, 0) == None


def test_draw_torch_rejects_a_iron_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "IRON", 5)
    with pytest.raises(TypeError):
        environment.draw_torch()


def test_draw_chain_paints_the_locked_chain_canvas():
    canvas = environment.draw_chain()
    assert (canvas.width, canvas.height) == (16, 160)
    assert _digest(canvas.pixels) == 'd499182935e370dde0b8bf651f2cb743977eb2ea75d5ce82fd74a5c877f7a359'


def test_draw_chain_keeps_the_locked_corner_pixel():
    assert environment.draw_chain().read(0, 0) == None


def test_draw_chain_rejects_a_brass_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "BRASS", 5)
    with pytest.raises(TypeError):
        environment.draw_chain()


def test_draw_ground_slam_paints_the_locked_ground_slam_canvas():
    canvas = environment.draw_ground_slam()
    assert (canvas.width, canvas.height) == (96, 24)
    assert _digest(canvas.pixels) == '6906edf663352dead57c29636b258d90cacef958f0021525fed1a2e28ed19227'


def test_draw_ground_slam_keeps_the_locked_corner_pixel():
    assert environment.draw_ground_slam().read(0, 0) == None


def test_draw_ground_slam_rejects_a_orange_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "ORANGE", 5)
    with pytest.raises(TypeError):
        environment.draw_ground_slam()


def test_speck_returns_the_dark_speck_on_the_origin():
    assert environment.speck(0, 0) == -18


def test_speck_returns_no_speck_when_the_cell_hash_misses():
    assert environment.speck(1, 0) == 0
    assert environment.speck(8, 8) == 0


def test_speck_rejects_a_column_that_is_not_a_number():
    with pytest.raises(TypeError):
        environment.speck("column", 0)


def test_hash2_returns_zero_at_the_origin():
    assert environment.hash2(0, 0) == 0


def test_hash2_changes_when_the_salt_changes_and_stays_in_a_byte():
    assert environment.hash2(0, 0, 3) == 124
    assert environment.hash2(400, 12, 1) == 201


def test_hash2_rejects_a_missing_column():
    with pytest.raises(TypeError):
        environment.hash2(None, 0)


def test_beveled_color_lightens_the_outer_corner():
    assert environment.beveled_color(0, 0, 0, 0, 6, 6, environment.GRAY) == (70, 68, 78, 255)


def test_beveled_color_keeps_the_interior_on_the_base_plus_speck():
    assert environment.beveled_color(3, 3, 0, 0, 6, 6, environment.GRAY) == (68, 66, 76, 255)


def test_beveled_color_rejects_a_base_that_is_not_a_color():
    with pytest.raises(TypeError):
        environment.beveled_color(0, 0, 0, 0, 2, 2, 5)


def test_bevel_block_paints_a_six_by_six_block():
    canvas = clipping_canvas(6, 6)
    environment.bevel_block(canvas, 0, 0, 6, 6, environment.GRAY)
    assert canvas.read(0, 0) == (70, 68, 78, 255)
    assert _digest(canvas.pixels) == '2ef955cdaf1acf5e493feb62a9b7fef1baf10edb96f9c3cb0ab212a27c8fee1d'


def test_bevel_block_with_an_empty_range_paints_nothing():
    canvas = clipping_canvas(4, 4, environment.INK)
    environment.bevel_block(canvas, 1, 1, 1, 3, environment.GRAY)
    assert canvas.pixels == [environment.INK] * 16


def test_bevel_block_rejects_a_pixel_outside_a_rejecting_canvas():
    canvas = rejecting_canvas(2, 2)
    with pytest.raises(ValueError, match=r"pixel outside the icon: \(2, 0\)"):
        environment.bevel_block(canvas, 0, 0, 3, 1, environment.GRAY)


def test_bevel_wrapped_block_wraps_pixels_back_onto_the_canvas():
    canvas = clipping_canvas(4, 4)
    environment.bevel_wrapped_block(canvas, 0, 0, 6, 2, environment.GRAY)
    assert _digest(canvas.pixels) == '0f5092fa188846c8140eb26f621000f275425205ab3a544e2e7459124adfb452'


def test_bevel_wrapped_block_with_an_empty_range_leaves_the_fill():
    canvas = clipping_canvas(3, 3, environment.INK)
    environment.bevel_wrapped_block(canvas, 0, 2, 2, 2, environment.GRAY)
    assert canvas.pixels == [environment.INK] * 9


def test_bevel_wrapped_block_rejects_a_base_that_is_not_a_color():
    with pytest.raises(TypeError):
        environment.bevel_wrapped_block(clipping_canvas(2, 2), 0, 0, 1, 1, 5)


def test_crack_paints_glow_on_every_third_step():
    canvas = clipping_canvas(8, 8)
    environment.crack(canvas, ((0, 0), (4, 0)), INK, RED)
    assert [canvas.read(x, 0) for x in range(5)] == [(200, 0, 0, 255), (0, 0, 0, 255), (0, 0, 0, 255), (200, 0, 0, 255), (0, 0, 0, 255)]


def test_crack_with_one_point_paints_nothing():
    canvas = clipping_canvas(2, 2, BLUE)
    environment.crack(canvas, ((0, 0),), INK, RED)
    assert canvas.pixels == [BLUE, BLUE, BLUE, BLUE]


def test_crack_rejects_points_that_are_missing():
    with pytest.raises(TypeError):
        environment.crack(clipping_canvas(2, 2), None, INK, RED)

def test_assert_fully_opaque_accepts_a_solid_image(tmp_path: Path) -> None:
    path = tmp_path / "solid.png"
    Image.new("RGBA", (2, 2), (10, 20, 30, 255)).save(path)
    environment.assert_fully_opaque(path, 2, 2)


def test_assert_fully_opaque_accepts_a_one_pixel_image(tmp_path: Path) -> None:
    path = tmp_path / "pixel.png"
    Image.new("RGBA", (1, 1), (1, 2, 3, 255)).save(path)
    environment.assert_fully_opaque(path, 1, 1)


def test_assert_fully_opaque_rejects_a_transparent_pixel(tmp_path: Path) -> None:
    path = tmp_path / "cut.png"
    image = Image.new("RGBA", (2, 1), (1, 2, 3, 255))
    image.putpixel((1, 0), (1, 2, 3, 0))
    image.save(path)
    with pytest.raises(SystemExit, match="not fully opaque"):
        environment.assert_fully_opaque(path, 2, 1)


def test_assert_cutout_accepts_one_clear_pixel_and_one_solid_pixel(tmp_path: Path) -> None:
    path = tmp_path / "cut.png"
    image = Image.new("RGBA", (2, 1), (0, 0, 0, 0))
    image.putpixel((1, 0), (4, 5, 6, 255))
    image.save(path)
    environment.assert_cutout(path, 2, 1)


def test_assert_cutout_accepts_a_larger_mixed_image(tmp_path: Path) -> None:
    path = tmp_path / "mixed.png"
    image = Image.new("RGBA", (3, 1), (8, 8, 8, 255))
    image.putpixel((0, 0), (0, 0, 0, 0))
    image.save(path)
    environment.assert_cutout(path, 3, 1)


def test_assert_cutout_rejects_a_fully_solid_image(tmp_path: Path) -> None:
    path = tmp_path / "solid.png"
    Image.new("RGBA", (1, 1), (1, 1, 1, 255)).save(path)
    with pytest.raises(SystemExit, match="no transparent pixel"):
        environment.assert_cutout(path, 1, 1)


def test_alpha_values_rejects_the_wrong_size(tmp_path: Path) -> None:
    path = tmp_path / "small.png"
    Image.new("RGBA", (1, 1), (1, 1, 1, 255)).save(path)
    with pytest.raises(SystemExit, match="expected"):
        environment.alpha_values(path, 2, 2)
