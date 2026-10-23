from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_actors as actors
import draw_dungeon_environment as environment
import draw_dwarf_pickaxe_swing as pickaxe
import draw_homestead_eat_frames as eat
import draw_hunter_frames as hunter
import draw_skill_icons as icons
from art_common import Palette, clipping_canvas, hunter_palette, load_pinned_colors, rejecting_canvas

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _digest(pixels) -> str:
    return hashlib.sha256(b"".join(bytes(pixel) for pixel in pixels)).hexdigest()


def _sprite_digest(sprite: hunter.Sprite) -> str:
    payload = repr(tuple(sorted(sprite.pixels.items()))).encode()
    return hashlib.sha256(payload).hexdigest()


def _image_digest(image: Image.Image) -> str:
    return hashlib.sha256(image.tobytes()).hexdigest()


def _signed_colors() -> set[tuple[int, int, int, int]]:
    folder = ROOT / "assets" / "source" / "hunter_signed"
    digests = {
        "knight_idle_side.png": "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e",
        "knight_run_side.png": "1e05a64f76044a614d8a080e234778ce8eab3c9bff819fa99de3e34b816466e7",
        "knight_attack_side.png": "ca089a0e93524161f020d9ef91f736e330d36f3530749b8296e918b088620b04",
    }
    return load_pinned_colors(folder, digests)


@pytest.fixture(scope="module")
def palette() -> Palette:
    return hunter_palette(_signed_colors())


@pytest.fixture(scope="module")
def idle_cell() -> hunter.Sprite:
    return hunter.cell_of(hunter.load_sheet("knight_idle_side.png"), 0)


@pytest.fixture(scope="module")
def attack_cells() -> tuple[hunter.Sprite, hunter.Sprite]:
    sheet = hunter.load_sheet("knight_attack_side.png")
    return hunter.cell_of(sheet, 0), hunter.cell_of(sheet, 2)


@pytest.fixture(scope="module")
def run_frames(idle_cell: hunter.Sprite) -> list[hunter.Sprite]:
    return hunter.build_run_cycle(idle_cell)


@pytest.fixture(scope="module")
def attack_frames(attack_cells: tuple[hunter.Sprite, hunter.Sprite]) -> tuple[hunter.Sprite, hunter.Sprite]:
    return hunter.build_attack_frames(*attack_cells)


def _long_sole() -> hunter.Sprite:
    return hunter.Sprite({(x, 8): RED for x in range(12)})


def _file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
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


def test_draw_ground_fill_rejects_a_purple_dk_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "PURPLE_DK", 5)
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


def test_draw_platform_rejects_a_iron_dk_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "IRON_DK", 5)
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


def test_draw_ladder_rejects_a_iron_dk_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "IRON_DK", 5)
    with pytest.raises(TypeError):
        environment.draw_ladder()


def test_draw_stake_paints_the_locked_stake_canvas():
    canvas = environment.draw_stake()
    assert (canvas.width, canvas.height) == (24, 112)
    assert _digest(canvas.pixels) == '942c14b4862bddeae0e1871995757436e1e6b5193d05f3b154e0550d60fd58ff'


def test_draw_stake_keeps_the_locked_corner_pixel():
    assert environment.draw_stake().read(0, 0) == None


def test_draw_stake_rejects_a_brass_dk_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "BRASS_DK", 5)
    with pytest.raises(TypeError):
        environment.draw_stake()


def test_draw_lip_paints_the_locked_lip_canvas():
    canvas = environment.draw_lip()
    assert (canvas.width, canvas.height) == (24, 96)
    assert _digest(canvas.pixels) == '00c04ef76b480b3674f0ac436c1c8575625ec0325bf3b1e1e19a0a81bbb8e05b'


def test_draw_lip_keeps_the_locked_corner_pixel():
    assert environment.draw_lip().read(0, 0) == (214, 164, 70, 255)


def test_draw_lip_rejects_a_brass_dk_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "BRASS_DK", 5)
    with pytest.raises(TypeError):
        environment.draw_lip()


def test_draw_pillar_paints_the_locked_pillar_canvas():
    canvas = environment.draw_pillar()
    assert (canvas.width, canvas.height) == (64, 512)
    assert _digest(canvas.pixels) == '3cd652fca2c3ccebec9791d5c50ca36e18d506738068f7ccbfdf2497561cdd1a'


def test_draw_pillar_keeps_the_locked_corner_pixel():
    assert environment.draw_pillar().read(0, 0) == (48, 46, 56, 255)


def test_draw_pillar_rejects_a_gray_dk_value_that_is_not_a_color(monkeypatch):
    monkeypatch.setattr(environment, "GRAY_DK", 5)
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
    expected = ['055abb6f1019c5a37ede4062234803ff313be600ec0f18610f3a220e0abc9940', '1b8a76f996c7e38ec06c4617654988f89271aed75659132b34a833d73508ee0e', 'f96d6738fc8766dd3547aa46339ac9c27dcaf4bdb26824a3e6985f80cd130735', '90e0ab0cc0fc6e49c4b519e3697cd15b77513cc04284edb8b942b41014ec1e32']
    assert [_file_digest(folder / f"dwarf_m_pickaxe_swing_f{index}.png") for index in range(4)] == expected


