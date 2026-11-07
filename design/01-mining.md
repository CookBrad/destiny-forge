# 01 — Mining & Gathering

Authority: Brad's GDD §4.2 ("multi-level mine and outdoor gathering nodes provide ores, gems, herbs, and fossils… structural backbone of weapons and armor (frames, cores, polish)… floors have increasing difficulty with small monsters and environmental hazards… rare nodes only appear after certain story progress or during specific seasons"), §4.6 (forge uses natural materials), §6 ("crystal-veined mines"). Engine: Rust + Bevy 0.15.

Build priority: **1 of 16**. Herbs and fossils are covered here as *gathering*; crop-side herbs live in 02-farming.

## 0. Standing rules (apply to every ticket below)

- Every player action gets **new in-house sprite frames** drawn by a deterministic `tools/draw_*.py` script (same pattern as `tools/draw_dwarf_pickaxe_swing.py`). Feedback cannot be only a UI banner, color tint, or tinted arc. Tier differences are new frames or new palette-baked PNGs written by the script. Runtime `Sprite.color` tinting does not count.
- Clean Code. No code comments. Names carry the intent. Small modules: `src/mining/*`.
- Zero warnings and zero errors on `cargo build`, `cargo clippy --all-targets`, and `cargo test`.
- **Dungeon platform/art churn is frozen.** The mine is its own top-down zone (`src/mining`, `GameState::Mine`). Do not edit `assets/dungeon/**` or `src/dungeon/setup/**` layouts. Mine art goes under `assets/mine/**`.
- Tickets with sprite or animation work are their own **[ART]** tickets. Gameplay tickets list the [ART] tickets they depend on. A gameplay ticket may merge with a placeholder only if the dependency is marked *soft*.

## 1. What already exists (do not redo)

| Shipped / in PR #95 | Where |
| --- | --- |
| `MaterialId::CopperOre`, `IronOre`, `PickaxeTier1` (Copper Pickaxe), `PickaxeTier2` (Iron Pickaxe) | `src/items/material.rs` |
| `OreKind {Copper, Iron}` with `hits_to_break` (2 / 4), `energy_reserve_required` (4 / 8), required tier (1 / 2) | `src/mining/ore.rs` |
| `PickaxeTier {Tier1, Tier2}`, energy per swing 4.0 / 3.0 | `src/mining/ore.rs` |
| Homestead mine entrance (KeyE) → `Mine` state; ladder (KeyE) back to homestead | `src/overworld/layout.rs`, `src/mining/interaction.rs`, `transition_tests.rs` |
| One mine layer, 30×20 tiles, 13 fixed `ORE_NODE_PLACEMENTS`, ladder tile (3,1), arrival (3,3) | `src/mining/layout.rs` |
| Nodes respawn on sleep (`respawn_ore_nodes_on_sleep`) | `src/mining/ore.rs` |
| Raw ore used directly in forge recipes (Iron Sword 5 gel + 3 iron ore; Iron Pickaxe 5 iron ore; copper-bound Iron Pickaxe 3 copper ore) | `assets/data/recipes.ron` |
| Dwarf pickaxe swing: 4 frames, 48×60 canvas, 16×28 dwarf at (16,16), 0.3 s total, faces the node through `flip_x` | `assets/player/non-combat/dwarf_m_pickaxe_swing_f0..3.png`, `src/mining/swing.rs` |
| Tool energy pool 100 | `src/core/day_cycle.rs` |

The mine is top-down and uses the dwarf homestead avatar, not the side-scroll knight.

## 2. Where it happens

1. **Hearthspire Crystal Mines** (GDD "crystal-veined mines"). Entered from the homestead entrance that already exists. 40 main floors in 4 strata, plus an endless Destiny Vault after the story finale (floor 41+).

| Stratum | Floors | Theme | Main ores | Gems | Monsters | Hazards |
| --- | --- | --- | --- | --- | --- | --- |
| Upper Shafts | 1–10 | timber-propped earth tunnels | Stone, Copper, Coal, Clay | Quartz, Amber | Dust Slime, Cave Bat (mine variant), Rock Crab | loose rubble, dark corners |
| Clockwork Galleries | 11–20 | abandoned steam drilling rigs, rails | Iron, Zinc, Coal | Topaz, Garnet | Gearmite, Copper Beetle, Steam Imp | steam vents, runaway minecarts |
| Crystal Veins | 21–30 | glowing aether-crystal caverns | Silver, Aether Crystal | Amethyst, Sapphire, Emerald | Crystal Wisp, Shardback Tortoise, Gloom Bat | crystal spike floors, darkness |
| Molten Deep | 31–40 | magma channels, brass pipes | Mythril, Obsidian, Sulfur | Ruby, Diamond | Magma Slug, Cinder Golem (mini-boss F40), Ember Moth | lava tiles, heat drain, cave-ins |
| Destiny Vault | 41+ (post-story) | prismatic ruin-mine | Starmetal, Destinium | Prismatic Shard | mixed elite variants | all, randomized |

2. **Floor structure**. Each floor is a top-down room built from a stratum template set plus seeded node scatter. Seed = `(profile_seed, calendar_day, floor)` so a floor stays stable for a day and changes after sleep. The current 30×20 single layer becomes **floor 1** (keep its 13 placements as the floor-1 template). Floors 5/10/15… are *rest floors*: no monsters, one elevator, one supply crate. Floor 10/20/30/40 rest floors also hold a stratum treasure chest the first time you reach them.
3. **Descending**. Each non-rest floor hides a **ladder down** under one random breakable rock (guaranteed after ≥ 40% of rocks are broken, or immediately after all monsters on the floor are defeated). Some floors also roll a **shaft hole** (5% per floor from F11): drop 2–4 floors, take 15 HP fall damage, and miss the elevator unlock in between.
4. **Elevator**. A brass cage lift at the mine entrance and on every 5th floor. Reaching floor 5n unlocks stop 5n. The panel lists unlocked stops.
5. **Surface gathering nodes** (outside the mine). They respawn on sleep, using the same respawn hook.

| Zone | Nodes | Notes |
| --- | --- | --- |
| Homestead | Stone boulder, small Copper outcrop (×2), Clay patch, Weeds/Herbs | starter supply; clearing boulders is also farm prep |
| Misty Forest (exists) | Copper outcrop, Mossy Stone, Glowcap herb, Wild Sage herb, Fossil dig spot | forest art is not frozen |
| Riverlands | River Stone, Zinc outcrop, Clay bank, Pearl clam (fishing-adjacent), Reedmint herb | needs 10-world-zones |
| Highland Ruins | Silver outcrop, Relic Rubble (fossils + clockwork parts), Sunstone (summer only) | story Act 2 |
| Floating Islets | Starmetal meteorite (1 per week), Skybloom herb | story Act 3 |
| Any zone, winter | Frost Crystal clusters | seasonal rare |
| Any zone, thunderstorm | Stormglass node (spawns where lightning hits, lasts until next phase) | weather rare |

## 3. Every entity and object

