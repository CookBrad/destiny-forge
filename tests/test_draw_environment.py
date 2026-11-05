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
INK = (0, 0, 0, 255)
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


def test_draw_floor_paints_the_locked_floor_canvas() -> None:
    canvas = environment.draw_floor()
    assert (canvas.width, canvas.height) == (128, 32)
    assert _digest(canvas.pixels) == "38a5ba5af0b8dcf89744390e441723b703047988a56139916f1ca9fb0f659815"


def test_draw_floor_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_floor().read(0, 0) == (214, 164, 70, 255)


def test_draw_floor_rejects_a_brown_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "BROWN", 5)
    with pytest.raises(TypeError):
        environment.draw_floor()


def test_draw_ground_fill_paints_the_locked_ground_fill_canvas() -> None:
    canvas = environment.draw_ground_fill()
    assert (canvas.width, canvas.height) == (128, 128)
    assert _digest(canvas.pixels) == "5ffaca6fb8d0286f8ce22b855d6fbc7e4c39c6d89047a312fe80d1d080efe571"


def test_draw_ground_fill_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_ground_fill().read(0, 0) == (16, 14, 20, 255)


def test_draw_ground_fill_rejects_a_purple_dark_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "PURPLE_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_ground_fill()


def test_draw_ceiling_beam_paints_the_locked_ceiling_beam_canvas() -> None:
    canvas = environment.draw_ceiling_beam()
    assert (canvas.width, canvas.height) == (128, 32)
    assert _digest(canvas.pixels) == "258454173e005ad5d90158b8ea3722e03b9004af8dafe97f0e881542223671f6"


def test_draw_ceiling_beam_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_ceiling_beam().read(0, 0) == (50, 48, 58, 255)


def test_draw_ceiling_beam_rejects_a_iron_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "IRON", 5)
    with pytest.raises(TypeError):
        environment.draw_ceiling_beam()


def test_draw_platform_paints_the_locked_platform_canvas() -> None:
    canvas = environment.draw_platform()
    assert (canvas.width, canvas.height) == (128, 32)
    assert _digest(canvas.pixels) == "77af776cfb64be50b9d2735f916f45e594fea6b1c89e4ed6079a784855dc89c1"


def test_draw_platform_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_platform().read(0, 0) == (230, 170, 255, 255)


def test_draw_platform_rejects_a_iron_dark_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "IRON_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_platform()


def test_draw_pit_paints_the_locked_pit_canvas() -> None:
    canvas = environment.draw_pit()
    assert (canvas.width, canvas.height) == (128, 128)
    assert _digest(canvas.pixels) == "5d7b769bffa2b77ff790fef64a7d1a9afb472b110c283144651abfaa9a86dda4"


def test_draw_pit_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_pit().read(0, 0) == (2, 0, 8, 255)


def test_draw_pit_rejects_a_void_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "VOID", 5)
    with pytest.raises(TypeError):
        environment.draw_pit()


def test_draw_ladder_paints_the_locked_ladder_canvas() -> None:
    canvas = environment.draw_ladder()
    assert (canvas.width, canvas.height) == (64, 256)
    assert _digest(canvas.pixels) == "5de2d5267d5be02c235dff5afb32baaf830c40344e0f44b3b6101d7d497f7b43"


def test_draw_ladder_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_ladder().read(0, 0) is None


def test_draw_ladder_rejects_a_iron_dark_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "IRON_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_ladder()


def test_draw_stake_paints_the_locked_stake_canvas() -> None:
    canvas = environment.draw_stake()
    assert (canvas.width, canvas.height) == (24, 112)
    assert _digest(canvas.pixels) == "942c14b4862bddeae0e1871995757436e1e6b5193d05f3b154e0550d60fd58ff"


def test_draw_stake_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_stake().read(0, 0) is None


def test_draw_stake_rejects_a_brass_dark_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "BRASS_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_stake()


def test_draw_lip_paints_the_locked_lip_canvas() -> None:
    canvas = environment.draw_lip()
    assert (canvas.width, canvas.height) == (24, 96)
    assert _digest(canvas.pixels) == "00c04ef76b480b3674f0ac436c1c8575625ec0325bf3b1e1e19a0a81bbb8e05b"