def test_pickaxe_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe, "FOLDER", folder)
    monkeypatch.setattr(pickaxe, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        pickaxe.main()


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


def test_sprite_paste_copies_another_sprite_by_an_offset() -> None:
    target = hunter.Sprite({(0, 0): RED})
    pasted = target.paste(hunter.Sprite({(1, 1): BLUE}), 2, 3)
    assert pasted is target
    assert target.pixels[(3, 4)] == BLUE
    assert target.pixels[(0, 0)] == RED


def test_sprite_shift_of_zero_returns_a_copy() -> None:
    sprite = hunter.Sprite({(1, 1): RED})
    moved = sprite.shift(0, 0)
    assert moved.pixels == {(1, 1): RED}
    assert moved is not sprite


def test_sprite_crop_drops_pixels_outside_the_rectangle() -> None:
    cropped = hunter.Sprite({(0, 0): RED, (3, 1): BLUE}).crop(0, 0, 2, 2)
    assert cropped.pixels == {(0, 0): RED}


def test_sprite_erase_rect_removes_only_the_rectangle() -> None:
    sprite = hunter.Sprite({(0, 0): RED, (2, 2): BLUE})
    sprite.erase_rect(0, 0, 1, 1)
    assert sprite.pixels == {(2, 2): BLUE}


def test_sprite_erase_if_drops_pixels_the_predicate_selects() -> None:
    sprite = hunter.Sprite({(0, 0): RED, (1, 0): BLUE})
    sprite.erase_if(lambda x, _y, _color: x == 0)
    assert sprite.pixels == {(1, 0): BLUE}


def test_sprite_bounds_rejects_an_empty_sprite() -> None:
    with pytest.raises(SystemExit, match="empty sprite"):
        hunter.Sprite().bounds()


def test_sprite_require_opaque_pixels_inside_accepts_a_pixel_on_the_last_cell() -> None:
    hunter.Sprite({(1, 1): RED}).require_opaque_pixels_inside(0, 0, width=2, height=2)


def test_sprite_require_opaque_pixels_inside_rejects_a_pixel_past_the_offset() -> None:
    with pytest.raises(SystemExit, match=r"opaque pixel would leave the cell at 2,0"):
        hunter.Sprite({(1, 0): RED}).require_opaque_pixels_inside(1, 0, width=2, height=2)


def test_colors_used_unions_the_groups_and_drops_duplicates() -> None:
    assert hunter.colors_used({(0, 0): RED}, {(1, 1): RED, (2, 2): BLUE}) == {RED, BLUE}


def test_colors_used_of_no_groups_is_empty() -> None:
    assert hunter.colors_used() == set()


def test_colors_used_rejects_a_group_without_values() -> None:
    with pytest.raises(AttributeError):
        hunter.colors_used(None)


def test_palette_of_collects_colors_from_each_sprite() -> None:
    assert hunter.palette_of(hunter.Sprite({(0, 0): RED}), hunter.Sprite({(1, 0): BLUE})) == {RED, BLUE}


def test_palette_of_an_empty_sprite_is_an_empty_set() -> None:
    assert hunter.palette_of(hunter.Sprite()) == set()


def test_palette_of_rejects_a_value_that_is_not_a_sprite() -> None:
    with pytest.raises(AttributeError):
        hunter.palette_of(object())


def test_round_half_up_rounds_a_half_away_from_zero_on_the_positive_side() -> None:
    assert hunter.round_half_up(0.5) == 1
    assert hunter.round_half_up(1.2) == 1


def test_round_half_up_of_a_small_negative_value_is_zero() -> None:
    assert hunter.round_half_up(-0.2) == 0


def test_round_half_up_rejects_a_value_that_is_not_a_number() -> None:
    with pytest.raises(TypeError):
        hunter.round_half_up("1")


def test_stance_contact_uses_the_center_of_the_longest_bottom_run() -> None:
    assert hunter.stance_contact(_long_sole()) == (8, 5.5)


def test_stance_contact_rejects_a_run_shorter_than_eight_pixels() -> None:
    short = hunter.Sprite({(x, 1): RED for x in range(3)})
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.stance_contact(short)


def test_widest_sole_returns_the_contact_row_and_its_ends() -> None:
    assert hunter.widest_sole(_long_sole()) == (8, 0, 11)


def test_widest_sole_rejects_a_sprite_with_no_long_run() -> None:
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.widest_sole(hunter.Sprite({(0, 0): RED}))


def test_widen_sole_toward_right_copies_the_rightmost_sole_pixel() -> None:
    sprite = _long_sole()
    hunter.widen_sole_toward_right(sprite)
    assert sprite.pixels[(12, 8)] == RED


def test_widen_sole_toward_left_copies_the_leftmost_sole_pixel() -> None:
    sprite = _long_sole()
    hunter.widen_sole_toward_left(sprite)
    assert sprite.pixels[(-1, 8)] == RED


def test_widen_sole_by_delta_grows_right_for_a_positive_delta_and_left_otherwise() -> None:
    right = _long_sole()
    hunter.widen_sole_by_delta(right, 0.4)
    assert (12, 8) in right.pixels
    left = _long_sole()
    hunter.widen_sole_by_delta(left, 0)
    assert (-1, 8) in left.pixels


def test_widen_sole_toward_right_rejects_a_sprite_with_no_sole() -> None:
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.widen_sole_toward_right(hunter.Sprite())


def test_shift_contact_to_keeps_a_sole_that_is_already_on_the_target() -> None:
    moved, offset_x, offset_y = hunter.shift_contact_to(_long_sole(), 5.5, 8)
    assert (offset_x, offset_y) == (0, 0)
    assert hunter.stance_contact(moved) == (8, 5.5)


def test_shift_contact_to_moves_the_sole_onto_an_integer_target() -> None:
    moved, offset_x, offset_y = hunter.shift_contact_to(_long_sole(), 8.5, 10)
    assert (offset_x, offset_y) == (3, 2)
    assert hunter.stance_contact(moved) == (10, 8.5)


def test_shift_contact_to_rejects_a_target_it_cannot_reach_in_four_widens() -> None:
    with pytest.raises(SystemExit, match=r"contact 5.0 cannot reach 5.2"):
        hunter.shift_contact_to(_long_sole(), 5.2, 8)


def test_plant_contact_lands_the_sole_inside_the_cell() -> None:
    planted = hunter.plant_contact(_long_sole(), 5.5, 4, 30, 20)
    assert hunter.stance_contact(planted) == (4, 5.5)


def test_plant_contact_rejects_an_empty_sprite() -> None:
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.plant_contact(hunter.Sprite(), 1.0, 1, 10, 10)


def test_plant_on_foot_plants_the_long_sole_on_the_hunter_foot_point() -> None:
    planted = hunter.plant_on_foot(_long_sole())
    assert hunter.stance_contact(planted) == (hunter.FOOT_ROW, hunter.FOOT_X)


def test_plant_on_foot_rejects_an_empty_sprite() -> None:
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.plant_on_foot(hunter.Sprite())


def test_bridge_vertical_gaps_fills_a_gap_of_two_pixels() -> None:
    bridged = hunter.bridge_vertical_gaps(hunter.Sprite({(0, 0): RED, (0, 3): BLUE}))
    assert bridged.pixels[(0, 1)] == BLUE
    assert bridged.pixels[(0, 2)] == BLUE


def test_bridge_vertical_gaps_leaves_a_gap_wider_than_the_limit() -> None:
    bridged = hunter.bridge_vertical_gaps(hunter.Sprite({(0, 0): RED, (0, 20): BLUE}))
    assert (0, 1) not in bridged.pixels


def test_bridge_vertical_gaps_rejects_a_sprite_that_is_missing() -> None:
    with pytest.raises(AttributeError):
        hunter.bridge_vertical_gaps(None)


def test_fill_cracks_fills_a_one_pixel_hole_between_neighbors() -> None:
    filled = hunter.fill_cracks(hunter.Sprite({(0, 1): RED, (2, 1): BLUE}), passes=1)
    assert filled.pixels[(1, 1)] == RED


def test_fill_cracks_with_zero_passes_leaves_the_hole() -> None:
    sprite = hunter.Sprite({(0, 1): RED, (2, 1): BLUE})
    assert (1, 1) not in hunter.fill_cracks(sprite, passes=0).pixels


def test_fill_cracks_of_an_empty_sprite_stays_empty() -> None:
    assert hunter.fill_cracks(hunter.Sprite()).pixels == {}


def test_rotate_of_a_tiny_angle_returns_a_copy() -> None:
    sprite = hunter.Sprite({(0, y): RED for y in range(21)})
    turned = hunter.rotate(sprite, 0.0, (0.0, 0.0))
    assert turned.pixels == sprite.pixels
    assert turned is not sprite


def test_rotate_of_ninety_degrees_keeps_the_pixel_count() -> None:
    sprite = hunter.Sprite({(0, y): RED for y in range(21)})
    assert len(hunter.rotate(sprite, 90.0, (0.0, 0.0)).pixels) == 21


def test_rotate_of_an_empty_sprite_stays_empty() -> None:
    assert hunter.rotate(hunter.Sprite(), 90.0, (0.0, 0.0)).pixels == {}


def test_blade_base_low_is_the_center_of_the_bottom_row() -> None:
    blade = hunter.Sprite({(0, y): RED for y in range(21)})
    assert hunter.blade_base_low(blade) == (0.0, 20.0)


def test_blade_base_left_is_the_middle_of_the_left_edge() -> None:
    blade = hunter.Sprite({(0, y): RED for y in range(21)})
    assert hunter.blade_base_left(blade) == (0.0, 10.0)


def test_blade_base_low_rejects_an_empty_blade() -> None:
    with pytest.raises(ValueError):
        hunter.blade_base_low(hunter.Sprite())


def test_aim_from_pivot_shortens_the_blade_to_the_requested_length() -> None:
    blade = hunter.Sprite({(0, y): RED for y in range(21)})
    aimed, pivot = hunter.aim_from_pivot(blade, (0.0, 20.0), 0.0, 10.0)
    assert pivot == (0.0, 20.0)
    assert len(aimed.pixels) == 11
    assert _sprite_digest(aimed) == "d2b288f39a76cec70cc7e69eb898914a3ad836cfc3f11e56847243ed59d38e6e"


def test_aim_low_blade_uses_the_low_base_and_the_same_length() -> None:
    blade = hunter.Sprite({(0, y): RED for y in range(21)})
    aimed, pivot = hunter.aim_low_blade(blade, 0.0, 10.0)
    assert pivot == (0.0, 20.0)
    assert len(aimed.pixels) == 11


def test_aim_left_blade_uses_the_left_base() -> None:
    blade = hunter.Sprite({(0, y): RED for y in range(21)})
    aimed, pivot = hunter.aim_left_blade(blade, 90.0, 8.0)
    assert pivot == (0.0, 10.0)
    assert len(aimed.pixels) == 17


def test_aim_from_pivot_rejects_a_length_that_removes_every_pixel() -> None:
    blade = hunter.Sprite({(0, 0): RED})
    with pytest.raises(SystemExit, match="shorten removed the sprite"):
        hunter.aim_from_pivot(blade, (10.0, 10.0), 0.0, 1.0)


def test_place_aimed_moves_the_pivot_onto_the_hand() -> None:
    blade = hunter.Sprite({(0, y): RED for y in range(21)})
    aimed, pivot = hunter.aim_from_pivot(blade, (0.0, 20.0), 0.0, 10.0)
    placed = hunter.place_aimed(aimed, pivot, (4.0, 6.0))
    assert placed.bounds() == (4, 6, 14, 6)


def test_place_aimed_of_an_empty_sprite_rejects_the_bounds() -> None:
    with pytest.raises(SystemExit, match="empty sprite"):
        hunter.place_aimed(hunter.Sprite(), (0.0, 0.0), (1.0, 1.0)).bounds()


def test_seat_planted_leg_moves_the_sole_to_the_foot_row() -> None:
    seated = hunter.seat_planted_leg(_long_sole(), 5.5)
    assert hunter.stance_contact(seated) == (hunter.FOOT_ROW, 5.5)


def test_seat_lifted_leg_does_not_drop_a_sole_that_is_already_above_the_foot() -> None:
    lifted = hunter.seat_lifted_leg(_long_sole())
    assert hunter.stance_contact(lifted) == (8, 5.5)


def test_seat_planted_leg_rejects_an_empty_leg() -> None:
    with pytest.raises(ValueError):
        hunter.seat_planted_leg(hunter.Sprite(), 5.5)


def test_swing_from_hip_with_no_angle_keeps_the_sole_contact() -> None:
    swung = hunter.swing_from_hip(_long_sole(), 0.0, 5.5, 8.0)
    assert hunter.stance_contact(swung) == (8, 5.5)


def test_swing_from_hip_rejects_an_empty_leg() -> None:
    with pytest.raises(ValueError):
        hunter.swing_from_hip(hunter.Sprite(), 10.0, 0.0, 0.0)


def test_load_sheet_reads_the_signed_idle_sheet() -> None:
    loaded = hunter.load_sheet("knight_idle_side.png")
    assert len(loaded.pixels) == 44600
    assert loaded.bounds() == (0, 1, 1191, 158)


def test_load_sheet_rejects_an_unknown_name() -> None:
    with pytest.raises(KeyError, match="nope.png"):
        hunter.load_sheet("nope.png")


def test_without_blade_removes_blade_pixels_from_the_idle_cell(idle_cell) -> None:
    stripped = hunter.without_blade(idle_cell)
    assert len(stripped.pixels) == 9753
    assert len(stripped.pixels) < len(idle_cell.pixels)


def test_without_blade_with_no_halo_still_drops_the_hot_blade(idle_cell) -> None:
    stripped = hunter.without_blade(idle_cell, halo=0)
    assert len(stripped.pixels) < len(idle_cell.pixels)
    assert len(stripped.pixels) >= 9753


def test_without_blade_rejects_a_missing_sprite() -> None:
    with pytest.raises(AttributeError):
        hunter.without_blade(None)


def test_chest_column_lists_the_torso_columns_of_the_idle_cell(idle_cell) -> None:
    columns = hunter.chest_column(idle_cell)
    assert min(columns) == 37
    assert max(columns) == 150


def test_chest_column_of_a_pixel_outside_the_torso_is_empty() -> None:
    assert hunter.chest_column(hunter.Sprite({(0, 0): RED})) == []


def test_chest_pin_x_is_the_middle_of_the_idle_torso(idle_cell) -> None:
    assert hunter.chest_pin_x(idle_cell) == 93.5


def test_chest_pin_x_rejects_a_sprite_with_no_torso_pixels() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.chest_pin_x(hunter.Sprite({(0, 0): RED}))


def test_drop_chest_column_removes_the_torso_pixels_on_that_column() -> None:
    sprite = hunter.Sprite({(4, 60): RED, (4, 10): BLUE})
    hunter.drop_chest_column(sprite, 4)
    assert sprite.pixels == {(4, 10): BLUE}


def test_shave_right_chest_edge_drops_the_rightmost_torso_column(idle_cell) -> None:
    sprite = idle_cell.copy()
    hunter.shave_right_chest_edge(sprite)
    assert max(hunter.chest_column(sprite)) < 150


def test_shave_left_chest_edge_drops_the_leftmost_torso_column(idle_cell) -> None:
    sprite = idle_cell.copy()
    hunter.shave_left_chest_edge(sprite)
    assert min(hunter.chest_column(sprite)) > 37


def test_shave_right_chest_edge_rejects_a_sprite_with_no_torso() -> None:
    with pytest.raises(ValueError):
        hunter.shave_right_chest_edge(hunter.Sprite())


def test_require_inside_accepts_the_idle_cell(idle_cell) -> None:
    hunter.require_inside(idle_cell)


def test_require_inside_rejects_a_pixel_left_of_the_cell() -> None:
    with pytest.raises(SystemExit, match=r"opaque pixel would leave the cell at -1,0"):
        hunter.require_inside(hunter.Sprite({(-1, 0): RED}))


def test_build_run_parts_returns_the_body_legs_and_sword(idle_cell) -> None:
    parts = hunter.build_run_parts(idle_cell)
    assert tuple(sorted(parts)) == ("body", "chest", "front", "rear", "sword")
    assert parts["chest"] == 93.5


def test_build_run_parts_rejects_an_empty_sprite() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.build_run_parts(hunter.Sprite())


def test_build_run_cycle_plants_frame_zero_and_frame_four_on_the_foot(run_frames) -> None:
    assert [hunter.stance_contact(frame) for frame in run_frames] == [
        (158, 81.5),
        (158, 76.5),
        (157, 90.5),
        (158, 77.25),
        (158, 81.5),
        (158, 94.0),
        (155, 116.0),
        (158, 98.5),
    ]
    assert [len(frame.pixels) for frame in run_frames] == [9947, 9764, 9735, 10010, 10099, 10126, 9866, 10158]


def test_check_run_accepts_the_eight_planted_frames(run_frames) -> None:
    assert hunter.check_run(run_frames) is None


def test_check_run_rejects_a_cycle_that_is_not_eight_frames() -> None:
    with pytest.raises(SystemExit, match="run cycle is not 8 frames"):
        hunter.check_run([])


def test_check_run_rejects_a_first_frame_that_misses_the_foot() -> None:
    frames = [_long_sole() for _ in range(8)]
    with pytest.raises(SystemExit, match=r"run frame 0 contact 5.5@8"):
        hunter.check_run(frames)


def test_compose_run_frame_builds_the_first_pose(idle_cell) -> None:
    parts = hunter.build_run_parts(idle_cell)
    frame = hunter.compose_run_frame(parts, hunter.RUN_POSES[0])
    assert frame.bounds()[1] >= 0
    assert len(frame.pixels) > 9000


def test_compose_run_frame_rejects_a_pose_without_a_plant(idle_cell) -> None:
    parts = hunter.build_run_parts(idle_cell)
    with pytest.raises(KeyError, match="bob"):
        hunter.compose_run_frame(parts, {})


def test_sheet_image_packs_one_sprite_into_the_hunter_cell() -> None:
    image = hunter.sheet_image([_long_sole()])
    assert image.size == (hunter.CELL_W, hunter.CELL_H)
    assert image.getpixel((0, 8)) == RED


def test_sheet_image_of_no_frames_has_zero_width() -> None:
    assert hunter.sheet_image([]).size == (0, hunter.CELL_H)


def test_sheet_image_rejects_a_pixel_below_the_cell() -> None:
    with pytest.raises(SystemExit, match=r"blit clipped 0,200"):
        hunter.sheet_image([hunter.Sprite({(0, 200): RED})])


def test_attack_hand_of_the_idle_cell_is_the_right_hand(idle_cell) -> None:
    assert hunter.attack_hand(idle_cell) == (125, 120)


def test_sword_hand_of_the_idle_cell_is_the_skin_cluster(idle_cell) -> None:
    assert hunter.sword_hand(idle_cell) == (120.28455284552845, 101.3089430894309)


def test_sword_hand_rejects_a_sprite_with_no_hand() -> None:
    with pytest.raises(SystemExit, match="no sword hand"):
        hunter.sword_hand(hunter.Sprite({(0, 0): RED}))


def test_build_attack_frames_keeps_both_feet_on_the_attack_sole(attack_frames) -> None:
    between, recover = attack_frames
    assert hunter.stance_contact(between) == (159, 87.75)
    assert hunter.stance_contact(recover) == (159, 87.75)
    assert len(between.pixels) == 10343
    assert len(recover.pixels) == 9907


def test_attack_with_sword_rejects_a_body_with_no_hand() -> None:
    with pytest.raises(SystemExit, match="no sword hand"):
        hunter._attack_with_sword(hunter.Sprite({(10, 10): RED}), hunter.Sprite(), 10.0, 20.0)


def test_align_chest_and_sole_rejects_a_sprite_with_no_chest() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.align_chest_and_sole(_long_sole(), 8)


def test_nudge_chest_x_rejects_a_sprite_with_no_chest() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.nudge_chest_x(hunter.Sprite({(0, 0): RED}))


def test_reposition_boots_returns_the_same_pixels_when_the_boots_are_missing() -> None:
    sprite = hunter.Sprite({(1, 1): RED})
    assert hunter.reposition_boots(sprite, 10.0, 20.0, 30).pixels == {(1, 1): RED}


def test_build_jump_lifts_the_idle_cell_off_the_ground_row(idle_cell) -> None:
    rear = hunter.leg_column(idle_cell, 62, 116, 82, 160)
    front = hunter.leg_column(idle_cell, 90, 112, 116, 160)
    jump = hunter.build_jump(idle_cell, rear, front)
    assert len(jump.pixels) == 10655
    assert all(y < hunter.FOOT_ROW for _x, y in jump.pixels)


def test_build_fall_plants_the_feet_on_the_signed_foot_point(idle_cell) -> None:
    rear = hunter.leg_column(idle_cell, 62, 116, 82, 160)
    front = hunter.leg_column(idle_cell, 90, 112, 116, 160)
    assert hunter.stance_contact(hunter.build_fall(idle_cell, rear, front)) == (158, 81.5)


def test_death_stagger_and_crumple_both_plant_on_the_signed_foot_point(idle_cell) -> None:
    rear = hunter.leg_column(idle_cell, 62, 116, 82, 160)
    front = hunter.leg_column(idle_cell, 90, 112, 116, 160)
    assert hunter.stance_contact(hunter._death_stagger(idle_cell, rear, front)) == (158, 81.5)
    assert hunter.stance_contact(hunter._death_crumple(idle_cell, rear, front)) == (158, 81.5)


def test_build_jump_rejects_an_empty_body() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.build_jump(hunter.Sprite(), hunter.Sprite({(0, 0): RED}), hunter.Sprite({(1, 0): RED}))


def test_hunter_main_writes_the_six_combat_sheets_with_the_locked_hashes(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "combat"
    folder.mkdir()
    monkeypatch.setattr(hunter, "OUT", folder)
    hunter.main()
    expected = {
        "knight_idle_side.png": "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e",
        "knight_run_side.png": "526b3330f501ad9ffbd1d0ad6f4b0ac68cf6a50cd8eca79523ba14bf87481e4d",
        "knight_attack_side.png": "40d133d023a793428387519cbaae041db3dd39b9f7895549e5f6344f95c19b01",
        "knight_jump_side.png": "ad88d241c0c54b3e0cb006189d990044b31f03207b4fbd0e236fe72b322e6e44",
        "knight_fall_side.png": "894c42e0758f97d489c2f21e51e58490fee7f3470ca9ad28b1b40bc765f6b268",
        "knight_death_side.png": "1d8fd43d20b063e6ed4c12302c6193982a5784e233bd82fa0cbfaaf6a0b04539",
    }
    for name, digest in expected.items():
        assert _file_digest(folder / name) == digest


def test_hunter_main_rejects_a_signed_sheet_whose_hash_does_not_match(monkeypatch) -> None:
    monkeypatch.setitem(hunter.SIGNED_SHA256, "knight_idle_side.png", "0" * 64)
    with pytest.raises(SystemExit, match="knight_idle_side.png hash"):
        hunter.main()


def test_new_canvas_is_a_rejecting_32_pixel_icon() -> None:
    canvas = icons.new_canvas()
    assert (canvas.width, canvas.height) == (32, 32)
    with pytest.raises(ValueError, match=r"pixel outside the icon"):
        canvas.put(32, 0, icons.STEEL)


def test_put_stores_a_pixel_and_rejects_one_outside_the_icon() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 1, 2, icons.STEEL)
    assert canvas.read(1, 2) == icons.STEEL
    with pytest.raises(ValueError, match=r"pixel outside the icon: \(40, 0\)"):
        icons.put(canvas, 40, 0, icons.STEEL)


def test_fills_span_paints_a_closed_horizontal_span() -> None:
    canvas = icons.new_canvas()
    icons.fills_span(canvas, 1, 0, 2, icons.STEEL)
    assert canvas.read(0, 1) == icons.STEEL
    assert canvas.read(2, 1) == icons.STEEL
    assert canvas.read(3, 1) is None


def test_fills_span_rejects_a_span_that_leaves_the_icon() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.fills_span(icons.new_canvas(), 0, 30, 32, icons.STEEL)


def test_neighbor_returns_none_outside_the_icon_and_for_a_clear_pixel() -> None:
    canvas = icons.new_canvas()
    assert icons.neighbor(canvas, 0, 0, -1, 0) is None
    assert icons.neighbor(canvas, 1, 1, 0, 0) is None


def test_neighbor_returns_the_stored_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 2, 2, icons.STEEL)
    assert icons.neighbor(canvas, 2, 3, 0, -1) == icons.STEEL


