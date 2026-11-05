from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_environment as environment
from art_common import clipping_canvas, rejecting_canvas

OPAQUE_PIECE_REPORT = (
    "wrote environment/wall.png 256x256\n"
    "wrote environment/floor_ground.png 128x32\n"
    "wrote environment/ground_fill.png 128x128\n"
    "wrote environment/ceiling_beam.png 128x32\n"
    "wrote environment/floor_platform.png 128x32\n"
    "wrote environment/floor_pit.png 128x128\n"
)
CUTOUT_PIECE_REPORT = (
    "wrote environment/floor_ladder.png 64x256\n"
    "wrote environment/pit_stake.png 24x112\n"
    "wrote environment/pit_lip.png 24x96\n"
    "wrote environment/pillar.png 64x512\n"
    "wrote environment/torch.png 32x64\n"
    "wrote environment/chain.png 16x160\n"
    "wrote vfx/ground_slam.png 96x24\n"
)
RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
PARTIAL = (1, 2, 3, 128)


def _digest(pixels: list[tuple[int, int, int, int]]) -> str:
    return hashlib.sha256(b"".join(bytes(pixel) for pixel in pixels)).hexdigest()


def _redirect_piece_folders(monkeypatch: pytest.MonkeyPatch, root: Path) -> tuple[Path, Path]:
    environment_folder = root / "environment"
    visual_effects_folder = root / "vfx"
    monkeypatch.setattr(environment, "ROOT", root)
    monkeypatch.setattr(environment, "ENV", environment_folder)
    monkeypatch.setattr(environment, "VFX", visual_effects_folder)
    return environment_folder, visual_effects_folder


def _saved_piece_record(path: Path) -> tuple[str, tuple[int, int], str, set[int]]:
    with Image.open(path) as image:
        width, height = image.size
        alphas = {image.getpixel((x, y))[3] for y in range(height) for x in range(width)}
        return path.name, image.size, image.mode, alphas


def _saved_pixels(path: Path) -> list[tuple[int, int, int, int]]:
    with Image.open(path) as image:
        width, height = image.size
        return [image.getpixel((x, y)) for y in range(height) for x in range(width)]


def _solid_canvas(width: int, height: int, color: tuple[int, int, int, int]):
    return clipping_canvas(width, height, color)


def _marked_cutout(width: int, height: int, color: tuple[int, int, int, int]):
    canvas = clipping_canvas(width, height)
    canvas.put(0, 0, color)
    return canvas


def test_draw_wall_paints_the_locked_wall_canvas() -> None:
    canvas = environment.draw_wall()
    assert (canvas.width, canvas.height) == (256, 256)
    assert _digest(canvas.pixels) == "31b21d8a6b151313ba959aa0401b1e2ce78b12e2e9cee0f21e4861318f60b000"


def test_draw_wall_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_wall().read(0, 0) == (32, 26, 28, 255)


def test_draw_wall_rejects_a_gray_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "GRAY", 5)
    with pytest.raises(TypeError):
        environment.draw_wall()


def test_bevel_block_paints_a_six_by_six_block() -> None:
    canvas = clipping_canvas(6, 6)
    environment.bevel_block(canvas, 0, 0, 6, 6, environment.GRAY)
    assert canvas.read(0, 0) == (70, 68, 78, 255)
    assert _digest(canvas.pixels) == "2ef955cdaf1acf5e493feb62a9b7fef1baf10edb96f9c3cb0ab212a27c8fee1d"


def test_bevel_block_with_an_empty_range_paints_nothing() -> None:
    canvas = clipping_canvas(4, 4, environment.INK)
    environment.bevel_block(canvas, 1, 1, 1, 3, environment.GRAY)
    assert canvas.pixels == [environment.INK] * 16


def test_bevel_block_rejects_a_pixel_outside_a_rejecting_canvas() -> None:
    canvas = rejecting_canvas(2, 2)
    with pytest.raises(ValueError, match=r"pixel outside the icon: \(2, 0\)"):
        environment.bevel_block(canvas, 0, 0, 3, 1, environment.GRAY)


