# 15 — Art & Presentation (pipeline, conventions, style, cross-system sprite standards)

Authority: GDD §8 ("cozy steampunk aesthetic: warm brass, copper, wood, glowing magical accents. Farm and village lived-in and inviting; hunts feature more dramatic lighting and magical effects. Creatures: magical beasts with distinctive silhouettes, readable tells, satisfying part-break animations; organic + crystalline/elemental traits"), §11 ("art style tests for farm vs. hunt presentation"). Brad's standing rule: **every player action needs new in-house sprite frames**.

Build priority: **15 of 16**, but **ART-01, ART-02, ART-03 should land before the first art tickets of mining/farming**. They define the conventions every [ART] ticket in docs 01–14 follows.

## 0. Standing rules (canonical statement)

1. Every player action = new in-house frames. No UI banner, color tint, tinted arc, or scale-pulse may stand in for an animation. Tier/variant differences are **separate baked PNGs** (palette/layer composites written by a draw script), never a runtime `Sprite.color` tint.
2. In-house frames are produced by deterministic Python scripts in `tools/draw_*.py` (PIL), which re-run byte-identically (current pattern: `draw_dwarf_pickaxe_swing.py`, `draw_homestead_eat_frames.py`, `draw_hunter_frames.py`, `draw_dungeon_actors.py`, `draw_skill_icons.py`). Hand-pixel PNGs are allowed when committed with a source `.aseprite`/layered file under `assets/source/inhouse/`.
3. **Dungeon platform/art churn is frozen**: `assets/dungeon/**` and the dungeon setup layout do not change. **Taste-signed hunter cells** never change.
4. Third-party art (0x72 dwarf base, Pixel Crawler workbench/anvil/trees, Dawnlike animals) is credited in `assets/ATTRIBUTION.txt`. The long-term goal is in-house replacements (Q3).
5. Clean Code, no code comments, zero warnings/errors (applies to tools/ Python too: no comment noise beyond the module docstring the existing scripts use; ruff-clean).
6. Each [ART] ticket ships: PNGs + draw script + loader constants (frame count, cell size, per-frame seconds) + a preview contact sheet (PNG in the PR body, not committed) + attribution lines if needed. Gameplay wiring is a separate [GAME] ticket unless trivial.

## 1. Already exists (current conventions, as found in code)

| Domain | Convention | Source |
| --- | --- | --- |
| Top-down avatar (homestead/forest/mine) | 0x72 `dwarf_m`, **16×28**, idle 4 / run 4 / eat 4 (in-house, 0.18 s, order 0,1,2,3,2,3), pickaxe swing 4 on a **48×60 canvas** (dwarf at 16,16), 0.3 s; side-only, `flip_x` for left | `overworld/sprites.rs`, `eating.rs`, `mining/swing.rs` |
| Side-scroll hunter | **343×160 cells**, sole at row 159, pins `IDLE_PIN_X 86`, `FOOT_PIN_X 81.5`; idle 4 (1 played), run 9 (cells 1–8 @0.08), attack 4, jump 1, fall 1, death 2; grit-neon style | `dungeon/sprites.rs`, `hunter_pose.rs`, ATTRIBUTION |
| Dungeon enemies | 4-frame strips, 0.10–0.18 s | `enemy_anim.rs` |
| World units | TILE = 32 | `graphics/world_units.rs` |
| Env tiles | 32×32 placeholder tiles with tints | `overworld/environment/*` |
| UI icons | skill icons 32×32 (in-house); inventory uses swatches + text | `ui/skills/*`, `inventory_window.rs` |
| Animals | Dawnlike 16×16, 4 frames per row | `overworld/sprites.rs` |

## 2. Where it applies

All zones, actors, items, UI. Two presentation modes (GDD "farm vs hunt"):
- **Life mode** (top-down: homestead, village, zones, mine): warm palette, soft daylight, chunky 16-px characters on 32-px tiles, cozy steampunk props.
- **Hunt mode** (side-scroll: dungeon + hunting grounds): dramatic lighting (rim light drawn into frames, darker backgrounds, parallax), the large grit-neon hunter cell, big monsters, saturated magic VFX.

## 3. Entities / standards

### 3.1 Canonical sprite spec table (all docs follow this)