def test_touches_empty_is_true_for_a_pixel_with_a_clear_side() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 1, 1, icons.STEEL)
    assert icons.touches_empty(canvas, 1, 1) is True


def test_touches_empty_is_false_when_all_four_sides_are_painted() -> None:
    canvas = icons.new_canvas()
    for x, y in ((1, 1), (0, 1), (2, 1), (1, 0), (1, 2)):
        icons.put(canvas, x, y, icons.STEEL)
    assert icons.touches_empty(canvas, 1, 1) is False


def test_outline_icon_paints_the_shared_outline_around_one_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.outline_icon(canvas)
    assert canvas.read(4, 3) == icons.OUTLINE
    assert canvas.read(4, 4) == icons.STEEL


def test_shade_outline_recolors_the_upper_outline() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.outline_icon(canvas)
    icons.shade_outline(canvas)
    assert canvas.read(4, 3) == icons.RIM_LIT


def test_finish_body_outlines_and_lights_the_upper_edge() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.finish_body(canvas)
    assert canvas.read(4, 4) == icons.STEEL
    assert canvas.read(4, 3) == icons.RIM_LIT


def test_finish_body_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(AttributeError):
        icons.finish_body(object())


def test_to_image_copies_the_canvas_pixels() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 0, 0, icons.STEEL)
    image = icons.to_image(canvas)
    assert image.size == (32, 32)
    assert image.getpixel((0, 0)) == icons.STEEL