def test_beveled_color_lightens_the_outer_corner() -> None:
    assert environment.beveled_color(0, 0, 0, 0, 6, 6, environment.GRAY) == (70, 68, 78, 255)


def test_beveled_color_keeps_the_interior_on_the_base_plus_speck() -> None:
    assert environment.beveled_color(3, 3, 0, 0, 6, 6, environment.GRAY) == (68, 66, 76, 255)


def test_beveled_color_rejects_a_base_that_is_not_a_color() -> None:
    with pytest.raises(TypeError):
        environment.beveled_color(0, 0, 0, 0, 2, 2, 5)


def test_bevel_wrapped_block_wraps_pixels_back_onto_the_canvas() -> None:
    canvas = clipping_canvas(4, 4)
    environment.bevel_wrapped_block(canvas, 0, 0, 6, 2, environment.GRAY)
    assert _digest(canvas.pixels) == "0f5092fa188846c8140eb26f621000f275425205ab3a544e2e7459124adfb452"


def test_bevel_wrapped_block_with_an_empty_range_leaves_the_fill() -> None:
    canvas = clipping_canvas(3, 3, environment.INK)
    environment.bevel_wrapped_block(canvas, 0, 2, 2, 2, environment.GRAY)
    assert canvas.pixels == [environment.INK] * 9


def test_bevel_wrapped_block_rejects_a_base_that_is_not_a_color() -> None:
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


