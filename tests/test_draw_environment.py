from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_environment as environment

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


def test_write_opaque_pieces_keeps_an_unrelated_file_in_the_folder(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]) -> None:
    environment_folder, visual_effects_folder = _redirect_piece_folders(monkeypatch, tmp_path)
    environment_folder.mkdir()
    note = environment_folder / "note.txt"
    note.write_text("keep")
    environment.write_opaque_pieces()
    assert note.read_text() == "keep"
    assert visual_effects_folder.exists() is False
    assert capsys.readouterr().out == OPAQUE_PIECE_REPORT


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


def test_save_checked_piece_writes_the_canvas_when_the_size_matches(tmp_path: Path) -> None:
    color = (9, 8, 7, 255)
    canvas = environment.Canvas(2, 3, color)
    path = tmp_path / "piece.png"
    environment.save_checked_piece(canvas, path, 2, 3)
    assert _saved_piece_record(path) == ("piece.png", (2, 3), "RGBA", {255})
    assert _saved_pixels(path) == [color, color, color, color, color, color]


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
