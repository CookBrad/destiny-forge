from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_dungeon_actors as actors
import draw_dungeon_environment as environment
import draw_dwarf_pickaxe_swing as pickaxe
import draw_dwarf_pickaxe_t2_swing as pickaxe_t2
import draw_dwarf_pickaxe_clank as pickaxe_clank
import draw_dwarf_pickaxe_t2_clank as pickaxe_t2_clank
import draw_dwarf_hoe_till as hoe
import draw_dwarf_water_empty as water_empty
import draw_dwarf_fishing_cast as fish
import draw_dwarf_facing as facing
import draw_dwarf_axe_chop as axe
import draw_dwarf_axe_chop_t2 as axe_t2
import draw_dwarf_axe_chop_t3 as axe_t3
import draw_dwarf_axe_chop_t4 as axe_t4
import draw_dwarf_axe_chop_t5 as axe_t5
import draw_dwarf_axe_charge_t2 as axe_charge_t2
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

def test_draw_slime_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_slime(object())


def test_draw_blob_paints_the_small_gel_blob(palette):
    canvas = actors.draw_blob(palette)
    assert (canvas.width, canvas.height) == (32, 32)
    assert _digest(canvas.pixels) == '22aac6bd18af5bbe108948db8d581a8ba2193b905ba5d4bdde883efe3c88d081'

def test_draw_blob_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_blob(object())


def test_draw_bolt_paints_the_slime_bolt(palette):
    canvas = actors.draw_bolt(palette)
    assert (canvas.width, canvas.height) == (16, 48)
    assert _digest(canvas.pixels) == '8ab4ed9b408d292621c7546ccdf688221d655fedbf701b160cd8dd0faff1c9fa'

def test_draw_bolt_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_bolt(object())


def test_draw_arrow_paints_the_brass_arrow(palette):
    canvas = actors.draw_arrow(palette)
    assert (canvas.width, canvas.height) == (16, 48)
    assert _digest(canvas.pixels) == '09b87796954105a9ef18380832881162ddc1a01865f0d3589f7e5561c2b3b795'

def test_draw_arrow_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_arrow(object())


def test_draw_bat_paints_the_signed_bat(palette):
    canvas = actors.draw_bat(palette)
    assert (canvas.width, canvas.height) == (96, 48)
    assert _digest(canvas.pixels) == '0b88b5d4b855b74804cf01c927fb6a69dc41e39581b654a785b42d0e4387c3ee'

def test_draw_bat_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_bat(object())


def test_draw_goblin_paints_the_signed_goblin(palette):
    canvas = actors.draw_goblin(palette)
    assert (canvas.width, canvas.height) == (80, 112)
    assert _digest(canvas.pixels) == '697b730b68859bf39985dade2053398b96f5430d78d5eb729f972e77aa622722'

def test_draw_goblin_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_goblin(object())


def test_draw_skeleton_paints_the_signed_skeleton(palette):
    canvas = actors.draw_skeleton(palette)
    assert (canvas.width, canvas.height) == (64, 144)
    assert _digest(canvas.pixels) == '16aa497935de235f69a4a9989133eb9998b91d2a3d69e3669c8b7219a44542b3'

def test_draw_skeleton_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_skeleton(object())


def test_draw_zombie_paints_the_signed_zombie(palette):
    canvas = actors.draw_zombie(palette)
    assert (canvas.width, canvas.height) == (96, 128)
    assert _digest(canvas.pixels) == 'f24c06a7c3135af33820f6ef6bd82fcc3f4075fa90c7ef9bbeca84fc7e0f1e8c'

def test_draw_zombie_rejects_a_palette_without_the_signed_ramps():
    with pytest.raises(AttributeError):
        actors.draw_zombie(object())


def test_draw_king_paints_the_signed_king_slime(palette):
    canvas = actors.draw_king(palette)
    assert (canvas.width, canvas.height) == (192, 192)
    assert _digest(canvas.pixels) == 'b84df8c2d3c144bf764515af1b2e2801aa5d0f0b010defd19cc6ebfdbcf28a8c'

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

def test_draw_stake_paints_the_locked_stake_canvas():
    canvas = environment.draw_stake()
    assert (canvas.width, canvas.height) == (24, 112)
    assert _digest(canvas.pixels) == '942c14b4862bddeae0e1871995757436e1e6b5193d05f3b154e0550d60fd58ff'


def test_draw_stake_keeps_the_locked_corner_pixel():
    assert environment.draw_stake().read(0, 0) == None

def test_draw_lip_paints_the_locked_lip_canvas():
    canvas = environment.draw_lip()
    assert (canvas.width, canvas.height) == (24, 96)
    assert _digest(canvas.pixels) == '00c04ef76b480b3674f0ac436c1c8575625ec0325bf3b1e1e19a0a81bbb8e05b'


def test_draw_lip_keeps_the_locked_corner_pixel():
    assert environment.draw_lip().read(0, 0) == (214, 164, 70, 255)

def test_draw_pillar_paints_the_locked_pillar_canvas():
    canvas = environment.draw_pillar()
    assert (canvas.width, canvas.height) == (64, 512)
    assert _digest(canvas.pixels) == '3cd652fca2c3ccebec9791d5c50ca36e18d506738068f7ccbfdf2497561cdd1a'


def test_draw_pillar_keeps_the_locked_corner_pixel():
    assert environment.draw_pillar().read(0, 0) == (48, 46, 56, 255)

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