| Kind | Native size | Canvas | Facings | Frame-time range | File layout |
| --- | --- | --- | --- | --- | --- |
| Dwarf body | 16×28 | 16×28 (locomotion), 24–64 × 28–60 (actions) | R + `flip_x`; back/front only where listed (ART-02) | 0.04–0.6 s | one PNG per frame `dwarf_m_<set>_f<n>.png` (existing pattern) |
| NPC body | 16×28 (Kit 16×24) | same | R+flip, front, back | 0.1–0.6 | strip per set `npc_<id>_<set>.png` |
| Hunter / companions | 343×160 cell | same | R + flip | 0.035–0.2 | strip `knight_<set>_side.png` / `companion_<id>_<set>.png` |
| Small monster (side) | 64×64 / 96×64 | — | R + flip | 0.06–0.2 | strip |
| Large monster | 384×256 / 512×320 / 640×384 | — | R + flip | 0.04–0.18 | strip per state + part overlays |
| Mine monster (top-down) | 16×16 – 64×64 | — | R + flip | 0.05–0.4 | strip |
| Animals | 16×16 – 48×48 | — | R + flip, front | 0.05–0.6 | strip |
| Tiles | 32×32 (autotile 16/47) | — | — | static/0.2–0.3 anim | atlas per tileset per season |
| Props | multiples of 16 | — | front | 0.06–0.4 | strip |
| VFX | 8×8 – 320×180 | — | — | 0.04–0.1 | strip |
| Item icons | 16×16 (shown ×3) | — | — | — | atlas `ui/icons/<category>.png` + index RON |
| HUD icons | 32×32 | — | — | — | |
| Portraits | 64×64 | — | front | — | per villager × 6 expressions |

### 3.2 Palette & style guide
- Master palette (`assets/source/inhouse/palette.gpl`): brass ramp (4), copper (4), iron/steel (4), wood (4), leaf greens (4 per season), sky (5 phase gradients), magic accents (aether cyan, ember orange, thunder yellow, frost blue, poison green, destiny prismatic: 3 each), outline `#222222` (existing dwarf outline), skin ramps.
- Outline: 1-px dark outline on characters/items. Tiles have no outline.
- Lighting: top-left key light in life mode. Hunt mode adds a colored rim light from the magic source.
- Readability: monster telegraph poses exaggerate silhouette change ≥ 25% of the body area; telegraph frames add 1–2 px of glow on the attacking part.
- Part-break art: every breakable part has an intact and a broken drawing with a silhouette change visible at 1×.

### 3.3 Animation principles checklist (for review)
Anticipation frame before every active hit; smear frame on fast swings; a 1-frame hold on impact (pairs with the existing hit-stop); follow-through/recover ≥ 2 frames; idle loops breathe (≥ 2 distinct frames); anchor pins keep the body from swimming (existing sole/pin rules); left/right via flip only (assets authored facing right).

## 4. Tools / pipeline

| Tool | Purpose | Status |
| --- | --- | --- |
| `tools/draw_*.py` per set family | deterministic frame generation | pattern exists |
| `tools/art_common.py` (new) | shared palette, outline, canvas/pin helpers, layer compositing, strip packing, contact-sheet export | ART-01 |
| `tools/draw_monster_rig.py` | modular rig bake (06 MON-03) | new |
| `tools/pack_icons.py` | 16×16 icon atlas + `icons.ron` index | ART-04 |
| `tools/season_bake.py` | palette remap of tilesets into 4 seasons + snow overlay merge | WLD-22 |
| `tools/art_lint.py` | checks size/frame count/palette conformity/transparent margins vs `sprites.ron` | ART-05 |
| `assets/data/sprites.ron` (new) | single registry: set id → file(s), cell, frames, per-frame seconds, pins, loop, hit frames | ART-03 |

## 5. ANIMATIONS (most detailed section): cross-system sprite inventory

This is the roll-up of every sprite set specified in docs 01–14, used to plan art capacity. Counts are approximate frames.