def test_draw_lip_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_lip().read(0, 0) == (214, 164, 70, 255)


def test_draw_lip_rejects_a_brass_dark_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "BRASS_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_lip()


def test_draw_pillar_paints_the_locked_pillar_canvas() -> None:
    canvas = environment.draw_pillar()
    assert (canvas.width, canvas.height) == (64, 512)
    assert _digest(canvas.pixels) == "3cd652fca2c3ccebec9791d5c50ca36e18d506738068f7ccbfdf2497561cdd1a"


def test_draw_pillar_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_pillar().read(0, 0) == (48, 46, 56, 255)


def test_draw_pillar_rejects_a_gray_dark_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "GRAY_DARK", 5)
    with pytest.raises(TypeError):
        environment.draw_pillar()


def test_draw_torch_paints_the_locked_torch_canvas() -> None:
    canvas = environment.draw_torch()
    assert (canvas.width, canvas.height) == (32, 64)
    assert _digest(canvas.pixels) == "a09fdee93972c6f1117599f7b850737e3250d4f8863d82725ddb3277fb44f82b"


def test_draw_torch_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_torch().read(0, 0) is None


def test_draw_torch_rejects_a_iron_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "IRON", 5)
    with pytest.raises(TypeError):
        environment.draw_torch()


def test_draw_chain_paints_the_locked_chain_canvas() -> None:
    canvas = environment.draw_chain()
    assert (canvas.width, canvas.height) == (16, 160)
    assert _digest(canvas.pixels) == "d499182935e370dde0b8bf651f2cb743977eb2ea75d5ce82fd74a5c877f7a359"


def test_draw_chain_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_chain().read(0, 0) is None


def test_draw_chain_rejects_a_brass_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "BRASS", 5)
    with pytest.raises(TypeError):
        environment.draw_chain()


def test_draw_ground_slam_paints_the_locked_ground_slam_canvas() -> None:
    canvas = environment.draw_ground_slam()
    assert (canvas.width, canvas.height) == (96, 24)
    assert _digest(canvas.pixels) == "6906edf663352dead57c29636b258d90cacef958f0021525fed1a2e28ed19227"


def test_draw_ground_slam_keeps_the_locked_corner_pixel() -> None:
    assert environment.draw_ground_slam().read(0, 0) is None


def test_draw_ground_slam_rejects_a_orange_value_that_is_not_a_color(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(environment, "ORANGE", 5)
    with pytest.raises(TypeError):
        environment.draw_ground_slam()


def test_speck_returns_the_dark_speck_on_the_origin() -> None:
    assert environment.speck(0, 0) == -18


def test_speck_returns_no_speck_when_the_cell_hash_misses() -> None:
    assert environment.speck(1, 0) == 0
    assert environment.speck(8, 8) == 0


def test_speck_rejects_a_column_that_is_not_a_number() -> None:
    with pytest.raises(TypeError):
        environment.speck("column", 0)


def test_hash2_returns_zero_at_the_origin() -> None:
    assert environment.hash2(0, 0) == 0


def test_hash2_changes_when_the_salt_changes_and_stays_in_a_byte() -> None:
    assert environment.hash2(0, 0, 3) == 124
    assert environment.hash2(400, 12, 1) == 201


def test_hash2_rejects_a_missing_column() -> None:
    with pytest.raises(TypeError):
        environment.hash2(None, 0)


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


def test_crack_paints_glow_on_every_third_step() -> None:
    canvas = clipping_canvas(8, 8)
    environment.crack(canvas, ((0, 0), (4, 0)), INK, RED)
    assert [canvas.read(x, 0) for x in range(5)] == [(200, 0, 0, 255), (0, 0, 0, 255), (0, 0, 0, 255), (200, 0, 0, 255), (0, 0, 0, 255)]


def test_crack_with_one_point_paints_nothing() -> None:
    canvas = clipping_canvas(2, 2, BLUE)
    environment.crack(canvas, ((0, 0),), INK, RED)
    assert canvas.pixels == [BLUE, BLUE, BLUE, BLUE]


def test_crack_rejects_points_that_are_missing() -> None:
    with pytest.raises(TypeError):
        environment.crack(clipping_canvas(2, 2), None, INK, RED)


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

