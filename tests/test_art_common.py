from __future__ import annotations

import hashlib
from pathlib import Path

import pytest
from PIL import Image

from art_common import (
    CLEAR,
    _frame_pixels,
    add_cast_rim,
    add_outline,
    clipping_canvas,
    composite_images,
    composite_layers,
    light_contour,
    outline_layer,
    pack_strip,
    paste_pixels,
    rejecting_canvas,
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
def test_add_outline_paints_the_eight_empty_neighbors_around_one_pixel() -> None:
    canvas = clipping_canvas(3, 3)
    canvas.put(1, 1, RED)
    add_outline(canvas, INK)
    assert canvas.read(1, 1) == RED
    assert [canvas.read(x, y) for y in range(3) for x in range(3)].count(INK) == 8
def test_add_outline_stops_at_the_corner_of_the_canvas() -> None:
    canvas = clipping_canvas(2, 2)
    canvas.put(0, 0, RED)
    add_outline(canvas, INK)
    assert canvas.read(1, 0) == INK
    assert canvas.read(0, 1) == INK
    assert canvas.read(1, 1) == INK
    assert canvas.opaque_at(0, 0) is True
def test_add_outline_rejects_a_partially_transparent_outline_color() -> None:
    canvas = clipping_canvas(3, 3)
    canvas.put(1, 1, RED)
    with pytest.raises(SystemExit, match="partial alpha"):
        add_outline(canvas, PARTIAL)
def test_add_cast_rim_recolors_a_pixel_whose_lower_right_is_empty() -> None:
    canvas = clipping_canvas(3, 3)
    canvas.put(0, 0, RED)
    canvas.put(1, 0, RED)
    add_cast_rim(canvas, INK)
    assert canvas.read(0, 0) == INK
    assert canvas.read(1, 0) == INK
def test_add_cast_rim_leaves_a_pixel_with_opaque_right_down_and_diagonal() -> None:
    canvas = clipping_canvas(2, 2, RED)
    add_cast_rim(canvas, INK)
    assert canvas.read(0, 0) == RED
    assert canvas.read(1, 0) == INK
    assert canvas.read(0, 1) == INK
    assert canvas.read(1, 1) == INK
def test_add_cast_rim_rejects_a_partially_transparent_rim_color() -> None:
    canvas = clipping_canvas(2, 2)
    canvas.put(0, 0, RED)
    with pytest.raises(SystemExit, match="partial alpha"):
        add_cast_rim(canvas, PARTIAL)
def test_light_contour_recolors_outline_with_empty_space_above() -> None:
    canvas = clipping_canvas(3, 3)
    canvas.put(1, 1, INK)
    canvas.put(1, 2, RED)
    light_contour(canvas, INK, BLUE)
    assert canvas.read(1, 1) == BLUE
    assert canvas.read(1, 2) == RED
def test_light_contour_recolors_a_left_edge_that_still_has_a_pixel_below() -> None:
    canvas = clipping_canvas(3, 3)
    canvas.put(1, 1, INK)
    canvas.put(2, 1, RED)
    canvas.put(1, 2, RED)
    light_contour(canvas, INK, BLUE)
    assert canvas.read(1, 1) == BLUE
def test_light_contour_rejects_a_partially_transparent_lit_color() -> None:
    canvas = clipping_canvas(2, 2)
    canvas.put(0, 0, INK)
    with pytest.raises(SystemExit, match="partial alpha"):
        light_contour(canvas, INK, PARTIAL)
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
def test_clipping_canvas_ignores_a_pixel_drawn_outside_the_icon() -> None:
    canvas = clipping_canvas(2, 2, BLUE)
    canvas.put(-1, 0, RED)
    canvas.put(2, 0, RED)
    assert canvas.pixels == [BLUE, BLUE, BLUE, BLUE]
def test_clipping_canvas_read_of_a_clear_pixel_is_none() -> None:
    canvas = clipping_canvas(1, 1)
    assert canvas.read(0, 0) is None
    assert canvas.read(5, 5) is None
    assert canvas.opaque_at(0, 0) is False
def test_clipping_canvas_rejects_a_partially_transparent_pixel() -> None:
    canvas = clipping_canvas(1, 1)
    with pytest.raises(SystemExit, match="partial alpha"):
        canvas.put(0, 0, PARTIAL)
def test_rejecting_canvas_stores_a_pixel_on_the_last_row() -> None:
    canvas = rejecting_canvas(2, 2)
    canvas.put(1, 1, RED)
    assert canvas.read(1, 1) == RED
    assert isinstance(canvas.image().getpixel((1, 1)), tuple)
def test_rejecting_canvas_rejects_a_pixel_outside_the_icon() -> None:
    canvas = rejecting_canvas(2, 2)
    with pytest.raises(ValueError, match=r"pixel outside the icon: \(2, 0\)"):
        canvas.put(2, 0, RED)
def test_rejecting_canvas_rejects_a_partially_transparent_pixel_inside_the_icon() -> None:
    canvas = rejecting_canvas(1, 1)
    with pytest.raises(SystemExit, match="partial alpha"):
        canvas.put(0, 0, PARTIAL)
def test_canvas_fill_paints_each_listed_point() -> None:
    canvas = clipping_canvas(3, 1)
    canvas.fill([(0, 0), (2, 0), (9, 9)], RED)
    assert canvas.read(0, 0) == RED
    assert canvas.read(1, 0) is None
    assert canvas.read(2, 0) == RED
def test_canvas_fill_rect_uses_a_half_open_range() -> None:
    canvas = clipping_canvas(4, 3)
    canvas.fill_rect(1, 0, 3, 2, RED)
    assert canvas.read(1, 0) == RED
    assert canvas.read(2, 1) == RED
    assert canvas.read(3, 0) is None
    assert canvas.read(1, 2) is None
def test_canvas_fill_rect_with_an_empty_range_paints_nothing() -> None:
    canvas = clipping_canvas(2, 2, BLUE)
    canvas.fill_rect(1, 1, 1, 2, RED)
    assert canvas.pixels == [BLUE, BLUE, BLUE, BLUE]
def test_canvas_blit_writes_the_shaded_pixel_map() -> None:
    canvas = clipping_canvas(2, 2)
    canvas.blit({(1, 0): RED})
    assert canvas.read(1, 0) == RED
def test_canvas_row_assignment_of_none_clears_the_pixel() -> None:
    canvas = clipping_canvas(2, 2)
    canvas[0][1] = RED
    assert canvas[0][1] == RED
    canvas[0][1] = None
    assert canvas[0][1] is None
    assert canvas.pixels[1] == CLEAR
def test_canvas_row_read_outside_the_row_is_none() -> None:
    canvas = clipping_canvas(1, 1)
    assert canvas[0][4] is None
def test_canvas_image_copies_the_stored_pixels() -> None:
    canvas = clipping_canvas(2, 1)
    canvas.put(1, 0, RED)
    image = canvas.image()
    assert image.getpixel((0, 0)) == CLEAR
    assert image.getpixel((1, 0)) == RED
def test_store_opaque_writes_a_fully_transparent_pixel_inside_the_buffer() -> None:
    canvas = clipping_canvas(1, 1, RED)
    canvas.store_opaque(0, 0, CLEAR)
    assert canvas.pixels == [CLEAR]
def test_opaque_at_is_true_only_for_a_fully_opaque_pixel() -> None:
    canvas = clipping_canvas(2, 1)
    canvas.put(0, 0, RED)
    canvas.store_opaque(1, 0, CLEAR)
    assert canvas.opaque_at(0, 0) is True
    assert canvas.opaque_at(1, 0) is False
    assert canvas.opaque_at(-1, 0) is False
def test_shared_helpers_do_not_restore_the_unused_contact_sheet() -> None:
    import art_common

    assert not hasattr(art_common, "contact_sheet")
    assert not hasattr(art_common, "place_on_pin")
    assert not hasattr(art_common, "Pin")
