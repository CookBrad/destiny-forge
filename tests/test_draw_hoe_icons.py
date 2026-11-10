from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

import draw_hoe_icons as hoe_icons
from art_common import rejecting_canvas

ROOT = Path(__file__).resolve().parents[1]
ICON_PIXEL_DIGESTS = (
    "52249a22cc52f0fd06d01418e33aa265640ad378f000050f07c7960148a89f23",
    "c48eeec05faedb0c4959b3b88fc7bcfb3d38956d59021fabaa186f9fba598aa6",
    "6bc0b62e2f71eee8b1cd755b2e6befc7d1d2d9006038be86ce5f31436e811aab",
    "79820f38444dcc8ab8bb4ec37af829f4dc25b28ab85dd134ad1f151bd9460916",
    "d520071e2d09887cb20f95af9360206955fdd3875ae838c55158bdf02cbd5b85",
)
HANDLE_PIXEL_DIGEST = "86a3dd0062dac885a0de607a05a27ab06e39081090f21c9174524dc2486ece89"
BLADE_PIXEL_DIGESTS = (
    "2ec8156561e82de88ba35c86f866a98eebcd5a02c985ca653ceebc74ea417b25",
    "71153f3200c99c2e7514db9e09ba2acb57b3d215d909fc8498275110e459c5e8",
    "1201bb25053b5cda4f6d70f702b3fff70a6b5c5890ba2d9a8f914abb202c54b6",
    "08bd0c3b311fcdef2a7bfb60a8ad9dba22b7edb9c5af6ec28d2696776ca0413b",
    "dfd0046d79c36c515260b17997cfd06fb380f3fbbde8fae229ec8a1e3e1cbbf2",
)
WOOD_PIXELS = (
    (8, 5), (7, 5),
    (7, 6), (6, 6),
    (6, 7), (5, 7),
    (5, 8), (4, 8),
    (4, 9), (3, 9),
    (3, 10), (2, 10),
    (2, 11), (3, 11),
    (2, 12), (3, 12), (4, 12),
    (3, 13), (4, 13),
)
TIER_ONE_BLADE = (
    (7, 2), (8, 2), (9, 2), (10, 2), (11, 2),
    (7, 3), (8, 3), (9, 3), (10, 3), (11, 3),
    (7, 4), (8, 4), (9, 4), (10, 4), (11, 4),
    (7, 5), (8, 5),
)
TIER_TWO_BLADE = (
    (7, 2), (8, 2), (9, 2), (10, 2), (11, 2), (12, 2),
    (7, 3), (8, 3), (9, 3), (10, 3), (11, 3), (12, 3),
    (7, 4), (8, 4), (9, 4), (10, 4), (11, 4), (12, 4),
    (7, 5), (8, 5),
)
TIER_FIVE_BLADE = tuple(
    (column, row)
    for row in range(2, 6)
    for column in range(5, 14)
)
CLEAR = (0, 0, 0, 0)
WOOD = (119, 92, 85, 255)
OUTLINE = (34, 34, 34, 255)
RUST = (138, 96, 62, 255)
RUST_EDGE = (98, 68, 44, 255)
COPPER = (196, 122, 64, 255)
COPPER_EDGE = (148, 86, 42, 255)
COPPER_HIGHLIGHT = (220, 156, 96, 255)
IRON = (132, 140, 148, 255)
IRON_EDGE = (72, 78, 86, 255)
IRON_HIGHLIGHT = (180, 188, 196, 255)
BRASS = (204, 164, 72, 255)
BRASS_EDGE = (148, 112, 36, 255)
BRASS_HIGHLIGHT = (236, 208, 120, 255)
AETHERSTEEL = (148, 176, 212, 255)
AETHERSTEEL_EDGE = (72, 108, 168, 255)
AETHERSTEEL_GLOW = (210, 236, 255, 255)
BAKED_COLORS = {
    1: {CLEAR, OUTLINE, WOOD, RUST, RUST_EDGE},
    2: {CLEAR, OUTLINE, WOOD, COPPER, COPPER_EDGE, COPPER_HIGHLIGHT},
    3: {CLEAR, OUTLINE, WOOD, IRON, IRON_EDGE, IRON_HIGHLIGHT},
    4: {CLEAR, OUTLINE, WOOD, BRASS, BRASS_EDGE, BRASS_HIGHLIGHT},
    5: {CLEAR, OUTLINE, WOOD, AETHERSTEEL, AETHERSTEEL_EDGE, AETHERSTEEL_GLOW},
}


