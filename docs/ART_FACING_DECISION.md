# ART-02 facing decision (AP1)

Record for `#180` / `design/15-art-presentation.md` open question Q1.

## Decision

Ship **back walk**, **front walk**, and **front idle** dwarf sets (4 frames each, 16×28). Use them for vertical-only homestead actions: doors, counters, ladders, boards, and beds. Keep tool actions side-only with runtime `flip_x`, matching the shipped pickaxe and hoe canvases.

## File names

- `assets/player/non-combat/dwarf_m_walk_back_f0..3.png`
- `assets/player/non-combat/dwarf_m_walk_front_f0..3.png`
- `assets/player/non-combat/dwarf_m_idle_front_f0..3.png`

Drawn by `tools/draw_dwarf_facing.py`. Side idle/run frames stay as they are.

## Follow-up

`#702` ART-08 wires vertical movement and door/ladder prompts to these sets once ART-03 registers them.
