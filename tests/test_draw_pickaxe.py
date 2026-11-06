from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dwarf_pickaxe_swing as pickaxe

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _image_digest(image: Image.Image) -> str:
    return hashlib.sha256(image.tobytes()).hexdigest()


def _file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_outline_around_adds_a_four_connected_outline():
    assert pickaxe.outline_around({(1, 1): pickaxe.WOOD}) == {(1, 1): (119, 92, 85, 255), (2, 1): (34, 34, 34, 255), (0, 1): (34, 34, 34, 255), (1, 2): (34, 34, 34, 255), (1, 0): (34, 34, 34, 255)}


def test_outline_around_of_an_empty_layer_stays_empty():
    assert pickaxe.outline_around({}) == {}


def test_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        pickaxe.outline_around(None)


def test_draw_frame_puts_the_pickaxe_behind_the_dwarf_on_the_first_pose():
    frame = pickaxe.draw_frame(pickaxe.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == 'd0287d0d44fa92c322cbb8109149d5855a57024c45bf5e3e3fa23d2d70716247'


def test_draw_frame_puts_the_pickaxe_in_front_on_the_third_pose():
    frame = pickaxe.draw_frame(pickaxe.POSES[2])
    assert _image_digest(frame) == 'cd2c29cf2401cf8a2edfc009c8b0bcdb16ca74cb681357344486adaea2be1aa3'
    assert _image_digest(frame) != 'd0287d0d44fa92c322cbb8109149d5855a57024c45bf5e3e3fa23d2d70716247'


def test_draw_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        pickaxe.draw_frame({})


def test_pickaxe_main_writes_four_swing_frames_with_the_locked_hashes(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe, "FOLDER", folder)
    monkeypatch.setattr(pickaxe, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png")
    pickaxe.main()
    expected = ['178712b08583b0f9feaeea05dd10e09c6a4f329f68e0f605a2252b8189896f3c', 'ab389f49539c78b3dc34c6e2a8e512cdab08827d1a8d1e7d42f672ba69481baa', 'b6721fb902df595f7e07f59a1b31fa829582be7c11845d3555256c52408204d0', 'ab3d0e9730d59ab71f7228ce0be1000d3cc90e0d01c26c5a1150ef4ab2266810']
    assert [_file_digest(folder / f"dwarf_m_pickaxe_swing_f{index}.png") for index in range(4)] == expected


def test_pickaxe_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe, "FOLDER", folder)
    monkeypatch.setattr(pickaxe, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        pickaxe.main()

def test_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert pickaxe.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert pickaxe.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        pickaxe.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_pickaxe_pixels_draws_an_upright_handle_from_the_grip() -> None:
    layer = pickaxe.pickaxe_pixels((24.0, 40.0), 0.0)
    wood = (119, 92, 85, 255)
    steel = (123, 137, 148, 255)
    outline = (34, 34, 34, 255)
    expected = {(x, y): wood for x in (23, 24) for y in range(28, 40)}
    expected.update({
        point: steel
        for point in (
            (18, 28),
            (19, 28),
            (20, 27),
            (20, 28),
            (21, 27),
            (21, 28),
            (22, 27),
            (23, 26),
            (23, 27),
            (24, 26),
            (24, 27),
            (25, 27),
            (26, 27),
            (26, 28),
            (27, 27),
            (27, 28),
            (28, 28),
            (29, 28),
        )
    })
    expected.update({
        point: outline
        for point in (
            (17, 28),
            (18, 27),
            (18, 29),
            (19, 27),
            (19, 29),
            (20, 26),
            (20, 29),
            (21, 26),
            (21, 29),
            (22, 26),
            (22, 28),
            (22, 29),
            (22, 30),
            (22, 31),
            (22, 32),
            (22, 33),
            (22, 34),
            (22, 35),
            (22, 36),
            (22, 37),
            (22, 38),
            (22, 39),
            (23, 25),
            (23, 40),
            (24, 25),
            (24, 40),
            (25, 26),
            (25, 28),
            (25, 29),
            (25, 30),
            (25, 31),
            (25, 32),
            (25, 33),
            (25, 34),
            (25, 35),
            (25, 36),
            (25, 37),
            (25, 38),
            (25, 39),
            (26, 26),
            (26, 29),
            (27, 26),
            (27, 29),
            (28, 27),
            (28, 29),
            (29, 27),
            (29, 29),
            (30, 28),
        )
    })
    assert layer == expected


def test_pickaxe_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert pickaxe.pickaxe_pixels((1.0, 1.0), 0.0) == {
        (0, 0): (119, 92, 85, 255),
        (1, 0): (119, 92, 85, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 1): (34, 34, 34, 255),
        (2, 0): (34, 34, 34, 255),
    }


def test_pickaxe_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe.pickaxe_pixels(None, 0.0)


def test_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    pickaxe.stroke(layer, [((1, 1), (3, 1))], 0.5, color=RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    pickaxe.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, color=BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        pickaxe.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def test_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe, "DWARF", sheet)
    assert pickaxe.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe, "DWARF", sheet)
    assert pickaxe.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        pickaxe.dwarf_pixels({})


def test_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe.fist_pixels((20.5, 27.5)) == {
        (18, 27): outline,
        (18, 28): outline,
        (19, 26): outline,
        (19, 27): skin,
        (19, 28): skin,
        (19, 29): outline,
        (20, 26): outline,
        (20, 27): skin,
        (20, 28): skin,
        (20, 29): outline,
        (21, 27): outline,
        (21, 28): outline,
    }


def test_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe.fist_pixels(None)
