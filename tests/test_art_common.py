from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

from art_common import (
    CLEAR,
    _frame_pixels,
    composite_images,
    composite_layers,
    outline_layer,
    pack_strip,
    paste_pixels,
    translate_pixels,
)

INK = (0, 0, 0, 255)
RED = (200, 0, 0, 255)
BLUE = (0, 0, 200, 255)
GREEN = (0, 180, 0, 255)
PARTIAL = (1, 2, 3, 128)


def test_pack_strip_places_each_frame_in_its_own_cell() -> None:
    image = pack_strip([{(0, 1): RED}, {(2, 0): BLUE}], 4, 3)
    assert image.size == (8, 3)
    assert image.getpixel((0, 1)) == RED
    assert image.getpixel((6, 0)) == BLUE
    assert image.getpixel((1, 1)) == CLEAR
def test_pack_strip_of_an_empty_frame_list_is_a_zero_width_strip() -> None:
    image = pack_strip([], 5, 2)
    assert image.size == (0, 2)
    assert image.mode == "RGBA"
def test_pack_strip_rejects_a_pixel_that_falls_outside_the_cell() -> None:
    with pytest.raises(SystemExit, match=r"blit clipped 4,0"):
        pack_strip([{(4, 0): RED}], 4, 2)
def test_frame_pixels_returns_a_dictionary_unchanged() -> None:
    frame = {(1, 2): RED}
    assert _frame_pixels(frame) is frame
def test_frame_pixels_reads_pixels_from_a_sprite_like_frame() -> None:
    frame = type("Frame", (), {"pixels": {(0, 0): BLUE}})()
    assert _frame_pixels(frame) == {(0, 0): BLUE}
def test_frame_pixels_rejects_a_frame_with_no_pixel_map() -> None:
    with pytest.raises(AttributeError):
        _frame_pixels(type("Frame", (), {})())
def test_composite_layers_lets_a_later_layer_cover_an_earlier_pixel() -> None:
    image = composite_layers(2, 2, [{(0, 0): RED, (1, 1): BLUE}, {(0, 0): GREEN}])
    assert image.getpixel((0, 0)) == GREEN
    assert image.getpixel((1, 1)) == BLUE
    assert image.getpixel((1, 0)) == CLEAR
def test_composite_layers_with_no_layers_is_a_clear_image() -> None:
    image = composite_layers(1, 1, [])
    assert image.getpixel((0, 0)) == CLEAR
def test_composite_layers_rejects_a_pixel_outside_the_image() -> None:
    with pytest.raises(IndexError):
        composite_layers(1, 1, [{(2, 0): RED}])
def test_composite_images_stacks_a_layer_at_its_origin() -> None:
    layer = Image.new("RGBA", (1, 1), RED)
    image = composite_images((2, 2), [(layer, (1, 1))])
    assert image.getpixel((1, 1)) == RED
    assert image.getpixel((0, 0)) == CLEAR
def test_composite_images_with_no_layers_stays_clear() -> None:
    image = composite_images((2, 1), [])
    assert image.getpixel((0, 0)) == CLEAR
    assert image.getpixel((1, 0)) == CLEAR
def test_composite_images_rejects_a_layer_that_is_not_rgba() -> None:
    layer = Image.new("RGB", (1, 1), (1, 2, 3))
    with pytest.raises(ValueError, match="images do not match"):
        composite_images((2, 2), [(layer, (0, 0))])
def test_outline_layer_adds_four_connected_neighbors_inside_the_bounds() -> None:
    outlined = outline_layer({(1, 1): RED}, INK, 3, 3)
    assert outlined[(1, 1)] == RED
    assert outlined[(2, 1)] == INK
    assert outlined[(0, 1)] == INK
    assert outlined[(1, 2)] == INK
    assert outlined[(1, 0)] == INK
    assert len(outlined) == 5
def test_outline_layer_does_not_grow_past_the_layer_bounds() -> None:
    outlined = outline_layer({(0, 0): RED}, INK, 1, 1)
    assert outlined == {(0, 0): RED}
def test_outline_layer_rejects_a_layer_that_is_not_a_mapping() -> None:
    with pytest.raises(TypeError):
        outline_layer(None, INK, 2, 2)
def test_paste_pixels_copies_a_layer_by_the_given_offset() -> None:
    target: dict[tuple[int, int], tuple[int, int, int, int]] = {(0, 0): BLUE}
    paste_pixels(target, {(1, 1): RED}, 2, 3)
    assert target[(3, 4)] == RED
    assert target[(0, 0)] == BLUE
def test_paste_pixels_with_no_offset_overwrites_the_same_point() -> None:
    target = {(0, 0): BLUE}
    paste_pixels(target, {(0, 0): RED})
    assert target == {(0, 0): RED}
def test_paste_pixels_rejects_a_layer_without_items() -> None:
    with pytest.raises(AttributeError):
        paste_pixels({}, [(0, 0)])
def test_translate_pixels_moves_every_point_by_the_offset() -> None:
    moved = translate_pixels({(1, 2): RED, (0, 0): BLUE}, -1, 4)
    assert moved == {(0, 6): RED, (-1, 4): BLUE}
def test_translate_pixels_with_a_zero_offset_returns_a_copy() -> None:
    source = {(1, 1): RED}
    moved = translate_pixels(source, 0, 0)
    assert moved == source
    assert moved is not source
def test_translate_pixels_rejects_a_missing_pixel_map() -> None:
    with pytest.raises(AttributeError):
        translate_pixels(None, 1, 0)
def test_shared_helpers_do_not_restore_the_unused_contact_sheet() -> None:
    import art_common

    assert not hasattr(art_common, "contact_sheet")
    assert not hasattr(art_common, "place_on_pin")
    assert not hasattr(art_common, "Pin")
