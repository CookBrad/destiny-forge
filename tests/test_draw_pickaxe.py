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