def _image_digest(image: Image.Image) -> str:
    return hashlib.sha256(image.tobytes()).hexdigest()


def _opaque(canvas) -> dict:
    found = {}
    for row in range(canvas.height):
        for column in range(canvas.width):
            color = canvas.read(column, row)
            if color is not None:
                found[(column, row)] = color
    return found


def _colors(image: Image.Image) -> set:
    width, height = image.size
    return {image.getpixel((column, row)) for row in range(height) for column in range(width)}


def test_main_writes_five_hoe_icons_with_the_locked_pixel_digests(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "FOLDER", tmp_path)
    hoe_icons.main()
    digests = [
        _image_digest(Image.open(tmp_path / f"hoe_t{tier}.png").convert("RGBA"))
        for tier in hoe_icons.HOE_TIERS
    ]
    assert digests == list(ICON_PIXEL_DIGESTS)
    assert len(set(digests)) == 5
    assert sorted(path.name for path in tmp_path.iterdir()) == [
        "hoe_t1.png",
        "hoe_t2.png",
        "hoe_t3.png",
        "hoe_t4.png",
        "hoe_t5.png",
    ]


def test_main_with_no_hoe_tiers_writes_nothing(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "FOLDER", tmp_path)
    monkeypatch.setattr(hoe_icons, "HOE_TIERS", ())
    hoe_icons.main()
    assert list(tmp_path.iterdir()) == []