| Doc | Character (player) | NPC/companion | Entity/world | Monster | VFX | UI/icons | Approx. frames |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 01 Mining | 24 sets (~130) | — | ~200 (nodes 84, props, hazards, tilesets) | 12 mine monsters (~330) | 14 sets (~70) | ~95 icons + HUD | **~830** |
| 02 Farming | 24 sets (~170) | — | ~450 (crops ~165, trees ~70, machines ~72, debris, sprinklers) | crows (13) | 7 sets (~60) | ~150 icons | **~840** |
| 03 Fishing | 19 sets (~250 with rods) | — | ~80 (bobbers, shadows, water) | — | — | ~75 icons + minigame UI (~45) | **~450** |
| 04 Forging | 16 sets (~75) | — | ~140 (stations L0–L5, transformations) | — | ~110 | ~400 icons (gear, skills, decos) | **~725** |
| 05 Combat | 31 shared sets + 6 weapon sheets (~715 hunter-cell frames) | — | traps (~22) | — | ~350 | ~120 HUD/pouch | **~1,200** |
| 06 Monsters | — | — | — | 12 large (~2,400 incl. Elder skins) + 10 small (~300) + pests | ~120 | ~105 portraits/parts | **~2,900** |
| 07 Animals | 15 sets (~60) | — | ~120 buildings/objects | — | ~30 | ~50 | 420 animal + **~680** |
| 08 Social | 12 sets (~45) | 14 NPC (~1,150) + 4 companions (~280) | festival dressing (~60) | — | ~60 | portraits 84 + UI | **~1,700** |
| 09 Economy | 8 sets (~35) | — | ~70 | — | ~25 | ~30 | **~160** |
| 10 World | 13 sets (~45) | — | tilesets ×4 seasons + props (~900) | critters (~40) | weather (~70) | map (~30) | **~1,100** |
| 11 Day/season | 10 sets (~35) | — | ~30 | — | ~20 | ~20 | **~105** |
| 12 Narrative | — | (reuse) | spectacle beats (~290) | (reuse MON) | — | — | **~290** |
| 13 UI | — | — | — | — | — | ~170 animated UI frames + static sets | **~300** |
| 14 Audio | 1 set (4) | — | gramophone (7) | — | — | ~45 | **~60** |
| 16 Saves | 2 sets (~10) | — | — | — | — | ~25 | **~35** |
| **Total** | | | | | | | **≈ 11,000 frames** |

**Implication for Forge (PM):** art is the critical path. Proposed throughput target: one [ART] ticket ≈ 20–60 frames. Prioritize **art for actions that already exist but have no frames** (the standing-rule debt), listed below, before any new content art.

### 5.1 Standing-rule debt (existing actions without frames)

| Existing action | Missing frames | Ticket |
| --- | --- | --- |
| Hoe till | F1 | FRM-01 |
| Watering | F4 | FRM-03 |
| Plant seeds / harvest (Hand) | F9, F11, F12 | FRM-06 |
| Crop growth (color layers) | FE4 Turnip/Potato | FRM-09 |
| Sleep at bed (hold E) | D1–D3 | CYC-03 |
| Forge craft (instant) | G1–G5 | FRG-01/02 |
| Carve (hold) | H17 | CMB-09 |
| Block / parry / Charge / Spin / hurt / idle | H1, H7, H28–H31 + smears CV1 | CMB-01/02 |
| Iron pickaxe (uses the T1 frames) | C2 + clank C9 | MIN-22 |
| Ore node hits (no crack states) | E1/E2 | MIN-02 |
| Zone transitions (fade only) | W1/W2, ladders | WLD-04, MIN-12 |
| Farm animals (third-party, no interactions) | in-house sets | ANM-01 |

### 5.2 Global presentation sets owned here

| # | Set | Frames | Size | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| AP1 | Dwarf back-facing walk + front walk + front idle (ART-02 decision) | 4 + 4 + 4 | 16×28 | 0.1 / 0.2 | unlocks vertical-facing actions for doors, counters, ladders |
| AP2 | Screen transitions: iris wipe, steam wipe (zone change), page flip (day end, owned by 11) | 8 / 8 | 320×180 | 0.04 | drawn |
| AP3 | Hunt-mode lighting overlays per ground (rim-light color grade drawn as gradient sprites) | 1–2 per ground | 320×180 | static/0.3 | "dramatic lighting" without shaders |
| AP4 | Parallax backgrounds per hunting ground (3 layers each) | 3 × 8 grounds | 640×180 | static, cloud layers 0.2 | |
| AP5 | Art-style test boards (GDD §11): farm vs. hunt side-by-side mockups | 2 boards | 960×540 | — | the first deliverable of ART-06 |

## 6. Inventory items

Icons for every item in docs 01–14 follow §3.1 (16×16, atlas). Quality star overlays (02), rarity frames (04), element/status icons (04/05) are shared atlases.

## 7. Processing (art pipeline steps)

