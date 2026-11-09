# 05 — Combat & Hunting

Standards: combat is Monster Hunter style against large magical creatures. Weapon types have distinct movesets, commitment and strengths, and the launch set is **Great Sword, Long Sword, Sword & Shield, Dual Blades, Hammer and Bow**. A hunt goes: accept a quest or free-hunt, track by clues (tracks, magical residue, roars, elemental signs), fight (read tells, break parts, apply status, use environmental traps), handle phases, elemental shifts and enrage, then carve or capture. Fainting has limited retries per hunt. Combat is built on readable mastery. Hunts run from afternoon to night or as multi-day expeditions. Four to five weapon types are must-have; capture is should-have. Monsters themselves are in 06.

Build priority: **5 of 16**.

## 0. Standing rules

- **Every hunter action gets new in-house frames on the side-scroll hunter (knight) sheets**, drawn by `tools/draw_hunter_frames.py`-style scripts. Today **Block, Parry, Charge, Spin, Thrust special, Carve, and hurt have no dedicated hunter frames**: `HunterPose` only has Idle/Run/Jump/Fall/Attack/Death, and the idle sheet shows one cell. Each of these needs an [ART] ticket.
- Arcs/slashes are drawn **smear frames inside the hunter cell** (or a drawn VFX sprite), never tinted geometric arcs.
- No code comments; zero warnings. **Dungeon platform/art churn is frozen**: Floor 1 / King Slime stay exactly as they are. New hunting grounds are new zones with their own art under `assets/hunt/<ground>/`. Hunter sheets (`assets/player/combat/`) are not dungeon platform art and may grow new sheets. The existing six PNGs stay byte-identical. That includes the **Taste-signed cells** (signed idle, run lean, attack chamber, attack thrust; see `assets/ATTRIBUTION.txt`), which must never be redrawn. New sets are new files built from the signed body, in the same grit-neon hunter style.
- Sprite/animation work is in separate [ART] tickets.

## 1. Already exists (do not redo)

| Exists | Where |
| --- | --- |
| Side-scroll combat in the dungeon: hunter cells **343×160**, sheets idle(4 identical cells, 1 shown), run(9, cells 1–8 cycle @0.08), attack(4: chamber/thrust/between/recover), jump(1), fall(1), death(2) | `dungeon/sprites.rs`, `hunter_pose.rs`, `graphics::HUNTER_CELL_PX` |
| Combo system `ComboStep {duration, hit_start, …}`, `HitShape {SwordArc, SpearThrust, SpearLunge}`, families Sword/Spear, 4 weapons | `combat/weapon.rs`, `attack.rs` |
| Block + parry, hit stop, hitboxes, projectiles, health, hurt, death (1.45 s knockback) | `combat/*.rs` |
| Specials Charge (0.4 s dash, 2 s CD), Spin (0.5 s), Thrust (spear) + skill bar UI with 4 icons | `special_moves.rs`, `skills.rs`, `ui/skill_bar.rs` |
| Floor 1 procedural dungeon + enemies slime/bat/goblin/skeleton/zombie (4-frame sheets) + King Slime boss with phases and ground slam | `dungeon/*` |
| Hold-to-carve (2.0 s base, interruptible), per-monster carve loot RON with bonus rolls, carve feedback UI | `dungeon/carve*.rs`, `ui/carve_feedback.rs` |
| Hunt day cost: 2 phase steps | `core/hunt_day_cost.rs` |
| Food buffs to attack/defense/carve speed | PR #96 |
| Health bars | `ui/health_bars.rs` |

## 2. Where it happens

- **Hunting Grounds** (new, side-scroll, multi-area). Each ground is 5–7 connected side-scroll **areas** (≈ 3–4 screens wide) joined by edge exits, with a **Base Camp** (area 1: supply box, gear box, tent bed, quest return). They are separate from the frozen dungeon.

| Ground | Zone (10) | Rank | Large monsters (06) | Environment traps |
| --- | --- | --- | --- | --- |
| Hearthspire Outskirts (training) | Village edge | Low | Bristleboar (village threat) | hay bales (fire), bell tower |
| Misty Forest Grounds | Misty Forest | Low/High | Thornback Briarwolf, Glowmoth Matriarch | vine snares, spore pods, deadfall logs |
| Riverlands Dam | Riverlands | Low/High | Tidecaller Kraken-Eel, Ironhide Behemoth (dam works) | sluice gate flood, dam turbines |
| Clockwork Galleries Deep (side-scroll mine ground, not the top-down mine) | Mine | High | Gearjaw Drillwyrm | steam pipes, ore-cart rail |
| Highland Ruins | Ruins | High | Stormwing Gryphon, Runeback Golem | collapsing pillars, rune pylons |
| Ember Caldera | Ruins edge | High/Master | Magmaw Salamander | lava vents, boulders |
| Floating Islets | Islets | Master | Prism Leviathan Ray, Aether Drake (Elder) | wind currents, crystal spires |
| Destiny Crater (finale) | — | Master | Elder Destiny Wyrm | — |

- **Floor 1 dungeon + King Slime** stays as the "village threat" tutorial hunt. It is unchanged and is offered as quest "Slime Season".
- **Top-down field combat-lite** (mine floors 01, forest small monsters) uses the dwarf avatar (CMB-04).

