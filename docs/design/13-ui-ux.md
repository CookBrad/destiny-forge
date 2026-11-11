# 13 — UI / UX (inventory, crafting with material sources, monster field guide, gear compare, remapping, combat assists)

Standards: inventory and crafting screens are clear and show where materials come from. A monster field guide fills in as the player researches or breaks parts. A simple gear comparison tool is informative. Controls are accessible, with remapping and optional simplified combat assists.

Build priority: **13 of 16**. UI tickets from other docs (forge window v2, fishing minigame, dialogue) are owned there. This doc owns the shared shells and components they use.

## 0. Standing rules

UI is a presentation layer. **UI feedback never substitutes for a player action's sprite frames** (an eat banner + eat frames, both). All UI art is in-house pixel art (9-slice brass frames, icons, cursors), not Bevy default rectangles/colored swatches once the matching ART ticket lands. No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

| Exists | Where |
| --- | --- |
| Title + profile picker + rename + quick start 1–3 | `ui/menu.rs`, `profile_picker.rs`, `title_profiles.rs` |
| Pause menu (I inventory, Esc resume, Q return home) + pause audio | `ui/menu.rs`, `pause_audio.rs` |
| Inventory window 32 slots, sort, swatch + text icons, coin footer placeholders | `ui/inventory_window.rs` (PR #96) |
| Forge window (recipe cycle) | `ui/forge_window.rs` |
| HUDs: day, tool energy, health bars, skill bar (bindable slots 1–9 via `SkillBindings`), loadout strip, food buff chip, carve feedback, eat feedback popup, interaction prompt | `ui/*` |
| Text-only control hints on the title screen | `ui/menu.rs` |
| Keyboard + mouse only; no gamepad code | — |

## 2. Where it happens

Global overlay layer in all states. Screens: Title, Pause hub (tabs), Inventory, Crafting stations (Forge/Workbench/Kitchen/Smelter/Jeweler/machines), Gear Stash & Compare, Field Guide (Monsters, Fish, Crops, Materials, Museum), Quest Log/Journal, Relationships, Calendar, World Map, Settings (Controls/Remap, Accessibility & Assists, Audio, Video), Shop, Dialogue, Hunt HUD, Reward screen, Shipping summary.

## 3. Shared components (entities)

`UiTheme` (brass 9-slice frames, fonts, colors), `ItemIcon` (PNG icon + quality star + count + fallback swatch), `Tooltip` (name, category, quality, stats, buffs, sell price, **sources**), **`MaterialSourceList`** (data-driven "where to get this": monster + method (carve/break part/capture/quest), node + mine floors, crop + season, fish + spot/time/weather, machine, shop; known sources only (seen), with "???" for undiscovered), `StatDelta` (green/red arrows), `TabBar`, `GridList` (keyboard/gamepad navigable), `Toast` (drawn frame queue), `ConfirmDialog`, `HotkeyGlyph` (shows the current binding for an action, keyboard or gamepad).

`InputMap` (new): action-based input (`Action::{MoveUp…, Interact, UseTool, Attack, Special1…, Evade, Guard, ItemUse, ItemCycle, Map, Inventory, Pause, …}`) → bindings for keyboard, mouse, and gamepad. This replaces direct `KeyCode` checks across systems (they currently hard-code KeyE/Escape/Digit1–9/WASD/Space).

## 4. Screens (requirements)

### 4.1 Inventory
- 32 slots (exists) + 8-slot toolbar (02 FRM-05) + separate pouches: Hunt Items (05, 24), Ore Satchel (01), Decorations (04), Key Items, Gear Stash (04).
- Icons (PNG), quality stars, stack counts, category filter tabs, sort (exists), drag/drop + keyboard move, split stack, trash (confirm), quick-stack to nearby chest (storage chests: new craftable, Workbench).
- Tooltip with **sources** and **uses** ("Used in: Iron Sword, Steam Sprinkler…").

### 4.2 Crafting screens (shared shell)
- Left: recipe list grouped by category with locks (silhouettes for unseen). Middle: selected recipe with components (icon, owned/needed colored), role glyphs (04), and **MaterialSourceList per missing component**. Right: output preview + stat compare (gear) or buff summary (food/potions). Bottom: Craft (hold to confirm for expensive items), quantity for consumables, **"Track"** pin that shows the missing materials on the HUD.

### 4.3 Monster Field Guide
- Pages per monster: portrait (silhouette → revealed after the first sighting), habitat (zones, season, time), threat rank, size records, hunts/captures counts. **Weakness table** (element/damage-type stars) revealed by: hunting 2×, or a Quillon research quest, or Professor 6♥. **Parts list**, each revealed when that part is first broken, with its break rewards. Drops by source (revealed when obtained). Tells hints (unlocked after 3 hunts: "Paws twice before charging").
- Tabs also for Fish (03), Crops (02), Materials (all MaterialIds: sources/uses), Museum (01).

### 4.4 Gear compare
- In the Gear Stash and Forge: side-by-side current vs. candidate: attack/defense, element/status, affinity, sharpness bar, slots, skills (level deltas incl. set bonus thresholds), resistances. Summary line: "+12 attack, −Fire Resist 2, gains Weakness Exploit 1". Loadout sets (save 5 loadouts named by matchup; 09 endgame).

### 4.5 Remapping
- Settings → Controls: list of actions per context (Homestead, Hunt, Menus) with keyboard and gamepad columns, rebind with a conflict warning, reset to defaults, per-profile save (bindings exist per profile for skill slots, generalize them).
- Gamepad support (Bevy gamepad input): default layout for Xbox/PlayStation glyph sets.
- Hold vs. toggle options (charge, guard, sprint).

### 4.6 Combat & life assists (accessibility)
| Assist | Effect | Default |
| --- | --- | --- |
| Simplified combos | one-button combo strings (auto-advance) | off |
| Auto-guard window | perfect guards on a wider window (0.15 → 0.3 s) | off |
| Hold-to-charge assist | auto-release at max charge | off |
| Target lock | camera/facing lock on a large monster | off |
| Telegraph boost | monster tells ×1.2 longer + an outline flash (drawn) | off |
| Show monster health | HP bar visible | off (05 Q5) |
| Slower enrage | enrage damage ×1.0 | off |
| Relaxed fishing | 03 | off |
| Forge minigame off | 04 | off |
| Text size / dyslexia-friendly font | 3 sizes | M |
| Colorblind palettes for element/status icons + quality stars (shape-coded) | 3 modes | off |
| Screen shake / flash reduction | | on (reduced flash) |
| Hold-E durations (carve, sleep) → tap-toggle | | off |

### 4.7 HUD
Homestead: clock/season/weather (11), energy, gold, toolbar, quest tracker (pinned), buff chips. Hunt: HP, stamina, sharpness, weapon gauge, status icons, pouch quick-bar (cycle with a scroll/bumpers), faints, timer, track gauge, minimap. Toasts: item gained, part broken, level up, heart up, quest updated (drawn frames, queued, max 3).

## 5. ANIMATIONS (most detailed section)

UI art is pixel art at 1× native, shown at 3× (matching the item icon convention 16×16 → 48 px slots). Brass theme. Animated UI elements are drawn frame strips (no tween-only color fades for key feedback; tween **movement** is fine).

### 5.1 UI animation sets

| # | Set | Frames | Size | Timing | Loop | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| U1 | Window open (brass shutters slide + rivet glint) | 6 | 9-slice frame | 0.04 | no | all main windows |
| U2 | Window close | 4 | — | 0.04 | no | |
| U3 | Tab switch gear-click | 4 | 32×16 | 0.04 | no | |
| U4 | Slot hover glow | 3 | 18×18 | 0.1 | yes | |
| U5 | Slot select pulse | 4 | 18×18 | 0.08 | yes | |
| U6 | Item pick-up wiggle (drag) | 3 | 16×16 | 0.06 | yes | |
| U7 | Item gained toast slide + icon pop | 5 | 128×24 | 0.05 | no | |
| U8 | Craft success burst (UI side; the world-side FRG anim is separate) | 6 | 48×48 | 0.05 | no | |
| U9 | Missing-material pulse (red rivet) | 3 | 8×8 | 0.15 | yes | crafting |
| U10 | Field Guide page turn | 6 | 320×200 | 0.05 | no | |
| U11 | Field Guide reveal stamp (part/weakness discovered) | 5 | 48×48 | 0.06 | no | the "fills in" moment |
| U12 | Silhouette → portrait reveal | 6 | 64×64 | 0.08 | no | |
| U13 | Stat delta arrows (up/down) bounce | 3 each | 8×8 | 0.1 | yes | gear compare |
| U14 | Cursor (brass gauntlet pointer) idle + click | 2 + 2 | 16×16 | 0.4 / 0.05 | | |
| U15 | Tile target cursor (homestead) normal/invalid/area | 2 ×3 | 32×32 | 0.3 | yes | 02 |
| U16 | Interaction prompt "E" key glyph bounce (per binding glyph) | 3 | 16×16 | 0.15 | yes | replaces text-only prompt |
| U17 | Rebind "press a key" pulse | 4 | 64×16 | 0.15 | yes | |
| U18 | Loading/autosave gear spinner | 8 | 16×16 | 0.06 | yes | 16 |
| U19 | Title screen: forge glow + steam + logo shine | 6 + 6 + 8 | 320×180 / logo 192×64 | 0.1 | yes | |
| U20 | Pause hub tab icons idle | 2 each ×8 | 16×16 | 0.4 | yes | |
| U21 | Toast frames: part broken, heart up, level up, quest | 4 each | 160×24 | 0.05 | no | drawn |
| U22 | Pouch quick-bar cycle | 4 | 64×24 | 0.04 | no | hunt HUD |
| U23 | Low HP / low stamina border pulse | 4 | 9-slice edge | 0.12 | yes | drawn vignette frames |
| U24 | Track pin icon | 2 | 16×16 | 0.4 | yes | |

### 5.2 Static UI art sets

9-slice frames (window, tooltip, button normal/hover/pressed/disabled, tab, slot, slot locked, header plate); fonts (pixel font 5×7 + 7×9, digits, outline variants, dyslexia-friendly alt); icon sets (category ×12, input glyphs keyboard ×60 / Xbox ×24 / PlayStation ×24, assist icons ×12, settings icons ×10); colorblind-safe shape overlays for elements/quality (circle/triangle/square/diamond/star).

## 6. Inventory items

Storage Chest (Workbench: 50 Wood + 1 Copper Bar), Big Chest (Iron), stack 1 each, placeable. No other items.

## 7. Processing

N/A. The "Track" pin in crafting watches inventory counts.

## 8. Hooks

Every system exposes data the shared components read: `MaterialSourceList` needs source data from carve/break/capture tables (05/06), node tables (01), crops (02), fish (03), machines (02/07), shops (09), plus `seen_materials` (04). The Field Guide reads 06/03/02/01 save data. Gear compare reads 04 skills/stats. Assists hook into 05/03/04. InputMap is used by every system.

## 9. Energy / time / risk

Time pauses in menus (11 Q1). No energy.

## 10. Progression / gating

Field Guide tabs unlock as systems unlock. Assists are always available. Remap is always available.

## 11. UI

(this document)

## 12. Audio

ui_open/close (brass shutters), tab_click, hover_tick, select_confirm, cancel, error_buzz (missing materials), item_pickup/drop, craft_confirm, page_turn, reveal_stamp, toast_pop per type, rebind_ok, autosave_tick. A separate UI volume slider (14).

## 13. Save data

Profile: `input_bindings` (generalize `skill_bindings`), `assists`, `ui_prefs` (text size, colorblind mode, flash reduction, minimap), `tracked_recipe`, `loadout_presets`, `seen_materials` (04), `field_guide` (06). Global settings: last profile (exists), video options.

## 14. Balance levers

Tooltip verbosity, assist magnitudes (guard window, telegraph ×), toast durations, track pin limits.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Bevy UI or immediate-mode? | Keep Bevy UI (existing pattern). Build the shared components in `src/ui/components/*`. |
| Q2 | Gamepad at launch? | Yes. It is cheap with Bevy, and the Steam audience expects it. |
| Q3 | Inventory icons from swatches → PNG | Fallback swatch until each item's icon ships (MIN-05 pattern). |
| Q4 | Where does source data live? | Derived from the existing data files (carve_loot.ron, nodes.ron, crops.ron, fish.ron, recipes.ron) by a `MaterialSources` index built at load. No hand-written duplicate list. |
| Q5 | Assists affect rewards? | No (accessibility, cozy). |

## 16. Ticket breakdown

- **UI-01 [GAME]** `InputMap` action layer (keyboard + mouse), migrate the existing hard-coded KeyCode checks with behavior unchanged + tests. deps: none.
- **UI-02 [ART]** Brass UI theme: 9-slice frames, buttons, tabs, slots, pixel fonts, cursor U14. deps: none.
- **UI-03 [UI]** Shared `ItemIcon` (PNG + quality star + count + swatch fallback) used by the inventory/forge. deps: none (MIN-05 implements the first cut; merge into this).
- **UI-04 [UI]** Apply the UI theme to existing windows (inventory, forge, pause, title) + U1/U2 open/close. deps: UI-02, UI-05.
- **UI-05 [ART]** UI animation sets U1–U9, U13, U16 (core). deps: UI-02.
- **UI-06 [GAME]** `MaterialSources` index derived from the data files + `seen_materials` tracking. deps: FRG-40 (seen tracking, or implement here).
- **UI-07 [UI]** `Tooltip` with sources + uses, used by the inventory and crafting. deps: UI-03, UI-06.
- **UI-08 [UI]** Crafting screen shell (list / components with sources / preview / craft / Track pin). deps: UI-07; FRG-10 adopts it.
- **UI-09 [UI]** Track pin HUD (missing materials). deps: UI-08, U24 art (in UI-05).
- **UI-10 [UI]** Pause hub with tabs (Inventory, Gear, Field Guide, Quests, Relationships, Calendar, Map, Settings) + tab icons U20. deps: UI-02.
- **UI-11 [ART]** Field Guide art: book frame, page turn U10, reveal stamp U11, silhouette reveal U12. deps: UI-02.
- **UI-12 [UI]** Field Guide shell + Monster tab (silhouette, habitat, weakness reveal rules, parts reveal on break, drops). deps: UI-11, MON-01, CMB-29.
- **UI-13 [UI]** Field Guide Materials tab (every MaterialId: sources/uses). deps: UI-12, UI-06.
- **UI-14 [UI]** Gear compare component (StatDelta, skills diff, summary line) + Gear Stash screen. deps: FRG-13, UI-05.
- **UI-15 [UI]** Loadout presets (5, named). deps: UI-14.
- **UI-16 [UI]** Settings: Controls remap (keyboard) with conflicts/reset + per-profile save + glyph art U17. deps: UI-01.
- **UI-17 [GAME]** Gamepad support in InputMap + default layouts + glyph sets (art in this ticket's ART pair UI-18). deps: UI-01.
- **UI-18 [ART]** Input glyph sets (keyboard/Xbox/PS) + HotkeyGlyph prompt U16 per glyph. deps: none.
- **UI-19 [UI]** Accessibility & assists settings page + hooks API (assist flags resource consumed by 03/04/05). deps: UI-10.
- **UI-20 [GAME]** Combat assists implementation (simplified combos, auto-guard, charge assist, target lock, telegraph boost, show HP, slower enrage). deps: UI-19, CMB-45, MON-04.
- **UI-21 [UI]** Text size + dyslexia font + colorblind shape overlays + flash reduction. deps: UI-02, UI-19.
- **UI-22 [UI]** Toast system (queued drawn frames U7/U21) replacing ad-hoc popups (keep the eat popup behavior). deps: UI-05.
- **UI-23 [UI]** Hunt HUD layout (HP/stamina/sharpness/gauge/status/pouch bar/faints/timer) + U22/U23. deps: CMB-08, CMB-58.
- **UI-24 [UI]** Inventory v2: pouches, category tabs, drag/drop, split, trash, quick-stack. deps: UI-03, UI-04.
- **UI-25 [GAME]** Storage chests (placeable, UI shares the inventory grid). deps: UI-24, FRG-23.
- **UI-26 [ART]** Title screen animation U19 + logo. deps: none.
- **UI-27 [AUD]** UI sound set + UI volume slider. deps: UI-04, AUD-01.
- **UI-28 [SAVE]** Persist bindings, assists, UI prefs, tracked recipe, presets. deps: UI-16, UI-19.

**Ticket count: 28.**

**Build first (top 3):** UI-01 (InputMap: unblocks remapping and gamepad, and stops new hard-coded keys), UI-02 → UI-03/UI-04 (theme + icons on existing windows), UI-06 → UI-07 (material sources + tooltip, the headline UI requirement).