def test_paint_sword_stores_the_bright_steel_at_the_tip() -> None:
    canvas = icons.new_canvas()
    icons.paint_sword(canvas)
    assert canvas.read(15, 3) == icons.STEEL_LT


def test_draw_sword_matches_the_locked_icon_pixels() -> None:
    image = icons.draw_sword()
    assert image.getpixel((0, 0)) == (0, 0, 0, 0)
    assert image.getpixel((15, 3)) == (232, 240, 248, 255)
    assert _image_digest(image) == "680106f437068abbfd8a6ba3110bb04ebb97bb2a719810094c0f6940fbff5c9b"


def test_paint_sword_rejects_a_pixel_if_the_canvas_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_sword(rejecting_canvas(4, 4))


def test_shield_spans_cover_the_shield_from_top_to_bottom() -> None:
    spans = icons.shield_spans()
    assert len(spans) == 21
    assert spans[0] == (6, 10, 21)
    assert spans[-1] == (26, 15, 16)


def test_paint_shield_fills_the_shield_body() -> None:
    canvas = icons.new_canvas()
    icons.paint_shield(canvas)
    assert sum(canvas.opaque_at(x, y) for y in range(32) for x in range(32)) == 316


def test_paint_shield_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_shield(rejecting_canvas(4, 4))


