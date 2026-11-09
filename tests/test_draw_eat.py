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


def test_reach_places_the_drumstick_in_the_lower_hand():
    base = Image.open(eat.BASE).convert("RGBA")
    original = base.copy()
    frame = eat.reach(base)
    assert frame.size == (16, 28)
    assert frame is not base
    assert base.tobytes() == original.tobytes()
    assert frame.getpixel((10, 21)) == (34, 34, 34, 255)
    assert frame.getpixel((10, 22)) == (204, 128, 64, 255)
    assert frame.getpixel((11, 23)) == (156, 84, 40, 255)
    assert frame.getpixel((14, 23)) == (238, 228, 204, 255)
    assert frame.getpixel((13, 25)) == (216, 165, 125, 255)
    assert frame.getpixel((14, 25)) == (34, 34, 34, 255)
    assert _image_digest(frame) == 'f7cae73ef598e4e802f8029753ad45c24d7e5e4308cb4284b5d9178b12ae1803'


def test_reach_on_a_short_canvas_leaves_every_pixel_clear():
    base = Image.new("RGBA", (16, 10), (0, 0, 0, 0))
    frame = eat.reach(base)
    assert frame.size == (16, 10)
    assert frame is not base
    assert frame.tobytes() == base.tobytes()


def test_reach_rejects_a_missing_image():
    with pytest.raises(AttributeError):
        eat.reach(None)


def test_lift_raises_the_drumstick_to_the_open_mouth():
    base = Image.open(eat.BASE).convert("RGBA")
    original = base.copy()
    frame = eat.lift(base)
    assert frame.size == (16, 28)
    assert frame is not base
    assert base.tobytes() == original.tobytes()
    assert frame.getpixel((9, 21)) == (34, 34, 34, 255)
    assert frame.getpixel((10, 21)) == (34, 34, 34, 255)
    assert frame.getpixel((12, 20)) == (204, 128, 64, 255)
    assert frame.getpixel((14, 20)) == (156, 84, 40, 255)
    assert frame.getpixel((14, 23)) == (216, 165, 125, 255)
    assert frame.getpixel((15, 23)) == (34, 34, 34, 255)
    assert _image_digest(frame) == '096816fefacfcddfa97b51975fca0e6a93edbc9d4b15b1d55e23d834b4a49d86'


def test_lift_on_a_short_canvas_paints_only_the_drumstick_top():
    base = Image.new("RGBA", (16, 20), (0, 0, 0, 0))
    frame = eat.lift(base)
    expected = Image.new("RGBA", (16, 20), (0, 0, 0, 0))
    expected.putpixel((12, 19), (34, 34, 34, 255))
    expected.putpixel((13, 19), (34, 34, 34, 255))
    expected.putpixel((14, 19), (34, 34, 34, 255))
    assert frame.size == (16, 20)
    assert frame.tobytes() == expected.tobytes()


def test_lift_rejects_a_missing_image():
    with pytest.raises(AttributeError):
        eat.lift(None)


def test_bite_dips_the_head_before_the_drumstick():
    base = Image.open(eat.BASE).convert("RGBA")
    original = base.copy()
    frame = eat.bite(base)
    assert frame.size == (16, 28)
    assert frame is not base
    assert base.tobytes() == original.tobytes()
    assert frame.getpixel((0, 1)) == base.getpixel((0, 0))
    assert frame.getpixel((11, 21)) == (204, 128, 64, 255)
    assert frame.getpixel((12, 21)) == (156, 84, 40, 255)
    assert frame.getpixel((15, 22)) == (238, 228, 204, 255)
    assert frame.getpixel((14, 24)) == (216, 165, 125, 255)
    assert frame.getpixel((15, 24)) == (34, 34, 34, 255)
    assert _image_digest(frame) == '3a63ba95bfc0ff7adb3e2e34ef030f4be2f56dc53803252146e5a1d3966bf9c5'


def test_bite_on_a_neck_high_canvas_shifts_the_top_row():
    base = Image.new("RGBA", (16, 20), (0, 0, 0, 0))
    base.putpixel((0, 0), (9, 9, 9, 255))
    base.putpixel((1, 19), (8, 8, 8, 255))
    frame = eat.bite(base)
    expected = Image.new("RGBA", (16, 20), (0, 0, 0, 0))
    expected.putpixel((0, 1), (9, 9, 9, 255))
    assert frame.tobytes() == expected.tobytes()
    assert base.getpixel((0, 0)) == (9, 9, 9, 255)
    assert base.getpixel((1, 19)) == (8, 8, 8, 255)


def test_bite_rejects_a_missing_image():
    with pytest.raises(AttributeError):
        eat.bite(None)