## 3. Entities and objects

Hunter (exists) + `Stamina` (new combat stamina, separate from tool energy), `Sharpness`, `WeaponState` (per-type resource: spirit gauge, demon gauge, charge level, bow charge/coating), `Evade` (i-frames), `StatusEffects` (player), `ItemPouch` (hunt items, 24 slots), `HuntQuest`, `HuntSession {quest, faints, time_limit, objectives}`, `TrackClue {kind, monster, value}`, `TrackGauge`, `MonsterPart {id, hp, broken, severable}`, `MonsterState {phase, enraged, element_mode, fatigue, limping}`, `StatusBuildup {poison, paralysis, sleep, ko, exhaust, blast}`, `Trap {kind, armed}`, `EnvTrap`, `BaseCamp`, `SupplyBox`, `Tent`, `Companion` (08), `CarveSpot` (exists; now per monster corpse with carve count), `CaptureState`, `RewardScreen`.

## 4. Weapons (launch subset, movesets)

Shared: draw/sheathe, walk drawn, evade roll (0.2 s i-frames), item use sheathed (SnS also drawn), sharpen (melee), knockdown/get-up, hurt, block (GS/SnS guard). Stamina: evade 25, guard chip 10, charge hold 10/s, DB demon 15/s, Bow charge 12/s. Max 100, regen 25/s.