def test_recolor_rim_leaves_an_empty_canvas_unchanged() -> None:
    canvas = icons.new_canvas()
    icons.recolor_rim(canvas, icons.BRASS_LT, icons.BRASS, icons.BRASS_DK)
    assert all(pixel == (0, 0, 0, 0) for pixel in canvas.pixels)


def test_recolor_rim_lights_the_top_of_a_single_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.recolor_rim(canvas, icons.BRASS_LT, icons.BRASS, icons.BRASS_DK)
    assert canvas.read(4, 4) == icons.BRASS_LT


def test_recolor_rim_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(TypeError):
        icons.recolor_rim(None, icons.BRASS_LT, icons.BRASS, icons.BRASS_DK)


def test_stamp_boss_paints_one_pixel() -> None:
    canvas = icons.new_canvas()
    icons.stamp_boss(canvas)
    assert sum(canvas.opaque_at(x, y) for y in range(32) for x in range(32)) == 1


def test_stamp_boss_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.stamp_boss(rejecting_canvas(2, 2))


def test_paint_boot_paints_the_locked_opaque_count() -> None:
    canvas = icons.new_canvas()
    icons.paint_boot(canvas)
    assert sum(canvas.opaque_at(x, y) for y in range(32) for x in range(32)) == 196


def test_paint_boot_leaves_the_early_rows_clear() -> None:
    canvas = icons.new_canvas()
    icons.paint_boot(canvas)
    assert canvas.read(8, 8) is None