| Entity | Component(s) | Notes |
| --- | --- | --- |
| `OreNode` | `OreNode {id, kind, hits_taken}`, `MineNodeVisual`, `Interactable` | extends existing; adds `NodeKind` beyond `OreKind` |
| `NodeKind` | `Stone, Copper, Coal, Clay, Zinc, Iron, Silver, AetherCrystal, Mythril, Obsidian, Sulfur, Starmetal, Destinium, GemNode(GemKind), GeodeNode(GeodeKind), MossyStone, RiverStone, RelicRubble, FrostCrystal, Stormglass, Sunstone` | data in `assets/data/nodes.ron` |
| `LadderDown` | spawned on reveal | interact = descend |
| `LadderUp` | exists (exit ladder) | on floor > 1 it goes up one floor; on floor 1 it goes home |
| `ShaftHole` | hazard + descend | |
| `Elevator` | `ElevatorCage {stops}` | entrance and 5n floors |
| `SupplyCrate` | rest floors | 1–3 torches/bombs/food |
| `TreasureChest` | first-visit stratum chests | fixed loot |
| `MineCart` (decor + hazard F11–20) | `RunawayCart {lane, speed}` | runs along a rail lane, 20 dmg |
| `SteamVent` | `PeriodicHazard {period, active}` | 2 s on / 3 s off, 8 dmg/s, pushes player |
| `FallingRock` | `TelegraphedHazard {delay}` | shadow 0.8 s, then 12 dmg |
| `CrystalSpikeTile` | `ToggleHazard` | 1.5 s up / 1.5 s down, 6 dmg |
| `LavaTile` | `DamageTile` | 10 dmg/s, ignites (burn status) |
| `CaveIn` | floor event F31–40 | rocks fall across 3×3 marks; 10 s to leave |
| `Darkness` | `MineLighting {radius}` | full-screen overlay, light radius set by lantern tier |
| `MineMonster` | `MineEnemy {kind}`, `Health`, AI | see §8 |
| `OreDrop` | world pickup, magnet radius 1.5 tiles | collected on touch |
| `Smelter` (homestead) | `Smelter {queue, fuel}` | next to the forge furnace |
| `GeodeCracker` (blacksmith NPC service, later a machine) | | 25 g per geode, or the in-house Geode Press |
| `BombThrown` | `Fuse {secs}` | 1.5 s fuse, 2-tile radius, breaks all nodes and damages monsters |

## 4. Tools and equipment, with tiers

### 4.1 Pickaxes (tool power gates node tier; energy per swing; hits)

| Tier | Item | Power | Energy/swing | Swing time | Special | Recipe (Forge) | Gate |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T1 | Copper Pickaxe (`PickaxeTier1`, exists) | 1 | 4.0 (exists) | 0.30 s | — | starter | — |
| T2 | Iron Pickaxe (`PickaxeTier2`, exists) | 2 | 3.0 (exists) | 0.30 s | — | migrate to 5 Iron Bar *or* 3 Copper Bar + 2 Coal | Forge Lv1 |
| T3 | Brasswork Pickaxe | 3 | 2.5 | 0.27 s | hold 0.6 s charge = 3-tile line strike | 5 Brass Bar + 2 Gear Spring | Forge Lv2 |
| T4 | Aether Pickaxe | 4 | 2.0 | 0.25 s | charge = 3×3 strike; +10% gem chance | 5 Silver Bar + 3 Aether Crystal + 1 Gearmite Core | Forge Lv3 |
| T5 | Mythril Drill-Pick | 5 | 1.5 | 0.22 s | charge = 5×3 cone; auto-collect drops | 5 Mythril Bar + 1 Cinder Heart + 2 Ruby | Forge Lv4 |
| T6 | Destiny Pick | 6 | 1.0 | 0.20 s | charge vents steam; reveals ladder | 3 Destinium Ingot + 1 Destiny Spark | Forge Lv5 + Destiny Binding |