def test_chew_closes_the_eyes_on_the_dipped_head():
    base = Image.open(eat.BASE).convert("RGBA")
    original = base.copy()
    frame = eat.chew(base)
    assert frame.size == (16, 28)
    assert frame is not base
    assert base.tobytes() == original.tobytes()
    assert frame.getpixel((8, 19)) == (216, 165, 125, 255)
    assert frame.getpixel((9, 19)) == (216, 165, 125, 255)
    assert frame.getpixel((8, 20)) == (34, 34, 34, 255)
    assert frame.getpixel((9, 20)) == (34, 34, 34, 255)
    assert frame.getpixel((11, 21)) == (216, 165, 125, 255)
    assert frame.getpixel((11, 22)) == (195, 141, 112, 255)
    assert frame.getpixel((13, 23)) == (204, 128, 64, 255)
    assert frame.getpixel((14, 23)) == (156, 84, 40, 255)
    assert frame.getpixel((15, 26)) == (216, 165, 125, 255)
    assert _image_digest(frame) == 'b23de02a493104ddb5c2ffd0713b5085b9941e36c71b9708ddef845358d04078'
    assert _image_digest(frame) != '3a63ba95bfc0ff7adb3e2e34ef030f4be2f56dc53803252146e5a1d3966bf9c5'


def test_chew_on_a_clear_canvas_paints_the_bitten_drumstick():
    base = Image.new("RGBA", (16, 28), (0, 0, 0, 0))
    frame = eat.chew(base)
    assert frame.getpixel((8, 19)) == (216, 165, 125, 255)
    assert frame.getpixel((9, 20)) == (34, 34, 34, 255)
    assert frame.getpixel((11, 21)) == (216, 165, 125, 255)
    assert frame.getpixel((11, 22)) == (195, 141, 112, 255)
    assert frame.getpixel((13, 23)) == (204, 128, 64, 255)
    assert frame.getpixel((15, 26)) == (216, 165, 125, 255)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == '131508757b7d644be3614ea557cf37b8bd1816171ece8567a80f8d11fa6eb625'


def test_chew_rejects_a_missing_image():
    with pytest.raises(AttributeError):
        eat.chew(None)


def test_stamp_paints_each_palette_key_at_the_origin():
    image = Image.new("RGBA", (6, 2), (0, 0, 0, 0))
    eat.stamp(image, ["a.smh", "d.b.."], 0, 0)
    assert image.getpixel((0, 0)) == (34, 34, 34, 255)
    assert image.getpixel((1, 0)) == (0, 0, 0, 0)
    assert image.getpixel((2, 0)) == (216, 165, 125, 255)
    assert image.getpixel((3, 0)) == (156, 84, 40, 255)
    assert image.getpixel((4, 0)) == (204, 128, 64, 255)
    assert image.getpixel((5, 0)) == (0, 0, 0, 0)
    assert image.getpixel((0, 1)) == (195, 141, 112, 255)
    assert image.getpixel((1, 1)) == (0, 0, 0, 0)
    assert image.getpixel((2, 1)) == (238, 228, 204, 255)
    assert image.getpixel((3, 1)) == (0, 0, 0, 0)
    assert image.getpixel((4, 1)) == (0, 0, 0, 0)
    assert image.getpixel((5, 1)) == (0, 0, 0, 0)


def test_stamp_ignores_a_key_past_the_right_edge():
    image = Image.new("RGBA", (2, 2), (1, 2, 3, 255))
    eat.stamp(image, ["aa"], 1, 1)
    assert image.getpixel((0, 0)) == (1, 2, 3, 255)
    assert image.getpixel((1, 0)) == (1, 2, 3, 255)
    assert image.getpixel((0, 1)) == (1, 2, 3, 255)
    assert image.getpixel((1, 1)) == (34, 34, 34, 255)


def test_stamp_rejects_an_unknown_palette_key():
    image = Image.new("RGBA", (1, 1), (0, 0, 0, 0))
    with pytest.raises(KeyError, match="'q'"):
        eat.stamp(image, ["q"], 0, 0)
    assert image.getpixel((0, 0)) == (0, 0, 0, 0)


def test_close_eyes_shifts_the_lids_by_the_head_offset():
    image = Image.new("RGBA", (10, 21), (1, 1, 1, 255))
    eat.close_eyes(image, 1)
    expected = Image.new("RGBA", (10, 21), (1, 1, 1, 255))
    expected.putpixel((8, 19), (216, 165, 125, 255))
    expected.putpixel((9, 19), (216, 165, 125, 255))
    expected.putpixel((8, 20), (34, 34, 34, 255))
    expected.putpixel((9, 20), (34, 34, 34, 255))
    assert image.tobytes() == expected.tobytes()


def test_close_eyes_with_zero_offset_paints_the_eye_row():
    image = Image.new("RGBA", (10, 20), (1, 1, 1, 255))
    eat.close_eyes(image, 0)
    expected = Image.new("RGBA", (10, 20), (1, 1, 1, 255))
    expected.putpixel((8, 18), (216, 165, 125, 255))
    expected.putpixel((9, 18), (216, 165, 125, 255))
    expected.putpixel((8, 19), (34, 34, 34, 255))
    expected.putpixel((9, 19), (34, 34, 34, 255))
    assert image.tobytes() == expected.tobytes()


def test_close_eyes_rejects_a_row_past_the_image():
    image = Image.new("RGBA", (10, 20), (1, 1, 1, 255))
    with pytest.raises(IndexError, match="image index out of range"):
        eat.close_eyes(image, 1)


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
