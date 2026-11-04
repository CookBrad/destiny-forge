from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_environment as environment
from art_common import clipping_canvas, rejecting_canvas

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
PARTIAL = (1, 2, 3, 128)


def _digest(pixels) -> str:
    return hashlib.sha256(b"".join(bytes(pixel) for pixel in pixels)).hexdigest()


def _redirect_piece_folders(monkeypatch, root: Path) -> tuple[Path, Path]:
    environment_folder = root / "environment"
    effects_folder = root / "effects"
    monkeypatch.setattr(environment, "ROOT", root)
    monkeypatch.setattr(environment, "ENV", environment_folder)
    monkeypatch.setattr(environment, "VFX", effects_folder)
    return environment_folder, effects_folder


def _solid_canvas(width: int, height: int, color: tuple[int, int, int, int]):
    return clipping_canvas(width, height, color)


def _marked_cutout(width: int, height: int, color: tuple[int, int, int, int]):
    canvas = clipping_canvas(width, height)
    canvas.put(0, 0, color)
    return canvas


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


def test_beveled_color_lightens_the_outer_corner():
    assert environment.beveled_color(0, 0, 0, 0, 6, 6, environment.GRAY) == (70, 68, 78, 255)


def test_beveled_color_keeps_the_interior_on_the_base_plus_speck():
    assert environment.beveled_color(3, 3, 0, 0, 6, 6, environment.GRAY) == (68, 66, 76, 255)


def test_beveled_color_rejects_a_base_that_is_not_a_color():
    with pytest.raises(TypeError):
        environment.beveled_color(0, 0, 0, 0, 2, 2, 5)


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


def test_clipping_canvas_ignores_a_pixel_drawn_outside_the_icon() -> None:
    canvas = clipping_canvas(2, 2, BLUE)
    canvas.put(-1, 0, RED)
    canvas.put(2, 0, RED)
    assert canvas.pixels == [BLUE, BLUE, BLUE, BLUE]


def test_clipping_canvas_read_of_a_clear_pixel_is_none() -> None:
    canvas = clipping_canvas(1, 1)
    assert canvas.read(0, 0) is None
    assert canvas.read(5, 5) is None
    assert canvas.opaque_at(0, 0) is False


def test_clipping_canvas_rejects_a_partially_transparent_pixel() -> None:
    canvas = clipping_canvas(1, 1)
    with pytest.raises(SystemExit, match="partial alpha"):
        canvas.put(0, 0, PARTIAL)


def test_write_opaque_pieces_saves_each_opaque_piece(tmp_path: Path, monkeypatch) -> None:
    environment_folder, _effects_folder = _redirect_piece_folders(monkeypatch, tmp_path / "forge")
    pieces = (
        (environment.draw_wall, "wall.png", 256, 256),
        (environment.draw_floor, "floor_ground.png", 128, 32),
        (environment.draw_ground_fill, "ground_fill.png", 128, 128),
        (environment.draw_ceiling_beam, "ceiling_beam.png", 128, 32),
        (environment.draw_platform, "floor_platform.png", 128, 32),
        (environment.draw_pit, "floor_pit.png", 128, 128),
    )
    expected = {name: (width, height, list(draw().pixels)) for draw, name, width, height in pieces}
    environment.write_opaque_pieces()
    assert sorted(path.name for path in environment_folder.iterdir()) == sorted(expected)
    for name, (width, height, pixels) in expected.items():
        image = Image.open(environment_folder / name)
        assert image.size == (width, height)
        assert image.mode == "RGBA"
        assert image.tobytes() == b"".join(bytes(pixel) for pixel in pixels)
        assert {pixel[3] for pixel in pixels} == {255}


