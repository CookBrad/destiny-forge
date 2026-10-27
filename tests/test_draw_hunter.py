from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_hunter_frames as hunter
from art_common import clipping_canvas

RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
INK = (0, 0, 0, 255)
ROOT = Path(__file__).resolve().parents[1]


def _sprite_digest(sprite: hunter.Sprite) -> str:
    payload = repr(tuple(sorted(sprite.pixels.items()))).encode()
    return hashlib.sha256(payload).hexdigest()


@pytest.fixture(scope="module")
def idle_cell() -> hunter.Sprite:
    return hunter.cell_of(hunter.load_sheet("knight_idle_side.png"), 0)


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


@pytest.fixture(scope="module")
def attack_cells() -> tuple[hunter.Sprite, hunter.Sprite]:
    sheet = hunter.load_sheet("knight_attack_side.png")
    return hunter.cell_of(sheet, 0), hunter.cell_of(sheet, 2)


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