def test_main_rejects_a_folder_path_that_is_a_file(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "tools"
    folder.write_text("not a folder")
    monkeypatch.setattr(hoe_icons, "FOLDER", folder)
    with pytest.raises(FileExistsError):
        hoe_icons.main()


def test_main_rejects_duplicate_hoe_icons_before_writing(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "FOLDER", tmp_path)
    icon = hoe_icons.draw_hoe_icon(1)
    monkeypatch.setattr(hoe_icons, "hoe_icon_records", lambda: [(1, icon), (2, icon)])
    with pytest.raises(ValueError, match="hoe icons are not unique"):
        hoe_icons.main()
    assert list(tmp_path.iterdir()) == []


def test_hoe_icon_records_returns_five_tiers_with_the_locked_pixel_digests() -> None:
    records = hoe_icons.hoe_icon_records()
    assert [tier for tier, _image in records] == [1, 2, 3, 4, 5]
    assert [_image_digest(image) for _tier, image in records] == list(ICON_PIXEL_DIGESTS)


def test_hoe_icon_records_with_no_tiers_returns_nothing(monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "HOE_TIERS", ())
    assert hoe_icons.hoe_icon_records() == []


def test_hoe_icon_records_rejects_a_tier_outside_the_hoe_set(monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "HOE_TIERS", (1, 9))
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.hoe_icon_records()


def test_hoe_icon_record_returns_the_tier_and_the_iron_icon() -> None:
    tier, image = hoe_icons.hoe_icon_record(3)
    assert tier == 3
    assert image.size == (16, 16)
    assert _image_digest(image) == ICON_PIXEL_DIGESTS[2]


def test_hoe_icon_record_spans_the_wood_hoe_and_the_aethersteel_hoe() -> None:
    first_tier, first_image = hoe_icons.hoe_icon_record(1)
    last_tier, last_image = hoe_icons.hoe_icon_record(5)
    assert (first_tier, last_tier) == (1, 5)
    assert _image_digest(first_image) == ICON_PIXEL_DIGESTS[0]
    assert _image_digest(last_image) == ICON_PIXEL_DIGESTS[4]
    assert _image_digest(first_image) != _image_digest(last_image)


def test_hoe_icon_record_rejects_a_missing_tier() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.hoe_icon_record(None)


def test_require_hoe_tier_returns_a_middle_tier() -> None:
    assert hoe_icons.require_hoe_tier(2) == 2
    assert hoe_icons.require_hoe_tier(3) == 3
    assert hoe_icons.require_hoe_tier(4) == 4


def test_require_hoe_tier_accepts_the_first_and_last_tiers() -> None:
    assert hoe_icons.require_hoe_tier(1) == 1
    assert hoe_icons.require_hoe_tier(5) == 5


def test_require_hoe_tier_rejects_a_missing_tier() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.require_hoe_tier(None)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.require_hoe_tier(True)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.require_hoe_tier(1.0)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.require_hoe_tier(0)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.require_hoe_tier(6)


def test_draw_hoe_icon_paints_the_locked_iron_hoe() -> None:
    icon = hoe_icons.draw_hoe_icon(3)
    assert icon.size == (16, 16)
    assert icon.mode == "RGBA"
    assert icon.getpixel((0, 0)) == CLEAR
    assert icon.getpixel((15, 15)) == CLEAR
    assert icon.getpixel((6, 2)) == IRON_HIGHLIGHT
    assert icon.getpixel((8, 3)) == IRON
    assert icon.getpixel((12, 4)) == IRON_EDGE
    assert icon.getpixel((7, 5)) == IRON
    assert icon.getpixel((6, 6)) == WOOD
    assert icon.getpixel((6, 1)) == OUTLINE
    assert _colors(icon) == BAKED_COLORS[3]
    assert _image_digest(icon) == ICON_PIXEL_DIGESTS[2]


def test_draw_hoe_icon_keeps_the_short_rust_blade_and_the_tall_aethersteel_blade() -> None:
    first = hoe_icons.draw_hoe_icon(1)
    last = hoe_icons.draw_hoe_icon(5)
    assert first.size == (16, 16)
    assert last.size == (16, 16)
    assert first.getpixel((11, 2)) == RUST
    assert first.getpixel((12, 2)) == OUTLINE
    assert first.getpixel((7, 5)) == RUST
    assert first.getpixel((7, 5)) != WOOD
    assert last.getpixel((5, 2)) == AETHERSTEEL_GLOW
    assert last.getpixel((13, 5)) == AETHERSTEEL_EDGE
    assert last.getpixel((7, 5)) != WOOD
    assert first.getpixel((0, 0)) == CLEAR
    assert last.getpixel((15, 0)) == CLEAR
    assert _colors(first) == BAKED_COLORS[1]
    assert _colors(last) == BAKED_COLORS[5]
    assert _image_digest(first) == ICON_PIXEL_DIGESTS[0]
    assert _image_digest(last) == ICON_PIXEL_DIGESTS[4]


def test_draw_hoe_icon_rejects_a_tier_past_five() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.draw_hoe_icon(6)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.draw_hoe_icon(None)


def test_the_five_hoe_icons_differ_from_each_other() -> None:
    icons = [hoe_icons.draw_hoe_icon(tier) for tier in hoe_icons.HOE_TIERS]
    digests = [_image_digest(icon) for icon in icons]
    assert digests == list(ICON_PIXEL_DIGESTS)
    assert len(set(digests)) == 5
    for left in range(5):
        for right in range(left + 1, 5):
            assert digests[left] != digests[right]
            assert icons[left].tobytes() != icons[right].tobytes()
    for tier, icon in zip(hoe_icons.HOE_TIERS, icons):
        assert icon.size == (16, 16)
        assert icon.mode == "RGBA"
        assert _colors(icon) == BAKED_COLORS[tier]
        for column in range(16):
            assert icon.getpixel((column, 0)) == CLEAR
            assert icon.getpixel((column, 15)) == CLEAR
        for row in range(16):
            assert icon.getpixel((0, row)) == CLEAR
            assert icon.getpixel((15, row)) == CLEAR
        raw_pixels = icon.tobytes()
        assert len(raw_pixels) == 16 * 16 * 4
        for index in range(0, len(raw_pixels), 4):
            pixel = raw_pixels[index:index + 4]
            assert pixel[3] in (0, 255)
            if pixel[3] == 0:
                assert pixel == b"\x00\x00\x00\x00"


def test_committed_hoe_icons_match_the_locked_pixel_digests() -> None:
    folder = ROOT / "assets" / "ui" / "tools"
    assert sorted(path.name for path in folder.iterdir()) == [
        "hoe_t1.png",
        "hoe_t2.png",
        "hoe_t3.png",
        "hoe_t4.png",
        "hoe_t5.png",
    ]
    digests = [
        _image_digest(Image.open(folder / f"hoe_t{tier}.png").convert("RGBA"))
        for tier in hoe_icons.HOE_TIERS
    ]
    assert digests == list(ICON_PIXEL_DIGESTS)
    assert len(set(digests)) == 5
    for left in range(5):
        assert digests[left] != digests[(left + 1) % 5]


def test_paint_handle_stamps_the_wood_shaft() -> None:
    canvas = rejecting_canvas(hoe_icons.ICON_SIZE, hoe_icons.ICON_SIZE)
    hoe_icons.paint_handle(canvas)
    assert _opaque(canvas) == {pixel: WOOD for pixel in WOOD_PIXELS}
    assert _image_digest(canvas.image()) == HANDLE_PIXEL_DIGEST


def test_paint_handle_leaves_the_blade_rows_and_the_canvas_edge_clear() -> None:
    canvas = rejecting_canvas(hoe_icons.ICON_SIZE, hoe_icons.ICON_SIZE)
    hoe_icons.paint_handle(canvas)
    assert canvas.read(0, 0) is None
    assert canvas.read(15, 15) is None
    assert canvas.read(7, 2) is None
    assert canvas.read(8, 4) is None
    assert canvas.read(1, 10) is None
    assert canvas.read(2, 10) == WOOD
    assert canvas.read(8, 5) == WOOD
    assert canvas.read(4, 13) == WOOD
    assert canvas.read(4, 14) is None


def test_paint_handle_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        hoe_icons.paint_handle(None)


def test_paint_blade_paints_the_locked_copper_head() -> None:
    canvas = rejecting_canvas(hoe_icons.ICON_SIZE, hoe_icons.ICON_SIZE)
    hoe_icons.paint_blade(canvas, 2)
    assert canvas.read(7, 2) == COPPER_HIGHLIGHT
    assert canvas.read(12, 2) == COPPER_HIGHLIGHT
    assert canvas.read(9, 3) == COPPER
    assert canvas.read(12, 4) == COPPER_EDGE
    assert canvas.read(8, 5) == COPPER
    assert canvas.read(6, 2) is None
    assert canvas.read(13, 2) is None
    assert canvas.read(7, 6) is None
    assert _image_digest(canvas.image()) == BLADE_PIXEL_DIGESTS[1]


def test_paint_blade_grows_from_the_rust_head_to_the_aethersteel_head() -> None:
    first = rejecting_canvas(hoe_icons.ICON_SIZE, hoe_icons.ICON_SIZE)
    last = rejecting_canvas(hoe_icons.ICON_SIZE, hoe_icons.ICON_SIZE)
    hoe_icons.paint_blade(first, 1)
    hoe_icons.paint_blade(last, 5)
    assert first.read(11, 2) == RUST
    assert first.read(12, 2) is None
    assert first.read(7, 4) == RUST_EDGE
    assert first.read(7, 5) == RUST
    assert last.read(5, 2) == AETHERSTEEL_GLOW
    assert last.read(13, 4) == AETHERSTEEL
    assert last.read(13, 5) == AETHERSTEEL_EDGE
    assert last.read(4, 2) is None
    assert last.read(7, 6) is None
    assert _image_digest(first.image()) == BLADE_PIXEL_DIGESTS[0]
    assert _image_digest(last.image()) == BLADE_PIXEL_DIGESTS[4]


def test_paint_blade_rejects_a_missing_canvas() -> None:
    with pytest.raises(AttributeError):
        hoe_icons.paint_blade(None, 1)
    canvas = rejecting_canvas(hoe_icons.ICON_SIZE, hoe_icons.ICON_SIZE)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.paint_blade(canvas, None)


def test_blade_pixels_returns_the_copper_head_and_its_collar() -> None:
    assert hoe_icons.blade_pixels(2) == list(TIER_TWO_BLADE)
    assert hoe_icons.blade_pixels(3)[0] == (6, 2)
    assert hoe_icons.blade_pixels(3)[-1] == (8, 5)
    assert len(hoe_icons.blade_pixels(3)) == 23
    assert hoe_icons.blade_pixels(4)[0] == (6, 2)
    assert (13, 2) in hoe_icons.blade_pixels(4)
    assert (13, 4) in hoe_icons.blade_pixels(4)
    assert hoe_icons.blade_pixels(4)[-1] == (8, 5)
    assert len(hoe_icons.blade_pixels(4)) == 26


def test_blade_pixels_spans_the_short_rust_head_and_the_tall_aethersteel_head() -> None:
    assert hoe_icons.blade_pixels(1) == list(TIER_ONE_BLADE)
    assert hoe_icons.blade_pixels(5) == list(TIER_FIVE_BLADE)
    assert (12, 2) not in hoe_icons.blade_pixels(1)
    assert (5, 2) not in hoe_icons.blade_pixels(1)
    assert (5, 5) in hoe_icons.blade_pixels(5)
    assert (13, 5) in hoe_icons.blade_pixels(5)
    assert hoe_icons.blade_pixels(5).count((7, 5)) == 1
    assert len(hoe_icons.blade_pixels(1)) == 17
    assert len(hoe_icons.blade_pixels(5)) == 36


def test_blade_pixels_rejects_a_missing_tier() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.blade_pixels(None)


def test_blade_span_returns_the_iron_head() -> None:
    assert hoe_icons.blade_span(2) == (7, 12, 2, 4)
    assert hoe_icons.blade_span(3) == (6, 12, 2, 4)
    assert hoe_icons.blade_span(4) == (6, 13, 2, 4)


def test_blade_span_grows_from_the_wood_hoe_to_the_aethersteel_hoe() -> None:
    assert hoe_icons.blade_span(1) == (7, 11, 2, 4)
    assert hoe_icons.blade_span(5) == (5, 13, 2, 5)
    left, right, top, bottom = hoe_icons.blade_span(1)
    last_left, last_right, last_top, last_bottom = hoe_icons.blade_span(5)
    assert last_left < left
    assert last_right > right
    assert last_top == top
    assert last_bottom > bottom


def test_blade_span_rejects_a_missing_tier() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.blade_span(None)


def test_blade_color_lights_the_copper_crown_and_darkens_the_cutting_edge() -> None:
    assert hoe_icons.blade_color(2, 7, 2) == COPPER_HIGHLIGHT
    assert hoe_icons.blade_color(2, 12, 2) == COPPER_HIGHLIGHT
    assert hoe_icons.blade_color(2, 7, 3) == COPPER
    assert hoe_icons.blade_color(2, 12, 3) == COPPER
    assert hoe_icons.blade_color(2, 7, 4) == COPPER_EDGE
    assert hoe_icons.blade_color(2, 12, 4) == COPPER_EDGE
    assert hoe_icons.blade_color(2, 7, 5) == COPPER
    assert hoe_icons.blade_color(2, 8, 5) == COPPER


def test_blade_color_keeps_tier_one_rust_flat_and_tier_five_glowing() -> None:
    assert hoe_icons.blade_color(1, 7, 2) == RUST
    assert hoe_icons.blade_color(1, 11, 3) == RUST
    assert hoe_icons.blade_color(1, 11, 4) == RUST_EDGE
    assert hoe_icons.blade_color(1, 8, 5) == RUST
    assert hoe_icons.blade_color(5, 5, 2) == AETHERSTEEL_GLOW
    assert hoe_icons.blade_color(5, 13, 2) == AETHERSTEEL_GLOW
    assert hoe_icons.blade_color(5, 5, 3) == AETHERSTEEL
    assert hoe_icons.blade_color(5, 13, 4) == AETHERSTEEL
    assert hoe_icons.blade_color(5, 5, 5) == AETHERSTEEL_EDGE
    assert hoe_icons.blade_color(5, 13, 5) == AETHERSTEEL_EDGE


def test_blade_color_rejects_a_pixel_that_is_not_on_the_blade() -> None:
    with pytest.raises(ValueError, match="pixel is not on the hoe blade"):
        hoe_icons.blade_color(1, 0, 0)
    with pytest.raises(ValueError, match="pixel is not on the hoe blade"):
        hoe_icons.blade_color(2, None, 2)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.blade_color(None, 7, 2)


def test_metal_colors_returns_the_baked_palette_from_the_hoe_frames() -> None:
    assert hoe_icons.metal_colors(1) == (RUST, RUST_EDGE, RUST)
    assert hoe_icons.metal_colors(2) == (COPPER, COPPER_EDGE, COPPER_HIGHLIGHT)
    assert hoe_icons.metal_colors(3) == (IRON, IRON_EDGE, IRON_HIGHLIGHT)
    assert hoe_icons.metal_colors(4) == (BRASS, BRASS_EDGE, BRASS_HIGHLIGHT)
    assert hoe_icons.metal_colors(5) == (AETHERSTEEL, AETHERSTEEL_EDGE, AETHERSTEEL_GLOW)
    assert hoe_icons.WOOD == WOOD
    assert hoe_icons.OUTLINE == OUTLINE


def test_metal_colors_keeps_tier_one_rust_flat_and_tier_five_bright() -> None:
    body, edge, highlight = hoe_icons.metal_colors(1)
    assert highlight == body == RUST
    assert edge == RUST_EDGE
    assert highlight != edge
    aethersteel, aethersteel_edge, glow = hoe_icons.metal_colors(5)
    assert aethersteel == AETHERSTEEL
    assert aethersteel_edge == AETHERSTEEL_EDGE
    assert glow == AETHERSTEEL_GLOW
    assert glow != aethersteel
    assert glow != aethersteel_edge
    assert aethersteel != body


def test_metal_colors_rejects_a_missing_tier() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.metal_colors(None)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.metal_colors(True)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.metal_colors(6)


def test_require_unique_hoe_icons_accepts_the_five_drawn_icons() -> None:
    hoe_icons.require_unique_hoe_icons(hoe_icons.hoe_icon_records())


def test_require_unique_hoe_icons_accepts_an_empty_list_and_one_icon() -> None:
    hoe_icons.require_unique_hoe_icons([])
    hoe_icons.require_unique_hoe_icons([(1, hoe_icons.draw_hoe_icon(1))])


def test_require_unique_hoe_icons_rejects_two_copies_of_one_icon() -> None:
    icon = hoe_icons.draw_hoe_icon(1)
    with pytest.raises(ValueError, match="hoe icons are not unique"):
        hoe_icons.require_unique_hoe_icons([(1, icon), (2, icon)])
    with pytest.raises(ValueError, match="not enough values to unpack"):
        hoe_icons.require_unique_hoe_icons([(1,)])
    with pytest.raises(AttributeError):
        hoe_icons.require_unique_hoe_icons([(1, None)])
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.require_unique_hoe_icons([(0, icon)])


def test_save_hoe_icon_writes_the_tier_one_icon(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "FOLDER", tmp_path)
    hoe_icons.save_hoe_icon(hoe_icons.hoe_icon_record(1))
    saved = Image.open(tmp_path / "hoe_t1.png").convert("RGBA")
    assert saved.size == (16, 16)
    assert saved.mode == "RGBA"
    assert _image_digest(saved) == ICON_PIXEL_DIGESTS[0]
    assert sorted(path.name for path in tmp_path.iterdir()) == ["hoe_t1.png"]


def test_save_hoe_icon_creates_a_missing_tools_folder(tmp_path, monkeypatch) -> None:
    folder = tmp_path / "ui" / "tools"
    monkeypatch.setattr(hoe_icons, "FOLDER", folder)
    hoe_icons.save_hoe_icon(hoe_icons.hoe_icon_record(5))
    saved = folder / "hoe_t5.png"
    assert saved.is_file()
    assert _image_digest(Image.open(saved).convert("RGBA")) == ICON_PIXEL_DIGESTS[4]


def test_save_hoe_icon_rejects_a_record_missing_the_image(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(hoe_icons, "FOLDER", tmp_path)
    with pytest.raises(ValueError, match="not enough values to unpack"):
        hoe_icons.save_hoe_icon((1,))
    with pytest.raises(ValueError, match="hoe icon must be a 16 by 16 RGBA image"):
        hoe_icons.save_hoe_icon((2, Image.new("RGBA", (8, 8))))
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.save_hoe_icon((0, hoe_icons.draw_hoe_icon(1)))
    assert list(tmp_path.iterdir()) == []


def test_require_hoe_icon_image_accepts_a_drawn_hoe() -> None:
    assert hoe_icons.require_hoe_icon_image(hoe_icons.draw_hoe_icon(3)) is None
    assert hoe_icons.require_hoe_icon_image(hoe_icons.draw_hoe_icon(1)) is None


def test_require_hoe_icon_image_accepts_a_blank_sixteen_by_sixteen_image() -> None:
    blank = Image.new("RGBA", (16, 16))
    assert hoe_icons.require_hoe_icon_image(blank) is None
    blank.putpixel((15, 15), (1, 2, 3, 255))
    assert hoe_icons.require_hoe_icon_image(blank) is None


def test_require_hoe_icon_image_rejects_a_missing_image() -> None:
    with pytest.raises(AttributeError):
        hoe_icons.require_hoe_icon_image(None)
    with pytest.raises(ValueError, match="hoe icon must be a 16 by 16 RGBA image"):
        hoe_icons.require_hoe_icon_image(Image.new("RGB", (16, 16)))
    with pytest.raises(ValueError, match="hoe icon must be a 16 by 16 RGBA image"):
        hoe_icons.require_hoe_icon_image(Image.new("RGBA", (16, 15)))


def test_hoe_icon_file_name_uses_the_tier_number() -> None:
    assert hoe_icons.hoe_icon_file_name(2) == "hoe_t2.png"
    assert hoe_icons.hoe_icon_file_name(3) == "hoe_t3.png"
    assert hoe_icons.hoe_icon_file_name(4) == "hoe_t4.png"


def test_hoe_icon_file_name_spans_the_first_and_last_tiers() -> None:
    assert hoe_icons.hoe_icon_file_name(1) == "hoe_t1.png"
    assert hoe_icons.hoe_icon_file_name(5) == "hoe_t5.png"


def test_hoe_icon_file_name_rejects_a_tier_of_zero() -> None:
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.hoe_icon_file_name(0)
    with pytest.raises(ValueError, match="hoe tier must be 1, 2, 3, 4, or 5"):
        hoe_icons.hoe_icon_file_name(None)