def test_paint_boot_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_boot(rejecting_canvas(4, 4))


def test_paint_boot_trail_keeps_a_pixel_that_was_already_there() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 1, 1, icons.STEEL)
    icons.paint_boot_trail(canvas)
    assert canvas.read(1, 1) == icons.STEEL


def test_paint_boot_trail_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_boot_trail(rejecting_canvas(2, 2))


def test_paint_wand_stores_the_purple_pixel() -> None:
    canvas = icons.new_canvas()
    icons.paint_wand(canvas)
    assert canvas.read(16, 10) == (214, 75, 255, 255)


def test_paint_wand_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_wand(rejecting_canvas(4, 4))


def test_paint_wand_orbit_runs_on_a_full_icon() -> None:
    canvas = icons.new_canvas()
    icons.paint_wand_orbit(canvas)
    assert canvas.width == 32


def test_paint_wand_orbit_rejects_a_canvas_that_is_too_small() -> None:
    with pytest.raises(ValueError, match="pixel outside the icon"):
        icons.paint_wand_orbit(rejecting_canvas(4, 4))


def test_ascii_preview_draws_the_sword_glyph() -> None:
    preview = icons.ascii_preview(icons.draw_sword())
    assert hashlib.sha256(preview.encode()).hexdigest() == "0cd91b14f39d92524413c774fedfa09aa0ecf72b740a95483f629b9102732d47"
    assert next(line for line in preview.splitlines() if line.strip()) == "              ++++"