def test_write_opaque_pieces_saves_each_solid_piece(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    environment.write_opaque_pieces()
    assert capsys.readouterr().out == OPAQUE_PIECE_REPORT
    assert visual_effects_folder.exists() is False
    assert sorted(path.name for path in environment_folder.iterdir()) == [
        "ceiling_beam.png",
        "floor_ground.png",
        "floor_pit.png",
        "floor_platform.png",
        "ground_fill.png",
        "wall.png",
    ]
    assert [
        _saved_piece_record(environment_folder / "wall.png"),
        _saved_piece_record(environment_folder / "floor_ground.png"),
        _saved_piece_record(environment_folder / "ground_fill.png"),
        _saved_piece_record(environment_folder / "ceiling_beam.png"),
        _saved_piece_record(environment_folder / "floor_platform.png"),
        _saved_piece_record(environment_folder / "floor_pit.png"),
    ] == [
        ("wall.png", (256, 256), "RGBA", {255}),
        ("floor_ground.png", (128, 32), "RGBA", {255}),
        ("ground_fill.png", (128, 128), "RGBA", {255}),
        ("ceiling_beam.png", (128, 32), "RGBA", {255}),
        ("floor_platform.png", (128, 32), "RGBA", {255}),
        ("floor_pit.png", (128, 128), "RGBA", {255}),
    ]
    assert _saved_pixels(environment_folder / "wall.png") == list(environment.draw_wall().pixels)
    assert _saved_pixels(environment_folder / "floor_ground.png") == list(environment.draw_floor().pixels)
    assert _saved_pixels(environment_folder / "ground_fill.png") == list(environment.draw_ground_fill().pixels)
    assert _saved_pixels(environment_folder / "ceiling_beam.png") == list(environment.draw_ceiling_beam().pixels)
    assert _saved_pixels(environment_folder / "floor_platform.png") == list(environment.draw_platform().pixels)
    assert _saved_pixels(environment_folder / "floor_pit.png") == list(environment.draw_pit().pixels)


def test_write_opaque_pieces_keeps_an_unrelated_file_in_the_folder(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    environment_folder.mkdir()
    note = environment_folder / "note.txt"
    note.write_text("keep")
    environment.write_opaque_pieces()
    assert note.read_text() == "keep"
    assert visual_effects_folder.exists() is False
    assert capsys.readouterr().out == OPAQUE_PIECE_REPORT


def test_write_opaque_pieces_replaces_an_existing_wall_image(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
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
    assert visual_effects_folder.exists() is False


def test_write_opaque_pieces_rejects_a_wall_that_is_not_fully_opaque(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    monkeypatch.setattr(
        environment,
        "draw_wall",
        lambda: environment.Canvas(environment.WALL_PATTERN, environment.WALL_PATTERN),
    )
    with pytest.raises(SystemExit) as raised:
        environment.write_opaque_pieces()
    assert str(raised.value) == f"{environment_folder / 'wall.png'} is not fully opaque: [0]"
    assert (environment_folder / "wall.png").is_file() is True
    assert (environment_folder / "floor_ground.png").exists() is False
    assert visual_effects_folder.exists() is False


def test_write_opaque_pieces_rejects_a_wall_canvas_of_the_wrong_size(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    monkeypatch.setattr(environment, "draw_wall", lambda: clipping_canvas(1, 1, environment.INK))
    with pytest.raises(SystemExit, match=r"canvas wall.png is \(1, 1\)"):
        environment.write_opaque_pieces()
    assert environment_folder.exists() is False
    assert visual_effects_folder.exists() is False


def test_write_cutout_pieces_saves_each_cutout_piece(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    environment.write_cutout_pieces()
    assert capsys.readouterr().out == CUTOUT_PIECE_REPORT
    assert sorted(path.name for path in environment_folder.iterdir()) == [
        "chain.png",
        "floor_ladder.png",
        "pillar.png",
        "pit_lip.png",
        "pit_stake.png",
        "torch.png",
    ]
    assert sorted(path.name for path in visual_effects_folder.iterdir()) == ["ground_slam.png"]
    assert [
        _saved_piece_record(environment_folder / "floor_ladder.png"),
        _saved_piece_record(environment_folder / "pit_stake.png"),
        _saved_piece_record(environment_folder / "pit_lip.png"),
        _saved_piece_record(environment_folder / "pillar.png"),
        _saved_piece_record(environment_folder / "torch.png"),
        _saved_piece_record(environment_folder / "chain.png"),
        _saved_piece_record(visual_effects_folder / "ground_slam.png"),
    ] == [
        ("floor_ladder.png", (64, 256), "RGBA", {0, 255}),
        ("pit_stake.png", (24, 112), "RGBA", {0, 255}),
        ("pit_lip.png", (24, 96), "RGBA", {0, 255}),
        ("pillar.png", (64, 512), "RGBA", {0, 255}),
        ("torch.png", (32, 64), "RGBA", {0, 255}),
        ("chain.png", (16, 160), "RGBA", {0, 255}),
        ("ground_slam.png", (96, 24), "RGBA", {0, 255}),
    ]
    assert _saved_pixels(environment_folder / "floor_ladder.png") == list(environment.draw_ladder().pixels)
    assert _saved_pixels(environment_folder / "pit_stake.png") == list(environment.draw_stake().pixels)
    assert _saved_pixels(environment_folder / "pit_lip.png") == list(environment.draw_lip().pixels)
    assert _saved_pixels(environment_folder / "pillar.png") == list(environment.draw_pillar().pixels)
    assert _saved_pixels(environment_folder / "torch.png") == list(environment.draw_torch().pixels)
    assert _saved_pixels(environment_folder / "chain.png") == list(environment.draw_chain().pixels)
    assert _saved_pixels(visual_effects_folder / "ground_slam.png") == list(environment.draw_ground_slam().pixels)


def test_write_cutout_pieces_keeps_an_unrelated_file_in_each_folder(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    environment_folder.mkdir()
    visual_effects_folder.mkdir()
    environment_note = environment_folder / "note.txt"
    visual_note = visual_effects_folder / "note.txt"
    environment_note.write_text("keep environment")
    visual_note.write_text("keep visual")
    environment.write_cutout_pieces()
    assert environment_note.read_text() == "keep environment"
    assert visual_note.read_text() == "keep visual"
    assert capsys.readouterr().out == CUTOUT_PIECE_REPORT


def test_write_cutout_pieces_replaces_an_existing_ladder_image(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    environment_folder, _visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
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


def test_write_cutout_pieces_rejects_a_ladder_with_no_transparent_pixel(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    monkeypatch.setattr(
        environment,
        "draw_ladder",
        lambda: environment.Canvas(environment.LADDER_W, environment.LADDER_H, environment.GRAY),
    )
    with pytest.raises(SystemExit) as raised:
        environment.write_cutout_pieces()
    assert str(raised.value) == f"{environment_folder / 'floor_ladder.png'} has no transparent pixel"
    assert (environment_folder / "floor_ladder.png").is_file() is True
    assert (environment_folder / "pit_stake.png").exists() is False
    assert visual_effects_folder.exists() is False


def test_write_cutout_pieces_rejects_a_ladder_canvas_of_the_wrong_size(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    monkeypatch.setattr(environment, "draw_ladder", lambda: clipping_canvas(1, 1))
    with pytest.raises(SystemExit, match=r"canvas floor_ladder.png is \(1, 1\)"):
        environment.write_cutout_pieces()
    assert environment_folder.exists() is False
    assert visual_effects_folder.exists() is False


def test_save_checked_piece_writes_the_canvas_when_the_size_matches(tmp_path: Path) -> None:
    color = (9, 8, 7, 255)
    canvas = environment.Canvas(2, 3, color)
    path = tmp_path / "piece.png"
    environment.save_checked_piece(canvas, path, 2, 3)
    assert _saved_piece_record(path) == ("piece.png", (2, 3), "RGBA", {255})
    assert _saved_pixels(path) == [color, color, color, color, color, color]


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


def test_save_checked_piece_creates_a_missing_parent_folder(tmp_path: Path) -> None:
    color = (4, 5, 6, 255)
    canvas = environment.Canvas(1, 1, color)
    path = tmp_path / "nested" / "room" / "piece.png"
    environment.save_checked_piece(canvas, path, 1, 1)
    assert _saved_piece_record(path) == ("piece.png", (1, 1), "RGBA", {255})
    assert _saved_pixels(path) == [color]


def test_save_checked_piece_rejects_a_canvas_with_the_wrong_size(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    canvas = environment.Canvas(2, 2, (9, 8, 7, 255))
    path = tmp_path / "missing" / "piece.png"

    def save_must_stay_idle(saved_canvas: environment.Canvas, saved_path: Path) -> None:
        raise AssertionError(f"{saved_path.name} {saved_canvas.width}")

    monkeypatch.setattr(environment, "save", save_must_stay_idle)
    with pytest.raises(SystemExit) as raised:
        environment.save_checked_piece(canvas, path, 4, 5)
    assert str(raised.value) == "canvas piece.png is (2, 2)"
    assert path.exists() is False
    assert path.parent.exists() is False


def test_save_checked_piece_rejects_a_canvas_whose_size_differs(tmp_path: Path) -> None:
    canvas = clipping_canvas(2, 1, environment.INK)
    path = tmp_path / "wide.png"
    with pytest.raises(SystemExit, match=r"canvas wide.png is \(2, 1\)"):
        environment.save_checked_piece(canvas, path, 1, 1)
    assert path.exists() is False


def test_assert_fully_opaque_accepts_a_solid_image(tmp_path: Path) -> None:
    path = tmp_path / "solid.png"
    Image.new("RGBA", (2, 2), (10, 20, 30, 255)).save(path)
    assert environment.assert_fully_opaque(path, 2, 2) is None


def test_assert_fully_opaque_accepts_a_one_pixel_image(tmp_path: Path) -> None:
    path = tmp_path / "pixel.png"
    Image.new("RGBA", (1, 1), (1, 2, 3, 255)).save(path)
    assert environment.assert_fully_opaque(path, 1, 1) is None


def test_assert_fully_opaque_rejects_a_transparent_pixel(tmp_path: Path) -> None:
    path = tmp_path / "cut.png"
    image = Image.new("RGBA", (2, 1), (1, 2, 3, 255))
    image.putpixel((1, 0), (1, 2, 3, 0))
    image.save(path)
    with pytest.raises(SystemExit) as raised:
        environment.assert_fully_opaque(path, 2, 1)
    assert str(raised.value) == f"{path} is not fully opaque: [0, 255]"


def test_assert_cutout_accepts_one_clear_pixel_beside_one_solid_pixel(tmp_path: Path) -> None:
    path = tmp_path / "cut.png"
    image = Image.new("RGBA", (2, 1), (0, 0, 0, 0))
    image.putpixel((1, 0), (4, 5, 6, 255))
    image.save(path)
    assert environment.assert_cutout(path, 2, 1) is None


def test_assert_cutout_accepts_a_larger_mixed_image(tmp_path: Path) -> None:
    path = tmp_path / "mixed.png"
    image = Image.new("RGBA", (3, 1), (8, 8, 8, 255))
    image.putpixel((0, 0), (0, 0, 0, 0))
    image.save(path)
    assert environment.assert_cutout(path, 3, 1) is None


def test_assert_cutout_rejects_a_fully_solid_image(tmp_path: Path) -> None:
    path = tmp_path / "solid.png"
    Image.new("RGBA", (1, 1), (1, 1, 1, 255)).save(path)
    with pytest.raises(SystemExit) as raised:
        environment.assert_cutout(path, 1, 1)
    assert str(raised.value) == f"{path} has no transparent pixel"


def test_assert_cutout_rejects_an_image_with_no_opaque_pixel(tmp_path: Path) -> None:
    path = tmp_path / "clear.png"
    Image.new("RGBA", (1, 1), (0, 0, 0, 0)).save(path)
    with pytest.raises(SystemExit) as raised:
        environment.assert_cutout(path, 1, 1)
    assert str(raised.value) == f"{path} has no opaque pixel"


def test_alpha_values_returns_the_solid_alpha(tmp_path: Path) -> None:
    path = tmp_path / "solid.png"
    Image.new("RGBA", (2, 2), (10, 20, 30, 255)).save(path)
    assert environment.alpha_values(path, 2, 2) == {255}


def test_alpha_values_returns_the_clear_alpha_of_one_pixel(tmp_path: Path) -> None:
    path = tmp_path / "pixel.png"
    Image.new("RGBA", (1, 1), (0, 0, 0, 0)).save(path)
    assert environment.alpha_values(path, 1, 1) == {0}


def test_alpha_values_returns_clear_with_opaque(tmp_path: Path) -> None:
    path = tmp_path / "mixed.png"
    image = Image.new("RGBA", (2, 1), (0, 0, 0, 0))
    image.putpixel((1, 0), (4, 5, 6, 255))
    image.save(path)
    assert environment.alpha_values(path, 2, 1) == {0, 255}


def test_alpha_values_rejects_a_mode_that_is_not_rgba(tmp_path: Path) -> None:
    path = tmp_path / "rgb.png"
    Image.new("RGB", (2, 2), (1, 2, 3)).save(path)
    with pytest.raises(SystemExit) as raised:
        environment.alpha_values(path, 2, 2)
    assert str(raised.value) == f"{path} mode RGB, expected RGBA"


def test_alpha_values_rejects_the_wrong_size(tmp_path: Path) -> None:
    path = tmp_path / "small.png"
    Image.new("RGBA", (1, 1), (1, 1, 1, 255)).save(path)
    with pytest.raises(SystemExit) as raised:
        environment.alpha_values(path, 2, 2)
    assert str(raised.value) == f"{path} is (1, 1), expected (2, 2)"