1. Spec (from the system doc's animation table) → 2. draw script / hand PNG → 3. `sprites.ron` entry → 4. `art_lint.py` passes → 5. contact sheet in the PR → 6. a [GAME] ticket wires it → 7. in-game xvfb screenshot check (existing practice in PR #96).

## 8. Hooks

Every doc's [ART] tickets depend on ART-01 (helpers), ART-03 (registry), ART-04 (icon atlas) where relevant.

## 9. Energy / time / risk

Risk: art volume (≈ 11k frames) is the schedule risk. Mitigations: modular rigs (06 Q1), palette-baked tiers (01/02/03), season bakes (10 Q3), launch scopes (8 monsters, 3 trees per weapon, 8 villagers first).

## 10. Progression / gating

Art lands per system in ticket order. No player-facing gating.

## 11. UI

Covered by 13 (theme, icons). Here: icon atlas and the portrait spec.

## 12. Audio

Frame-sync bindings (14 §5.1) read hit/contact frames from `sprites.ron`.

## 13. Save data

None (presentation only).

## 14. Balance levers

Per-frame seconds in `sprites.ron` (animation speed tuning without code), hit-frame indices, canvas pins.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Up/down facings for the dwarf? | **Add back + front walk/idle (AP1)** and use them for vertical-only actions (doors, counters, ladders, boards, beds). Keep tool actions side-only with flip (as shipped) to contain frame counts. |
| Q2 | "Grit-neon" hunter vs. cozy steampunk | Keep it: it matches GDD "hunts feature dramatic lighting". Add brass/copper accents to new hunter weapon sheets to tie them to the world. |
| Q3 | Replace third-party art (0x72 dwarf base, Pixel Crawler, Dawnlike)? | Gradually. Animals first (ANM-42), then the anvil/workbench station art with the forge level art (FRG-30). The dwarf base stays (all in-house frames derive from it). |
| Q4 | One PNG per frame (dwarf) vs strips? | New sets use **strips** + `sprites.ron`. Keep the existing per-frame dwarf files and migrate them in ART-03 without changing pixels. |
| Q5 | Resolution / scale policy | Native pixel art, nearest-neighbor, integer scale. Follow the 3× homestead scale branch for life mode. |

## 16. Ticket breakdown

- **ART-01 [ART]** `tools/art_common.py` (palette, outline, canvas/pin, layer composite, strip pack, contact sheet) + refactor the existing draw scripts to use it with byte-identical outputs. deps: none.
- **ART-02 [ART]** Dwarf back/front walk + front idle (AP1) + decision record in `docs/`. deps: ART-01.
- **ART-03 [GAME]** `assets/data/sprites.ron` registry + loader (cell, frames, seconds, pins, hit frames, loop), migrating existing sets (dwarf, hunter, enemies) with behavior unchanged. deps: none.
- **ART-04 [ART]** Icon atlas pipeline (`tools/pack_icons.py`, `icons.ron`) + first atlas (existing MaterialIds). deps: ART-01; pairs with UI-03/MIN-05.
- **ART-05 [ART]** `tools/art_lint.py` + test that every `sprites.ron` entry matches its PNG dimensions/frame counts. deps: ART-03.
- **ART-06 [ART]** Art-style test boards: farm vs. hunt (AP5), the GDD §11 next step. deps: ART-01.
- **ART-07 [ART]** Master palette file + style guide page (`docs/ART_STYLE.md`). deps: ART-06.
- **ART-08 [GAME]** Wire the dwarf back/front facing into movement (vertical input picks the AP1 sets) + doors/ladders use them. deps: ART-02, ART-03.
- **ART-09 [ART]** Screen transitions AP2 (iris, steam wipe). deps: ART-01.
- **ART-10 [GAME]** Transition system uses the AP2 wipes for zone changes. deps: ART-09, WLD-11.
- **ART-11 [ART]** Hunt-mode lighting overlays + parallax for the first ground (Outskirts) AP3/AP4. deps: CMB-20.
- **ART-12 [ART]** Contact-sheet/xvfb screenshot helper script for PR evidence (generalizes `pr96-xvfb` practice). deps: ART-01.
- **ART-13 [ART]** Third-party replacement plan execution: anvil/workbench in-house (with FRG-30) + attribution cleanup. deps: FRG-30.

**Ticket count: 13.**

**Build first (top 3):** ART-01 (shared draw helpers), ART-03 (sprite registry), ART-02 (dwarf back/front facing decision + frames). All three precede the system art tickets.