def test_write_opaque_pieces_replaces_an_existing_wall_image(tmp_path: Path, monkeypatch) -> None:
    environment_folder, _effects_folder = _redirect_piece_folders(monkeypatch, tmp_path / "forge")
    environment_folder.mkdir(parents=True)
    Image.new("RGBA", (1, 1), (9, 8, 7, 255)).save(environment_folder / "wall.png")
    monkeypatch.setattr(environment, "draw_wall", lambda: _solid_canvas(256, 256, environment.MORTAR_DARK))
    monkeypatch.setattr(environment, "draw_floor", lambda: _solid_canvas(128, 32, environment.BROWN))
    monkeypatch.setattr(environment, "draw_ground_fill", lambda: _solid_canvas(128, 128, environment.INK))
    monkeypatch.setattr(environment, "draw_ceiling_beam", lambda: _solid_canvas(128, 32, environment.IRON))
    monkeypatch.setattr(environment, "draw_platform", lambda: _solid_canvas(128, 32, environment.PURPLE_LIGHT))
    monkeypatch.setattr(environment, "draw_pit", lambda: _solid_canvas(128, 128, environment.VOID))
    environment.write_opaque_pieces()
    image = Image.open(environment_folder / "wall.png")
    assert image.size == (256, 256)
    assert image.getpixel((0, 0)) == environment.MORTAR_DARK


def test_write_opaque_pieces_rejects_a_wall_canvas_of_the_wrong_size(tmp_path: Path, monkeypatch) -> None:
    environment_folder, _effects_folder = _redirect_piece_folders(monkeypatch, tmp_path / "forge")
    monkeypatch.setattr(environment, "draw_wall", lambda: clipping_canvas(1, 1, environment.INK))
    with pytest.raises(SystemExit, match=r"canvas wall.png is \(1, 1\)"):
        environment.write_opaque_pieces()
    assert environment_folder.exists() is False


def test_write_cutout_pieces_saves_each_cutout_piece(tmp_path: Path, monkeypatch) -> None:
    environment_folder, effects_folder = _redirect_piece_folders(monkeypatch, tmp_path / "forge")
    pieces = (
        (environment.draw_ladder, environment_folder / "floor_ladder.png", 64, 256),
        (environment.draw_stake, environment_folder / "pit_stake.png", 24, 112),
        (environment.draw_lip, environment_folder / "pit_lip.png", 24, 96),
        (environment.draw_pillar, environment_folder / "pillar.png", 64, 512),
        (environment.draw_torch, environment_folder / "torch.png", 32, 64),
        (environment.draw_chain, environment_folder / "chain.png", 16, 160),
        (environment.draw_ground_slam, effects_folder / "ground_slam.png", 96, 24),
    )
    expected = {path: (width, height, list(draw().pixels)) for draw, path, width, height in pieces}
    environment.write_cutout_pieces()
    assert sorted(environment_folder.iterdir()) == sorted(path for path in expected if path.parent == environment_folder)
    assert sorted(effects_folder.iterdir()) == [effects_folder / "ground_slam.png"]
    for path, (width, height, pixels) in expected.items():
        image = Image.open(path)
        assert image.size == (width, height)
        assert image.mode == "RGBA"
        assert image.tobytes() == b"".join(bytes(pixel) for pixel in pixels)
        assert {pixel[3] for pixel in pixels} == {0, 255}


def test_write_cutout_pieces_replaces_an_existing_ladder_image(tmp_path: Path, monkeypatch) -> None:
    environment_folder, _effects_folder = _redirect_piece_folders(monkeypatch, tmp_path / "forge")
    environment_folder.mkdir(parents=True)
    Image.new("RGBA", (1, 1), (9, 8, 7, 255)).save(environment_folder / "floor_ladder.png")
    monkeypatch.setattr(environment, "draw_ladder", lambda: _marked_cutout(64, 256, environment.IRON_DARK))
    monkeypatch.setattr(environment, "draw_stake", lambda: _marked_cutout(24, 112, environment.BRASS_DARK))
    monkeypatch.setattr(environment, "draw_lip", lambda: _marked_cutout(24, 96, environment.BRASS))
    monkeypatch.setattr(environment, "draw_pillar", lambda: _marked_cutout(64, 512, environment.GRAY_DARK))
    monkeypatch.setattr(environment, "draw_torch", lambda: _marked_cutout(32, 64, environment.IRON))
    monkeypatch.setattr(environment, "draw_chain", lambda: _marked_cutout(16, 160, environment.BRASS))
    monkeypatch.setattr(environment, "draw_ground_slam", lambda: _marked_cutout(96, 24, environment.ORANGE))
    environment.write_cutout_pieces()
    image = Image.open(environment_folder / "floor_ladder.png")
    assert image.size == (64, 256)
    assert image.getpixel((0, 0)) == environment.IRON_DARK
    assert image.getpixel((1, 0)) == (0, 0, 0, 0)


