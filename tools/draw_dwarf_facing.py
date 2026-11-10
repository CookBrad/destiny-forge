from __future__ import annotations

from pathlib import Path

from art_common import composite_layers, require_opaque_color

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / "assets" / "player" / "non-combat"

CANVAS_WIDTH = 16
CANVAS_HEIGHT = 28

OUTLINE = require_opaque_color((34, 34, 34, 255))
SKIN = require_opaque_color((216, 165, 125, 255))
SKIN_SHADE = require_opaque_color((195, 141, 112, 255))
TUNIC = require_opaque_color((218, 78, 56, 255))
TUNIC_LIGHT = require_opaque_color((228, 110, 51, 255))
STEEL = require_opaque_color((123, 137, 148, 255))
BOOT = require_opaque_color((119, 92, 85, 255))
HAIR = require_opaque_color((92, 64, 51, 255))

PALETTE = {
    "a": OUTLINE,
    "s": SKIN,
    "d": SKIN_SHADE,
    "t": TUNIC,
    "h": TUNIC_LIGHT,
    "m": STEEL,
    "b": BOOT,
    "r": HAIR,
}

FRONT_IDLE_BASE = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "......aaaa......",
    "....aarrrraa....",
    "...arrrrrrrra...",
    "..arrrssssrrra..",
    "..arssssssssra..",
    "..assddddddssa..",
    "..asddddddddsa..",
    "...assssssssa...",
    "...atttttttta...",
    "..atttttttttta..",
    "..atthttthttta..",
    "..atttttttttta..",
    "...atttttttta...",
    "...amtttttma....",
    "...ammtttmma....",
    "...ammmmmmma....",
    "...abbaaabba....",
    "...aba..aba.....",
    "...aa....aa.....",
    "................",
    "................",
]

BACK_IDLE_BASE = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "......aaaa......",
    "....aarrrraa....",
    "...arrrrrrrra...",
    "..arrrrrrrrrra..",
    "..arrrrrrrrrra..",
    "..arrrrrrrrrra..",
    "..arrrrrrrrrra..",
    "...arrrrrrra....",
    "...atttttttta...",
    "..atttttttttta..",
    "..atttttttttta..",
    "..atthttthttta..",
    "...atttttttta...",
    "...amtttttma....",
    "...ammtttmma....",
    "...ammmmmmma....",
    "...abbaaabba....",
    "...aba..aba.....",
    "...aa....aa.....",
    "................",
    "................",
]


def idle_front_frames():
    breathe = [
        FRONT_IDLE_BASE,
        with_boots(FRONT_IDLE_BASE, "...abbaaabba....", "....aba.aba.....", "....aa...aa....."),
        FRONT_IDLE_BASE,
        with_boots(FRONT_IDLE_BASE, "....abbaabba....", "....aba.aba.....", "....aa...aa....."),
    ]
    return breathe


def walk_front_frames():
    return [
        with_boots(FRONT_IDLE_BASE, "...abba..bba....", "...aba...aba....", "...aa.....aa...."),
        with_boots(FRONT_IDLE_BASE, "....abbaabba....", "....aba.aba.....", "....aa...aa....."),
        with_boots(FRONT_IDLE_BASE, "...bba..abba....", "...aba...aba....", "...aa.....aa...."),
        with_boots(FRONT_IDLE_BASE, "....abbaabba....", "...aba...aba....", "...aa.....aa...."),
    ]


def walk_back_frames():
    return [
        with_boots(BACK_IDLE_BASE, "...abba..bba....", "...aba...aba....", "...aa.....aa...."),
        with_boots(BACK_IDLE_BASE, "....abbaabba....", "....aba.aba.....", "....aa...aa....."),
        with_boots(BACK_IDLE_BASE, "...bba..abba....", "...aba...aba....", "...aa.....aa...."),
        with_boots(BACK_IDLE_BASE, "....abbaabba....", "...aba...aba....", "...aa.....aa...."),
    ]


def with_boots(base, row23, row24, row25):
    rows = list(base)
    rows[23] = row23
    rows[24] = row24
    rows[25] = row25
    return rows


SETS = (
    ("dwarf_m_idle_front", idle_front_frames),
    ("dwarf_m_walk_front", walk_front_frames),
    ("dwarf_m_walk_back", walk_back_frames),
)


def main():
    for stem, builder in SETS:
        for index, pattern in enumerate(builder()):
            require_pattern_size(pattern)
            draw_frame(pattern).save(FOLDER / f"{stem}_f{index}.png")


def draw_frame(pattern):
    require_pattern_size(pattern)
    return composite_layers(CANVAS_WIDTH, CANVAS_HEIGHT, [stamp_pattern(pattern)])


def stamp_pattern(pattern):
    layer = {}
    for row_index, line in enumerate(pattern):
        for column_index, key in enumerate(line):
            if key == ".":
                continue
            layer[(column_index, row_index)] = PALETTE[key]
    return layer


def require_pattern_size(pattern):
    if len(pattern) != CANVAS_HEIGHT:
        raise ValueError(f"pattern height {len(pattern)} != {CANVAS_HEIGHT}")
    for line in pattern:
        if len(line) != CANVAS_WIDTH:
            raise ValueError(f"pattern width {len(line)} != {CANVAS_WIDTH}")


if __name__ == "__main__":
    main()