def test_pickaxe_t2_outline_around_wraps_a_single_pixel():
    assert pickaxe_t2.outline_around({(1, 1): pickaxe_t2.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_pickaxe_t2_outline_around_of_an_empty_layer_stays_empty():
    assert pickaxe_t2.outline_around({}) == {}


def test_pickaxe_t2_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        pickaxe_t2.outline_around(None)


def test_draw_pickaxe_t2_frame_puts_the_pickaxe_behind_the_dwarf_on_the_first_pose():
    frame = pickaxe_t2.draw_frame(pickaxe_t2.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "ce7ccea0b0e3a50face412f357d1e585cd5368b45f6895353ee1be61df934919"


def test_draw_pickaxe_t2_frame_puts_the_pickaxe_in_front_on_the_third_pose():
    frame = pickaxe_t2.draw_frame(pickaxe_t2.POSES[2])
    assert _image_digest(frame) == "6126f77c348dd8b22fa0e90192b9a03e84301dbc2252c77c45ecf305a84a8ab1"
    assert _image_digest(frame) != "ce7ccea0b0e3a50face412f357d1e585cd5368b45f6895353ee1be61df934919"


def test_draw_pickaxe_t2_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        pickaxe_t2.draw_frame({})


def test_pickaxe_t2_main_writes_four_swing_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe_t2, "FOLDER", folder)
    monkeypatch.setattr(
        pickaxe_t2, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    pickaxe_t2.main()
    expected = [
        "ce7ccea0b0e3a50face412f357d1e585cd5368b45f6895353ee1be61df934919",
        "8c218e6cce6fcabd662beda9886c1e89bacf981a0515ca210acf501750ec5c87",
        "6126f77c348dd8b22fa0e90192b9a03e84301dbc2252c77c45ecf305a84a8ab1",
        "3079488cf8007ebcbbef2d0af0ea93f9b47fceb7b141d6d8c14ed70531dfe063",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_pickaxe_t2_swing_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_pickaxe_t2_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe_t2, "FOLDER", folder)
    monkeypatch.setattr(pickaxe_t2, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        pickaxe_t2.main()


def test_pickaxe_t2_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert pickaxe_t2.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_pickaxe_t2_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert pickaxe_t2.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_pickaxe_t2_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert pickaxe_t2.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_pickaxe_t2_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_pickaxe_t2_pixels_draws_an_iron_head_from_the_grip() -> None:
    assert pickaxe_t2.pickaxe_pixels((24.0, 40.0), 0.0) == {(23, 27): (132, 140, 148, 255), (24, 27): (132, 140, 148, 255), (23, 28): (119, 92, 85, 255), (24, 28): (119, 92, 85, 255), (23, 29): (119, 92, 85, 255), (24, 29): (119, 92, 85, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (23, 26): (132, 140, 148, 255), (24, 26): (132, 140, 148, 255), (20, 27): (132, 140, 148, 255), (21, 27): (132, 140, 148, 255), (22, 27): (132, 140, 148, 255), (25, 27): (132, 140, 148, 255), (26, 27): (132, 140, 148, 255), (27, 27): (132, 140, 148, 255), (17, 28): (132, 140, 148, 255), (18, 28): (132, 140, 148, 255), (19, 28): (132, 140, 148, 255), (20, 28): (132, 140, 148, 255), (21, 28): (132, 140, 148, 255), (26, 28): (132, 140, 148, 255), (27, 28): (132, 140, 148, 255), (28, 28): (132, 140, 148, 255), (29, 28): (132, 140, 148, 255), (30, 28): (132, 140, 148, 255), (17, 29): (132, 140, 148, 255), (18, 29): (132, 140, 148, 255), (29, 29): (132, 140, 148, 255), (30, 29): (132, 140, 148, 255), (22, 28): (34, 34, 34, 255), (25, 28): (34, 34, 34, 255), (22, 29): (34, 34, 34, 255), (25, 29): (34, 34, 34, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (22, 26): (34, 34, 34, 255), (23, 25): (34, 34, 34, 255), (25, 26): (34, 34, 34, 255), (24, 25): (34, 34, 34, 255), (19, 27): (34, 34, 34, 255), (20, 26): (34, 34, 34, 255), (21, 26): (34, 34, 34, 255), (26, 26): (34, 34, 34, 255), (28, 27): (34, 34, 34, 255), (27, 26): (34, 34, 34, 255), (16, 28): (34, 34, 34, 255), (17, 27): (34, 34, 34, 255), (18, 27): (34, 34, 34, 255), (19, 29): (34, 34, 34, 255), (20, 29): (34, 34, 34, 255), (21, 29): (34, 34, 34, 255), (26, 29): (34, 34, 34, 255), (27, 29): (34, 34, 34, 255), (28, 29): (34, 34, 34, 255), (29, 27): (34, 34, 34, 255), (31, 28): (34, 34, 34, 255), (30, 27): (34, 34, 34, 255), (16, 29): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (31, 29): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255)}


def test_pickaxe_t2_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert pickaxe_t2.pickaxe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_pickaxe_t2_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2.pickaxe_pixels(None, 0.0)


def test_pickaxe_t2_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    pickaxe_t2.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_pickaxe_t2_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    pickaxe_t2.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_pickaxe_t2_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_pickaxe_t2_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_pickaxe_t2_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_t2, "DWARF", sheet)
    assert pickaxe_t2.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_pickaxe_t2_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_t2, "DWARF", sheet)
    assert pickaxe_t2.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_pickaxe_t2_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_t2, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        pickaxe_t2.dwarf_pixels({})


def test_pickaxe_t2_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe_t2.fist_pixels((20.5, 27.5)) == {
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


def test_pickaxe_t2_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe_t2.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_pickaxe_t2_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2.fist_pixels(None)


def test_draw_pickaxe_t2_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = pickaxe_t2.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_pickaxe_t2_frames_differ_from_pickaxe_t1_pixel_digests() -> None:
    assert [
        _image_digest(pickaxe_t2.draw_frame(pose)) for pose in pickaxe_t2.POSES
    ] != [
        _image_digest(pickaxe.draw_frame(pose)) for pose in pickaxe.POSES
    ]



def test_pickaxe_clank_outline_around_wraps_a_single_pixel():
    assert pickaxe_clank.outline_around({(1, 1): pickaxe_clank.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_pickaxe_clank_outline_around_of_an_empty_layer_stays_empty():
    assert pickaxe_clank.outline_around({}) == {}


def test_pickaxe_clank_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        pickaxe_clank.outline_around(None)


def test_draw_pickaxe_clank_frame_jolts_back_on_the_second_pose():
    frame = pickaxe_clank.draw_frame(pickaxe_clank.POSES[1])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "a337e269e7f7646de79f3ca5e991a458a4f7243540bdb2041ce70d81e06c8dc4"


def test_draw_pickaxe_clank_frame_starts_from_the_blocked_hit_pose():
    frame = pickaxe_clank.draw_frame(pickaxe_clank.POSES[0])
    assert _image_digest(frame) == "7e379dd945403404da5316e90dc609ee9afac8d87cc123466f16d7023a3a5423"
    assert _image_digest(frame) != "a337e269e7f7646de79f3ca5e991a458a4f7243540bdb2041ce70d81e06c8dc4"


def test_draw_pickaxe_clank_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        pickaxe_clank.draw_frame({})


def test_pickaxe_clank_main_writes_three_recoil_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe_clank, "FOLDER", folder)
    monkeypatch.setattr(
        pickaxe_clank, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    pickaxe_clank.main()
    expected = [
        "7e379dd945403404da5316e90dc609ee9afac8d87cc123466f16d7023a3a5423",
        "a337e269e7f7646de79f3ca5e991a458a4f7243540bdb2041ce70d81e06c8dc4",
        "261ffafa9a05cc84e0dfa61b46c6579445aa121a7b24feabe5ccbb3b344602d2",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_pickaxe_clank_f{index}.png").convert("RGBA"))
        for index in range(3)
    ] == expected


def test_pickaxe_clank_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe_clank, "FOLDER", folder)
    monkeypatch.setattr(pickaxe_clank, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        pickaxe_clank.main()


def test_pickaxe_clank_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert pickaxe_clank.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_pickaxe_clank_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert pickaxe_clank.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_pickaxe_clank_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert pickaxe_clank.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_pickaxe_clank_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        pickaxe_clank.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_pickaxe_clank_pixels_draws_a_steel_head_from_the_grip() -> None:
    assert pickaxe_clank.pickaxe_pixels((24.0, 40.0), 0.0) == {(23, 27): (123, 137, 148, 255), (24, 27): (123, 137, 148, 255), (23, 28): (119, 92, 85, 255), (24, 28): (119, 92, 85, 255), (23, 29): (119, 92, 85, 255), (24, 29): (119, 92, 85, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (23, 26): (123, 137, 148, 255), (24, 26): (123, 137, 148, 255), (20, 27): (123, 137, 148, 255), (21, 27): (123, 137, 148, 255), (22, 27): (123, 137, 148, 255), (25, 27): (123, 137, 148, 255), (26, 27): (123, 137, 148, 255), (27, 27): (123, 137, 148, 255), (18, 28): (123, 137, 148, 255), (19, 28): (123, 137, 148, 255), (20, 28): (123, 137, 148, 255), (21, 28): (123, 137, 148, 255), (26, 28): (123, 137, 148, 255), (27, 28): (123, 137, 148, 255), (28, 28): (123, 137, 148, 255), (29, 28): (123, 137, 148, 255), (22, 28): (34, 34, 34, 255), (25, 28): (34, 34, 34, 255), (22, 29): (34, 34, 34, 255), (25, 29): (34, 34, 34, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (22, 26): (34, 34, 34, 255), (23, 25): (34, 34, 34, 255), (25, 26): (34, 34, 34, 255), (24, 25): (34, 34, 34, 255), (19, 27): (34, 34, 34, 255), (20, 26): (34, 34, 34, 255), (21, 26): (34, 34, 34, 255), (26, 26): (34, 34, 34, 255), (28, 27): (34, 34, 34, 255), (27, 26): (34, 34, 34, 255), (17, 28): (34, 34, 34, 255), (18, 29): (34, 34, 34, 255), (18, 27): (34, 34, 34, 255), (19, 29): (34, 34, 34, 255), (20, 29): (34, 34, 34, 255), (21, 29): (34, 34, 34, 255), (26, 29): (34, 34, 34, 255), (27, 29): (34, 34, 34, 255), (28, 29): (34, 34, 34, 255), (30, 28): (34, 34, 34, 255), (29, 29): (34, 34, 34, 255), (29, 27): (34, 34, 34, 255)}


def test_pickaxe_clank_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert pickaxe_clank.pickaxe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_pickaxe_clank_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe_clank.pickaxe_pixels(None, 0.0)


def test_pickaxe_clank_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    pickaxe_clank.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_pickaxe_clank_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    pickaxe_clank.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_pickaxe_clank_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        pickaxe_clank.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_pickaxe_clank_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_pickaxe_clank_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_clank_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_clank, "DWARF", sheet)
    assert pickaxe_clank.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_pickaxe_clank_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_clank_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_clank, "DWARF", sheet)
    assert pickaxe_clank.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_pickaxe_clank_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_clank_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_clank, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        pickaxe_clank.dwarf_pixels({})


def test_pickaxe_clank_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe_clank.fist_pixels((20.5, 27.5)) == {
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


def test_pickaxe_clank_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe_clank.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_pickaxe_clank_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe_clank.fist_pixels(None)


def test_draw_pickaxe_clank_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = pickaxe_clank.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_pickaxe_clank_frames_differ_from_pickaxe_swing_pixel_digests() -> None:
    assert [
        _image_digest(pickaxe_clank.draw_frame(pose)) for pose in pickaxe_clank.POSES
    ] != [
        _image_digest(pickaxe.draw_frame(pose)) for pose in pickaxe.POSES[:3]
    ]



def test_pickaxe_t2_clank_outline_around_wraps_a_single_pixel():
    assert pickaxe_t2_clank.outline_around({(1, 1): pickaxe_t2_clank.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_pickaxe_t2_clank_outline_around_of_an_empty_layer_stays_empty():
    assert pickaxe_t2_clank.outline_around({}) == {}


def test_pickaxe_t2_clank_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        pickaxe_t2_clank.outline_around(None)


def test_draw_pickaxe_t2_clank_frame_jolts_back_on_the_second_pose():
    frame = pickaxe_t2_clank.draw_frame(pickaxe_t2_clank.POSES[1])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "03d1628eaa8e57314c4ea2103f44845c457221979fec683704c25ccc7ad9038c"


def test_draw_pickaxe_t2_clank_frame_starts_from_the_blocked_hit_pose():
    frame = pickaxe_t2_clank.draw_frame(pickaxe_t2_clank.POSES[0])
    assert _image_digest(frame) == "bb697b80ae75c202af2dded2de34a6b7e61c1c05d32120e7bf94634afb2029e3"
    assert _image_digest(frame) != "03d1628eaa8e57314c4ea2103f44845c457221979fec683704c25ccc7ad9038c"


def test_draw_pickaxe_t2_clank_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        pickaxe_t2_clank.draw_frame({})


def test_pickaxe_t2_clank_main_writes_three_recoil_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe_t2_clank, "FOLDER", folder)
    monkeypatch.setattr(
        pickaxe_t2_clank, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    pickaxe_t2_clank.main()
    expected = [
        "bb697b80ae75c202af2dded2de34a6b7e61c1c05d32120e7bf94634afb2029e3",
        "03d1628eaa8e57314c4ea2103f44845c457221979fec683704c25ccc7ad9038c",
        "16ec32e5fde205afb8fbf76fd2502d7534fda4d1ecca275eb9d5fa7a9cf8ea2d",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_pickaxe_t2_clank_f{index}.png").convert("RGBA"))
        for index in range(3)
    ] == expected


def test_pickaxe_t2_clank_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(pickaxe_t2_clank, "FOLDER", folder)
    monkeypatch.setattr(pickaxe_t2_clank, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        pickaxe_t2_clank.main()


def test_pickaxe_t2_clank_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert pickaxe_t2_clank.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_pickaxe_t2_clank_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert pickaxe_t2_clank.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_pickaxe_t2_clank_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert pickaxe_t2_clank.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_pickaxe_t2_clank_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2_clank.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_pickaxe_t2_clank_pixels_draws_an_iron_head_from_the_grip() -> None:
    assert pickaxe_t2_clank.pickaxe_pixels((24.0, 40.0), 0.0) == {(23, 27): (132, 140, 148, 255), (24, 27): (132, 140, 148, 255), (23, 28): (119, 92, 85, 255), (24, 28): (119, 92, 85, 255), (23, 29): (119, 92, 85, 255), (24, 29): (119, 92, 85, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (23, 26): (132, 140, 148, 255), (24, 26): (132, 140, 148, 255), (20, 27): (132, 140, 148, 255), (21, 27): (132, 140, 148, 255), (22, 27): (132, 140, 148, 255), (25, 27): (132, 140, 148, 255), (26, 27): (132, 140, 148, 255), (27, 27): (132, 140, 148, 255), (17, 28): (132, 140, 148, 255), (18, 28): (132, 140, 148, 255), (19, 28): (132, 140, 148, 255), (20, 28): (132, 140, 148, 255), (21, 28): (132, 140, 148, 255), (26, 28): (132, 140, 148, 255), (27, 28): (132, 140, 148, 255), (28, 28): (132, 140, 148, 255), (29, 28): (132, 140, 148, 255), (30, 28): (132, 140, 148, 255), (17, 29): (132, 140, 148, 255), (18, 29): (132, 140, 148, 255), (29, 29): (132, 140, 148, 255), (30, 29): (132, 140, 148, 255), (22, 28): (34, 34, 34, 255), (25, 28): (34, 34, 34, 255), (22, 29): (34, 34, 34, 255), (25, 29): (34, 34, 34, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (22, 26): (34, 34, 34, 255), (23, 25): (34, 34, 34, 255), (25, 26): (34, 34, 34, 255), (24, 25): (34, 34, 34, 255), (19, 27): (34, 34, 34, 255), (20, 26): (34, 34, 34, 255), (21, 26): (34, 34, 34, 255), (26, 26): (34, 34, 34, 255), (28, 27): (34, 34, 34, 255), (27, 26): (34, 34, 34, 255), (16, 28): (34, 34, 34, 255), (17, 27): (34, 34, 34, 255), (18, 27): (34, 34, 34, 255), (19, 29): (34, 34, 34, 255), (20, 29): (34, 34, 34, 255), (21, 29): (34, 34, 34, 255), (26, 29): (34, 34, 34, 255), (27, 29): (34, 34, 34, 255), (28, 29): (34, 34, 34, 255), (29, 27): (34, 34, 34, 255), (31, 28): (34, 34, 34, 255), (30, 27): (34, 34, 34, 255), (16, 29): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (31, 29): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255)}


def test_pickaxe_t2_clank_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert pickaxe_t2_clank.pickaxe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_pickaxe_t2_clank_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2_clank.pickaxe_pixels(None, 0.0)


def test_pickaxe_t2_clank_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    pickaxe_t2_clank.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_pickaxe_t2_clank_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    pickaxe_t2_clank.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_pickaxe_t2_clank_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2_clank.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_pickaxe_t2_clank_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_pickaxe_t2_clank_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_t2_clank_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_t2_clank, "DWARF", sheet)
    assert pickaxe_t2_clank.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_pickaxe_t2_clank_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_t2_clank_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_t2_clank, "DWARF", sheet)
    assert pickaxe_t2_clank.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_pickaxe_t2_clank_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_pickaxe_t2_clank_dwarf_sheet(sheet)
    monkeypatch.setattr(pickaxe_t2_clank, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        pickaxe_t2_clank.dwarf_pixels({})


def test_pickaxe_t2_clank_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe_t2_clank.fist_pixels((20.5, 27.5)) == {
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


def test_pickaxe_t2_clank_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert pickaxe_t2_clank.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_pickaxe_t2_clank_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        pickaxe_t2_clank.fist_pixels(None)


def test_draw_pickaxe_t2_clank_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = pickaxe_t2_clank.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_pickaxe_t2_clank_frames_differ_from_pickaxe_t2_swing_and_t1_clank_pixel_digests() -> None:
    frames = [_image_digest(pickaxe_t2_clank.draw_frame(pose)) for pose in pickaxe_t2_clank.POSES]
    assert frames != [_image_digest(pickaxe_t2.draw_frame(pose)) for pose in pickaxe_t2.POSES[:3]]
    assert frames != [_image_digest(pickaxe_clank.draw_frame(pose)) for pose in pickaxe_clank.POSES]


def test_hoe_outline_around_wraps_a_single_pixel():
    assert hoe.outline_around({(1, 1): hoe.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_hoe_outline_around_of_an_empty_layer_stays_empty():
    assert hoe.outline_around({}) == {}


def test_hoe_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        hoe.outline_around(None)


def test_draw_frame_puts_the_hoe_behind_the_dwarf_on_the_first_pose():
    frame = hoe.draw_frame(hoe.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "c19e0ef0d03a56f1affb35e6eed00a48920d51d4f095adacbaa8a77e3f2d4aab"


def test_draw_frame_puts_the_hoe_in_front_on_the_third_pose():
    frame = hoe.draw_frame(hoe.POSES[2])
    assert _image_digest(frame) == "1416b9358c85391279aabf0d6361de66e539578191b9a06fc5567185c23899cb"
    assert _image_digest(frame) != "c19e0ef0d03a56f1affb35e6eed00a48920d51d4f095adacbaa8a77e3f2d4aab"


def test_draw_hoe_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        hoe.draw_frame({})


def test_hoe_main_writes_four_till_frames_with_the_locked_hashes(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(hoe, "FOLDER", folder)
    monkeypatch.setattr(
        hoe, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    hoe.main()
    expected = [
        "c19e0ef0d03a56f1affb35e6eed00a48920d51d4f095adacbaa8a77e3f2d4aab",
        "a81f4d6456dfec0eafafc0ab4f2b3832f5396fb23958b6b9e014d6bad3fe569a",
        "1416b9358c85391279aabf0d6361de66e539578191b9a06fc5567185c23899cb",
        "39242e8243013006807f39286c6f9aefc627619bcf83f0eebc90caf1e3ff7d4d",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_hoe_t1_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_hoe_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(hoe, "FOLDER", folder)
    monkeypatch.setattr(hoe, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        hoe.main()


def test_hoe_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert hoe.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_hoe_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert hoe.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_hoe_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert hoe.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_hoe_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        hoe.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_hoe_pixels_draws_an_upright_blade_from_the_grip() -> None:
    wood = (119, 92, 85, 255)
    rust = (138, 96, 62, 255)
    rust_edge = (98, 68, 44, 255)
    outline = (34, 34, 34, 255)
    assert hoe.hoe_pixels((24.0, 40.0), 0.0) == {
        (21, 25): outline,
        (21, 26): outline,
        (22, 24): outline,
        (22, 25): rust,
        (22, 26): rust,
        (22, 27): outline,
        (22, 28): outline,
        (22, 29): outline,
        (22, 30): outline,
        (22, 31): outline,
        (22, 32): outline,
        (22, 33): outline,
        (22, 34): outline,
        (22, 35): outline,
        (22, 36): outline,
        (22, 37): outline,
        (22, 38): outline,
        (22, 39): outline,
        (23, 24): outline,
        (23, 25): rust,
        (23, 26): rust,
        (23, 27): wood,
        (23, 28): wood,
        (23, 29): wood,
        (23, 30): wood,
        (23, 31): wood,
        (23, 32): wood,
        (23, 33): wood,
        (23, 34): wood,
        (23, 35): wood,
        (23, 36): wood,
        (23, 37): wood,
        (23, 38): wood,
        (23, 39): wood,
        (23, 40): outline,
        (24, 24): outline,
        (24, 25): rust,
        (24, 26): rust,
        (24, 27): wood,
        (24, 28): wood,
        (24, 29): wood,
        (24, 30): wood,
        (24, 31): wood,
        (24, 32): wood,
        (24, 33): wood,
        (24, 34): wood,
        (24, 35): wood,
        (24, 36): wood,
        (24, 37): wood,
        (24, 38): wood,
        (24, 39): wood,
        (24, 40): outline,
        (25, 24): outline,
        (25, 25): rust,
        (25, 26): rust,
        (25, 27): outline,
        (25, 28): outline,
        (25, 29): outline,
        (25, 30): outline,
        (25, 31): outline,
        (25, 32): outline,
        (25, 33): outline,
        (25, 34): outline,
        (25, 35): outline,
        (25, 36): outline,
        (25, 37): outline,
        (25, 38): outline,
        (25, 39): outline,
        (26, 24): outline,
        (26, 25): rust,
        (26, 26): rust,
        (26, 27): outline,
        (27, 24): outline,
        (27, 25): rust,
        (27, 26): rust,
        (27, 27): outline,
        (28, 24): outline,
        (28, 25): rust,
        (28, 26): rust,
        (28, 27): outline,
        (29, 23): outline,
        (29, 24): rust_edge,
        (29, 25): rust_edge,
        (29, 26): rust,
        (29, 27): outline,
        (30, 23): outline,
        (30, 24): rust_edge,
        (30, 25): rust_edge,
        (30, 26): rust,
        (30, 27): outline,
        (31, 24): outline,
        (31, 25): outline,
        (31, 26): outline,
    }


def test_hoe_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert hoe.hoe_pixels((1.0, 1.0), 0.0) == {
        (0, 0): (119, 92, 85, 255),
        (1, 0): (119, 92, 85, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 1): (34, 34, 34, 255),
        (2, 0): (34, 34, 34, 255),
    }


def test_hoe_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        hoe.hoe_pixels(None, 0.0)


def test_hoe_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    hoe.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_hoe_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    hoe.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_hoe_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        hoe.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_hoe_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_hoe_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_hoe_dwarf_sheet(sheet)
    monkeypatch.setattr(hoe, "DWARF", sheet)
    assert hoe.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_hoe_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_hoe_dwarf_sheet(sheet)
    monkeypatch.setattr(hoe, "DWARF", sheet)
    assert hoe.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_hoe_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_hoe_dwarf_sheet(sheet)
    monkeypatch.setattr(hoe, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        hoe.dwarf_pixels({})


def test_hoe_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert hoe.fist_pixels((20.5, 27.5)) == {
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


def test_hoe_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert hoe.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_hoe_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        hoe.fist_pixels(None)


def test_draw_hoe_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = hoe.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_fish_outline_around_wraps_a_single_pixel():
    assert fish.outline_around({(1, 1): fish.BAMBOO}) == {
        (1, 1): (166, 140, 72, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_fish_outline_around_of_an_empty_layer_stays_empty():
    assert fish.outline_around({}) == {}


def test_fish_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        fish.outline_around(None)


def test_draw_frame_puts_the_rod_behind_the_dwarf_on_cast_charge():
    frame = fish.draw_frame(fish.CAST_CHARGE[0])
    assert frame.size == (64, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "f6147316bcb98d5f94a9f9eefe91ff710f273de3becf52a713a8742bb0fded22"


def test_draw_frame_puts_the_rod_in_front_on_cast_release_hit():
    frame = fish.draw_frame(fish.CAST_RELEASE[2])
    assert _image_digest(frame) == "5ebc74c7f61827185917c64f8489b78204cb12fbaa44c020fc3b3967c410d4e4"
    assert _image_digest(frame) != "f6147316bcb98d5f94a9f9eefe91ff710f273de3becf52a713a8742bb0fded22"


def test_draw_fish_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        fish.draw_frame({})


def test_fish_main_writes_bamboo_fs1_fs3_frames_with_the_locked_hashes(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(fish, "FOLDER", folder)
    monkeypatch.setattr(
        fish, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    fish.main()
    expected = {
        "dwarf_m_rod_bamboo_cast_charge": [
            "f6147316bcb98d5f94a9f9eefe91ff710f273de3becf52a713a8742bb0fded22",
            "054a774523606ce6f6c7bf8c0d1ceb72f61312c0d71cf9d4d73a2211a23e0d83",
            "393d768df2be00775bf9c59b10b1ef8b3ee7d4fe33408a6f9fe0682a76999e0a",
        ],
        "dwarf_m_rod_bamboo_cast_release": [
            "66d50178e3fdc475235604209e60a12d7c81d67b67082d7283b42922ec339629",
            "7a52b1fcabc4b635336f0acddcfeddfa9f6dab41b0e2676bbc3a88bd239b0ce2",
            "5ebc74c7f61827185917c64f8489b78204cb12fbaa44c020fc3b3967c410d4e4",
            "8bdccb1b9895fd44cbd3f5d1186dc8940a49cc7afa3eca497cdea02a5b4e14da",
        ],
        "dwarf_m_rod_bamboo_wait": [
            "baa1451d78bb13e88e7a0c3bc3b89dac2dea44b521bbfb104c7c72bbf202958d",
            "68712570f115cd27e941125572beaa5d1abb65d70d37a063015120100a31bb98",
            "36338253b8c859f9cfccf94c1255babef607071a7b8d62a97db98d10eb801bff",
            "c03a33af6033308080cc6cbbea8e5193093f2d4b7e7dd765f13ed0a5f5e49bda",
        ],
    }
    for stem, digests in expected.items():
        assert [
            _image_digest(Image.open(folder / f"{stem}_f{index}.png").convert("RGBA"))
            for index in range(len(digests))
        ] == digests


def test_fish_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(fish, "FOLDER", folder)
    monkeypatch.setattr(fish, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        fish.main()


def test_fish_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert fish.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_fish_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert fish.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_fish_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert fish.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_fish_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        fish.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_rod_pixels_draws_an_upright_bamboo_shaft_from_the_grip() -> None:
    bamboo = (166, 140, 72, 255)
    outline = (34, 34, 34, 255)
    assert fish.rod_pixels({"grip": (24.0, 40.0), "angle": 0.0, "line": False}) == {
        (22, 18): outline,
        (22, 19): outline,
        (22, 20): outline,
        (22, 21): outline,
        (22, 22): outline,
        (22, 23): outline,
        (22, 24): outline,
        (22, 25): outline,
        (22, 26): outline,
        (22, 27): outline,
        (22, 28): outline,
        (22, 29): outline,
        (22, 30): outline,
        (22, 31): outline,
        (22, 32): outline,
        (22, 33): outline,
        (22, 34): outline,
        (22, 35): outline,
        (22, 36): outline,
        (22, 37): outline,
        (22, 38): outline,
        (22, 39): outline,
        (23, 17): outline,
        (23, 18): bamboo,
        (23, 19): bamboo,
        (23, 20): bamboo,
        (23, 21): bamboo,
        (23, 22): bamboo,
        (23, 23): bamboo,
        (23, 24): bamboo,
        (23, 25): bamboo,
        (23, 26): bamboo,
        (23, 27): bamboo,
        (23, 28): bamboo,
        (23, 29): bamboo,
        (23, 30): bamboo,
        (23, 31): bamboo,
        (23, 32): bamboo,
        (23, 33): bamboo,
        (23, 34): bamboo,
        (23, 35): bamboo,
        (23, 36): bamboo,
        (23, 37): bamboo,
        (23, 38): bamboo,
        (23, 39): bamboo,
        (23, 40): outline,
        (24, 17): outline,
        (24, 18): bamboo,
        (24, 19): bamboo,
        (24, 20): bamboo,
        (24, 21): bamboo,
        (24, 22): bamboo,
        (24, 23): bamboo,
        (24, 24): bamboo,
        (24, 25): bamboo,
        (24, 26): bamboo,
        (24, 27): bamboo,
        (24, 28): bamboo,
        (24, 29): bamboo,
        (24, 30): bamboo,
        (24, 31): bamboo,
        (24, 32): bamboo,
        (24, 33): bamboo,
        (24, 34): bamboo,
        (24, 35): bamboo,
        (24, 36): bamboo,
        (24, 37): bamboo,
        (24, 38): bamboo,
        (24, 39): bamboo,
        (24, 40): outline,
        (25, 18): outline,
        (25, 19): outline,
        (25, 20): outline,
        (25, 21): outline,
        (25, 22): outline,
        (25, 23): outline,
        (25, 24): outline,
        (25, 25): outline,
        (25, 26): outline,
        (25, 27): outline,
        (25, 28): outline,
        (25, 29): outline,
        (25, 30): outline,
        (25, 31): outline,
        (25, 32): outline,
        (25, 33): outline,
        (25, 34): outline,
        (25, 35): outline,
        (25, 36): outline,
        (25, 37): outline,
        (25, 38): outline,
        (25, 39): outline,
    }


def test_rod_pixels_clips_an_upward_shaft_to_the_canvas() -> None:
    assert fish.rod_pixels({"grip": (1.0, 1.0), "angle": 0.0, "line": False}) == {
        (0, 0): (166, 140, 72, 255),
        (1, 0): (166, 140, 72, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 1): (34, 34, 34, 255),
        (2, 0): (34, 34, 34, 255),
    }


def test_rod_pixels_rejects_a_pose_without_a_grip() -> None:
    with pytest.raises(KeyError, match="grip"):
        fish.rod_pixels({})


def test_fish_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    fish.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_fish_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    fish.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_fish_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        fish.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_fish_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_fish_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_fish_dwarf_sheet(sheet)
    monkeypatch.setattr(fish, "DWARF", sheet)
    assert fish.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_fish_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_fish_dwarf_sheet(sheet)
    monkeypatch.setattr(fish, "DWARF", sheet)
    assert fish.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_fish_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_fish_dwarf_sheet(sheet)
    monkeypatch.setattr(fish, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        fish.dwarf_pixels({})


def test_fish_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert fish.fist_pixels((20.5, 27.5)) == {
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


def test_fish_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert fish.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_fish_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        fish.fist_pixels(None)


def test_draw_fish_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = fish.draw_frame(
        {
            "lean_x": 0,
            "drop_y": 0,
            "grip": (1.0, 1.0),
            "angle": 0.0,
            "in_front": False,
            "line": False,
        }
    )
    assert frame.size == (64, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "237ff3117b956a412ed8006338f5b597943e7cb35732e41c21821d315365332c"



def test_facing_stamp_pattern_paints_palette_keys() -> None:
    assert facing.stamp_pattern(["a..............."] + ["." * 16] * 27) == {
        (0, 0): (34, 34, 34, 255)
    }


def test_facing_stamp_pattern_of_an_empty_grid_stays_empty() -> None:
    assert facing.stamp_pattern(["." * 16] * 28) == {}


def test_facing_stamp_pattern_rejects_a_missing_pattern() -> None:
    with pytest.raises(TypeError):
        facing.stamp_pattern(None)


def test_facing_require_pattern_size_accepts_a_full_canvas() -> None:
    facing.require_pattern_size(facing.FRONT_IDLE_BASE)


def test_facing_require_pattern_size_rejects_a_short_pattern() -> None:
    with pytest.raises(ValueError, match="height"):
        facing.require_pattern_size(["." * 16])


def test_facing_require_pattern_size_rejects_a_narrow_row() -> None:
    with pytest.raises(ValueError, match="width"):
        facing.require_pattern_size(["." * 15] + ["." * 16] * 27)


def test_facing_with_boots_replaces_the_boot_rows() -> None:
    rows = facing.with_boots(facing.FRONT_IDLE_BASE, "A" * 16, "B" * 16, "C" * 16)
    assert rows[23] == "A" * 16
    assert rows[24] == "B" * 16
    assert rows[25] == "C" * 16
    assert rows[0] == facing.FRONT_IDLE_BASE[0]


def test_facing_with_boots_on_a_blank_canvas_keeps_other_rows() -> None:
    blank = ["." * 16] * 28
    rows = facing.with_boots(blank, "b" * 16, "b" * 16, "a" * 16)
    assert rows[22] == "." * 16
    assert rows[23] == "b" * 16


def test_facing_with_boots_rejects_a_missing_base() -> None:
    with pytest.raises(TypeError):
        facing.with_boots(None, "." * 16, "." * 16, "." * 16)


def test_facing_draw_frame_matches_the_locked_front_idle_digest() -> None:
    frame = facing.draw_frame(facing.FRONT_IDLE_BASE)
    assert frame.size == (16, 28)
    assert _image_digest(frame) == "4156e79d3c292ff8e87ecbf33160aeec033a4e790903755a80e2f2fc6a4de909"


def test_facing_draw_frame_with_a_single_outline_pixel() -> None:
    pattern = ["a..............."] + ["." * 16] * 27
    frame = facing.draw_frame(pattern)
    assert frame.getpixel((0, 0)) == (34, 34, 34, 255)
    assert _image_digest(frame) == "e4ee68e2c8e4ad0fd3243a294e0252b32c1b255a609704420fb238569037e0bb"


def test_facing_draw_frame_rejects_an_empty_pattern() -> None:
    with pytest.raises(ValueError, match="height"):
        facing.draw_frame([])


def test_facing_main_writes_ap1_sets_with_the_locked_pixel_digests(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(facing, "FOLDER", folder)
    facing.main()
    expected = {
        "dwarf_m_idle_front": [
            "4156e79d3c292ff8e87ecbf33160aeec033a4e790903755a80e2f2fc6a4de909",
            "e3c3f8802111c17d7cfdf58ccfe8bcf16b55f5969fe61a0feb8a5a439fc77752",
            "4156e79d3c292ff8e87ecbf33160aeec033a4e790903755a80e2f2fc6a4de909",
            "372dd7f0979183cdbdbe104e3764104a7a82682e6bdd09976ec27a64cc6908b8",
        ],
        "dwarf_m_walk_front": [
            "03d4043cd8f06ad15727f99258a257db0a841f3f193dfcd2835a83c6d2b4b510",
            "372dd7f0979183cdbdbe104e3764104a7a82682e6bdd09976ec27a64cc6908b8",
            "c11ac07dadb54627a6c84fc0a4bc2c0e5833161438c75ccc3146ea2f2fedb9ac",
            "8b0023cde9b738a6c3b44dcaee4b6144b7ad540e73e00a166a27fe731b8a30fc",
        ],
        "dwarf_m_walk_back": [
            "9c9e138a07909ff11c8fa6105817ac607531bd9aa01551445a390dc1333c8c77",
            "633a1b4825c9ad3c56c79e317f12a7333881510988a72940f0a16afdc4b0bed7",
            "e8a6c3bc04caabedbe4aea10f75d6060540e43380f9e3a29116b7543eaf14f84",
            "e7a87c2522c8ebc18d53e5d93e91a1a8bf122a23b26162e7e76e1acbba32ba92",
        ],
    }
    for stem, digests in expected.items():
        assert [
            _image_digest(Image.open(folder / f"{stem}_f{index}.png").convert("RGBA"))
            for index in range(4)
        ] == digests


def test_facing_idle_front_frames_returns_four_patterns() -> None:
    frames = facing.idle_front_frames()
    assert len(frames) == 4
    assert frames[0] == facing.FRONT_IDLE_BASE


def test_facing_walk_builders_return_four_boot_variants() -> None:
    front_boots = [
        ("...abba..bba....", "...aba...aba....", "...aa.....aa...."),
        ("....abbaabba....", "....aba.aba.....", "....aa...aa....."),
        ("...bba..abba....", "...aba...aba....", "...aa.....aa...."),
        ("....abbaabba....", "...aba...aba....", "...aa.....aa...."),
    ]
    back_boots = [
        ("...abba..bba....", "...aba...aba....", "...aa.....aa...."),
        ("....abbaabba....", "....aba.aba.....", "....aa...aa....."),
        ("...bba..abba....", "...aba...aba....", "...aa.....aa...."),
        ("....abbaabba....", "...aba...aba....", "...aa.....aa...."),
    ]
    front_frames = facing.walk_front_frames()
    back_frames = facing.walk_back_frames()
    assert len(front_frames) == 4
    assert len(back_frames) == 4
    for frame, (row23, row24, row25) in zip(front_frames, front_boots, strict=True):
        assert frame[23] == row23
        assert frame[24] == row24
        assert frame[25] == row25
    for frame, (row23, row24, row25) in zip(back_frames, back_boots, strict=True):
        assert frame[23] == row23
        assert frame[24] == row24
        assert frame[25] == row25


def test_facing_back_walk_frames_show_hair_on_the_back_of_the_head() -> None:
    assert facing.walk_back_frames()[0][10] == "..arrrrrrrrrra.."



def test_axe_outline_around_wraps_a_single_pixel():
    assert axe.outline_around({(1, 1): axe.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_axe_outline_around_of_an_empty_layer_stays_empty():
    assert axe.outline_around({}) == {}


def test_axe_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        axe.outline_around(None)


def test_draw_frame_puts_the_axe_behind_the_dwarf_on_the_first_pose():
    frame = axe.draw_frame(axe.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "f66ea4b3b6fa55db0968182b2b86ee12b41f0ad352e1bc41adde469c6b3ba7bb"


def test_draw_frame_puts_the_axe_in_front_on_the_third_pose():
    frame = axe.draw_frame(axe.POSES[2])
    assert _image_digest(frame) == "79d130072ec9d1433fa376c65258dd62a12aa5e18812e2eb5aee083f9b12075c"
    assert _image_digest(frame) != "f66ea4b3b6fa55db0968182b2b86ee12b41f0ad352e1bc41adde469c6b3ba7bb"


def test_draw_axe_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        axe.draw_frame({})


def test_axe_main_writes_four_chop_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe, "FOLDER", folder)
    monkeypatch.setattr(
        axe, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    axe.main()
    expected = [
        "f66ea4b3b6fa55db0968182b2b86ee12b41f0ad352e1bc41adde469c6b3ba7bb",
        "358bd87e889dd6009b25c7176ca5d6707c9e2f325c099c1a842f98b02c12be1a",
        "79d130072ec9d1433fa376c65258dd62a12aa5e18812e2eb5aee083f9b12075c",
        "242941e74793cd799fdd1c2b95f2a597ef16fc1fb67649491952963eb4004c7e",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_axe_t1_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_axe_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe, "FOLDER", folder)
    monkeypatch.setattr(axe, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        axe.main()


def test_axe_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert axe.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_axe_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert axe.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_axe_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert axe.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_axe_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        axe.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_axe_pixels_draws_an_upright_blade_from_the_grip() -> None:
    assert axe.axe_pixels((24.0, 40.0), 0.0) == {(23, 26): (123, 137, 148, 255), (24, 26): (123, 137, 148, 255), (23, 27): (123, 137, 148, 255), (24, 27): (123, 137, 148, 255), (23, 28): (123, 137, 148, 255), (24, 28): (123, 137, 148, 255), (23, 29): (119, 92, 85, 255), (24, 29): (119, 92, 85, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (23, 25): (98, 112, 122, 255), (24, 25): (98, 112, 122, 255), (20, 26): (123, 137, 148, 255), (21, 26): (123, 137, 148, 255), (22, 26): (123, 137, 148, 255), (25, 26): (123, 137, 148, 255), (26, 26): (123, 137, 148, 255), (27, 26): (123, 137, 148, 255), (18, 27): (123, 137, 148, 255), (19, 27): (123, 137, 148, 255), (20, 27): (123, 137, 148, 255), (21, 27): (123, 137, 148, 255), (22, 27): (123, 137, 148, 255), (25, 27): (123, 137, 148, 255), (26, 27): (123, 137, 148, 255), (27, 27): (123, 137, 148, 255), (28, 27): (123, 137, 148, 255), (29, 27): (123, 137, 148, 255), (18, 28): (123, 137, 148, 255), (19, 28): (123, 137, 148, 255), (20, 28): (123, 137, 148, 255), (21, 28): (123, 137, 148, 255), (22, 28): (123, 137, 148, 255), (25, 28): (123, 137, 148, 255), (26, 28): (123, 137, 148, 255), (27, 28): (123, 137, 148, 255), (28, 28): (123, 137, 148, 255), (29, 28): (123, 137, 148, 255), (18, 25): (98, 112, 122, 255), (19, 25): (98, 112, 122, 255), (20, 25): (98, 112, 122, 255), (21, 25): (98, 112, 122, 255), (22, 25): (98, 112, 122, 255), (25, 25): (98, 112, 122, 255), (26, 25): (98, 112, 122, 255), (27, 25): (98, 112, 122, 255), (28, 25): (98, 112, 122, 255), (29, 25): (98, 112, 122, 255), (22, 29): (34, 34, 34, 255), (25, 29): (34, 34, 34, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (23, 24): (34, 34, 34, 255), (24, 24): (34, 34, 34, 255), (19, 26): (34, 34, 34, 255), (28, 26): (34, 34, 34, 255), (17, 27): (34, 34, 34, 255), (18, 26): (34, 34, 34, 255), (30, 27): (34, 34, 34, 255), (29, 26): (34, 34, 34, 255), (17, 28): (34, 34, 34, 255), (18, 29): (34, 34, 34, 255), (19, 29): (34, 34, 34, 255), (20, 29): (34, 34, 34, 255), (21, 29): (34, 34, 34, 255), (26, 29): (34, 34, 34, 255), (27, 29): (34, 34, 34, 255), (28, 29): (34, 34, 34, 255), (30, 28): (34, 34, 34, 255), (29, 29): (34, 34, 34, 255), (17, 25): (34, 34, 34, 255), (18, 24): (34, 34, 34, 255), (19, 24): (34, 34, 34, 255), (20, 24): (34, 34, 34, 255), (21, 24): (34, 34, 34, 255), (22, 24): (34, 34, 34, 255), (25, 24): (34, 34, 34, 255), (26, 24): (34, 34, 34, 255), (27, 24): (34, 34, 34, 255), (28, 24): (34, 34, 34, 255), (30, 25): (34, 34, 34, 255), (29, 24): (34, 34, 34, 255)}


def test_axe_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert axe.axe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_axe_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe.axe_pixels(None, 0.0)


def test_axe_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    axe.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_axe_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    axe.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_axe_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        axe.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_axe_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_axe_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_dwarf_sheet(sheet)
    monkeypatch.setattr(axe, "DWARF", sheet)
    assert axe.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_dwarf_sheet(sheet)
    monkeypatch.setattr(axe, "DWARF", sheet)
    assert axe.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_dwarf_sheet(sheet)
    monkeypatch.setattr(axe, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        axe.dwarf_pixels({})


def test_axe_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe.fist_pixels((20.5, 27.5)) == {
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


def test_axe_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_axe_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe.fist_pixels(None)


def test_draw_axe_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = axe.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"



def test_axe_t2_outline_around_wraps_a_single_pixel():
    assert axe_t2.outline_around({(1, 1): axe_t2.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_axe_t2_outline_around_of_an_empty_layer_stays_empty():
    assert axe_t2.outline_around({}) == {}


def test_axe_t2_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        axe_t2.outline_around(None)


def test_draw_axe_t2_frame_puts_the_axe_behind_the_dwarf_on_the_first_pose():
    frame = axe_t2.draw_frame(axe_t2.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "7b2f872fe344fba4f6dc4cbbe9d3a5e389d7dbf14f9eb6e2e11a62fa912e551e"


def test_draw_axe_t2_frame_puts_the_axe_in_front_on_the_third_pose():
    frame = axe_t2.draw_frame(axe_t2.POSES[2])
    assert _image_digest(frame) == "e4dd81a6814db49427123f58c4658f3c24e99540e991e99f70b08a590f88a948"
    assert _image_digest(frame) != "7b2f872fe344fba4f6dc4cbbe9d3a5e389d7dbf14f9eb6e2e11a62fa912e551e"


def test_draw_axe_t2_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        axe_t2.draw_frame({})


def test_axe_t2_main_writes_four_chop_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t2, "FOLDER", folder)
    monkeypatch.setattr(
        axe_t2, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    axe_t2.main()
    expected = [
        "7b2f872fe344fba4f6dc4cbbe9d3a5e389d7dbf14f9eb6e2e11a62fa912e551e",
        "aa2027dee9f27e1fef9209f95c6d63783e2ab68f1d8d83f9c5273755c06713a8",
        "e4dd81a6814db49427123f58c4658f3c24e99540e991e99f70b08a590f88a948",
        "7e179594ac73ced6b7a1a02ce7a0f4ab0a25f3706787c0c456715465c6d928ad",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_axe_t2_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_axe_t2_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t2, "FOLDER", folder)
    monkeypatch.setattr(axe_t2, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        axe_t2.main()


def test_axe_t2_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert axe_t2.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_axe_t2_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert axe_t2.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_axe_t2_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert axe_t2.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_axe_t2_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        axe_t2.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_axe_t2_pixels_draws_a_copper_blade_from_the_grip() -> None:
    assert axe_t2.axe_pixels((24.0, 40.0), 0.0) == {(23, 26): (196, 122, 64, 255), (24, 26): (196, 122, 64, 255), (23, 27): (119, 92, 85, 255), (24, 27): (119, 92, 85, 255), (23, 28): (220, 156, 96, 255), (24, 28): (220, 156, 96, 255), (23, 29): (196, 122, 64, 255), (24, 29): (196, 122, 64, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (23, 25): (148, 86, 42, 255), (24, 25): (148, 86, 42, 255), (20, 26): (196, 122, 64, 255), (21, 26): (196, 122, 64, 255), (22, 26): (196, 122, 64, 255), (25, 26): (196, 122, 64, 255), (26, 26): (196, 122, 64, 255), (27, 26): (196, 122, 64, 255), (18, 27): (196, 122, 64, 255), (19, 27): (196, 122, 64, 255), (20, 27): (196, 122, 64, 255), (21, 27): (196, 122, 64, 255), (22, 27): (196, 122, 64, 255), (25, 27): (196, 122, 64, 255), (26, 27): (196, 122, 64, 255), (27, 27): (196, 122, 64, 255), (28, 27): (196, 122, 64, 255), (29, 27): (196, 122, 64, 255), (17, 28): (220, 156, 96, 255), (18, 28): (220, 156, 96, 255), (19, 28): (220, 156, 96, 255), (20, 28): (220, 156, 96, 255), (21, 28): (220, 156, 96, 255), (22, 28): (220, 156, 96, 255), (25, 28): (220, 156, 96, 255), (26, 28): (220, 156, 96, 255), (27, 28): (220, 156, 96, 255), (28, 28): (220, 156, 96, 255), (29, 28): (220, 156, 96, 255), (30, 28): (220, 156, 96, 255), (17, 29): (196, 122, 64, 255), (18, 29): (196, 122, 64, 255), (19, 29): (196, 122, 64, 255), (20, 29): (196, 122, 64, 255), (21, 29): (196, 122, 64, 255), (22, 29): (196, 122, 64, 255), (25, 29): (196, 122, 64, 255), (26, 29): (196, 122, 64, 255), (27, 29): (196, 122, 64, 255), (28, 29): (196, 122, 64, 255), (29, 29): (196, 122, 64, 255), (30, 29): (196, 122, 64, 255), (17, 25): (148, 86, 42, 255), (18, 25): (148, 86, 42, 255), (19, 25): (148, 86, 42, 255), (20, 25): (148, 86, 42, 255), (21, 25): (148, 86, 42, 255), (22, 25): (148, 86, 42, 255), (25, 25): (148, 86, 42, 255), (26, 25): (148, 86, 42, 255), (27, 25): (148, 86, 42, 255), (28, 25): (148, 86, 42, 255), (29, 25): (148, 86, 42, 255), (30, 25): (148, 86, 42, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (23, 24): (34, 34, 34, 255), (24, 24): (34, 34, 34, 255), (19, 26): (34, 34, 34, 255), (28, 26): (34, 34, 34, 255), (17, 27): (34, 34, 34, 255), (18, 26): (34, 34, 34, 255), (30, 27): (34, 34, 34, 255), (29, 26): (34, 34, 34, 255), (16, 28): (34, 34, 34, 255), (31, 28): (34, 34, 34, 255), (16, 29): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (19, 30): (34, 34, 34, 255), (20, 30): (34, 34, 34, 255), (21, 30): (34, 34, 34, 255), (26, 30): (34, 34, 34, 255), (27, 30): (34, 34, 34, 255), (28, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (31, 29): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255), (16, 25): (34, 34, 34, 255), (17, 26): (34, 34, 34, 255), (17, 24): (34, 34, 34, 255), (18, 24): (34, 34, 34, 255), (19, 24): (34, 34, 34, 255), (20, 24): (34, 34, 34, 255), (21, 24): (34, 34, 34, 255), (22, 24): (34, 34, 34, 255), (25, 24): (34, 34, 34, 255), (26, 24): (34, 34, 34, 255), (27, 24): (34, 34, 34, 255), (28, 24): (34, 34, 34, 255), (29, 24): (34, 34, 34, 255), (31, 25): (34, 34, 34, 255), (30, 26): (34, 34, 34, 255), (30, 24): (34, 34, 34, 255)}


def test_axe_t2_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert axe_t2.axe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_axe_t2_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t2.axe_pixels(None, 0.0)


def test_axe_t2_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    axe_t2.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_axe_t2_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    axe_t2.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_axe_t2_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        axe_t2.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_axe_t2_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_axe_t2_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t2, "DWARF", sheet)
    assert axe_t2.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t2_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t2, "DWARF", sheet)
    assert axe_t2.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t2_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t2, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        axe_t2.dwarf_pixels({})


def test_axe_t2_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t2.fist_pixels((20.5, 27.5)) == {
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


def test_axe_t2_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t2.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_axe_t2_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t2.fist_pixels(None)


def test_draw_axe_t2_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = axe_t2.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_axe_t2_frames_differ_from_axe_t1_pixel_digests() -> None:
    assert [
        _image_digest(axe_t2.draw_frame(pose)) for pose in axe_t2.POSES
    ] != [
        _image_digest(axe.draw_frame(pose)) for pose in axe.POSES
    ]



def test_axe_t3_outline_around_wraps_a_single_pixel():
    assert axe_t3.outline_around({(1, 1): axe_t3.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_axe_t3_outline_around_of_an_empty_layer_stays_empty():
    assert axe_t3.outline_around({}) == {}


def test_axe_t3_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        axe_t3.outline_around(None)


def test_draw_axe_t3_frame_puts_the_axe_behind_the_dwarf_on_the_first_pose():
    frame = axe_t3.draw_frame(axe_t3.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "fee55701f442b44f875c3f992b27d1983bff77ffa568a48ce083293504d4e8b4"


def test_draw_axe_t3_frame_puts_the_axe_in_front_on_the_third_pose():
    frame = axe_t3.draw_frame(axe_t3.POSES[2])
    assert _image_digest(frame) == "1670adc135981f0520b0c913d64d397c71020360c1620030d225c216d1131855"
    assert _image_digest(frame) != "fee55701f442b44f875c3f992b27d1983bff77ffa568a48ce083293504d4e8b4"


def test_draw_axe_t3_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        axe_t3.draw_frame({})


def test_axe_t3_main_writes_four_chop_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t3, "FOLDER", folder)
    monkeypatch.setattr(
        axe_t3, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    axe_t3.main()
    expected = [
        "fee55701f442b44f875c3f992b27d1983bff77ffa568a48ce083293504d4e8b4",
        "3bfa2bf414737415d0082698b791a6a4e404c1da451cccea6e0ce5acaaeb2c32",
        "1670adc135981f0520b0c913d64d397c71020360c1620030d225c216d1131855",
        "8cd9b2abeff684dde2ef9be212794339c394d80a3f9bc8c470918b4de86ce6e2",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_axe_t3_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_axe_t3_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t3, "FOLDER", folder)
    monkeypatch.setattr(axe_t3, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        axe_t3.main()


def test_axe_t3_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert axe_t3.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_axe_t3_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert axe_t3.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_axe_t3_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert axe_t3.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_axe_t3_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        axe_t3.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_axe_t3_pixels_draws_an_iron_blade_from_the_grip() -> None:
    assert axe_t3.axe_pixels((24.0, 40.0), 0.0) == {(23, 25): (132, 140, 148, 255), (24, 25): (132, 140, 148, 255), (23, 26): (132, 140, 148, 255), (24, 26): (132, 140, 148, 255), (23, 27): (132, 140, 148, 255), (24, 27): (132, 140, 148, 255), (23, 28): (180, 188, 196, 255), (24, 28): (180, 188, 196, 255), (23, 29): (132, 140, 148, 255), (24, 29): (132, 140, 148, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (21, 25): (132, 140, 148, 255), (22, 25): (132, 140, 148, 255), (25, 25): (132, 140, 148, 255), (26, 25): (132, 140, 148, 255), (19, 26): (132, 140, 148, 255), (20, 26): (132, 140, 148, 255), (21, 26): (132, 140, 148, 255), (22, 26): (132, 140, 148, 255), (25, 26): (132, 140, 148, 255), (26, 26): (132, 140, 148, 255), (27, 26): (132, 140, 148, 255), (28, 26): (132, 140, 148, 255), (17, 27): (132, 140, 148, 255), (18, 27): (132, 140, 148, 255), (19, 27): (132, 140, 148, 255), (20, 27): (132, 140, 148, 255), (21, 27): (132, 140, 148, 255), (22, 27): (132, 140, 148, 255), (25, 27): (132, 140, 148, 255), (26, 27): (132, 140, 148, 255), (27, 27): (132, 140, 148, 255), (28, 27): (132, 140, 148, 255), (29, 27): (132, 140, 148, 255), (30, 27): (132, 140, 148, 255), (16, 28): (132, 140, 148, 255), (17, 28): (180, 188, 196, 255), (18, 28): (180, 188, 196, 255), (19, 28): (180, 188, 196, 255), (20, 28): (180, 188, 196, 255), (21, 28): (180, 188, 196, 255), (22, 28): (180, 188, 196, 255), (25, 28): (180, 188, 196, 255), (26, 28): (180, 188, 196, 255), (27, 28): (180, 188, 196, 255), (28, 28): (180, 188, 196, 255), (29, 28): (180, 188, 196, 255), (30, 28): (180, 188, 196, 255), (31, 28): (132, 140, 148, 255), (17, 29): (132, 140, 148, 255), (18, 29): (132, 140, 148, 255), (19, 29): (132, 140, 148, 255), (20, 29): (132, 140, 148, 255), (21, 29): (132, 140, 148, 255), (22, 29): (132, 140, 148, 255), (25, 29): (132, 140, 148, 255), (26, 29): (132, 140, 148, 255), (27, 29): (132, 140, 148, 255), (28, 29): (132, 140, 148, 255), (29, 29): (132, 140, 148, 255), (30, 29): (132, 140, 148, 255), (17, 24): (72, 78, 86, 255), (18, 24): (72, 78, 86, 255), (19, 24): (72, 78, 86, 255), (20, 24): (72, 78, 86, 255), (21, 24): (72, 78, 86, 255), (22, 24): (72, 78, 86, 255), (23, 24): (72, 78, 86, 255), (24, 24): (72, 78, 86, 255), (25, 24): (72, 78, 86, 255), (26, 24): (72, 78, 86, 255), (27, 24): (72, 78, 86, 255), (28, 24): (72, 78, 86, 255), (29, 24): (72, 78, 86, 255), (30, 24): (72, 78, 86, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (20, 25): (34, 34, 34, 255), (27, 25): (34, 34, 34, 255), (18, 26): (34, 34, 34, 255), (19, 25): (34, 34, 34, 255), (29, 26): (34, 34, 34, 255), (28, 25): (34, 34, 34, 255), (16, 27): (34, 34, 34, 255), (17, 26): (34, 34, 34, 255), (31, 27): (34, 34, 34, 255), (30, 26): (34, 34, 34, 255), (15, 28): (34, 34, 34, 255), (16, 29): (34, 34, 34, 255), (32, 28): (34, 34, 34, 255), (31, 29): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (19, 30): (34, 34, 34, 255), (20, 30): (34, 34, 34, 255), (21, 30): (34, 34, 34, 255), (26, 30): (34, 34, 34, 255), (27, 30): (34, 34, 34, 255), (28, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255), (16, 24): (34, 34, 34, 255), (17, 25): (34, 34, 34, 255), (17, 23): (34, 34, 34, 255), (18, 25): (34, 34, 34, 255), (18, 23): (34, 34, 34, 255), (19, 23): (34, 34, 34, 255), (20, 23): (34, 34, 34, 255), (21, 23): (34, 34, 34, 255), (22, 23): (34, 34, 34, 255), (23, 23): (34, 34, 34, 255), (24, 23): (34, 34, 34, 255), (25, 23): (34, 34, 34, 255), (26, 23): (34, 34, 34, 255), (27, 23): (34, 34, 34, 255), (28, 23): (34, 34, 34, 255), (29, 25): (34, 34, 34, 255), (29, 23): (34, 34, 34, 255), (31, 24): (34, 34, 34, 255), (30, 25): (34, 34, 34, 255), (30, 23): (34, 34, 34, 255)}


def test_axe_t3_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert axe_t3.axe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_axe_t3_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t3.axe_pixels(None, 0.0)


def test_axe_t3_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    axe_t3.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_axe_t3_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    axe_t3.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_axe_t3_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        axe_t3.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_axe_t3_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_axe_t3_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t3_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t3, "DWARF", sheet)
    assert axe_t3.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t3_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t3_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t3, "DWARF", sheet)
    assert axe_t3.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t3_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t3_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t3, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        axe_t3.dwarf_pixels({})


def test_axe_t3_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t3.fist_pixels((20.5, 27.5)) == {
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


def test_axe_t3_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t3.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_axe_t3_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t3.fist_pixels(None)


def test_draw_axe_t3_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = axe_t3.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_axe_t3_frames_differ_from_axe_t1_and_t2_pixel_digests() -> None:
    t3 = [_image_digest(axe_t3.draw_frame(pose)) for pose in axe_t3.POSES]
    assert t3 != [_image_digest(axe.draw_frame(pose)) for pose in axe.POSES]
    assert t3 != [_image_digest(axe_t2.draw_frame(pose)) for pose in axe_t2.POSES]



def test_axe_t4_outline_around_wraps_a_single_pixel():
    assert axe_t4.outline_around({(1, 1): axe_t4.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_axe_t4_outline_around_of_an_empty_layer_stays_empty():
    assert axe_t4.outline_around({}) == {}


def test_axe_t4_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        axe_t4.outline_around(None)


def test_draw_axe_t4_frame_puts_the_axe_behind_the_dwarf_on_the_first_pose():
    frame = axe_t4.draw_frame(axe_t4.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "e83876e5a5f2c77305404c951b76370a97de94e5588f540cdd7b84500dba6d4a"


def test_draw_axe_t4_frame_puts_the_axe_in_front_on_the_third_pose():
    frame = axe_t4.draw_frame(axe_t4.POSES[2])
    assert _image_digest(frame) == "d965d78edcabeda59b57777f7196c492e37e46fd34f4a76fb1e7ddd08b809649"
    assert _image_digest(frame) != "e83876e5a5f2c77305404c951b76370a97de94e5588f540cdd7b84500dba6d4a"


def test_draw_axe_t4_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        axe_t4.draw_frame({})


def test_axe_t4_main_writes_four_chop_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t4, "FOLDER", folder)
    monkeypatch.setattr(
        axe_t4, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    axe_t4.main()
    expected = [
        "e83876e5a5f2c77305404c951b76370a97de94e5588f540cdd7b84500dba6d4a",
        "9ee018e2bc3707060928ae4979c695e2aa16f11aaccf0d8f85a71fde1906e8a2",
        "d965d78edcabeda59b57777f7196c492e37e46fd34f4a76fb1e7ddd08b809649",
        "e671d85d9c2e45991be871b124da2eb02fe69e46abfcb7b56346ae42dba0b4b6",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_axe_t4_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_axe_t4_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t4, "FOLDER", folder)
    monkeypatch.setattr(axe_t4, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        axe_t4.main()


def test_axe_t4_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert axe_t4.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_axe_t4_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert axe_t4.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_axe_t4_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert axe_t4.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_axe_t4_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        axe_t4.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_axe_t4_pixels_draws_a_brass_blade_from_the_grip() -> None:
    assert axe_t4.axe_pixels((24.0, 40.0), 0.0) == {(23, 25): (204, 164, 72, 255), (24, 25): (204, 164, 72, 255), (23, 26): (204, 164, 72, 255), (24, 26): (204, 164, 72, 255), (23, 27): (119, 92, 85, 255), (24, 27): (119, 92, 85, 255), (23, 28): (236, 208, 120, 255), (24, 28): (236, 208, 120, 255), (23, 29): (204, 164, 72, 255), (24, 29): (204, 164, 72, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (21, 25): (204, 164, 72, 255), (22, 25): (204, 164, 72, 255), (25, 25): (204, 164, 72, 255), (26, 25): (204, 164, 72, 255), (19, 26): (204, 164, 72, 255), (20, 26): (204, 164, 72, 255), (21, 26): (204, 164, 72, 255), (22, 26): (204, 164, 72, 255), (25, 26): (204, 164, 72, 255), (26, 26): (204, 164, 72, 255), (27, 26): (204, 164, 72, 255), (28, 26): (204, 164, 72, 255), (17, 27): (204, 164, 72, 255), (18, 27): (204, 164, 72, 255), (19, 27): (204, 164, 72, 255), (20, 27): (204, 164, 72, 255), (21, 27): (204, 164, 72, 255), (26, 27): (204, 164, 72, 255), (27, 27): (204, 164, 72, 255), (28, 27): (204, 164, 72, 255), (29, 27): (204, 164, 72, 255), (30, 27): (204, 164, 72, 255), (16, 28): (236, 208, 120, 255), (17, 28): (236, 208, 120, 255), (18, 28): (236, 208, 120, 255), (19, 28): (236, 208, 120, 255), (20, 28): (236, 208, 120, 255), (21, 28): (236, 208, 120, 255), (22, 28): (236, 208, 120, 255), (25, 28): (236, 208, 120, 255), (26, 28): (236, 208, 120, 255), (27, 28): (236, 208, 120, 255), (28, 28): (236, 208, 120, 255), (29, 28): (236, 208, 120, 255), (30, 28): (236, 208, 120, 255), (31, 28): (236, 208, 120, 255), (16, 29): (204, 164, 72, 255), (17, 29): (204, 164, 72, 255), (18, 29): (204, 164, 72, 255), (19, 29): (204, 164, 72, 255), (20, 29): (204, 164, 72, 255), (21, 29): (204, 164, 72, 255), (22, 29): (204, 164, 72, 255), (25, 29): (204, 164, 72, 255), (26, 29): (204, 164, 72, 255), (27, 29): (204, 164, 72, 255), (28, 29): (204, 164, 72, 255), (29, 29): (204, 164, 72, 255), (30, 29): (204, 164, 72, 255), (31, 29): (204, 164, 72, 255), (16, 24): (148, 112, 36, 255), (17, 24): (148, 112, 36, 255), (18, 24): (148, 112, 36, 255), (19, 24): (148, 112, 36, 255), (20, 24): (148, 112, 36, 255), (21, 24): (148, 112, 36, 255), (22, 24): (148, 112, 36, 255), (23, 24): (148, 112, 36, 255), (24, 24): (148, 112, 36, 255), (25, 24): (148, 112, 36, 255), (26, 24): (148, 112, 36, 255), (27, 24): (148, 112, 36, 255), (28, 24): (148, 112, 36, 255), (29, 24): (148, 112, 36, 255), (30, 24): (148, 112, 36, 255), (31, 24): (148, 112, 36, 255), (22, 27): (34, 34, 34, 255), (25, 27): (34, 34, 34, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (20, 25): (34, 34, 34, 255), (27, 25): (34, 34, 34, 255), (18, 26): (34, 34, 34, 255), (19, 25): (34, 34, 34, 255), (29, 26): (34, 34, 34, 255), (28, 25): (34, 34, 34, 255), (16, 27): (34, 34, 34, 255), (17, 26): (34, 34, 34, 255), (31, 27): (34, 34, 34, 255), (30, 26): (34, 34, 34, 255), (15, 28): (34, 34, 34, 255), (32, 28): (34, 34, 34, 255), (15, 29): (34, 34, 34, 255), (16, 30): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (19, 30): (34, 34, 34, 255), (20, 30): (34, 34, 34, 255), (21, 30): (34, 34, 34, 255), (26, 30): (34, 34, 34, 255), (27, 30): (34, 34, 34, 255), (28, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255), (32, 29): (34, 34, 34, 255), (31, 30): (34, 34, 34, 255), (15, 24): (34, 34, 34, 255), (16, 25): (34, 34, 34, 255), (16, 23): (34, 34, 34, 255), (17, 25): (34, 34, 34, 255), (17, 23): (34, 34, 34, 255), (18, 25): (34, 34, 34, 255), (18, 23): (34, 34, 34, 255), (19, 23): (34, 34, 34, 255), (20, 23): (34, 34, 34, 255), (21, 23): (34, 34, 34, 255), (22, 23): (34, 34, 34, 255), (23, 23): (34, 34, 34, 255), (24, 23): (34, 34, 34, 255), (25, 23): (34, 34, 34, 255), (26, 23): (34, 34, 34, 255), (27, 23): (34, 34, 34, 255), (28, 23): (34, 34, 34, 255), (29, 25): (34, 34, 34, 255), (29, 23): (34, 34, 34, 255), (30, 25): (34, 34, 34, 255), (30, 23): (34, 34, 34, 255), (32, 24): (34, 34, 34, 255), (31, 25): (34, 34, 34, 255), (31, 23): (34, 34, 34, 255)}


def test_axe_t4_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert axe_t4.axe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_axe_t4_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t4.axe_pixels(None, 0.0)


def test_axe_t4_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    axe_t4.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_axe_t4_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    axe_t4.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_axe_t4_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        axe_t4.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_axe_t4_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_axe_t4_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t4_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t4, "DWARF", sheet)
    assert axe_t4.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t4_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t4_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t4, "DWARF", sheet)
    assert axe_t4.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t4_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t4_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t4, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        axe_t4.dwarf_pixels({})


def test_axe_t4_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t4.fist_pixels((20.5, 27.5)) == {
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


def test_axe_t4_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t4.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_axe_t4_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t4.fist_pixels(None)


def test_draw_axe_t4_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = axe_t4.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_axe_t4_frames_differ_from_lower_tier_pixel_digests() -> None:
    brass_frame_digests = [_image_digest(axe_t4.draw_frame(pose)) for pose in axe_t4.POSES]
    assert brass_frame_digests != [_image_digest(axe.draw_frame(pose)) for pose in axe.POSES]
    assert brass_frame_digests != [_image_digest(axe_t2.draw_frame(pose)) for pose in axe_t2.POSES]
    assert brass_frame_digests != [_image_digest(axe_t3.draw_frame(pose)) for pose in axe_t3.POSES]



def test_axe_t5_outline_around_wraps_a_single_pixel():
    assert axe_t5.outline_around({(1, 1): axe_t5.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_axe_t5_outline_around_of_an_empty_layer_stays_empty():
    assert axe_t5.outline_around({}) == {}


def test_axe_t5_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        axe_t5.outline_around(None)


def test_draw_axe_t5_frame_puts_the_axe_behind_the_dwarf_on_the_first_pose():
    frame = axe_t5.draw_frame(axe_t5.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "f819e01ae547bb67b6b5ccbbf68c621e10040ab0f41e7e6f110da89933c1effe"


def test_draw_axe_t5_frame_puts_the_axe_in_front_on_the_third_pose():
    frame = axe_t5.draw_frame(axe_t5.POSES[2])
    assert _image_digest(frame) == "fe78a0920e614e530311d86e874d35f38f9d1a68256c4a4b7895767c6bc1dc68"
    assert _image_digest(frame) != "f819e01ae547bb67b6b5ccbbf68c621e10040ab0f41e7e6f110da89933c1effe"


def test_draw_axe_t5_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        axe_t5.draw_frame({})


def test_axe_t5_main_writes_four_chop_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t5, "FOLDER", folder)
    monkeypatch.setattr(
        axe_t5, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    axe_t5.main()
    expected = [
        "f819e01ae547bb67b6b5ccbbf68c621e10040ab0f41e7e6f110da89933c1effe",
        "d08a867b5c145cc008894054c1f0cef7edeb559e2d2ad3e61939d8578969e5a4",
        "fe78a0920e614e530311d86e874d35f38f9d1a68256c4a4b7895767c6bc1dc68",
        "9dbb95687de6144bcf3b15b6debc85cd9771a8e46761928a80e44840fcfd8660",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_axe_t5_f{index}.png").convert("RGBA"))
        for index in range(4)
    ] == expected


def test_axe_t5_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_t5, "FOLDER", folder)
    monkeypatch.setattr(axe_t5, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        axe_t5.main()


def test_axe_t5_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert axe_t5.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_axe_t5_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert axe_t5.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_axe_t5_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert axe_t5.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_axe_t5_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        axe_t5.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_axe_t5_pixels_draws_an_aethersteel_blade_from_the_grip() -> None:
    assert axe_t5.axe_pixels((24.0, 40.0), 0.0) == {(23, 25): (148, 176, 212, 255), (24, 25): (148, 176, 212, 255), (23, 26): (119, 92, 85, 255), (24, 26): (119, 92, 85, 255), (23, 27): (148, 176, 212, 255), (24, 27): (148, 176, 212, 255), (23, 28): (210, 236, 255, 255), (24, 28): (210, 236, 255, 255), (23, 29): (148, 176, 212, 255), (24, 29): (148, 176, 212, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (22, 24): (72, 108, 168, 255), (23, 24): (72, 108, 168, 255), (24, 24): (72, 108, 168, 255), (25, 24): (72, 108, 168, 255), (20, 25): (148, 176, 212, 255), (21, 25): (148, 176, 212, 255), (22, 25): (148, 176, 212, 255), (25, 25): (148, 176, 212, 255), (26, 25): (148, 176, 212, 255), (27, 25): (148, 176, 212, 255), (18, 26): (148, 176, 212, 255), (19, 26): (148, 176, 212, 255), (20, 26): (148, 176, 212, 255), (21, 26): (148, 176, 212, 255), (22, 26): (148, 176, 212, 255), (25, 26): (148, 176, 212, 255), (26, 26): (148, 176, 212, 255), (27, 26): (148, 176, 212, 255), (28, 26): (148, 176, 212, 255), (29, 26): (148, 176, 212, 255), (16, 27): (148, 176, 212, 255), (17, 27): (148, 176, 212, 255), (18, 27): (148, 176, 212, 255), (19, 27): (148, 176, 212, 255), (20, 27): (148, 176, 212, 255), (21, 27): (148, 176, 212, 255), (22, 27): (148, 176, 212, 255), (25, 27): (148, 176, 212, 255), (26, 27): (148, 176, 212, 255), (27, 27): (148, 176, 212, 255), (28, 27): (148, 176, 212, 255), (29, 27): (148, 176, 212, 255), (30, 27): (148, 176, 212, 255), (31, 27): (148, 176, 212, 255), (15, 28): (148, 176, 212, 255), (16, 28): (210, 236, 255, 255), (17, 28): (210, 236, 255, 255), (18, 28): (210, 236, 255, 255), (19, 28): (210, 236, 255, 255), (20, 28): (210, 236, 255, 255), (21, 28): (210, 236, 255, 255), (22, 28): (210, 236, 255, 255), (25, 28): (210, 236, 255, 255), (26, 28): (210, 236, 255, 255), (27, 28): (210, 236, 255, 255), (28, 28): (210, 236, 255, 255), (29, 28): (210, 236, 255, 255), (30, 28): (210, 236, 255, 255), (31, 28): (210, 236, 255, 255), (32, 28): (148, 176, 212, 255), (16, 29): (148, 176, 212, 255), (17, 29): (148, 176, 212, 255), (18, 29): (148, 176, 212, 255), (19, 29): (148, 176, 212, 255), (20, 29): (148, 176, 212, 255), (21, 29): (148, 176, 212, 255), (22, 29): (148, 176, 212, 255), (25, 29): (148, 176, 212, 255), (26, 29): (148, 176, 212, 255), (27, 29): (148, 176, 212, 255), (28, 29): (148, 176, 212, 255), (29, 29): (148, 176, 212, 255), (30, 29): (148, 176, 212, 255), (31, 29): (148, 176, 212, 255), (16, 24): (72, 108, 168, 255), (17, 24): (72, 108, 168, 255), (18, 24): (72, 108, 168, 255), (19, 24): (72, 108, 168, 255), (20, 24): (72, 108, 168, 255), (21, 24): (72, 108, 168, 255), (26, 24): (72, 108, 168, 255), (27, 24): (72, 108, 168, 255), (28, 24): (72, 108, 168, 255), (29, 24): (72, 108, 168, 255), (30, 24): (72, 108, 168, 255), (31, 24): (72, 108, 168, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (22, 23): (34, 34, 34, 255), (23, 23): (34, 34, 34, 255), (24, 23): (34, 34, 34, 255), (25, 23): (34, 34, 34, 255), (19, 25): (34, 34, 34, 255), (28, 25): (34, 34, 34, 255), (17, 26): (34, 34, 34, 255), (18, 25): (34, 34, 34, 255), (30, 26): (34, 34, 34, 255), (29, 25): (34, 34, 34, 255), (15, 27): (34, 34, 34, 255), (16, 26): (34, 34, 34, 255), (32, 27): (34, 34, 34, 255), (31, 26): (34, 34, 34, 255), (14, 28): (34, 34, 34, 255), (15, 29): (34, 34, 34, 255), (33, 28): (34, 34, 34, 255), (32, 29): (34, 34, 34, 255), (16, 30): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (19, 30): (34, 34, 34, 255), (20, 30): (34, 34, 34, 255), (21, 30): (34, 34, 34, 255), (26, 30): (34, 34, 34, 255), (27, 30): (34, 34, 34, 255), (28, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255), (31, 30): (34, 34, 34, 255), (15, 24): (34, 34, 34, 255), (16, 25): (34, 34, 34, 255), (16, 23): (34, 34, 34, 255), (17, 25): (34, 34, 34, 255), (17, 23): (34, 34, 34, 255), (18, 23): (34, 34, 34, 255), (19, 23): (34, 34, 34, 255), (20, 23): (34, 34, 34, 255), (21, 23): (34, 34, 34, 255), (26, 23): (34, 34, 34, 255), (27, 23): (34, 34, 34, 255), (28, 23): (34, 34, 34, 255), (29, 23): (34, 34, 34, 255), (30, 25): (34, 34, 34, 255), (30, 23): (34, 34, 34, 255), (32, 24): (34, 34, 34, 255), (31, 25): (34, 34, 34, 255), (31, 23): (34, 34, 34, 255)}


def test_axe_t5_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert axe_t5.axe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_axe_t5_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t5.axe_pixels(None, 0.0)


def test_axe_t5_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    axe_t5.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_axe_t5_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    axe_t5.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_axe_t5_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        axe_t5.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_axe_t5_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_axe_t5_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t5_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t5, "DWARF", sheet)
    assert axe_t5.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t5_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t5_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t5, "DWARF", sheet)
    assert axe_t5.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_t5_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_t5_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_t5, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        axe_t5.dwarf_pixels({})


def test_axe_t5_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t5.fist_pixels((20.5, 27.5)) == {
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


def test_axe_t5_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_t5.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_axe_t5_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_t5.fist_pixels(None)


def test_draw_axe_t5_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = axe_t5.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


def test_axe_t5_frames_differ_from_lower_tier_pixel_digests() -> None:
    aethersteel_digests = [_image_digest(axe_t5.draw_frame(pose)) for pose in axe_t5.POSES]
    assert aethersteel_digests != [_image_digest(axe.draw_frame(pose)) for pose in axe.POSES]
    assert aethersteel_digests != [_image_digest(axe_t2.draw_frame(pose)) for pose in axe_t2.POSES]
    assert aethersteel_digests != [_image_digest(axe_t3.draw_frame(pose)) for pose in axe_t3.POSES]
    assert aethersteel_digests != [_image_digest(axe_t4.draw_frame(pose)) for pose in axe_t4.POSES]



def test_axe_charge_t2_outline_around_wraps_a_single_pixel():
    assert axe_charge_t2.outline_around({(1, 1): axe_charge_t2.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_axe_charge_t2_outline_around_of_an_empty_layer_stays_empty():
    assert axe_charge_t2.outline_around({}) == {}


def test_axe_charge_t2_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        axe_charge_t2.outline_around(None)


def test_draw_axe_charge_t2_frame_raises_the_axe_behind_the_dwarf_on_the_first_pose():
    frame = axe_charge_t2.draw_frame(axe_charge_t2.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "d56f0a9023d50e9a128b20692cc900c9c058f56f55c8e323865c59139419b8c4"


def test_draw_axe_charge_t2_frame_peaks_on_the_third_pose():
    frame = axe_charge_t2.draw_frame(axe_charge_t2.POSES[2])
    assert _image_digest(frame) == "823309aeff4fda2fc0232f6bf2b1ea645a1e040c85e2a42712764d6450bbfdf5"
    assert _image_digest(frame) != "d56f0a9023d50e9a128b20692cc900c9c058f56f55c8e323865c59139419b8c4"


def test_draw_axe_charge_t2_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        axe_charge_t2.draw_frame({})


def test_axe_charge_t2_main_writes_three_charge_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_charge_t2, "FOLDER", folder)
    monkeypatch.setattr(
        axe_charge_t2, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    axe_charge_t2.main()
    expected = [
        "d56f0a9023d50e9a128b20692cc900c9c058f56f55c8e323865c59139419b8c4",
        "02548923e83749e0de7486c5c40de73d69e45439a904bdd450254d26beb76ccb",
        "823309aeff4fda2fc0232f6bf2b1ea645a1e040c85e2a42712764d6450bbfdf5",
    ]
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_axe_charge_t2_f{index}.png").convert("RGBA"))
        for index in range(3)
    ] == expected


def test_axe_charge_t2_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(axe_charge_t2, "FOLDER", folder)
    monkeypatch.setattr(axe_charge_t2, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        axe_charge_t2.main()


def test_axe_charge_t2_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert axe_charge_t2.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_axe_charge_t2_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert axe_charge_t2.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_axe_charge_t2_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert axe_charge_t2.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_axe_charge_t2_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        axe_charge_t2.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_axe_charge_t2_pixels_draws_a_copper_blade_with_a_charge_pip() -> None:
    assert axe_charge_t2.axe_pixels((24.0, 40.0), 0.0) == {(23, 26): (196, 122, 64, 255), (24, 26): (196, 122, 64, 255), (23, 27): (119, 92, 85, 255), (24, 27): (119, 92, 85, 255), (23, 28): (220, 156, 96, 255), (24, 28): (220, 156, 96, 255), (23, 29): (196, 122, 64, 255), (24, 29): (196, 122, 64, 255), (23, 30): (119, 92, 85, 255), (24, 30): (119, 92, 85, 255), (23, 31): (119, 92, 85, 255), (24, 31): (119, 92, 85, 255), (23, 32): (119, 92, 85, 255), (24, 32): (119, 92, 85, 255), (23, 33): (119, 92, 85, 255), (24, 33): (119, 92, 85, 255), (23, 34): (119, 92, 85, 255), (24, 34): (119, 92, 85, 255), (23, 35): (119, 92, 85, 255), (24, 35): (119, 92, 85, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (23, 25): (148, 86, 42, 255), (24, 25): (148, 86, 42, 255), (20, 26): (196, 122, 64, 255), (21, 26): (196, 122, 64, 255), (22, 26): (196, 122, 64, 255), (25, 26): (196, 122, 64, 255), (26, 26): (196, 122, 64, 255), (27, 26): (196, 122, 64, 255), (18, 27): (196, 122, 64, 255), (19, 27): (196, 122, 64, 255), (20, 27): (196, 122, 64, 255), (21, 27): (196, 122, 64, 255), (22, 27): (196, 122, 64, 255), (25, 27): (196, 122, 64, 255), (26, 27): (196, 122, 64, 255), (27, 27): (196, 122, 64, 255), (28, 27): (196, 122, 64, 255), (29, 27): (196, 122, 64, 255), (17, 28): (220, 156, 96, 255), (18, 28): (220, 156, 96, 255), (19, 28): (220, 156, 96, 255), (20, 28): (220, 156, 96, 255), (21, 28): (220, 156, 96, 255), (22, 28): (220, 156, 96, 255), (25, 28): (220, 156, 96, 255), (26, 28): (220, 156, 96, 255), (27, 28): (220, 156, 96, 255), (28, 28): (220, 156, 96, 255), (29, 28): (220, 156, 96, 255), (30, 28): (220, 156, 96, 255), (17, 29): (196, 122, 64, 255), (18, 29): (196, 122, 64, 255), (19, 29): (196, 122, 64, 255), (20, 29): (196, 122, 64, 255), (21, 29): (196, 122, 64, 255), (22, 29): (196, 122, 64, 255), (25, 29): (196, 122, 64, 255), (26, 29): (196, 122, 64, 255), (27, 29): (196, 122, 64, 255), (28, 29): (196, 122, 64, 255), (29, 29): (196, 122, 64, 255), (30, 29): (196, 122, 64, 255), (17, 25): (148, 86, 42, 255), (18, 25): (148, 86, 42, 255), (19, 25): (148, 86, 42, 255), (20, 25): (148, 86, 42, 255), (21, 25): (148, 86, 42, 255), (22, 25): (148, 86, 42, 255), (25, 25): (148, 86, 42, 255), (26, 25): (148, 86, 42, 255), (27, 25): (148, 86, 42, 255), (28, 25): (148, 86, 42, 255), (29, 25): (148, 86, 42, 255), (30, 25): (148, 86, 42, 255), (24, 24): (220, 156, 96, 255), (24, 23): (148, 86, 42, 255), (22, 30): (34, 34, 34, 255), (25, 30): (34, 34, 34, 255), (22, 31): (34, 34, 34, 255), (25, 31): (34, 34, 34, 255), (22, 32): (34, 34, 34, 255), (25, 32): (34, 34, 34, 255), (22, 33): (34, 34, 34, 255), (25, 33): (34, 34, 34, 255), (22, 34): (34, 34, 34, 255), (25, 34): (34, 34, 34, 255), (22, 35): (34, 34, 34, 255), (25, 35): (34, 34, 34, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (23, 24): (34, 34, 34, 255), (19, 26): (34, 34, 34, 255), (28, 26): (34, 34, 34, 255), (17, 27): (34, 34, 34, 255), (18, 26): (34, 34, 34, 255), (30, 27): (34, 34, 34, 255), (29, 26): (34, 34, 34, 255), (16, 28): (34, 34, 34, 255), (31, 28): (34, 34, 34, 255), (16, 29): (34, 34, 34, 255), (17, 30): (34, 34, 34, 255), (18, 30): (34, 34, 34, 255), (19, 30): (34, 34, 34, 255), (20, 30): (34, 34, 34, 255), (21, 30): (34, 34, 34, 255), (26, 30): (34, 34, 34, 255), (27, 30): (34, 34, 34, 255), (28, 30): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (31, 29): (34, 34, 34, 255), (30, 30): (34, 34, 34, 255), (16, 25): (34, 34, 34, 255), (17, 26): (34, 34, 34, 255), (17, 24): (34, 34, 34, 255), (18, 24): (34, 34, 34, 255), (19, 24): (34, 34, 34, 255), (20, 24): (34, 34, 34, 255), (21, 24): (34, 34, 34, 255), (22, 24): (34, 34, 34, 255), (25, 24): (34, 34, 34, 255), (26, 24): (34, 34, 34, 255), (27, 24): (34, 34, 34, 255), (28, 24): (34, 34, 34, 255), (29, 24): (34, 34, 34, 255), (31, 25): (34, 34, 34, 255), (30, 26): (34, 34, 34, 255), (30, 24): (34, 34, 34, 255), (25, 23): (34, 34, 34, 255), (23, 23): (34, 34, 34, 255), (24, 22): (34, 34, 34, 255)}


def test_axe_charge_t2_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert axe_charge_t2.axe_pixels((1.0, 1.0), 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (1, -15): (220, 156, 96, 255), (1, -16): (148, 86, 42, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_axe_charge_t2_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_charge_t2.axe_pixels(None, 0.0)


def test_axe_charge_t2_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    axe_charge_t2.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_axe_charge_t2_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    axe_charge_t2.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_axe_charge_t2_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        axe_charge_t2.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_axe_charge_t2_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_axe_charge_t2_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_charge_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_charge_t2, "DWARF", sheet)
    assert axe_charge_t2.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_charge_t2_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_charge_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_charge_t2, "DWARF", sheet)
    assert axe_charge_t2.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_axe_charge_t2_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_axe_charge_t2_dwarf_sheet(sheet)
    monkeypatch.setattr(axe_charge_t2, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        axe_charge_t2.dwarf_pixels({})


def test_axe_charge_t2_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_charge_t2.fist_pixels((20.5, 27.5)) == {
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


def test_axe_charge_t2_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert axe_charge_t2.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_axe_charge_t2_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        axe_charge_t2.fist_pixels(None)


def test_draw_axe_charge_t2_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = axe_charge_t2.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "a4ae4a03ca49a44e452066867860264a9ed97b1b3a48563e4234eec5205c646f"


def test_axe_charge_t2_frames_differ_from_axe_t2_chop_pixel_digests() -> None:
    assert [
        _image_digest(axe_charge_t2.draw_frame(pose)) for pose in axe_charge_t2.POSES
    ] != [
        _image_digest(axe_t2.draw_frame(pose)) for pose in axe_t2.POSES[:3]
    ]



def test_water_empty_outline_around_wraps_a_single_pixel():
    assert water_empty.outline_around({(1, 1): water_empty.WOOD}) == {
        (1, 1): (119, 92, 85, 255),
        (2, 1): (34, 34, 34, 255),
        (0, 1): (34, 34, 34, 255),
        (1, 2): (34, 34, 34, 255),
        (1, 0): (34, 34, 34, 255),
    }


def test_water_empty_outline_around_of_an_empty_layer_stays_empty():
    assert water_empty.outline_around({}) == {}


def test_water_empty_outline_around_rejects_a_missing_layer():
    with pytest.raises(TypeError):
        water_empty.outline_around(None)


def test_draw_water_empty_frame_shakes_left_on_the_first_pose():
    frame = water_empty.draw_frame(water_empty.POSES[0])
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0)) == (0, 0, 0, 0)
    assert _image_digest(frame) == "434fa1833b6cde62a529e77515fe585e2f9e546a57cbab4ac8591ebde46f3dba"


def test_draw_water_empty_frame_shakes_right_on_the_second_pose():
    frame = water_empty.draw_frame(water_empty.POSES[1])
    assert _image_digest(frame) == "23c0a445ffc2ee66375560599221414ffe494dc232e347a45082170d6e0cc1f5"
    assert _image_digest(frame) != "434fa1833b6cde62a529e77515fe585e2f9e546a57cbab4ac8591ebde46f3dba"


def test_draw_water_empty_frame_rejects_a_pose_without_a_grip():
    with pytest.raises(KeyError, match="grip"):
        water_empty.draw_frame({})


def test_water_empty_main_writes_three_shake_frames_with_the_locked_pixel_digests(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(water_empty, "FOLDER", folder)
    monkeypatch.setattr(
        water_empty, "DWARF", ROOT / "assets" / "player" / "non-combat" / "dwarf_m_idle_anim_f0.png"
    )
    water_empty.main()
    expected = ['434fa1833b6cde62a529e77515fe585e2f9e546a57cbab4ac8591ebde46f3dba', '23c0a445ffc2ee66375560599221414ffe494dc232e347a45082170d6e0cc1f5', '43250f786b1102db028d061b0693303208edb2be97e0a60d24ac39f75f1833e6']
    assert [
        _image_digest(Image.open(folder / f"dwarf_m_water_empty_f{index}.png").convert("RGBA"))
        for index in range(3)
    ] == expected


def test_water_empty_main_rejects_a_missing_dwarf_sheet(tmp_path, monkeypatch):
    folder = tmp_path / "frames"
    folder.mkdir()
    monkeypatch.setattr(water_empty, "FOLDER", folder)
    monkeypatch.setattr(water_empty, "DWARF", folder / "missing.png")
    with pytest.raises(FileNotFoundError):
        water_empty.main()


def test_water_empty_distance_to_segment_is_zero_on_the_segment_end() -> None:
    assert water_empty.distance_to_segment(3, 4, 0, 0, 3, 4) == 0


def test_water_empty_distance_to_segment_measures_a_point_beside_a_horizontal_segment() -> None:
    assert water_empty.distance_to_segment(1, 2, 0, 0, 2, 0) == 2


def test_water_empty_distance_to_segment_of_a_zero_length_segment_is_the_point_distance() -> None:
    assert water_empty.distance_to_segment(5, 5, 2, 2, 2, 2) == (3**2 + 3**2) ** 0.5


def test_water_empty_distance_to_segment_rejects_a_missing_coordinate() -> None:
    with pytest.raises(TypeError):
        water_empty.distance_to_segment(None, 0, 0, 0, 1, 0)


def test_water_empty_can_pixels_draws_a_tin_can_without_a_stream() -> None:
    assert water_empty.can_pixels((24.0, 40.0), 0.0, 0.0) == {(23, 32): (96, 100, 106, 255), (24, 32): (96, 100, 106, 255), (23, 33): (96, 100, 106, 255), (24, 33): (96, 100, 106, 255), (23, 34): (96, 100, 106, 255), (24, 34): (96, 100, 106, 255), (23, 35): (96, 100, 106, 255), (24, 35): (96, 100, 106, 255), (23, 36): (119, 92, 85, 255), (24, 36): (119, 92, 85, 255), (23, 37): (119, 92, 85, 255), (24, 37): (119, 92, 85, 255), (23, 38): (119, 92, 85, 255), (24, 38): (119, 92, 85, 255), (23, 39): (119, 92, 85, 255), (24, 39): (119, 92, 85, 255), (24, 31): (96, 100, 106, 255), (25, 31): (148, 152, 156, 255), (26, 31): (148, 152, 156, 255), (27, 31): (148, 152, 156, 255), (28, 31): (148, 152, 156, 255), (29, 31): (148, 152, 156, 255), (25, 32): (148, 152, 156, 255), (26, 32): (148, 152, 156, 255), (27, 32): (148, 152, 156, 255), (28, 32): (148, 152, 156, 255), (29, 32): (148, 152, 156, 255), (22, 28): (148, 152, 156, 255), (23, 28): (148, 152, 156, 255), (22, 29): (148, 152, 156, 255), (23, 29): (148, 152, 156, 255), (22, 30): (148, 152, 156, 255), (23, 30): (148, 152, 156, 255), (22, 31): (148, 152, 156, 255), (23, 31): (96, 100, 106, 255), (22, 32): (148, 152, 156, 255), (22, 33): (148, 152, 156, 255), (22, 34): (148, 152, 156, 255), (22, 35): (148, 152, 156, 255), (24, 28): (148, 152, 156, 255), (24, 29): (148, 152, 156, 255), (24, 30): (148, 152, 156, 255), (25, 28): (148, 152, 156, 255), (25, 29): (148, 152, 156, 255), (25, 30): (148, 152, 156, 255), (25, 33): (148, 152, 156, 255), (25, 34): (148, 152, 156, 255), (25, 35): (148, 152, 156, 255), (22, 36): (34, 34, 34, 255), (25, 36): (34, 34, 34, 255), (22, 37): (34, 34, 34, 255), (25, 37): (34, 34, 34, 255), (22, 38): (34, 34, 34, 255), (25, 38): (34, 34, 34, 255), (22, 39): (34, 34, 34, 255), (23, 40): (34, 34, 34, 255), (25, 39): (34, 34, 34, 255), (24, 40): (34, 34, 34, 255), (26, 30): (34, 34, 34, 255), (27, 30): (34, 34, 34, 255), (28, 30): (34, 34, 34, 255), (30, 31): (34, 34, 34, 255), (29, 30): (34, 34, 34, 255), (26, 33): (34, 34, 34, 255), (27, 33): (34, 34, 34, 255), (28, 33): (34, 34, 34, 255), (30, 32): (34, 34, 34, 255), (29, 33): (34, 34, 34, 255), (21, 28): (34, 34, 34, 255), (22, 27): (34, 34, 34, 255), (23, 27): (34, 34, 34, 255), (21, 29): (34, 34, 34, 255), (21, 30): (34, 34, 34, 255), (21, 31): (34, 34, 34, 255), (21, 32): (34, 34, 34, 255), (21, 33): (34, 34, 34, 255), (21, 34): (34, 34, 34, 255), (21, 35): (34, 34, 34, 255), (24, 27): (34, 34, 34, 255), (26, 28): (34, 34, 34, 255), (25, 27): (34, 34, 34, 255), (26, 29): (34, 34, 34, 255), (26, 34): (34, 34, 34, 255), (26, 35): (34, 34, 34, 255)}


def test_water_empty_can_pixels_clips_an_upward_handle_to_the_canvas() -> None:
    assert water_empty.can_pixels((1.0, 1.0), 0.0, 0.0) == {(0, 0): (119, 92, 85, 255), (1, 0): (119, 92, 85, 255), (0, 1): (34, 34, 34, 255), (2, 0): (34, 34, 34, 255), (1, 1): (34, 34, 34, 255)}


def test_water_empty_can_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        water_empty.can_pixels(None, 0.0, 0.0)


def test_water_empty_stroke_paints_pixels_whose_centers_fall_inside_the_radius() -> None:
    layer = {}
    water_empty.stroke(layer, [((1, 1), (3, 1))], 0.5, RED)
    assert layer == {(1, 0): RED, (1, 1): RED, (2, 0): RED, (2, 1): RED}


def test_water_empty_stroke_with_a_zero_radius_paints_centers_on_the_segment() -> None:
    layer = {}
    water_empty.stroke(layer, [((0.5, 0.5), (2.5, 0.5))], 0.0, BLUE)
    assert layer == {(0, 0): BLUE, (1, 0): BLUE, (2, 0): BLUE}


def test_water_empty_stroke_rejects_a_missing_layer() -> None:
    with pytest.raises(TypeError):
        water_empty.stroke(None, [((1, 1), (3, 1))], 0.5, RED)


def _write_water_empty_dwarf_sheet(path: Path) -> None:
    image = Image.new("RGBA", (2, 23), (0, 0, 0, 0))
    image.putpixel((0, 21), BLUE)
    image.putpixel((0, 22), RED)
    image.putpixel((1, 22), GREEN)
    image.save(path)


def test_water_empty_dwarf_pixels_moves_the_head_while_the_body_stays_put(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_water_empty_dwarf_sheet(sheet)
    monkeypatch.setattr(water_empty, "DWARF", sheet)
    assert water_empty.dwarf_pixels({"lean_x": 3, "drop_y": -1}) == {
        (19, 36): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_water_empty_dwarf_pixels_omits_a_fully_transparent_pixel(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_water_empty_dwarf_sheet(sheet)
    monkeypatch.setattr(water_empty, "DWARF", sheet)
    assert water_empty.dwarf_pixels({"lean_x": 0, "drop_y": 0}) == {
        (16, 37): BLUE,
        (16, 38): RED,
        (17, 38): GREEN,
    }


def test_water_empty_dwarf_pixels_rejects_a_pose_without_a_lean(tmp_path, monkeypatch) -> None:
    sheet = tmp_path / "dwarf.png"
    _write_water_empty_dwarf_sheet(sheet)
    monkeypatch.setattr(water_empty, "DWARF", sheet)
    with pytest.raises(KeyError, match="lean_x"):
        water_empty.dwarf_pixels({})


def test_water_empty_fist_pixels_paints_a_skin_block_around_the_grip() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert water_empty.fist_pixels((20.5, 27.5)) == {
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


def test_water_empty_fist_pixels_keeps_skin_past_the_left_edge() -> None:
    skin = (216, 165, 125, 255)
    outline = (34, 34, 34, 255)
    assert water_empty.fist_pixels((0.2, 0.2)) == {
        (-1, 0): skin,
        (-1, 1): skin,
        (0, 0): skin,
        (0, 1): skin,
        (0, 2): outline,
        (1, 0): outline,
        (1, 1): outline,
    }


def test_water_empty_fist_pixels_rejects_a_missing_grip() -> None:
    with pytest.raises(TypeError):
        water_empty.fist_pixels(None)


def test_draw_water_empty_frame_with_a_corner_grip_stays_on_the_canvas() -> None:
    frame = water_empty.draw_frame(
        {"lean_x": 0, "drop_y": 0, "grip": (1.0, 1.0), "angle": 0.0, "stream": 0.0, "in_front": False}
    )
    assert frame.size == (48, 60)
    assert frame.getpixel((0, 0))[3] > 0
    assert _image_digest(frame) == "b4db7bfb2738e61555739b7e0b2d1bacfc291d0df25239d13c67a48be2b89309"


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
    assert isinstance(parts, hunter.RunParts)
    assert parts.chest == 93.5
    assert isinstance(parts.body, hunter.Sprite)
    assert isinstance(parts.rear, hunter.Sprite)
    assert isinstance(parts.front, hunter.Sprite)
    assert isinstance(parts.sword, hunter.Sprite)


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


def test_compose_run_frame_rejects_a_pose_that_is_not_a_run_pose(idle_cell) -> None:
    parts = hunter.build_run_parts(idle_cell)
    with pytest.raises(AttributeError, match="bob"):
        hunter.compose_run_frame(parts, {})


def test_sheet_image_packs_one_sprite_into_the_hunter_cell() -> None:
    image = hunter.sheet_image([_long_sole()])
    assert image.size == (hunter.CELL_WIDTH, hunter.CELL_HEIGHT)
    assert image.getpixel((0, 8)) == RED


def test_sheet_image_of_no_frames_has_zero_width() -> None:
    assert hunter.sheet_image([]).size == (0, hunter.CELL_HEIGHT)


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
    monkeypatch.setattr(hunter, "OUTPUT_DIRECTORY", folder)
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
    assert canvas.read(15, 3) == icons.STEEL_LIGHT


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
    icons.recolor_rim(canvas, icons.BRASS_LIGHT, icons.BRASS, icons.BRASS_DARK)
    assert all(pixel == (0, 0, 0, 0) for pixel in canvas.pixels)


def test_recolor_rim_lights_the_top_of_a_single_pixel() -> None:
    canvas = icons.new_canvas()
    icons.put(canvas, 4, 4, icons.STEEL)
    icons.recolor_rim(canvas, icons.BRASS_LIGHT, icons.BRASS, icons.BRASS_DARK)
    assert canvas.read(4, 4) == icons.BRASS_LIGHT


def test_recolor_rim_rejects_a_canvas_that_is_not_a_canvas() -> None:
    with pytest.raises(TypeError):
        icons.recolor_rim(None, icons.BRASS_LIGHT, icons.BRASS, icons.BRASS_DARK)


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
    monkeypatch.setattr(icons, "OUTPUT_DIRECTORY", folder)
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
    monkeypatch.setattr(icons, "OUTPUT_DIRECTORY", folder)
    monkeypatch.setattr(icons.sys, "argv", ["draw_skill_icons.py", "--check"])
    with pytest.raises(SystemExit, match="missing"):
        icons.main()