Rule (extends PR #95): `required_tier(node) ≤ pickaxe.power`, otherwise the "clank" outcome (no damage, 1 energy, recoil frames). Hits to break = `max(1, node.hardness − (power − node.required_tier))`. Lowest-power swings are never free.

### 4.2 Support gear

| Item | Tiers | Effect | Source |
| --- | --- | --- | --- |
| Lantern | Tin Lantern (r 3 tiles), Brass Lantern (r 5), Aether Lamp (r 7, reveals gem nodes) | light radius in Darkness | Forge recipes; Tin Lantern from F1 chest |
| Torch (consumable, stack 99) | — | place for a permanent light on the floor until you leave | 1 Wood + 1 Coal at workbench |
| Mining Bomb | Cherry Bomb (r1), Steam Bomb (r2), Mega Bomb (r3) | 1.5 s fuse | Workbench: coal + sulfur + copper/iron/silver bar |
| Staircase Charge | — | instantly creates a ladder down | 99 Stone + 1 Steam Bomb |
| Miner's Helmet (armor head piece, Low rank) | — | −50% falling-rock damage | Forge: 3 Iron Bar + 1 Rock Crab Shell |
| Ore Satchel | upgrade | auto-sort ore into a separate 12-slot pouch | Forge Lv2, 2 Leather |
| Miner's Ring (decoration-socket item later) | — | +1 ore per break 15% | F20 chest |

## 5. ANIMATIONS (most detailed section)

Art conventions for this doc (see README §Art conventions):
- **Homestead/mine avatar**: 0x72 `dwarf_m` base, 16×28 px body. Action frames are authored on a **48×60 canvas** with the dwarf at (16,16), the same as the shipped pickaxe swing, so the sprite center does not move. All frames are authored **facing right**. Left = runtime `flip_x`, as `mining/interaction.rs` already does. Up/down facings: see Open Question Q1. Proposed: side-only for launch, which matches all current homestead sets.
- **Palette**: dwarf palette + one tool material ramp per tier (3 shades + outline `#222222`).
- **Timing** is listed per frame. A total is given where gameplay needs it (hit-frame index = frame that applies damage).
- **Pixel scale**: native 1×, rendered with the existing world scale (TILE = 32 world units).

### 5.1 Character (dwarf) sprite sets

| # | Set | Files | Frames | Canvas | Facing | Timing | Loop | Hit/trigger frame | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| C1 | Pickaxe swing T1 copper | `dwarf_m_pickaxe_swing_f0..3` (**exists**) | 4 | 48×60 | R (+flip) | 0.075 s ×4 = 0.30 s | no | f2 | shipped; keep |
| C2 | Pickaxe swing T2 iron | `dwarf_m_pickaxe_t2_swing_f0..3` | 4 | 48×60 | R | 0.075 ×4 | no | f2 | new frames with an iron-head palette and wider pick head; do not tint T1 |
| C3 | Pickaxe swing T3 brasswork | `..._t3_swing_f0..3` | 4 | 48×60 | R | 0.0675 ×4 = 0.27 | no | f2 | brass head + gear hub on the handle |
| C4 | Pickaxe swing T4 aether | `..._t4_swing_f0..3` | 4 | 48×60 | R | 0.0625 ×4 = 0.25 | no | f2 | crystal head with a 1-px glow edge |
| C5 | Pickaxe swing T5 mythril drill | `..._t5_swing_f0..3` | 4 | 48×60 | R | 0.055 ×4 = 0.22 | no | f2 | drill bit; f2–f3 bit-spin variants |
| C6 | Pickaxe swing T6 destiny | `..._t6_swing_f0..3` | 4 | 48×60 | R | 0.05 ×4 = 0.20 | no | f2 | prismatic head |
| C7 | Charge wind-up (T3+) | `dwarf_m_pickaxe_charge_f0..2` per tier (T3–T6) | 3 | 48×60 | R | f0 0.2 s, f1–f2 loop at 0.12 s while held | f1↔f2 | — | the pick rises over the head; tier head per tier = 4 sets × 3 |
| C8 | Charge release slam | `dwarf_m_pickaxe_slam_f0..4` per tier (T3–T6) | 5 | 64×60 | R | 0.06 ×5 = 0.30 | no | f3 | wider canvas for the overhead slam |
| C9 | Clank recoil (tier too low) | `dwarf_m_pickaxe_clank_f0..2` (one set; pick head drawn per tier → 6 variants) | 3 | 48×60 | R | 0.08, 0.10, 0.12 | no | — | arms jolt back, 1-px body shake; plays after the C1–C6 hit frame |
| C10 | Ore pickup bend | `dwarf_m_pickup_f0..2` | 3 | 16×28 | R | 0.08 ×3 | no | f1 | shared by every pickup (forage, fish, drops) |
| C11 | Ladder climb down | `dwarf_m_ladder_down_f0..3` | 4 | 16×32 | back view (single facing) | 0.12 ×4, then fade | no | — | the only back-facing set; Q1 |
| C12 | Ladder climb up | `dwarf_m_ladder_up_f0..3` | 4 | 16×32 | back view | 0.12 ×4 | no | — | |
| C13 | Elevator ride idle | `dwarf_m_elevator_f0..1` | 2 | 16×28 | front | 0.4 ×2 | yes | — | foot tap |
| C14 | Lantern idle | `dwarf_m_lantern_idle_f0..3` | 4 | 24×28 | R | 0.15 ×4 | yes | — | lantern held in the off hand; replaces idle in the mine while a lantern is equipped |
| C15 | Lantern walk | `dwarf_m_lantern_run_f0..3` | 4 | 24×28 | R | 0.1 ×4 | yes | — | |
| C16 | Bomb throw | `dwarf_m_throw_f0..3` | 4 | 32×36 | R | 0.07, 0.07, 0.05, 0.12 | no | f2 (spawn bomb) | fishing (FSH) and the farm seed fling use their own sets |
| C17 | Place torch | `dwarf_m_place_f0..2` | 3 | 24×28 | R | 0.1 ×3 | no | f2 | shared with farming machine placement |
| C18 | Hurt flinch (mine) | `dwarf_m_hurt_f0..1` | 2 | 16×28 | R | 0.1 ×2 | no | — | white-edge frame drawn in, not a runtime tint |
| C19 | Knock-down / faint | `dwarf_m_faint_f0..4` | 5 | 28×28 | R | 0.1 ×4, hold f4 1.0 s | no | — | also used at 0 energy |
| C20 | Exhausted walk (≤ 10% energy) | `dwarf_m_tired_run_f0..3` | 4 | 16×28 | R | 0.14 ×4 | yes | — | slump posture |
| C21 | Smelter load | `dwarf_m_load_f0..3` | 4 | 32×32 | R | 0.09 ×4 | no | f2 | shovel ore in; shared with farm machines |
| C22 | Geode crack (Geode Press) | `dwarf_m_press_f0..3` | 4 | 32×32 | R | 0.1 ×4 | no | f3 | lever pull |
| C23 | Shaft fall | `dwarf_m_fall_f0..2` | 3 | 16×32 | front | 0.08 ×3, then screen fade | no | — | |
| C24 | Surface forage pluck (herb/fossil) | `dwarf_m_forage_f0..3` | 4 | 24×28 | R | 0.08 ×4 | no | f2 | crouch dig; the farming harvest pull is a separate, heavier set (FRM) |

Character total: **24 sets, about 130 frames** (C2–C6 and the C7–C9 tier variants make up most of it).

### 5.2 Tool / held-item sprites

| # | Set | Frames | Size | Notes |
| --- | --- | --- | --- | --- |
| T1 | Pickaxe world/drop icon per tier (6) | 1 each | 16×16 | also used as the inventory icon base |
| T2 | Lantern world sprite (3 tiers) with flame flicker | 3 each | 8×12 | 0.12 s loop; attaches to C14/C15 hand anchor |
| T3 | Placed torch | 4 | 8×16 | flame loop at 0.1 s |
| T4 | Bomb (3 tiers) fuse | 4 | 12×12 | fuse spark loop at 0.08 s, last 0.3 s blink at 0.05 s |

### 5.3 Entity / world sprites

| # | Set | Frames | Cell | Facing | Timing | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| E1 | Ore node states, per NodeKind (21 kinds) | intact, crack-1, crack-2, depleted rubble = 4 | 32×32 | n/a | static per state | 21×4 = 84 frames; crack frames are separate drawings, not a tint |
| E2 | Node shatter burst (per ore color family, 8 families) | 5 | 48×48 | n/a | 0.05 ×5 | plays on the break hit |
| E3 | Gem node sparkle (8 gems) | 4 | 32×32 | n/a | 0.15 s, idle sparkle every 2 s | |
| E4 | Geode node wobble (4 kinds) | 3 | 32×32 | n/a | 0.1 when hit | |
| E5 | Ladder-down reveal | 3 | 32×32 | — | 0.1 ×3 | dust lifts, ladder top shows |
| E6 | Ladder down idle | 1 | 32×32 | — | — | |
| E7 | Shaft hole | 4 | 32×32 | — | 0.15 loop (dust drift) | |
| E8 | Elevator cage doors | 6 (closed→open) | 48×64 | front | 0.07 ×6 | reverse plays to close |
| E9 | Elevator gauge needle | 4 | 16×16 | — | 0.1 loop while riding | |
| E10 | Supply crate open | 3 | 32×32 | — | 0.08 ×3 | |
| E11 | Treasure chest open (stratum skin ×4) | 4 | 32×32 | — | 0.08 ×4 | |
| E12 | Runaway minecart | 4 | 32×32 | R (+flip) | 0.06 loop | |
| E13 | Steam vent | idle 2, erupt 6 | 32×64 | — | idle 0.3; erupt 0.06 | the vent gets a sprite; the damage area is not only a tint |
| E14 | Falling rock: shadow grow 4 + rock fall 3 + shatter 4 | 11 | 32×48 | — | shadow 0.2 ×4 (0.8 s telegraph), fall 0.05 ×3, shatter 0.06 ×4 | |
| E15 | Crystal spike tile | 4 (down→up) | 32×32 | — | 0.05 ×4 rise | |
| E16 | Lava tile | 4 | 32×32 | — | 0.2 loop | + 3-frame edge variants (N/E/S/W) as autotile = 16 extra frames |
| E17 | Cave-in warning crack | 3 | 32×32 | — | 0.15 loop | |
| E18 | Floor tilesets per stratum (4 + vault) | 16-tile autotile + 6 decor props each | 32×32 | — | static | timber props, rails, crystals, pipes |
| E19 | Darkness overlay mask | 3 radii | 256×256 gradient | — | static | the light circle is a sprite mask; flicker alternates two masks at 0.2 s |
| E20 | Smelter (homestead) idle / working | idle 1, working 4 | 48×64 | front | 0.12 loop | sits next to `forge_furnace.png` |
| E21 | Smelter bar pop-out | 3 | 16×16 | — | 0.1 ×3 | |
| E22 | Geode Press machine idle / working | 1 + 4 | 32×48 | front | 0.1 loop | |
| E23 | Surface nodes (forest/river/ruin/islet skins) | uses E1 state set, zone recolor drawn per zone | 32×32 | — | — | counted inside E1's 21 kinds |
| E24 | Fossil dig spot (wiggling worms) | 3 | 16×16 | — | 0.2 loop | |

### 5.4 Mine monster sprite sets

Mine monsters are top-down. Each needs **idle 4, move 4, telegraph 2–3, attack 3–4, hurt 2, death 5** unless noted. All are right-authored with `flip_x`. Cells listed below.

| # | Monster | Floors | Cell | Idle | Move | Telegraph | Attack | Hurt | Death | Extra |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| M1 | Dust Slime | 1–10 | 24×24 | 4 @0.14 | 4 @0.12 hop | 2 @0.15 squash | 3 @0.08 lunge | 2 @0.1 | 5 @0.08 splat | — |
| M2 | Mine Bat | 1–20 | 24×16 | 4 @0.1 flap | (idle = move) | 2 @0.12 | 3 @0.06 dive | 2 | 4 | new mine skin; dungeon `bat.png` is frozen and not reused |
| M3 | Rock Crab | 3–15 | 32×24 | 2 @0.4 (disguised as a rock, 1 frame closed) | 4 @0.1 scuttle | 3 @0.12 claw raise | 3 @0.07 snip | 2 | 5 | "reveal" 3 @0.08 when the player is within 2 tiles |
| M4 | Gearmite | 11–20 | 16×16 | 4 @0.1 | 4 @0.08 | 2 @0.1 | 4 @0.05 bite | 2 | 5 (springs pop) | swarms of 3–5 |
| M5 | Copper Beetle | 11–20 | 32×32 | 4 @0.15 | 4 @0.12 | 3 @0.15 horn lower | 4 @0.07 charge | 2 | 5 | — |
| M6 | Steam Imp | 14–20 | 24×32 | 4 @0.12 hover | 4 | 3 @0.12 | 4 @0.08 steam puff projectile | 2 | 5 | projectile 3 frames 12×12 |
| M7 | Crystal Wisp | 21–30 | 24×24 | 4 @0.12 pulse | — | 3 @0.15 charge glow | 3 shard shot | 2 | 6 (shatter) | blinks: 3 @0.06 |
| M8 | Shardback Tortoise | 21–30 | 48×32 | 4 @0.2 | 4 @0.18 | 3 @0.2 | 4 @0.08 shell spin | 2 | 5 | part break: shard back 2-state |
| M9 | Gloom Bat | 21–30 | 32×20 | 4 @0.09 | — | 2 | 3 dive | 2 | 4 | dims nearby light while alive |
| M10 | Magma Slug | 31–40 | 32×20 | 4 @0.2 | 4 @0.2 | 2 | 3 spit | 2 | 5 | leaves a lava trail 3-frame tile |
| M11 | Ember Moth | 31–40 | 24×24 | 4 @0.08 | — | 2 | 3 ember drop | 2 | 4 | |
| M12 | Cinder Golem (F40 mini-boss) | 40 | 64×64 | 4 @0.18 | 6 @0.16 | 4 @0.15 fist raise | 5 @0.08 slam | 2 | 8 crumble | phase-2 molten skin set (idle 4, move 6) |

### 5.5 VFX sprites

| # | Set | Frames | Cell | Timing | Trigger |
| --- | --- | --- | --- | --- | --- |
| V1 | Pick impact spark (metal ore) | 3 | 16×16 | 0.04 ×3 | C*-f2 on a successful hit |
| V2 | Pick impact dust (stone/clay) | 4 | 16×16 | 0.05 ×4 | |
| V3 | Clank sparks (white, blocked) | 3 | 16×16 | 0.04 ×3 | C9 |
| V4 | Gem glint on drop | 4 | 8×8 | 0.08 ×4 | |
| V5 | Ore chunk fly-to-player | 1 per ore item (uses the item icon) + 2-frame trail | 8×8 | 0.05 | |
| V6 | Charge glow ring (T3–T6) | 4 | 32×32 | 0.08 loop | C7 |
| V7 | Slam shockwave | 5 | 96×32 | 0.05 ×5 | C8-f3 |
| V8 | Bomb explosion | 7 | 64×64 | 0.05 ×7 | |
| V9 | Smelter fire mouth | 4 | 16×16 | 0.1 loop | E20 working |
| V10 | Smelter steam puff | 5 | 24×24 | 0.08 ×5 | every 1.5 s while working |
| V11 | Geode open burst | 5 | 32×32 | 0.06 ×5 | |
| V12 | Elevator arrive steam | 4 | 48×24 | 0.07 ×4 | |
| V13 | Monster hit flash (per monster) | drawn white silhouette frame per monster M1–M12 | per cell | 0.06 | not a tint |
| V14 | Lava burn status flame on dwarf | 3 | 16×16 | 0.1 loop | |

### 5.6 UI icons (inventory, 16×16 native, shown at 3× in slots; 32×32 for HUD)

| Group | Icons |
| --- | --- |
| Ores (13) | Stone, Copper Ore (art replaces the current swatch), Coal, Clay, Zinc Ore, Iron Ore, Silver Ore, Aether Crystal, Mythril Ore, Obsidian, Sulfur, Starmetal Chunk, Destinium Ore |
| Bars (9) | Copper Bar, Brass Bar, Iron Bar, Steel Bar, Silver Bar, Aethersteel Bar, Mythril Bar, Starsteel Bar, Destinium Ingot |
| Gems (9) | Quartz, Amber, Topaz, Garnet, Amethyst, Sapphire, Emerald, Ruby, Diamond, + Prismatic Shard = 10 |
| Geodes (4) | Rough Geode, Crystal Geode, Magma Geode, Clockwork Geode |
| Gathering (9) | Glowcap, Wild Sage, Reedmint, Skybloom, Mossy Stone, River Stone, Pearl, Frost Crystal, Stormglass, Sunstone |
| Fossils & relics (8) | Ammonite, Trilobite, Wyrm Tooth, Amber-Fly, Gear Fossil, Relic Plate, Old Cog, Ancient Core |
| Tools (12) | 6 pickaxes, 3 lanterns, torch, 3 bombs, staircase charge |
| Monster drops (12) | Dust Gel, Bat Wing (mine), Rock Crab Shell, Gear Spring, Gearmite Core, Beetle Carapace, Steam Imp Horn, Wisp Mote, Tortoise Shard, Gloom Membrane, Magma Gland, Ember Dust, Cinder Heart (13) |
| HUD (32×32) | Floor badge frame, elevator panel buttons (number font 5×7), lantern charge pip, darkness/heat status icons |

## 6. Inventory items

Stack: 99 unless noted (`MAX_STACK` exists). Prices are the **sell** value (shipping bin / shop). Buy price = sell × 2.5 where sold. Quality does not apply to ores.

| Item | Category | Stack | Sell (g) | Source |
| --- | --- | --- | --- | --- |
| Stone | Ore | 999* | 2 | Stone nodes everywhere |
| Clay | Ore | 99 | 4 | Clay patches, F1–10 |
| Coal | Ore | 99 | 15 | Coal nodes F1–20, Molten Deep, Gearmite drop |
| Copper Ore (exists) | Ore | 99 | 5 | F1–10, forest, homestead |
| Zinc Ore | Ore | 99 | 8 | F11–20, riverlands |
| Iron Ore (exists) | Ore | 99 | 10 | F6–20 |
| Silver Ore | Ore | 99 | 25 | F21–30, highland ruins |
| Aether Crystal | Ore | 99 | 40 | F21–30 crystal nodes |
| Mythril Ore | Ore | 99 | 60 | F31–40 |
| Obsidian | Ore | 99 | 35 | F31–40 |
| Sulfur | Ore | 99 | 12 | F31–40, steam vents (forage) |
| Starmetal Chunk | Ore | 99 | 150 | Floating islet meteorites, Vault |
| Destinium Ore | Ore | 99 | 300 | Vault 41+ only |
| Copper Bar | Bar | 99 | 30 | Smelt 5 Copper Ore + 1 Coal |
| Brass Bar | Bar | 99 | 60 | Smelt 3 Copper Ore + 2 Zinc Ore + 1 Coal |
| Iron Bar | Bar | 99 | 70 | Smelt 5 Iron Ore + 1 Coal |
| Steel Bar | Bar | 99 | 120 | Smelt 1 Iron Bar + 2 Coal (Forge Lv2) |
| Silver Bar | Bar | 99 | 160 | Smelt 5 Silver Ore + 1 Coal |
| Aethersteel Bar | Bar | 99 | 300 | Smelt 1 Steel Bar + 2 Aether Crystal (Forge Lv3) |
| Mythril Bar | Bar | 99 | 400 | Smelt 5 Mythril Ore + 2 Coal |
| Starsteel Bar | Bar | 99 | 900 | Smelt 1 Mythril Bar + 3 Starmetal (Forge Lv4) |
| Destinium Ingot | Bar | 99 | 2000 | Smelt 5 Destinium + 1 Prismatic Shard (Forge Lv5) |
| Gems (Quartz 20, Amber 40, Topaz 80, Garnet 80, Amethyst 100, Sapphire 140, Emerald 160, Ruby 180, Diamond 400) | Gem | 99 | as listed | gem nodes by stratum, geodes |
| Prismatic Shard | Gem | 99 | 1500 | Vault, Clockwork Geode 0.5%, Cinder Golem 5% |
| Rough / Crystal / Magma / Clockwork Geode | Geode | 99 | 10/25/40/60 | 1 in 15 rocks per stratum, geode nodes |
| Herbs (Glowcap 12, Wild Sage 8, Reedmint 10, Skybloom 60) | Herb | 99 | — | surface nodes; used in 04-forging potions/coatings |
| Fossils (8) | Fossil | 20 | 50–250 | dig spots, geodes, relic rubble; donating to the Museum gives reputation (09) |
| Monster drops (13, §5.6) | Monster | 99 | 5–500 | mine monsters |
| Torch | Tool | 99 | 2 | craft |
| Bombs | Tool | 20 | 25/50/100 | craft |
| Pickaxes / Lanterns | Tool | 1 | not sellable | forge |

*Stone stack 999 is a lever (Q4). Proposed: raise `MAX_STACK` for the Stone category only.

## 7. Processing steps

1. **Break node** → drops raw ore/stone/gem/geode (`OreDrop`).
2. **Smelt** at the **Smelter** (new homestead station next to the forge furnace): pick a bar recipe, insert ore + coal, then the timer runs on in-game time:
   - Copper/Brass/Iron bar: 1 day phase (90 s real). Steel/Silver: 1 phase. Aethersteel/Mythril: 2 phases. Starsteel/Destinium: overnight (completes on sleep).
   - Queue size: 1 (Lv1), 3 (Lv2), 5 (Lv3+). Output collects in the smelter until taken.
3. **Forge** (04): bars are the *frame/core* components. Gems become *polish/decorations*. Monster parts provide *skills/element*. This is the GDD's "frames, cores, polish".
4. **Geode cracking**: Hearthspire Blacksmith (Ottilie, see 08) charges 25 g, or the Geode Press machine (Forge Lv2, 3 Iron Bar + 1 Gear Spring) is free and takes 1 energy. Loot tables per geode kind are in `assets/data/geodes.ron`.
5. **Grinding**: the Gem Grinder (workbench) turns 3 low gems into gem dust (forge polish) or cuts a gem into a **Decoration blank** (04 decorations).
6. **Stone → construction**: stone and clay go into farm machines (02), fences (07), and the homestead path.
7. **Recipe migration**: once the Smelter ships, PR #95's raw-ore costs in `recipes.ron` become bar costs (Iron Sword: 5 Slime Gel + 2 Iron Bar; Iron Pickaxe: 3 Iron Bar; copper-bound Iron Pickaxe: 3 Copper Bar + 1 Coal). One ticket, data + test updates.

## 8. Mine monster behavior (combat-lite, top-down)

Depends on 05-combat ticket **CMB-04** (top-down field melee with the equipped weapon; the hunter's weapon family maps to a dwarf swing set). Until that lands, mine floors spawn monsters only from F3 behind a feature flag.

| Monster | HP | Dmg | Behavior | Drops |
| --- | --- | --- | --- | --- |
| Dust Slime | 20 | 5 | hop toward the player within 5 tiles, lunge | Dust Gel 80%, Slime Gel 20% |
| Mine Bat | 14 | 4 | erratic flight, dives every 3 s | Bat Wing |
| Rock Crab | 40 | 8 | disguised as a stone node; reveals at 2 tiles; pickaxe hits do 2× | Rock Crab Shell, Copper Ore |
| Gearmite | 10 | 3 | swarms, flees at 30% HP, eats ore drops on the ground | Gear Spring, Gearmite Core 10%, Coal |
| Copper Beetle | 60 | 12 | telegraphed straight charge; stuns itself on walls | Beetle Carapace, Copper Ore ×3 |
| Steam Imp | 30 | 6 | keeps 4 tiles away, steam-puff projectile | Steam Imp Horn, Sulfur |
| Crystal Wisp | 35 | 9 | blinks, 3-shard spread; immune to blunt while shielded | Wisp Mote, Aether Crystal |
| Shardback Tortoise | 120 | 14 | shell spin; breakable back shard (part break → Tortoise Shard ×2) | Tortoise Shard |
| Gloom Bat | 28 | 8 | dims light radius by 2 within 4 tiles | Gloom Membrane |
| Magma Slug | 70 | 10 | leaves a lava trail that lasts 6 s | Magma Gland, Obsidian |
| Ember Moth | 22 | 7 | drops embers that burn | Ember Dust |
| Cinder Golem | 600 | 25 | F40 mini-boss; 2 phases; molten armor break | Cinder Heart (100%), Mythril Ore ×5, Ruby 30% |

## 9. Hooks into other systems

- **Forge (04)**: bars are required in all metal recipes. Gems feed decorations and polish. Aether Crystal and Starmetal gate High/Master rank. Cinder Heart and Gearmite Core gate tool tiers. Forge level gates the smelt recipes (Steel Lv2, Aethersteel Lv3, Starsteel Lv4, Destinium Lv5). Forge upgrades themselves cost mine materials (04 §upgrade table).
- **Hunting (05/06)**: Sulfur + Coal → bombs and *Barrel Bomb* traps for hunts. Stormglass/Frost Crystal → elemental coatings. Gems → element-tuned decorations used for matchups. Rock Crab Shell → Miner's Helmet (Low-rank armor). Large monster *Ironhide Behemoth* (06) lives in the Clockwork Galleries hunting ground and drops ore-flavored parts.
- **Farming (02)**: stone/clay/copper bars build sprinklers, machines, and the Geode Press. Coal fuels the Kiln. Clearing homestead boulders frees farm tiles.
- **Fishing (03)**: underground lakes on F20 and F35 have mine fish (Cave Eel, Lava Koi). Pearl clams in the riverlands.
- **Social (08)**: Ottilie the blacksmith cracks geodes; gem gifts are loved by several villagers; Mine Foreman Brunwald's quest line (find his lost crew cog on F15/F25/F35) unlocks the elevator to F40 and the Mythril smelt recipe early.
- **Progression (09)**: first reach of F10/20/30/40 grants reputation +10/+20/+30/+50. Fossils donated to the Museum add reputation.
- **Day cycle (11)**: the mine is a time sink (phases advance in the mine too, Q2). Rare-node rolls read the season/weather.
- **Narrative (12)**: Act 1 ends with the forge's Copper Heart restored (needs F10 chest Copper Heart Coil). Destiny Vault unlocks after the finale.
- **Saves (16)**: §13.

## 10. Energy, time, risk

- Energy per swing by tier (§4.1). Bomb 0 energy. Charge strike = 2× swing cost. Clank = 1 energy.
- The energy reserve rule (PR #95: node needs reserve ≥ X) is kept. Low energy (≤ 10%) uses the exhausted walk C20 and −20% move speed.
- 0 energy in the mine → faint (C19) → wake at home next morning with −10% gold (max 500 g), lose 1 random non-tool stack from this mine trip, and start with 50% energy.
- HP 0 from monsters/hazards → the same faint result.
- Time: phases advance in the mine at the same 90 s rate (Q2). Evening in the mine adds a "you're tired" prompt. After Evening + 90 s, a forced faint at 2 AM equivalent.
- Risk scales with depth: monster density 0.5 → 2.5 per 100 tiles; hazard count 0 → 6 per floor.

## 11. Progression and gating

| Gate | Unlocks |
| --- | --- |
| Start | F1–5, Copper Pickaxe, surface copper/stone |
| Smelter built (quest "Rekindle", Forge Lv1) | bars |
| Iron Pickaxe | iron (F6+) |
| Elevator repaired (Brunwald quest 1 or reach F5) | elevator stops |
| Lantern (F1 chest) | F11+ darkness becomes playable |
| Brasswork Pickaxe | silver tier nodes still need T3; Zinc is T2 |
| Story Act 2 | Highland Ruins surface nodes; Sunstone summer |
| Aether Pickaxe | Mythril (F31+) |
| Cinder Golem defeated | Mythril Drill-Pick recipe; Molten Deep shortcut |
| Story finale | Destiny Vault 41+, Destinium |
| Season/weather | Frost Crystal (winter), Sunstone (summer), Stormglass (thunderstorm) |

Node required tier: Stone/Clay/Copper/Coal 1; Zinc/Iron 2; Silver/Obsidian/Sulfur/Aether Crystal 3; Mythril 4; Starmetal 5; Destinium 6.
Hardness (base hits): Stone 1, Clay 1, Copper 2 (exists), Coal 2, Zinc 3, Iron 4 (exists), Silver 5, Aether 5, Obsidian 6, Sulfur 3, Mythril 7, Starmetal 8, Destinium 10.

## 12. UI

- **Floor badge** HUD top-left in the mine: "B12 — Clockwork Galleries", with a stratum icon.
- **Elevator panel**: a grid of unlocked stops (5, 10, …), keyboard/gamepad navigable, number font sprites.
- **Node prompt** (`interaction_prompt.rs` exists): adds "Needs Iron Pickaxe" when the tier is too low and "Hits: 2/4".
- **Smelter window**: recipe list, input counts with **source hints** ("Iron Ore — Mine B6–B20"), fuel slot, queue, time remaining. Uses the 13-ui-ux material-source component.
- **Geode result card**: reveals contents with the V11 burst.
- **Darkness**: overlay plus a lantern status pip.
- **Fossil/museum collection page** (in the 13-ui-ux collections tab).
- **Mine map**: none at launch. A mini-map of the current floor shows a revealed ladder (Aether Lamp only).

## 13. Audio

| Cue | Type | Notes |
| --- | --- | --- |
| `pick_hit_stone`, `pick_hit_metal`, `pick_hit_crystal` (3 variants each, ±5% pitch) | sfx | on hit frame f2 |
| `pick_clank` | sfx | tier too low |
| `node_break_*` per family (stone, metal, crystal, gem) | sfx | |
| `charge_hum` loop, `slam_impact` | sfx | T3+ |
| `ladder_reveal`, `ladder_climb` (4 steps) | sfx | |
| `elevator_doors`, `elevator_ride` loop, `elevator_ding` | sfx | |
| `bomb_fuse` loop, `bomb_blast` | sfx | |
| `steam_vent_hiss`, `rock_rumble` (telegraph), `rock_impact`, `lava_bubble` loop, `cavein_rumble` | sfx | |
| `smelter_roar` loop, `bar_clink`, `geode_crack`, `gem_chime` | sfx | |
| Per mine monster: `idle`, `telegraph`, `attack`, `hurt`, `death` (12 × 5) | sfx | the telegraph sound is required for readability |
| Music: `mine_upper`, `mine_clockwork`, `mine_crystal`, `mine_molten`, `mine_vault` | music | steampunk ambient, 60–90 s loops; crossfade 1.5 s on stratum change |
| Ambience beds per stratum (drips, gears, crystal hum, magma) | ambient | |

Uses the existing `src/audio/sfx.rs` and `music.rs` plumbing.

## 14. Save data

Extend `PlayerProfile` (bump `version`; add a migration fixture like `profile_v6_24_slots.ron`):
- `mine: MineProgress { deepest_floor: u16, elevator_stops: BitSet/Vec<u16>, stratum_chests_opened: u8 bitmask, vault_unlocked: bool }`
- `mine_floor_today: Option<{floor, broken_node_ids, ladder_revealed}>`: only if mid-floor save is allowed (Q5: proposed no; the floor resets on exit).
- `surface_nodes: Vec<{zone, node_id, depleted_until_day}>` (generalizes the current ore-node respawn data).
- `smelter: SmelterState { queue: Vec<{recipe, finishes_at: (day, phase)}>, output: Vec<(MaterialId, u32)> }`
- `geode_press`, lantern/pickaxe tiers are inventory items (no extra field).
- `museum_donations: Vec<MaterialId>`.

## 15. Balance levers (all RON data, `assets/data/nodes.ron`, `smelting.rs`, `mine_floors.ron`)

Node hardness and required tier; drop counts (min/max) and bonus chance; pickaxe energy/swing and swing speed; energy reserve rule; smelt input counts, coal, time; floor monster density; hazard counts and damage; ladder-reveal threshold (40%); shaft hole chance; geode rate (1/15) and geode tables; gem node rate per stratum; surface node counts and respawn days; faint penalty (gold %, item loss); sell prices; elevator interval (5).

## 16. Open design questions (with proposed answers)

| # | Question | Proposed answer |
| --- | --- | --- |
| Q1 | Do homestead/mine actions need up/down facings? | **No for launch.** All current dwarf sets are side-only with `flip_x`. Keep that, and add back-view sets only for ladders (C11/C12). Revisit after the art pass (15). |
| Q2 | Does time pass in the mine? | **Yes**, same 90 s phases. Otherwise the mine is a free-time exploit and the day loop breaks. |
| Q3 | Keep raw ore in forge recipes (PR #95) or require bars? | **Require bars** after the Smelter ships (MIN-06 migration). Raw ore is only for smelting. That gives the "processing step" the GDD implies. |
| Q4 | Stone stack 999? | Yes for Stone only. Everything else stays at 99. |
| Q5 | Save mid-floor? | No. Leaving or sleeping resets the floor. Deepest floor and elevator stops persist. |
| Q6 | Top-down mine combat or the side-scroll dungeon engine? | **Top-down combat-lite** (CMB-04). The dungeon side-scroller is frozen and belongs to hunts. |
| Q7 | Fixed or procedural floors? | Template + seeded scatter per stratum (4–6 templates each). Floor 1 keeps the PR #95 layout. |
| Q8 | Who cracks geodes before the Geode Press? | Ottilie (blacksmith), 25 g. That gives an early gold sink. |
| Q9 | Ladder reveal rule | Under a random rock, guaranteed after 40% broken or all monsters cleared. |
| Q10 | Faint penalty harshness | Mild (cozy tone): ≤10% gold capped at 500, 1 stack lost, never tools/quest items. |

## 17. Ticket breakdown

Legend: **[ART]** sprite/animation only (PNG + draw script + loader/anim wiring constants), **[GAME]** gameplay/logic, **[DATA]** RON/balance, **[UI]**, **[AUD]**, **[SAVE]**. "deps" lists hard dependencies; *soft* means a placeholder is OK until the ART ticket merges.

Foundation
- **MIN-01 [GAME]** `NodeKind` + `nodes.ron` data table (hardness, required tier, drops, sell price) replacing the hardcoded `OreKind` matches, with Copper/Iron behavior unchanged and tests kept. deps: PR #95.
- **MIN-02 [ART]** Ore node state sprites (intact/crack-1/crack-2/rubble) for Stone, Copper, Iron, Coal, Clay + shatter burst E2 (stone, copper, iron families) + impact VFX V1/V2/V3. deps: none.
- **MIN-03 [GAME]** Node visuals switch states per hit and play shatter on break, wired to MIN-02 sprites. deps: MIN-01, MIN-02.
- **MIN-04 [ART]** 16×16 item icons for Stone, Coal, Clay, Copper Ore, Iron Ore, Copper Bar, Iron Bar, Copper/Iron Pickaxe (replacing swatches for these items only). deps: none.
- **MIN-05 [UI]** Inventory slots render item icon PNGs when present, with swatch fallback. deps: MIN-04 (soft); shared with UI-03.
- **MIN-06 [ART]** Smelter station sprites E20/E21 + fire/steam VFX V9/V10 + dwarf smelter-load frames C21. deps: none.
- **MIN-07 [GAME]** Smelter station on the homestead: queue (size 1), coal fuel, phase-based timer, collect output; `smelting.ron`. deps: MIN-01, MIN-06.
- **MIN-08 [UI]** Smelter window with recipe list, inputs with source hints, timer. deps: MIN-07.
- **MIN-09 [DATA]** Migrate `recipes.ron` metal costs from raw ore to bars; update forge tests. deps: MIN-07.
- **MIN-10 [SAVE]** Persist the smelter queue/output and the generalized node depletion; profile version bump + migration fixture. deps: MIN-07.

Mine floors
- **MIN-11 [GAME]** `MineFloor` resource: floor index, seeded scatter from templates, current layout becomes the F1 template; HUD floor number placeholder. deps: MIN-01.
- **MIN-12 [ART]** Ladder reveal/idle E5/E6 + dwarf ladder down/up C11/C12. deps: none.
- **MIN-13 [GAME]** Ladder-down reveal rule and descend/ascend between floors (KeyE), with headless app tests like `transition_tests.rs`. deps: MIN-11, MIN-12.
- **MIN-14 [ART]** Upper Shafts tileset + 6 props (E18 stratum 1). deps: none.
- **MIN-15 [GAME]** Stratum 1 floor templates (5) using MIN-14 tiles; F1–10 distributions. deps: MIN-11, MIN-14.
- **MIN-16 [ART]** Elevator cage E8, gauge E9, steam V12, dwarf elevator idle C13. deps: none.
- **MIN-17 [GAME]** Elevator: unlock stops at 5n, ride between stops, entrance cage. deps: MIN-13, MIN-16.
- **MIN-18 [UI]** Elevator panel + floor badge HUD (+ number font sprites, part of this ticket's art). deps: MIN-17.
- **MIN-19 [SAVE]** `MineProgress` (deepest floor, stops, chests) persisted. deps: MIN-17.
- **MIN-20 [ART]** Rest-floor supply crate E10 + stratum chest E11 (4 skins). deps: none.
- **MIN-21 [GAME]** Rest floors (5n) with crate + first-visit chest loot (F10 Copper Heart Coil for the story). deps: MIN-15, MIN-20.

Tools
- **MIN-22 [ART]** Iron Pickaxe swing frames C2 + clank recoil C9 (T1, T2 heads) + V3. deps: none.
- **MIN-23 [GAME]** Per-tier swing sprite selection and clank outcome animation (replace any non-frame feedback). deps: MIN-22.
- **MIN-24 [ART]** Pickup bend C10 + ore fly V5 + gem glint V4. deps: MIN-04.
- **MIN-25 [GAME]** Ore drops become world pickups with magnet + pickup animation (instead of straight-to-inventory). deps: MIN-24.
- **MIN-26 [ART]** Lantern idle/walk C14/C15 + lantern held T2 (tin) + darkness masks E19. deps: none.
- **MIN-27 [GAME]** Darkness overlay F11+ and lantern light radius; Tin Lantern in the F1 chest. deps: MIN-21, MIN-26.
- **MIN-28 [ART]** Torch placed T3 + place frames C17. deps: none.
- **MIN-29 [GAME]** Torch craft + place (light source until floor exit). deps: MIN-27, MIN-28.
- **MIN-30 [ART]** Bomb throw C16 + bomb fuse T4 (3 tiers) + explosion V8. deps: none.
- **MIN-31 [GAME]** Bombs: craft, throw, fuse, radius break/damage. deps: MIN-30, MIN-01.
- **MIN-32 [ART]** T3 Brasswork swing C3 + charge C7 + slam C8 + V6/V7. deps: none.
- **MIN-33 [GAME]** T3 pickaxe + charge-line strike + forge recipe. deps: MIN-32, MIN-09, FRG-31 (Forge L2).
- **MIN-34 [ART]** T4/T5/T6 swing, charge, and slam sets (C4–C6 + their C7/C8/C9 variants). deps: MIN-32.
- **MIN-35 [GAME]** T4–T6 pickaxes and their area strikes/specials (gem chance, auto-collect, ladder reveal). deps: MIN-34, MIN-33.

Content per stratum
- **MIN-36 [ART]** Node sprites for Zinc, Silver, Aether Crystal, Mythril, Obsidian, Sulfur + their shatter families. deps: MIN-02.
- **MIN-37 [ART]** Clockwork Galleries tileset + props + minecart E12 + steam vent E13. deps: none.
- **MIN-38 [GAME]** Stratum 2 (F11–20) templates, Zinc/Iron/Coal distribution, brass smelt. deps: MIN-15, MIN-36, MIN-37.
- **MIN-39 [ART]** Crystal Veins tileset + spike tile E15. deps: none.
- **MIN-40 [GAME]** Stratum 3 (F21–30). deps: MIN-38, MIN-39.
- **MIN-41 [ART]** Molten Deep tileset + lava autotile E16 + cave-in E17 + burn VFX V14. deps: none.
- **MIN-42 [GAME]** Stratum 4 (F31–40). deps: MIN-40, MIN-41.
- **MIN-43 [ART]** Falling rock E14 + hurt C18 + faint C19 + exhausted walk C20 + shaft fall C23 + shaft hole E7. deps: none.
- **MIN-44 [GAME]** Hazards: falling rock, steam vent, spikes, lava, cave-in, runaway cart (one system, data-driven). deps: MIN-43, MIN-37/39/41 (per hazard, soft).
- **MIN-45 [GAME]** Faint/exhaustion outcome (wake at home, penalties). deps: MIN-43; shared with 11-day CYC-07.
- **MIN-46 [GAME]** Shaft holes (drop 2–4 floors, fall damage). deps: MIN-13, MIN-43.

Gems, geodes, gathering
- **MIN-47 [ART]** Gem nodes E3 (8) + geode nodes E4 (4) + gem/geode icons. deps: MIN-02.
- **MIN-48 [GAME]** Gem/geode nodes + 1-in-15 rock geode roll; `geodes.ron`. deps: MIN-47, MIN-01.
- **MIN-49 [ART]** Geode Press E22 + press frames C22 + geode open V11. deps: none.
- **MIN-50 [GAME]** Geode cracking: Ottilie service (stub vendor until 08) + Geode Press machine. deps: MIN-48, MIN-49.
- **MIN-51 [ART]** Surface node zone skins + forage frames C24 + herb/fossil icons + dig spot E24. deps: MIN-02.
- **MIN-52 [GAME]** Surface nodes on homestead/forest (copper, stone, herbs, fossil dig) respawning on sleep. deps: MIN-51, MIN-01, MIN-10.
- **MIN-53 [GAME]** Seasonal/weather rare nodes (Frost Crystal, Sunstone, Stormglass). deps: MIN-52, CYC-02 seasons, CYC-05 weather.

Mine monsters (each monster = 1 ART + 1 GAME ticket)
- **MIN-54/55 [ART]/[GAME]** Dust Slime (M1). GAME deps: CMB-04, MIN-15.
- **MIN-56/57** Mine Bat (M2). **MIN-58/59** Rock Crab (M3). **MIN-60/61** Gearmite (M4). **MIN-62/63** Copper Beetle (M5). **MIN-64/65** Steam Imp (M6 + projectile). **MIN-66/67** Crystal Wisp (M7). **MIN-68/69** Shardback Tortoise (M8, part break). **MIN-70/71** Gloom Bat (M9). **MIN-72/73** Magma Slug (M10). **MIN-74/75** Ember Moth (M11). **MIN-76/77** Cinder Golem mini-boss (M12, 2 phases). Each GAME ticket deps on its ART ticket + CMB-04 + its stratum ticket.
- **MIN-78 [ART]** Monster hit-flash silhouettes V13 for all mine monsters (batched with each monster's ART if preferred).

Polish
- **MIN-79 [AUD]** Mining sfx set (pick hits, clank, breaks, ladder, elevator, smelter, geode). deps: MIN-03, MIN-07, MIN-17.
- **MIN-80 [AUD]** Mine music per stratum + ambience beds + crossfade. deps: MIN-15.
- **MIN-81 [DATA]** Mining balance pass (energy/swing, drop rates, smelt times, prices). deps: MIN-42.
- **MIN-82 [GAME]** Destiny Vault endless floors 41+ (Starmetal/Destinium, elite variants). deps: MIN-42, NAR-16.
- **MIN-83 [GAME]** Ore Satchel + Miner's Helmet + Miner's Ring items. deps: MIN-09, FRG-21.
- **MIN-84 [GAME]** Mine quest hooks (Brunwald cogs F15/25/35). deps: SOC-20, MIN-40.

**Ticket count: 84** (ids 01–84, including 12 paired monster ART/GAME = 24).

**Build first (top 3):** MIN-01 (NodeKind data table), MIN-06 → MIN-07 (Smelter art + Smelter station: bars are the forge spine), MIN-11 → MIN-13 with MIN-12 (multi-floor mine with ladder art).
