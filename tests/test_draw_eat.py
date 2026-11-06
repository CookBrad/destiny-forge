from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_homestead_eat_frames as eat

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _image_digest(image: Image.Image) -> str:
    return hashlib.sha256(image.tobytes()).hexdigest()


def _file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_dip_head_moves_the_head_down_by_the_requested_rows():
    base = Image.open(eat.BASE).convert("RGBA")
    dipped = eat.dip_head(base, 1)
    assert dipped.getpixel((0, 1)) == base.getpixel((0, 0))
    assert _image_digest(dipped) == 'ecb536c446c2e925d1570e3ae3e149a0a0c2be1323136237e53508147441e5df'


def test_dip_head_with_zero_rows_matches_the_original_bytes():
    base = Image.open(eat.BASE).convert("RGBA")
    assert _image_digest(eat.dip_head(base, 0)) == _image_digest(base)
    assert _image_digest(base) == 'eac9009a8cafdbf9afe115c99e93d4ce26ef7e32c3bb78a2983c104b71ecddff'


def test_dip_head_rejects_an_image_that_cannot_be_cropped():
    with pytest.raises(AttributeError):
        eat.dip_head(None, 1)


def test_eat_main_writes_four_bite_frames_with_the_locked_hashes(tmp_path, monkeypatch):
    folder = tmp_path / "eat"
    folder.mkdir()
    source = ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    monkeypatch.setattr(eat, "SPRITES", folder)
    monkeypatch.setattr(eat, "BASE", source)
    eat.main()
    expected = ['2e9ac659b91799b15631cc03e0561fb687e500d5d1c20f0f9d47af3e2fbe93e4', 'd16370f51bb2e06741145dc54fe700fc71d61992a4a5b7c23958f8313da914a6', 'dbce1c9ffc807d9700b42ce44ffbd5b6f254c5c078d5fc064316e50ecb51fd2c', '92784f9ae9823df9e3953ea5ea76e9ca941664422d526ae9947c53dad189115c']
    assert [_file_digest(folder / f"dwarf_m_eat_anim_f{index}.png") for index in range(4)] == expected


def test_eat_main_rejects_a_missing_base_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "eat"
    folder.mkdir()
    monkeypatch.setattr(eat, "SPRITES", folder)
    monkeypatch.setattr(eat, "BASE", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        eat.main()