def test_ascii_preview_of_a_clear_icon_is_blank_lines() -> None:
    preview = icons.ascii_preview(icons.to_image(icons.new_canvas()))
    assert preview == "\n" * 31


def test_ascii_preview_rejects_an_image_with_no_pixels() -> None:
    with pytest.raises(AttributeError):
        icons.ascii_preview(object())


def test_skill_main_writes_the_four_icons_with_the_locked_hashes(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "skills"
    folder.mkdir()
    monkeypatch.setattr(icons, "OUT_DIR", folder)
    monkeypatch.setattr(icons.sys, "argv", ["draw_skill_icons.py"])
    icons.main()
    expected = {
        "short_wep.png": "ff5a3b96d7eb7149a93573a43cd7b768c42c8876df84a386196adc1e43c47c6d",
        "shield.png": "d09c9853fd5bac9946cddc1b878b65b76a68b04150811ec6e0aae9f4c31eafb9",
        "boot.png": "496fe8e3de666315615f721b20ce8f6a961ad4f3841db036a5c467079e7068a0",
        "wand.png": "f0f78fdf6267a855090e52364c30229002fb2f2d2eb45a68b9bb4b5ac62eac82",
    }
    for name, digest in expected.items():
        assert _file_digest(folder / name) == digest


def test_build_run_cycle_lets_airborne_frames_leave_the_foot_row(run_frames) -> None:
    assert hunter.stance_contact(run_frames[2]) == (157, 90.5)
    assert hunter.stance_contact(run_frames[6]) == (155, 116.0)


def test_build_run_cycle_rejects_an_empty_sprite() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.build_run_cycle(hunter.Sprite())


def test_build_fall_rejects_an_empty_body() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter.build_fall(hunter.Sprite(), _long_sole(), _long_sole())


def test_death_stagger_rejects_an_empty_body() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter._death_stagger(hunter.Sprite(), _long_sole(), _long_sole())


def test_death_crumple_rejects_an_empty_body() -> None:
    with pytest.raises(SystemExit, match="no chest pixels"):
        hunter._death_crumple(hunter.Sprite(), _long_sole(), _long_sole())


def test_drop_chest_column_leaves_a_pixel_outside_the_torso() -> None:
    sprite = hunter.Sprite({(4, 10): RED})
    hunter.drop_chest_column(sprite, 4)
    assert sprite.pixels == {(4, 10): RED}


def test_drop_chest_column_rejects_a_missing_sprite() -> None:
    with pytest.raises(AttributeError):
        hunter.drop_chest_column(None, 0)


def test_widen_sole_toward_left_rejects_a_sprite_with_no_sole() -> None:
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.widen_sole_toward_left(hunter.Sprite())


def test_widen_sole_by_delta_rejects_a_sprite_with_no_sole() -> None:
    with pytest.raises(SystemExit, match="no stance contact"):
        hunter.widen_sole_by_delta(hunter.Sprite(), 1.0)


def test_seat_lifted_leg_rejects_an_empty_leg() -> None:
    with pytest.raises(ValueError):
        hunter.seat_lifted_leg(hunter.Sprite())


def test_load_sheet_reads_the_signed_attack_sheet_bounds() -> None:
    loaded = hunter.load_sheet("knight_attack_side.png")
    assert loaded.bounds() == (0, 0, 1371, 159)


def test_outline_icon_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(AttributeError):
        icons.outline_icon(object())


def test_shade_outline_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(AttributeError):
        icons.shade_outline(object())


def test_neighbor_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(TypeError):
        icons.neighbor(None, 0, 0, 1, 0)


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


def test_skill_main_check_rejects_a_missing_icon(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "skills"
    folder.mkdir()
    monkeypatch.setattr(icons, "OUT_DIR", folder)
    monkeypatch.setattr(icons.sys, "argv", ["draw_skill_icons.py", "--check"])
    with pytest.raises(SystemExit, match="missing"):
        icons.main()
