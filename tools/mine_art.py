"""Build the 48x64 mine entrance sprite from AI-generated art."""

from __future__ import annotations

from ai_art_common import fit_to_canvas, load_clean

TARGET_W = 48
TARGET_H = 64


def build_mine_entrance():
    img = load_clean("mine_entrance")
    return fit_to_canvas(
        img,
        TARGET_W,
        TARGET_H,
        ground_y=TARGET_H - 1,
        pad=(12, 12, 12, 20),
    )