def test_write_cutout_pieces_rejects_a_ladder_canvas_of_the_wrong_size(tmp_path: Path, monkeypatch) -> None:
    environment_folder, effects_folder = _redirect_piece_folders(monkeypatch, tmp_path / "forge")
    monkeypatch.setattr(environment, "draw_ladder", lambda: clipping_canvas(1, 1))
    with pytest.raises(SystemExit, match=r"canvas floor_ladder.png is \(1, 1\)"):
        environment.write_cutout_pieces()
    assert environment_folder.exists() is False
    assert effects_folder.exists() is False


def test_save_checked_piece_writes_a_canvas_whose_size_matches(tmp_path: Path) -> None:
    canvas = clipping_canvas(2, 2, environment.INK)
    canvas.put(1, 0, environment.BRASS)
    path = tmp_path / "piece.png"
    environment.save_checked_piece(canvas, path, 2, 2)
    image = Image.open(path)
    assert image.size == (2, 2)
    assert image.mode == "RGBA"
    assert image.tobytes() == b"".join(bytes(pixel) for pixel in canvas.pixels)


def test_save_checked_piece_writes_a_one_pixel_canvas(tmp_path: Path) -> None:
    canvas = clipping_canvas(1, 1, environment.GRAY)
    path = tmp_path / "pixel.png"
    environment.save_checked_piece(canvas, path, 1, 1)
    image = Image.open(path)
    assert image.size == (1, 1)
    assert image.mode == "RGBA"
    assert image.getpixel((0, 0)) == environment.GRAY


def test_save_checked_piece_rejects_a_canvas_whose_size_differs(tmp_path: Path) -> None:
    canvas = clipping_canvas(2, 1, environment.INK)
    path = tmp_path / "wide.png"
    with pytest.raises(SystemExit, match=r"canvas wide.png is \(2, 1\)"):
        environment.save_checked_piece(canvas, path, 1, 1)
    assert path.exists() is False


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


def test_assert_cutout_accepts_one_clear_pixel_beside_one_solid_pixel(tmp_path: Path) -> None:
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


def test_assert_cutout_rejects_a_fully_clear_image(tmp_path: Path) -> None:
    path = tmp_path / "clear.png"
    Image.new("RGBA", (1, 1), (0, 0, 0, 0)).save(path)
    with pytest.raises(SystemExit, match="no opaque pixel"):
        environment.assert_cutout(path, 1, 1)


def test_alpha_values_returns_clear_with_opaque(tmp_path: Path) -> None:
    path = tmp_path / "mixed.png"
    image = Image.new("RGBA", (2, 1), (0, 0, 0, 0))
    image.putpixel((1, 0), (4, 5, 6, 255))
    image.save(path)
    assert environment.alpha_values(path, 2, 1) == {0, 255}


def test_alpha_values_returns_the_opaque_channel_of_a_one_pixel_image(tmp_path: Path) -> None:
    path = tmp_path / "pixel.png"
    Image.new("RGBA", (1, 1), (1, 2, 3, 255)).save(path)
    assert environment.alpha_values(path, 1, 1) == {255}


def test_alpha_values_rejects_the_wrong_size(tmp_path: Path) -> None:
    path = tmp_path / "small.png"
    Image.new("RGBA", (1, 1), (1, 1, 1, 255)).save(path)
    with pytest.raises(SystemExit, match="expected"):
        environment.alpha_values(path, 2, 2)


def test_alpha_values_rejects_a_red_green_blue_image(tmp_path: Path) -> None:
    path = tmp_path / "color.png"
    Image.new("RGB", (2, 2), (1, 2, 3)).save(path)
    with pytest.raises(SystemExit, match="expected RGBA"):
        environment.alpha_values(path, 2, 2)
