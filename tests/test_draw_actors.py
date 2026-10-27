from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_actors as actors
from art_common import Palette, clipping_canvas, hunter_palette, load_pinned_colors

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _digest(pixels) -> str:
    return hashlib.sha256(b"".join(bytes(pixel) for pixel in pixels)).hexdigest()


@pytest.fixture(scope="module")
def palette() -> Palette:
    return hunter_palette(_signed_colors())


def _file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _signed_colors() -> set[tuple[int, int, int, int]]:
    folder = ROOT / "assets" / "source" / "hunter_signed"
    digests = {
        "knight_idle_side.png": "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e",
        "knight_run_side.png": "1e05a64f76044a614d8a080e234778ce8eab3c9bff819fa99de3e34b816466e7",
        "knight_attack_side.png": "ca089a0e93524161f020d9ef91f736e330d36f3530749b8296e918b088620b04",
    }
    return load_pinned_colors(folder, digests)

def test_volume_shades_points_from_the_near_ramp_stop_to_the_far_one():
    painted = actors.volume([(0, 0), (4, 2), (8, 1)], 1, 1, (RED, BLUE, GREEN), 4)
    assert painted == {(0, 0): (0, 0, 200, 255), (4, 2): (0, 0, 200, 255), (8, 1): (200, 0, 0, 255)}


def test_volume_paints_every_point_with_the_only_ramp_color():
    assert actors.volume([(0, 0), (1, 1)], 0, 0, (RED,), 4) == {(0, 0): (200, 0, 0, 255), (1, 1): (200, 0, 0, 255)}


def test_volume_returns_nothing_for_an_empty_point_list():
    assert actors.volume([], 0, 0, (RED, BLUE), 4) == {}


def test_volume_rejects_points_when_the_ramp_is_empty():
    with pytest.raises(IndexError):
        actors.volume([(0, 0)], 0, 0, (), 4)


def test_draw_slime_paints_the_signed_gel_body(palette):
    canvas = actors.draw_slime(palette)
    assert (canvas.width, canvas.height) == (64, 64)
    assert _digest(canvas.pixels) == '8f97217f716f24351aa973bf14c8dd02de39d21fdd41dec2a158c52ab76f1c5c'