| Weapon | Resource | Core moves | Strength | Status |
| --- | --- | --- | --- | --- |
| **Sword & Shield** (existing Sword family) | — | Chop → Side Slash → Lateral → Rising (existing combo); Shield Bash → Hard Bash (KO); Backstep → Perfect Rush; Guard; item use while drawn; Charge (existing special) → "Steam Dash"; Spin (existing) → "Spinning Rush" | fast, KO from bash, item use | exists as 3-step combo, needs frames for bash/guard/backstep |
| **Great Sword** | charge level 0–3 | Draw Slash; Charged Overhead L1–3 (hold, 0.5/1.0/1.5 s); Wide Slash; Tackle (hyper armor during charge); Strong Charge; True Charged Slash (L3 finisher); Guard (blade) | massive single hits, part break | new |
| **Long Sword** | Spirit Gauge 0–100 + level (white/yellow/red) | Vertical → Thrust → Rising; Fade Slash; Spirit Blade 1-2-3 → Spirit Roundslash (level up); Foresight Slash (counter, 0.3 s window, i-frames) | counters, mobility | new |
| **Dual Blades** | Demon Gauge 0–100 | Double Slash → Lunging → Rising; Demon Mode (drains stamina, faster/stronger); Demon Dance; Demon Dash (evade); Archdemon Mode (gauge full, sheathe-less) | status application, evasion | new |
| **Hammer** | charge level 0–3 | Overhead 1-2 → Upswing; Charged Side Blow L1, Upswing L2, Big Bang/Spinning Bludgeon L3; Focus (hold while moving) | KO (head), head breaks, stun | new (EPICS #34) |
| **Bow** | charge level 1–3 + coatings | Nock/Draw; Shot L1–3 (rapid/spread/pierce by bow row); Arc Shot (up-angle volley); Power Shot; Dodge-Shot (charge+1); Coating swap (Blaze/Frost/Shock/Poison/Sleep/Para/Power/Close) | elemental, ranged, safe distance | new; reuses the projectile system |
| Spear (legacy) | — | keep existing Poke/Thrust/Lunge | — | kept, not expanded (04 Q2) |

Side-scroll adaptation: attacks are left/right with up-variants (rising/arc). Jump attacks exist; a jump-attack "mount" is post-launch (Q4).

## 5. ANIMATIONS (most detailed section)

Conventions: hunter **cell 343×160**, sole line at row 159 (existing `IDLE_SOLE_GAP`/`FOOT_PIN_X` rules), body pinned to `FOOT_PIN_X = 81.5`. **Authored facing right; left = `flip_x`** (existing `apply_facing`). Horizontal strip PNGs, one per set, `knight_<set>_side.png`, cell count = frames. Weapon is drawn in the frames (each weapon type is its own sheet set). The **tier/row look** of a weapon = a separate weapon-layer PNG per row family, composited by the draw script into baked sheets (Q2). Timing is per frame. Hit = active frames. Cancel = earliest frame that can be interrupted.

### 5.1 Hunter shared sets (all weapons)

| # | Set | Frames | Timing (s/frame) | Loop | Active / i-frames | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| H1 | Idle breathing | 4 | 0.18 | yes | — | **replaces the 1-shown-cell idle**; per weapon type sheathed back silhouette (7 variants) |
| H2 | Run (sheathed) | 8 | 0.08 (exists) | yes | — | exists. Per-weapon back silhouette baked = 6 new variants |
| H3 | Walk drawn (per weapon) | 8 | 0.10 | yes | — | ×6 weapons |
| H4 | Draw weapon | 4 | 0.06 | no | f2 = drawn | ×6 |
| H5 | Sheathe | 4 | 0.08 | no | — | ×6 |
| H6 | Evade roll | 6 | 0.05 | no | f1–f4 i-frames (0.2 s) | shared body; weapon on the back drawn per type ×6 |
| H7 | Hurt flinch (light) | 3 | 0.06 | no | — | white silhouette frame as f0 (drawn, not tint) |
| H8 | Knockdown + lie + get-up | 4 + 1 + 4 | 0.07 / hold 0.4 / 0.08 | no | get-up f0–f2 i-frames | |
| H9 | Launched (big hit) air tumble | 4 | 0.06 | yes until land | — | lands into H8 lie |
| H10 | Faint / cart | 6 | 0.1, then fade | no | — | existing death sheet (2) kept for the dungeon; hunts use H10 then the cart-out sequence H11 |
| H11 | Carted out (clockwork stretcher bots carry the hunter) | 6 | 0.12 loop | yes | — | entity + hunter composite |
| H12 | Drink potion | 6 | 0.1, 0.1, 0.2, 0.2, 0.15, 0.15 | no | f5 heal applies; "flex" f4–f5 | sheathed |
| H13 | Eat field ration | 6 | 0.12 | no | f4 | |
| H14 | Sharpen (whetstone) | 8 | 0.12 (3 strokes) | no | f7 restores | ×5 melee weapon types |
| H15 | Throw (bomb/pod/tranq/paintball/tracker) | 5 | 0.06, 0.06, 0.04, 0.08, 0.1 | no | f2 spawn | |
| H16 | Place trap / barrel bomb | 6 | 0.1 | no | f4 trap armed | |
| H17 | **Carve** (crouch + knife saw loop + pull) | 2 + 4 loop + 2 | 0.1 / 0.12 / 0.1 | loop | end = loot | **currently no frame for the existing carve** |
| H18 | Capture-sack (after capture) | 4 | 0.12 | no | — | |
| H19 | Gather (herb/bug/bone node in a hunt) | 4 | 0.1 | no | f3 | |
| H20 | Inspect track clue (crouch + look) | 4 | 0.12 | no | f3 gauge tick | |
| H21 | Tracker moth release (raise hand) | 3 | 0.1 | no | f2 | |
| H22 | Climb ledge / ladder (side) | 6 | 0.08 | yes | — | grounds have ledges; do not alter dungeon ladders |
| H23 | Ledge hang + pull-up | 2 + 4 | 0.1 | — | — | |
| H24 | Slide (slope) | 2 | 0.1 | yes | — | |
| H25 | Status: poisoned (green bubbles drawn on body), burning (stop-drop-roll 6), frozen (ice-encased 2), paralyzed (twitch 2), stunned (stars 4), sleeping (doze 2), soiled/slimed (2) | 2–6 each | 0.1–0.2 | yes | burning roll f0–f5 i-frames off | 7 sets |
| H26 | Victory / quest-clear pose | 4 | 0.15 | no | — | per weapon ×6 |
| H27 | Wave/signal (companion command) | 3 | 0.1 | no | — | |
| H28 | Block (existing mechanic, no frames today) generic shield/arm | 3 (raise, hold, chip-recoil) | 0.05/hold/0.06 | hold | — | for SnS; GS/Hammer use their own |
| H29 | Parry success (existing mechanic) | 4 | 0.04 | no | — | |
| H30 | Charge dash special (existing) | 4 | 0.1 | no | f0–f3 active | **no frames today** |
| H31 | Spin special (existing) | 6 | 0.083 | no | f1–f5 active | **no frames today** |

### 5.2 Sword & Shield sheet (`knight_sns_*`)

| Move | Frames | Timing | Active | Cancel | Notes |
| --- | --- | --- | --- | --- | --- |
| Chop (combo 1) | 5 | 0.05 | f2–f3 | f4 | the existing attack strip (4) is the base; add a smear frame |
| Side Slash (2) | 5 | 0.05 | f2–f3 | f4 | |
| Lateral Slash (3) | 6 | 0.05 | f2–f4 | f5 | |
| Rising Slash (up) | 5 | 0.05 | f2–f3 | f4 | |
| Shield Bash | 5 | 0.06 | f2 | f4 | KO value 15 |
| Hard Bash (follow) | 6 | 0.06 | f3 | f5 | KO 30 |
| Backstep | 4 | 0.05 | — (f0–f1 i-frame) | f3 | |
| Perfect Rush (3-hit after backstep) | 10 | 0.045 | f2, f5, f8 | f9 | |
| Jump slash (air) | 4 | 0.05 | f1–f2 | — | |
| Guard (raise/hold/chip) | 3 | | | | H28 SnS variant |
| Item use drawn | 6 | 0.1 | | | SnS-only, uses the shield hand |
| Steam Dash (Charge special) | 4 | 0.1 | f0–f3 | — | replaces generic H30 for SnS |
| Spinning Rush (Spin special) | 6 | 0.083 | f1–f5 | — | |

### 5.3 Great Sword sheet (`knight_gs_*`)

| Move | Frames | Timing | Active | Notes |
| --- | --- | --- | --- | --- |
| Draw Slash | 7 | 0.07 | f4–f5 | |
| Charge hold (L1/L2/L3 shown by a drawn blade glow per level) | 3 base + 2 loop per level = 9 | 0.1 / 0.12 loop | — | level change at 0.5/1.0/1.5 s; flash frame per level |
| Charged Overhead release | 6 | 0.05 | f2–f3 | damage ×1.0/1.4/1.9/2.4 by level |
| Wide Slash | 7 | 0.06 | f3–f4 | |
| Tackle (hyper armor) | 4 | 0.06 | f1–f2 | |
| Strong Charge hold/release | 9 | as above | | |
| True Charged Slash (2-hit) | 10 | 0.06 | f3, f7 | |
| Guard (blade) | 3 | hold | | chip damage |
| Jump slash | 5 | 0.06 | f2–f3 | |
| Unsheathe-run slash | 5 | 0.06 | f3 | |

### 5.4 Long Sword sheet (`knight_ls_*`)

| Move | Frames | Timing | Active | Notes |
| --- | --- | --- | --- | --- |
| Vertical | 5 | 0.05 | f2–f3 | |
| Thrust | 4 | 0.05 | f2 | |
| Rising | 5 | 0.05 | f2–f3 | |
| Fade Slash (back-hop) | 6 | 0.05 | f2–f3 | |
| Spirit Blade I / II / III | 6 / 6 / 8 | 0.05 | 2–3 hits each | the blade glow color per spirit level is drawn as 3 recolors baked |
| Spirit Roundslash | 9 | 0.05 | f4–f6 | levels the gauge up |
| Foresight Slash (counter) | 8 | 0.04 | i-frames f0–f3, hit f5–f6 | |
| Spirit level-up flash on sheathe | 4 | 0.06 | — | |
| Jump slash | 5 | 0.05 | | |

### 5.5 Dual Blades sheet (`knight_db_*`)

| Move | Frames | Timing | Active | Notes |
| --- | --- | --- | --- | --- |
| Double Slash | 5 | 0.04 | f1, f3 | |
| Lunging Strike | 5 | 0.045 | f2–f3 | |
| Rising Slash | 5 | 0.04 | f2 | |
| Demon Mode enter | 6 | 0.05 | — | red aura drawn (VFX CV6) |
| Demon idle / run | 4 / 8 | 0.12 / 0.07 | | separate sheets (arms crossed pose) |
| Demon Dance (6-hit) | 14 | 0.035 | 6 hits | |
| Demon Dash (evade) | 4 | 0.04 | i-frames f0–f2 | |
| Archdemon idle | 4 | 0.12 | | |
| Blade Dance (archdemon) | 10 | 0.04 | 5 hits | |

### 5.6 Hammer sheet (`knight_hammer_*`)

| Move | Frames | Timing | Active | Notes |
| --- | --- | --- | --- | --- |
| Overhead Smash I | 6 | 0.06 | f3 | KO 20 |
| Overhead Smash II | 6 | 0.06 | f3 | KO 20 |
| Upswing | 6 | 0.06 | f3 | KO 35, launches small monsters |
| Charge hold L1–L3 (walk while charging) | 3 + 4 walk-charge | 0.1 | — | head glow per level drawn |
| Charged Side Blow (L1) | 5 | 0.05 | f2 | |
| Charged Upswing (L2) | 6 | 0.05 | f3 | |
| Big Bang (L3, 3-hit) | 12 | 0.05 | f3, f7, f10 | KO 50 head |
| Spinning Bludgeon (L3 + move) | 8 loop + 4 finisher | 0.05 | each loop f2/f6 | |
| Focus Blow (Hammer variant of Charge special) | 6 | 0.06 | f4 | |

### 5.7 Bow sheet (`knight_bow_*`)

| Move | Frames | Timing | Active | Notes |
| --- | --- | --- | --- | --- |
| Draw + nock | 4 | 0.05 | — | |
| Hold aim (straight / up 30° / up 60°) | 2 ×3 | 0.12 loop | — | aim angle sets |
| Charge glow L1→L3 | 3 | per level | — | |
| Release | 3 | 0.04 | f0 spawns arrow(s) | |
| Arc Shot | 6 | 0.06 | f3 volley | |
| Power Shot | 7 | 0.05 | f3, f5 | |
| Dodge-Shot (roll + nock) | 6 | 0.05 | i-frames f0–f2 | +1 charge |
| Coating swap | 4 | 0.08 | — | |
| Melee stab (close) | 4 | 0.05 | f2 | |

### 5.8 Projectile / VFX / UI sprites for combat

| # | Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| CV1 | Slash smears per weapon (horizontal, rising, overhead) | 3 ×3 ×5 melee | 96×96 | 0.04 | **drawn smear sprites**, replacing the tinted-arc rule violations |
| CV2 | Hit sparks: cut (white), blunt (yellow ring), pierce (dot), crit (star burst), weak (dull) | 4 each | 48×48 | 0.04 | |
| CV3 | Element hit puffs ×8 (Fire/Water/Thunder/Ice/Aether + Poison/Para/Sleep) | 5 each | 48×48 | 0.05 | |
| CV4 | Part break burst + debris chunk | 7 + 3 | 96×96 | 0.05 | |
| CV5 | KO stars over the monster head | 4 | 32×16 | 0.1 loop | |
| CV6 | Demon aura (DB) | 4 | cell overlay 343×160 | 0.08 | drawn, not tint |
| CV7 | Spirit gauge flare (LS) white/yellow/red | 4 ×3 | 96×96 | 0.06 | |
| CV8 | Charge level flashes (GS/Hammer/Bow) | 3 ×3 | 64×64 | 0.05 | |
| CV9 | Arrows: normal + 8 coatings | 2 ×9 | 24×6 | 0.08 | **new hunt arrow set**; the dungeon `arrow.png` is not touched |
| CV10 | Bomb/barrel explosion | 7 | 96×96 | 0.05 | can share MIN V8 scaled art if allowed (Q6) |
| CV11 | Traps: Pitfall (arm 4, trigger 6, struggle loop 4), Shock (arm 4, active loop 4) | 22 | 64×32 | 0.08 | |
| CV12 | Tranq smoke | 5 | 64×64 | 0.08 | |
| CV13 | Flash Pod burst | 5 | 128×128 | 0.04 | |
| CV14 | Tracker moths (glowing trail) + clue sparkles (track, residue, scratch, shed scale, elemental scorch) | 4 + 5×3 | 16×16 | 0.1 | |
| CV15 | Paintball splat on monster (map reveal) | 3 | 16×16 | 0.08 | |
| CV16 | Healing sparkles | 5 | 48×48 | 0.08 | |
| CV17 | Clockwork cart bots (carting) | 6 | 96×48 | 0.12 | |
| CV18 | Enrage steam/breath puffs | 5 | 64×64 | 0.06 | per monster element tint drawn ×5 |
| UI | Stamina bar, sharpness gauge (5 colors), weapon gauges (spirit/demon/charge/bow coating), status icons (player ×7, monster ×6), item pouch icons (~30), quest board cards, track gauge, minimap icons (camp, monster, clue), faint counter skulls, reward screen frames | — | 16×16 / 32×32 | — | 13-ui-ux owns layout |

**Approximate frame budget**: shared H-sets ≈ 260 frames (with per-weapon variants); SnS ≈ 75; GS ≈ 75; LS ≈ 75; DB ≈ 90; Hammer ≈ 85; Bow ≈ 55; VFX ≈ 350. That is **≈ 1,060 new frames** at 343×160 (hunter) or small VFX cells. This is the single largest art cost in the game. It is why weapon types ship one at a time (tickets below).

## 6. Inventory items (hunt items: pouch 24 slots, MH-style caps)

| Item | Cap | Price | Source |
| --- | --- | --- | --- |
| Potion / Mega Potion | 10 / 10 | 66 / 180 buy | Still (02) / shop |
| Antidote, Nulberry-like Cleanse (Reedmint) | 10 | 50 | Still |
| Whetstone | 20 (infinite variant at High rank) | 15 | Workbench / supply box |
| Hardening Tonic, Might Draught, Breaker Tonic, Cool/Hot Drink, Moon Draught | 5 | 120–400 | Still |
| Field Ration (dried pack), Smoked Fish | 10 | — | 02/03 |
| Trap Tool, Net | 2 / 10 | 200 / 10 | Workbench |
| Pitfall Trap, Shock Trap | 1 each carried | — | Workbench |
| Tranq Bomb | 8 | 80 | Workbench |
| Flash Pod, Smoke Pod, Barrel Bomb (S/L), Paintball, Tracker Moth jar | 5–10 | 30–150 | Workbench |
| Arrow coatings ×8 | 20–50 | 15–60 | Still |
| Lubricant (sharpness restore) | 10 | 60 | Steam Press |
| Supply box items (quest-provided) | — | — | Base Camp, returned at quest end |

Hunt rewards are materials (06 per monster: carves, part breaks, capture, quest rewards) + gold + reputation + Hunter Rank points.

## 7. Hunting flow (processing steps)

1. **Accept**: the Quest Board at Hearthspire Hunter's Lodge (or the homestead mailbox for village requests). Types: Slay, Capture, Hunt (either), Gather, Tempered (High+), Arena challenge (post-launch). Each has a reward, a time limit in phases (Low 2 phases, High 3, Master overnight expedition), and a faint limit (3 default, 2 for tempered, 1 for arena).
2. **Prepare**: eat a Hearth Meal (04), pick gear (stash), pack the pouch, optional companion (08).
3. **Depart**: the hunt day cost (exists: 2 phase steps) now uses the quest's time cost. **Expeditions** let you sleep in the Base Camp tent (1 night) for multi-day hunts.
4. **Track**: clues spawn in areas (footprints, magical residue, claw marks, shed scales, roar echoes, elemental signs such as scorch/frost/static). Interact (H20) → +track gauge. At 100 the monster's area is shown and Tracker Moths (CV14) guide you. Paintball (CV15) keeps it marked. The Tracker skill lowers the needed amount.
5. **Fight**: read tells (06). Break parts (per-part HP; severable tails need cutting damage). Apply status (buildup thresholds rise ×1.5 per proc). KO through blunt head hits. Exhaust via SnS/Hammer. Use env traps. The monster changes area at HP thresholds and flees to its nest when limping.
6. **Phases/enrage**: phase 2 at 60% HP (new moves, sometimes an elemental shift), phase 3 at 30% (Elders), enrage from damage taken/time (×1.2 speed, ×1.3 damage, steam breath CV18, 25–40 s). Fatigue (drool) after 3 min enraged: slower, some moves fail.
7. **Capture**: limping (≤ 20% HP Low, 15% High) → trap (pitfall/shock) → 2 Tranq Bombs → captured. Rewards: no carves, +1 capture-only reward roll, higher rare chance, plus a Field Guide "capture" entry.
8. **Carve**: corpse carve count 3 (+1 Carving Master). Uses the existing hold-to-carve (2.0 s, food buff speed). Broken parts that drop on the ground are carvable once. Severed tail: 1 carve.
9. **Faint (carting)**: HP 0 → H10/H11 → back at Base Camp with full HP and −1 faint. When faints reach the limit → quest failed, return home, no rewards, keep what was carved (cozy). Q3 covers this.
10. **Return**: the reward screen shows quest rewards, break rewards, carves, capture rewards, gold, reputation, and Hunter Rank points. Then back to the village/homestead with the phase advanced.

## 8. Hooks

- **Forge (04)**: every monster part → weapon/armor rows; part-break-only materials gate top rows; weapons are forged per type; sharpness from rows; element/status from infusion.
- **Farming/Fishing/Animals**: potions, coatings, meals, tonics (Breaker Tonic), traps (Sleepcap, Paralyshroom), Monster Bait (03), Kraken-Eel fishing trigger.
- **Mining**: bombs/barrel bombs (coal/sulfur), miner gear, Gearjaw Drillwyrm in the Galleries ground, CMB-04 field combat-lite in the mine.
- **Social (08)**: companions (Hask the retired hunter, Wren the harbormaster-archer), Hunter's Lodge NPC Captain Vell gives quests, heart rewards via hunt requests, festivals (Hunter's Moon tournament).
- **Monsters (06)**: behaviors, parts, phases, seasonal/weather variants.
- **Progression (09)**: Hunter Rank (HR) gates quests; reputation from hunts; Sparks from Elders.
- **Day cycle (11)**: time cost; night hunts change monster behavior (nocturnal); seasons alter spawns.
- **UI (13)**: field guide, minimap, HUD, combat assists.

## 9. Energy, time, risk

- Combat **Stamina** (new) is separate from homestead **Tool Energy** (exists). Proposed coupling: you start a hunt with max stamina = 100 if tool energy ≥ 30, else 80 ("tired hunter"). Food restores both.
- Time: quest time limits in phases. Free-hunt runs until you return.
- Risk: faints (limited), item consumption, quest fail = no reward. Time pressure costs farm time (the core tension of the dual fantasy).

## 10. Progression / gating

Hunter Rank (HR) 1–8 Low, 9–20 High, 21+ Master (from quest points). Weapon unlocks: SnS (start, exists), GS after the Bristleboar quest (Hask teaches), Hammer after King Slime (EPICS #34), LS/DB through Hunter's Lodge HR 3/5, Bow through Wren 3♥ or HR 4. Rank gates the hunting grounds and monster ranks. Forge level gates gear rank (04).

## 11. UI

HUD: HP (exists), stamina bar (new), sharpness gauge, weapon gauge, status icons, pouch quick-bar (radial or cycle; 13), faint counter, quest timer, track gauge, minimap/area map, boss health is hidden (MH-style; assist toggle shows it, Q5), damage numbers (toggle), part-break toasts (a **drawn** toast frame plus a world VFX; the rule requires the in-world CV4 anim, the toast alone is not enough), reward screen, quest board, expedition camp menu.

## 12. Audio

Per weapon: swing whooshes (light/heavy), hit impacts (cut/blunt/pierce/crit), charge-up hums (GS/Hammer/Bow 3 levels), spirit/demon mode stingers, guard clang, parry ting (exists), evade roll, sharpen scrape, potion gulp, bomb fuse/boom, trap set/trigger, tranq hiss, flash pop, carve squelch (new), capture net, part-break crunch (distinct per material: bone, crystal, shell, gear), KO thud + bird tweet, faint jingle + cart bots, quest clear fanfare, quest fail. Music: per-ground exploration (calm) → combat (dynamic orchestral + mechanical) crossfade on monster aggro → enraged layer → victory sting. Monster roars live in 06.

## 13. Save data

`hunter: {hr, hr_points, unlocked_weapons, quests_completed, quest_flags}`, `pouch` (hunt item loadout presets ×3), `field_guide` (06), `active_expedition: Option<{quest, day, camp_state}>` (save at the tent only), per-weapon mastery counters (optional). Mid-hunt saves are not allowed except at the expedition tent (Q7).

## 14. Balance levers

Motion values per move; hit-stop; i-frame windows; stamina costs/regen; sharpness bars and damage multipliers (Red 0.5, Yellow 0.75, Green 1.0, Blue 1.2, White 1.32); element multipliers; status thresholds and growth; KO/exhaust thresholds; part HP; enrage triggers/duration; capture thresholds; faint limit; time limits; carve counts; reward tables; Hunter Rank points; companion damage.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Where do large-monster hunts happen, given the frozen dungeon? | **New side-scroll Hunting Grounds** built on the same combat/physics code, with new art per ground. The dungeon stays as the tutorial. |
| Q2 | Weapon row looks (Iron vs Slime GS) | Per-row **weapon layer** baked into per-row sheets by the script. Launch: 1 look per tree (3–4 per weapon type), not per row. |
| Q3 | Faint limit failure penalty | Quest fails with no quest reward, but carves/gathers are kept. Cozy, and still meaningful. |
| Q4 | Mounting? | Post-launch. Jump attacks exist, so mounting is a later extension. |
| Q5 | Show monster HP bars? | Hidden by default (readable mastery: the limp/drool tells matter). Assist toggle "Show monster health" in 13. |
| Q6 | Share bomb art between the mine and hunts? | Different scale/perspective, so make separate drawings. Allow the mine V8 as a placeholder only. |
| Q7 | Saving mid-hunt | Only at the expedition tent (sleep). Otherwise quitting returns to the pre-hunt state. |
| Q8 | Two energy systems confusing? | Keep both, but couple them (§9) and give them distinct HUD bars and names: "Energy" (home) vs "Stamina" (hunt). |
| Q9 | Weapon roster order | SnS (exists) → Hammer → Great Sword → Bow → Long Sword → Dual Blades. Hammer is first because EPICS #34 already plans it and KO/head breaks show off part breaking. |

## 16. Ticket breakdown

Hunter fundamentals (art for existing mechanics first):
- **CMB-01 [ART]** Hunter idle breathing H1 (SnS) + hurt flinch H7 + block H28 + parry H29. deps: none.
- **CMB-02 [ART]** Charge dash H30 + Spin H31 frames + slash smears CV1 for the existing sword/spear moves. deps: none.
- **CMB-03 [GAME]** Wire H1/H7/H28–H31 + CV1 into the existing animation playback (new `HunterSheet` variants); dungeon art untouched. deps: CMB-01, CMB-02.
- **CMB-04 [GAME]** Top-down field combat-lite for the dwarf (mine/forest small monsters): equipped weapon family → dwarf swing, hit detection, HP, knockback. deps: CMB-05.
- **CMB-05 [ART]** Dwarf top-down field strike sets: SnS slash, GS/Hammer heavy, LS/DB quick, Bow shot (4 sets × 4 frames, 64×60) + dwarf hurt (MIN C18 shared). deps: none.
- **CMB-06 [ART]** Evade roll H6 + knockdown/get-up H8 + launched H9. deps: none.
- **CMB-07 [GAME]** Combat stamina + evade roll with i-frames + knockdown states. deps: CMB-06.
- **CMB-08 [UI]** Stamina bar HUD + tool energy coupling at hunt start. deps: CMB-07.
- **CMB-09 [ART]** Carve crouch/loop/pull H17. deps: none.
- **CMB-10 [GAME]** Play H17 during the existing hold-to-carve; carve count per corpse (3). deps: CMB-09.
- **CMB-11 [ART]** Drink potion H12 + eat H13 + throw H15 + heal VFX CV16. deps: none.
- **CMB-12 [GAME]** Item pouch (24, caps) + use items in the hunt (potion, ration, bombs) + quick-use UI. deps: CMB-11, UI-24 (soft).
- **CMB-13 [ART]** Sharpen H14 (SnS) + sharpness gauge UI sprites. deps: none.
- **CMB-14 [GAME]** Sharpness system + Whetstone. deps: CMB-13, FRG-04.

Hunting structure:
- **CMB-15 [GAME]** `HuntQuest` + Quest Board (data `quests.ron`), accept/abandon, time limit, faint limit, objectives. deps: none (uses the existing dungeon as the first quest "Slime Season").
- **CMB-16 [ART]** Faint H10 + carted-out bots H11/CV17 + victory pose H26 (SnS). deps: none.
- **CMB-17 [GAME]** Faint/cart → Base Camp respawn, limit → fail; integrates with the existing player death. deps: CMB-15, CMB-16.
- **CMB-18 [UI]** Reward screen (quest/break/carve/capture/gold/rep/HR). deps: CMB-15.
- **CMB-19 [GAME]** Hunting Ground framework: multi-area side-scroll map, edge exits, Base Camp (supply box, gear box, tent), loads ground data. deps: CMB-15.
- **CMB-20 [ART]** Hearthspire Outskirts ground tileset + parallax + camp props (new art, not dungeon). deps: none.
- **CMB-21 [GAME]** Outskirts ground (3 areas) with Bristleboar quest. deps: CMB-19, CMB-20, MON-07.
- **CMB-22 [ART]** Ledge climb/hang/slide H22–H24. deps: none.
- **CMB-23 [GAME]** Ledges/slopes traversal in grounds. deps: CMB-22, CMB-19.
- **CMB-24 [ART]** Track clue sprites + tracker moths CV14 + inspect H20 + moth release H21 + paintball CV15. deps: none.
- **CMB-25 [GAME]** Tracking: clue spawns, track gauge, reveal, moths, paintball. deps: CMB-24, CMB-19.
- **CMB-26 [GAME]** Monster area-change and nest-flee (limp) behavior hooks. deps: CMB-19, MON-05.
- **CMB-27 [UI]** Area map/minimap (camp, clues, marked monster). deps: CMB-25.
- **CMB-28 [GAME]** Expeditions: multi-day hunt with tent sleep and save. deps: CMB-19, CYC-06.

Fight systems:
- **CMB-29 [GAME]** Part system: per-part HP, break, sever (cut only), break rewards, part-specific hurtboxes. deps: MON-01.
- **CMB-30 [ART]** Part-break burst CV4 + hit sparks CV2 + KO stars CV5. deps: none.
- **CMB-31 [GAME]** Damage model v2: motion values, cut/blunt/pierce, hitzones, crit/affinity, sharpness mult. deps: CMB-29, CMB-14.
- **CMB-32 [GAME]** Monster status buildup (poison/para/sleep/blast/KO/exhaust) with rising thresholds. deps: CMB-31.
- **CMB-33 [ART]** Element/status hit puffs CV3 + status icons. deps: none.
- **CMB-34 [GAME]** Element damage + weaknesses + infusion read-through. deps: CMB-31, FRG-15.
- **CMB-35 [ART]** Player status sets H25 (7). deps: none.
- **CMB-36 [GAME]** Player statuses (poison/burn/freeze/para/stun/sleep/slimed) + cures. deps: CMB-35.
- **CMB-37 [GAME]** Phases/enrage/fatigue/elemental shift framework (data-driven per monster). deps: MON-01.
- **CMB-38 [ART]** Enrage puffs CV18. deps: none.
- **CMB-39 [ART]** Place trap H16 + traps CV11 + tranq smoke CV12 + capture-sack H18. deps: none.
- **CMB-40 [GAME]** Traps (pitfall/shock) + capture + capture rewards. deps: CMB-39, CMB-37, FRG-23.
- **CMB-41 [GAME]** Environmental traps framework (per-ground hazards that hurt monsters: deadfall, sluice flood, pillar collapse). deps: CMB-19, MON-05.
- **CMB-42 [ART]** Flash pod CV13 + bomb explosion CV10. deps: none.
- **CMB-43 [GAME]** Flash/smoke pods, barrel bombs. deps: CMB-42, CMB-12.

Weapon types (each type = ART sheets ticket(s) + GAME ticket):
- **CMB-44 [ART]** SnS full sheet §5.2 (bash, backstep, perfect rush, guard, item-use drawn, specials) + draw/sheathe/walk-drawn. deps: CMB-01.
- **CMB-45 [GAME]** SnS moveset upgrade of the existing Sword family (bash KO, backstep, perfect rush, drawn item use). deps: CMB-44, CMB-31.
- **CMB-46 [ART]** Hammer sheet §5.6 + charge flashes CV8 + smears. deps: none.
- **CMB-47 [GAME]** Hammer family (charge levels, KO, Big Bang, spinning bludgeon) + first rows. deps: CMB-46, CMB-31, CMB-32.
- **CMB-48 [ART]** Great Sword sheet §5.3. deps: none.
- **CMB-49 [GAME]** Great Sword family (charge, tackle hyper armor, TCS, guard). deps: CMB-48, CMB-31.
- **CMB-50 [ART]** Bow sheet §5.7 + arrows/coatings CV9. deps: none.
- **CMB-51 [GAME]** Bow family (charge levels, arc/power shot, dodge-shot, coatings; reuse projectiles). deps: CMB-50, CMB-34, FRM-63 (coatings, soft).
- **CMB-52 [ART]** Long Sword sheet §5.4 + spirit flares CV7. deps: none.
- **CMB-53 [GAME]** Long Sword family (spirit gauge/levels, foresight counter). deps: CMB-52, CMB-31.
- **CMB-54 [ART]** Dual Blades sheets §5.5 + demon aura CV6. deps: none.
- **CMB-55 [GAME]** Dual Blades family (demon/archdemon, dance, dash). deps: CMB-54, CMB-07, CMB-32.
- **CMB-56 [ART]** Per-weapon shared variants: idle/run back silhouettes, roll, sharpen, victory for GS/LS/DB/Hammer/Bow. deps: CMB-06, CMB-13, CMB-16 (per weapon; can split ×5).
- **CMB-57 [GAME]** Weapon unlock flow (Hask/Lodge/Wren) + weapon select in the gear stash. deps: CMB-45, FRG-04.

Polish:
- **CMB-58 [UI]** Weapon gauges HUD (charge/spirit/demon/coating) + sharpness + status icons. deps: CMB-14, CMB-53, CMB-55.
- **CMB-59 [UI]** Combat assists (auto-guard window, hold-to-charge assist, target lock, show monster HP, slower enrage). deps: UI-19.
- **CMB-60 [AUD]** Weapon sfx per type + impacts by damage type. deps: per weapon GAME ticket.
- **CMB-61 [AUD]** Dynamic hunt music system (explore → combat → enrage → victory). deps: CMB-19.
- **CMB-62 [DATA]** Motion value / stamina / sharpness balance pass. deps: CMB-55.
- **CMB-63 [SAVE]** Hunter Rank, quests, pouch presets, expedition save. deps: CMB-15, CMB-28.
- **CMB-64 [GAME]** Hunter Rank progression + quest gating. deps: CMB-15, CMB-18.

**Ticket count: 64.**

**Build first (top 3):** CMB-01 + CMB-02 → CMB-03 (frames for existing block/parry/charge/spin/hurt/idle, a standing-rule debt), CMB-09 → CMB-10 (carve animation on the shipped carve), CMB-15 (Quest Board + HuntQuest, so hunts become quests and the existing dungeon becomes quest #1).