def test_draw_slime_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_slime(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 2123


def test_draw_slime_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_slime(object())


def test_draw_blob_paints_the_small_gel_blob(palette):
    canvas = actors.draw_blob(palette)
    assert (canvas.width, canvas.height) == (32, 32)
    assert _digest(canvas.pixels) == '22aac6bd18af5bbe108948db8d581a8ba2193b905ba5d4bdde883efe3c88d081'


def test_draw_blob_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_blob(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 675


def test_draw_blob_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_blob(object())


def test_draw_bolt_paints_the_slime_bolt(palette):
    canvas = actors.draw_bolt(palette)
    assert (canvas.width, canvas.height) == (16, 48)
    assert _digest(canvas.pixels) == '8ab4ed9b408d292621c7546ccdf688221d655fedbf701b160cd8dd0faff1c9fa'


def test_draw_bolt_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_bolt(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 449


def test_draw_bolt_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_bolt(object())


def test_draw_arrow_paints_the_brass_arrow(palette):
    canvas = actors.draw_arrow(palette)
    assert (canvas.width, canvas.height) == (16, 48)
    assert _digest(canvas.pixels) == '09b87796954105a9ef18380832881162ddc1a01865f0d3589f7e5561c2b3b795'


def test_draw_arrow_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_arrow(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 360


def test_draw_arrow_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_arrow(object())


def test_draw_bat_paints_the_signed_bat(palette):
    canvas = actors.draw_bat(palette)
    assert (canvas.width, canvas.height) == (96, 48)
    assert _digest(canvas.pixels) == '0b88b5d4b855b74804cf01c927fb6a69dc41e39581b654a785b42d0e4387c3ee'


def test_draw_bat_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_bat(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 2315


def test_draw_bat_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_bat(object())


def test_draw_goblin_paints_the_signed_goblin(palette):
    canvas = actors.draw_goblin(palette)
    assert (canvas.width, canvas.height) == (80, 112)
    assert _digest(canvas.pixels) == '697b730b68859bf39985dade2053398b96f5430d78d5eb729f972e77aa622722'


def test_draw_goblin_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_goblin(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 3823


def test_draw_goblin_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_goblin(object())


def test_draw_skeleton_paints_the_signed_skeleton(palette):
    canvas = actors.draw_skeleton(palette)
    assert (canvas.width, canvas.height) == (64, 144)
    assert _digest(canvas.pixels) == '16aa497935de235f69a4a9989133eb9998b91d2a3d69e3669c8b7219a44542b3'


def test_draw_skeleton_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_skeleton(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 4724


def test_draw_skeleton_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_skeleton(object())


def test_draw_zombie_paints_the_signed_zombie(palette):
    canvas = actors.draw_zombie(palette)
    assert (canvas.width, canvas.height) == (96, 128)
    assert _digest(canvas.pixels) == 'f24c06a7c3135af33820f6ef6bd82fcc3f4075fa90c7ef9bbeca84fc7e0f1e8c'


def test_draw_zombie_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_zombie(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 5642


def test_draw_zombie_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_zombie(object())


def test_draw_king_paints_the_signed_king_slime(palette):
    canvas = actors.draw_king(palette)
    assert (canvas.width, canvas.height) == (192, 192)
    assert _digest(canvas.pixels) == 'b84df8c2d3c144bf764515af1b2e2801aa5d0f0b010defd19cc6ebfdbcf28a8c'


def test_draw_king_keeps_the_top_left_pixel_clear_and_counts_opaque_pixels(palette):
    canvas = actors.draw_king(palette)
    assert canvas.read(0, 0) is None
    assert sum(pixel[3] == 255 for pixel in canvas.pixels) == 21381


def test_draw_king_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_king(object())


def test_crown_spike_paints_a_brass_tip_on_the_canvas(palette):
    canvas = clipping_canvas(40, 40)
    actors.crown_spike(canvas, 20, 2, 30, 8, palette)
    assert canvas.read(20, 3) == (251, 214, 119, 255)
    assert _digest(canvas.pixels) == 'fb2e1de581912e84a9d52b517679135f10dfa2703cc3d0f6fed78f4abcd6f772'


def test_crown_spike_uses_a_waist_of_at_least_three_pixels(palette):
    canvas = clipping_canvas(40, 40)
    actors.crown_spike(canvas, 20, 2, 30, 2, palette)
    assert any(canvas.opaque_at(x, 16) for x in range(40))


def test_crown_spike_rejects_a_palette_without_brass():
    with pytest.raises(AttributeError):
        actors.crown_spike(clipping_canvas(10, 10), 5, 0, 8, 3, object())


def test_actor_main_writes_the_nine_actor_sheets_with_the_locked_hashes(tmp_path, monkeypatch):
    enemies = tmp_path / "enemies"
    projectiles = tmp_path / "projectiles"
    enemies.mkdir()
    projectiles.mkdir()
    monkeypatch.setattr(actors, "ENEMIES", enemies)
    monkeypatch.setattr(actors, "PROJECTILES", projectiles)
    actors.main()
    expected = {
        "slime": "2c43383e1de7e8766333db63a15049f84f16ac46e5987e0910d888c369e88629",
        "bat": "c79640699a68c063d8db74e317358d05686c80a8deee6ec2c8ab45f805dd3065",
        "goblin": "0220381fe08503a3c57bfc7be31bc798bd997cc469ee6d425066bd0470f81219",
        "skeleton": "929eaa7d74a25eb4c98f9bcd27e4a5e9bf826f7e9ade40ce1690f647c9a8f0d8",
        "zombie": "bb8a4265421c61051c43e4295d1370217d34f1c95220fddbba2a92fd3520317a",
        "king_slime": "385a797ff241a5c33dd5db7abebe25e7a52f2aaf2d8817b8e5adce4494cf28a8",
        "arrow": "ac168a6bd2fcfe5987bc992c3a1228eb27bcc80801a657b8dbb5e6c66d5087d2",
        "slime_bolt": "c528803d19924168492a4e9e299a626809da72b2d10850d19476a0608823b2fa",
        "slime_blob": "4ae5b56f11bc3acd5188fb00dc05b73b047e7cdac433b30cadfda96ebee4bc25",
    }
    for name, digest in expected.items():
        folder = enemies if (enemies / f"{name}.png").exists() else projectiles
        assert _file_digest(folder / f"{name}.png") == digest
    assert len(list(enemies.glob("*.png"))) + len(list(projectiles.glob("*.png"))) == 9


def test_actor_main_rejects_a_signed_sheet_whose_hash_does_not_match(monkeypatch):
    monkeypatch.setitem(actors.SIGNED_SHA256, "knight_idle_side.png", "0" * 64)
    with pytest.raises(SystemExit, match="knight_idle_side.png hash"):
        actors.main()


def test_actor_main_writes_a_64_pixel_slime_sheet(tmp_path, monkeypatch) -> None:
    enemies = tmp_path / "enemies"
    projectiles = tmp_path / "projectiles"
    enemies.mkdir()
    projectiles.mkdir()
    monkeypatch.setattr(actors, "ENEMIES", enemies)
    monkeypatch.setattr(actors, "PROJECTILES", projectiles)
    actors.main()
    with Image.open(enemies / "slime.png") as image:
        assert image.size == (64, 64)
