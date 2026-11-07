# 03 — Fishing

Authority: GDD §4.3 ("mini-game fishing at multiple locations and times of day/season. Fish can be cooked, used as bait, or processed into oils, scales, and bones for gear. Legendary fish drop unique materials for endgame weapons or special forge recipes"), §3 (Prepare & Gather).

Build priority: **3 of 16**. Nothing exists yet (EPICS #27–30 open; landmark #63 "fishing dock" open).

## 0. Standing rules

New in-house dwarf frames for every action (cast, wait, hook, reel, catch, fail, pot). The minigame bar is UI and needs drawn UI sprites. **The player character still animates during the minigame**, so a UI-only minigame does not satisfy the rule. No code comments; zero warnings. Dungeon art untouched. Sprite/animation work goes in separate [ART] tickets.

## 1. Already exists

Nothing fishing-specific. Reuse: homestead top-down movement and `flip_x` facing, `interaction_prompt.rs`, inventory (stack 99, categories), `DayClock` phases, PR #96 food-buff pipeline, forge `recipes.ron`, `audio/sfx.rs`.

## 2. Where it happens

| Spot | Zone | Water type | Seasons/times | Unlock |
| --- | --- | --- | --- | --- |
| Homestead Pond + Fishing Dock | Homestead | pond | all | start (dock repaired day 2 by quest "Old Dock") |
| Hearthspire Canal | Village (10) | canal | all | village access |
| Mistbrook Stream | Misty Forest (exists) | stream | spring–fall, frozen winter except the ice hole | start |
| Clockwork Dam Reservoir + Spillway | Riverlands | lake + rapids | all | Act 1 |
| Mine Lake B20 / Lava Pool B35 | Mine (01) | cave lake / lava | all | reach floor |
| Ruin Aqueduct | Highland Ruins | aqueduct | all | Act 2 |
| Cloud Shoals | Floating Islets | sky (cloud fishing) | all, best in storms | Act 3 |
| Ice Fishing Hole | Forest/Reservoir in winter | ice | winter | Iron rod |

Each spot has a `FishingZone` rect. Water tiles allow casting. Fish shadows spawn in the zone (§5.3).

## 3. Entities and objects

`FishingSpot {zone_id, water: WaterKind}`, `Bobber {state: Flying|Floating|Nibble|Bite|Hooked}`, `FishShadow {size, species_hint}`, `CrabPot {bait, ready_on_day}`, `FishingMinigame` resource (catch zone, fish position, progress, treasure), `TreasureChest` (minigame), `FishPond` building (07 link, post-launch), `FishSmoker` machine, `Aquarium` (museum collection, optional), `FishingDock` (repairable).

## 4. Tools and tiers

| Rod | Cast range | Catch zone | Bait slot | Tackle slots | Recipe | Gate |
| --- | --- | --- | --- | --- | --- | --- |
| Bamboo Rod | 4 tiles | 64 px | no | 0 | starter (dock quest) | — |
| Copper Rod | 5 | 72 | yes | 0 | 3 Copper Bar + 10 Wood | Forge Lv1 |
| Iron Rod | 6 | 80 | yes | 1 | 3 Iron Bar + 1 Fang | Forge Lv1, ice fishing |
| Brass Reel Rod | 7 | 88, auto-reel ×1.1 | yes | 1 | 3 Brass Bar + 2 Gear Spring | Lv2 |
| Aether Rod | 8 (can cast into lava/sky) | 96 | yes | 2 | 3 Aethersteel + 1 Wisp Mote | Lv3 |
| Destiny Rod | 9 | 104, legendary bite +50% | yes | 2 | 2 Starsteel + 1 Destiny Spark | Lv5 |

Energy per cast: 4 (Bamboo) → 2 (Destiny). A failed catch costs no extra.

Bait (stack 99): Worm Bait (dig spots/compost, 1 Fiber), **Slime Bait** (Slime Gel ×2 → 5 bait; this is the hunt→fish hook), Gel-Glow Bait (night fish ×2), Magnet Bait (treasure ×3; Copper Ore + Slime Gel), Monster Bait (any monster part + Fish Oil: rare fish weight ↑, required for legendaries), Storm Bait (Stormcell; Cloud Shoals).
Tackle (durability 20 catches): Spinner (bite speed +), Brass Bobber (catch zone +16 px), Lead Weight (fish moves less), Trap Bobber (progress drains slower), Treasure Lens (treasure chance ×2), Gear Reel (auto-reel 15% of the time).
Crab Pot (10 Wood + 3 Copper Bar): passive catch overnight with bait.

## 5. ANIMATIONS (most detailed section)

Dwarf 16×28 on a **64×60 canvas** (rod extends forward, dwarf at (16,16)), facing **right** with `flip_x`. Casting toward up/down water uses the side frame facing the nearest horizontal water direction. Back/front cast sets are open question Q1 (proposed: defer). Timings are per frame.

### 5.1 Character sets

| # | Set | Frames | Canvas | Facing | Timing | Loop | Trigger | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FS1 | Cast wind-up (power charge) | 3 | 64×60 | R | f0 0.1, f1↔f2 0.12 while charging | f1↔f2 | — | rod behind the head; per-rod tip drawn |
| FS2 | Cast release | 4 | 64×60 | R | 0.05, 0.05, 0.06, 0.1 | no | f2 spawns bobber | |
| FS3 | Waiting idle with rod | 4 | 64×60 | R | 0.3 ×4 | yes | — | breathing + rod tip bob |
| FS4 | Nibble twitch | 2 | 64×60 | R | 0.08 ×2 | no | — | plays with bobber nibble |
| FS5 | Hook yank | 3 | 64×60 | R | 0.04, 0.05, 0.08 | no | f1 = hook check | |
| FS6 | Reel loop (neutral) | 4 | 64×60 | R | 0.1 ×4 | yes | — | during the minigame |
| FS7 | Reel strain (fish pulling away) | 4 | 64×60 | R | 0.08 ×4 | yes | — | when the fish is outside the catch zone; rod bends further |
| FS8 | Reel winning (fish in zone) | 4 | 64×60 | R | 0.1 ×4 | yes | — | upright, cranking |
| FS9 | Catch land | 4 | 64×60 | R | 0.06 ×4 | no | f3 | fish flies to hand |
| FS10 | Catch hold-up | 2 | 24×40 | front | 0.3 ×2 | no | — | **shared with FRM F12**; fish icon at the hand anchor; legendary uses FS11 |
| FS11 | Legendary hold-up (both arms, sparkle) | 4 | 40×48 | front | 0.15 ×4 | no | — | |
| FS12 | Line snap / fail | 4 | 64×60 | R | 0.05, 0.08, 0.12, 0.2 | no | — | stumbles back |
| FS13 | Reel in empty (cancel) | 3 | 64×60 | R | 0.07 ×3 | no | — | |
| FS14 | Crab pot place | shared `dwarf_m_place` (01 C17) | | | | | | |
| FS15 | Crab pot check/haul | 4 | 32×36 | R | 0.09 ×4 | no | f3 | |
| FS16 | Bait attach | 3 | 24×28 | R | 0.1 ×3 | no | — | plays when bait is equipped at cast |
| FS17 | Ice hole chip (winter) | uses the pickaxe swing C1 | | | | | | 3 hits opens the hole |
| FS18 | Treasure open | 3 | 24×28 | front | 0.1 ×3 | no | — | |

Per-rod variation: FS1–FS9, FS12, and FS13 include the rod in-frame. **6 rods × 12 sets ≈ 6 × 41 frames = ~246 frames.** To keep scope: draw the dwarf body once per set and composite rod layers in the draw script at build time (baked PNGs, not runtime tint). The script outputs one PNG per rod per frame.

### 5.2 Bobber, line, tackle sprites

| # | Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| FB1 | Bobber flight spin | 4 | 8×8 | 0.05 loop | along the arc |
| FB2 | Bobber land splash | 4 | 24×16 | 0.05 ×4 | |
| FB3 | Bobber float idle | 4 | 8×12 | 0.25 loop | per bobber type (default, Brass Bobber, Trap Bobber) = 3 skins |
| FB4 | Bobber nibble dip | 3 | 8×12 | 0.06 ×3 | |
| FB5 | Bobber bite plunge + "!" | 3 + 1 | 8×16 / 8×8 | 0.05 | the "!" is drawn as a sprite above the dwarf |
| FB6 | Fishing line | procedural 1-px line, Bevy Gizmos or a stretched 1×1 sprite | — | — | the line is not an action frame; OK |
| FB7 | Crab pot in water idle bob + ready shimmer | 2 + 2 | 16×16 | 0.4 / 0.2 | |
| FB8 | Lava bobber / sky bobber variants | 4 + 4 | 8×12 | 0.25 | |

### 5.3 World / entity sprites

| # | Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| FW1 | Fish shadow small/medium/large/legendary | 4 each | 16×8 / 24×10 / 32×12 / 48×16 | 0.15 swim loop | `flip_x` by direction; reveals spot richness |
| FW2 | Fish jump splash (ambient tell for a rich spot) | 5 | 24×24 | 0.06 ×5 | |
| FW3 | Water ripple ring | 4 | 32×16 | 0.1 ×4 | |
| FW4 | Pond/stream/canal water animated autotiles | 3 frames × 16 tiles per water kind (pond, stream, canal, reservoir, cave, lava, cloud) | 32×32 | 0.3 loop | lava reuses MIN E16 |
| FW5 | Fishing dock broken/repaired | 2 | 96×64 | static | |
| FW6 | Ice hole (winter) | 3 states | 32×32 | — | |
| FW7 | Fish Smoker machine idle/working/ready | 1 + 4 + 2 | 32×48 | 0.12 | uses FRM machine system |
| FW8 | Cloud shoal drifting clouds | 6 | 64×32 | 0.2 loop | islets |

### 5.4 Minigame UI sprites (drawn PNGs, 1× pixel art, shown at 3×)

| # | Set | Frames | Size | Notes |
| --- | --- | --- | --- | --- |
| FU1 | Vertical reel frame (brass gauge style) | 1 | 24×150 | steampunk gauge |
| FU2 | Catch zone bar per rod (6 heights) | 1 each + 2-frame "in-zone glow" | 16×64..104 | |
| FU3 | Fish marker icons (difficulty silhouettes: dart, smooth, sinker, floater, mixed) + legendary | 2 each (wiggle) | 16×16 | 0.12 loop |
| FU4 | Progress meter fill (gauge needle + 8 fill states) | 9 | 8×150 | |
| FU5 | Treasure chest marker | 3 (appear, idle, captured) | 16×16 | |
| FU6 | Perfect catch stamp | 4 | 48×16 | 0.06 |
| FU7 | Cast power meter | 12 fill states | 8×48 | |
| FU8 | Catch result card frame | 1 | 120×64 | name, size, quality star |

### 5.5 Item icons (16×16)

30 regular fish + 5 legendary = 35. Processed: Fillet, Fish Oil, Fish Scale ×3 grades (Common/Shimmer/Mythic), Fish Bone, Roe, Aged Roe, Smoked Fish (×quality), Chum. Bait ×6, tackle ×6, rods ×6, crab pot, trash ×4 (Driftwood, Rusty Cog, Soggy Boot, Broken Lens), treasure items (shared). ≈ 75 icons.

## 6. Fish (data: `assets/data/fish.ron`)

Columns: location; seasons; time (M=Morning, A=Afternoon, E=Evening); weather; difficulty 1–100; behavior; sell (Normal). Fish have quality (from perfect catches + rod tier) and size (cosmetic, for the field guide).

| Fish | Where | Season | Time | Weather | Diff | Behavior | Sell |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Pond Perch | Pond, Canal | all | any | any | 15 | smooth | 20 |
| Brass Carp | Pond, Reservoir | all | M/A | any | 20 | sinker | 30 |
| Mist Minnow | Stream | Sp/Su/Fa | M | any | 10 | dart | 15 |
| Glowtrout | Stream | Sp/Fa | E | any | 40 | mixed | 60 |
| Rainbow Gearfish | Canal | Su | A | sun | 45 | dart | 75 |
| Sunbass | Reservoir | Su | A | sun | 35 | smooth | 55 |
| Rain Eel | Stream, Reservoir | Sp/Fa | E | rain | 55 | mixed | 90 |
| Cogshell Crab (pot) | Pond, Canal | all | — | — | — | pot | 40 |
| Copper Crayfish (pot) | Stream | all | — | — | — | pot | 25 |
| River Pike | Spillway | all | M/A | any | 50 | dart | 80 |
| Spillway Salmon | Spillway | Fa | any | any | 45 | floater | 85 |
| Dam Catfish | Reservoir | Sp/Fa | E | rain | 60 | sinker | 110 |
| Frost Cod | Ice hole | Wi | any | any | 40 | smooth | 70 |
| Icicle Pike | Ice hole | Wi | M | snow | 65 | dart | 120 |
| Snowfin Char | Ice hole | Wi | E | any | 55 | mixed | 100 |
| Cave Eel | Mine B20 | all | any | — | 55 | mixed | 120 |
| Blind Glowfish | Mine B20 | all | any | — | 40 | floater | 90 |
| Crystal Shrimp (pot) | Mine B20 | all | — | — | — | pot | 70 |
| Lava Koi | Mine B35 | all | any | — | 75 | dart | 300 |
| Cinder Eel | Mine B35 | all | any | — | 80 | mixed | 350 |
| Aqueduct Gar | Ruin Aqueduct | all | A | any | 60 | dart | 150 |
| Relic Turtle | Ruin Aqueduct | Su/Fa | M | rain | 70 | sinker | 220 |
| Clockwork Pufferfish | Canal | Su | A | sun | 65 | floater | 160 |
| Cloudray | Cloud Shoals | all | A | any | 70 | floater | 260 |
| Skyskipper | Cloud Shoals | Sp/Su | M | sun | 60 | dart | 200 |
| Thunderfin | Cloud Shoals | all | any | storm | 85 | mixed | 400 |
| Moonjelly | Pond | Fa/Wi | E | clear | 50 | floater | 140 |
| Emberfin Goby | Stream (hot spring) | Wi | any | any | 45 | smooth | 90 |
| Mudskipper | Pond | Su | M | rain | 30 | dart | 35 |
| Seaweed / Algae (trash-tier) | any | — | — | — | — | — | 5 |

Legendary (one catch each, then rare re-spawn post-game; Monster Bait required; rod gate):

| Legendary | Where | Condition | Diff | Unique drop → use |
| --- | --- | --- | --- | --- |
| Old Brassjaw | Homestead Pond | Fall, rain, Iron Rod+ | 85 | **Brassjaw Gear-Scale** → "Pond King" Hammer upgrade + Forge Lv3 upgrade material |
| Mistwyrm Trout | Mistbrook | Spring, Evening, Brass Rod+ | 90 | **Mistwyrm Fin** → Dual Blades water branch (Master rank) |
| Dam Leviathan Pike | Spillway | Summer, storm | 95 | **Leviathan Spine** → Bow endgame "Spillway Longbow" |
| Magmaw | Mine B35 Lava | any, Aether Rod | 98 | **Magmaw Heartfin** → fire decoration jewel (Lv4) |
| Sky Wyrmling | Cloud Shoals | storm, Destiny Rod | 100 | **Skywyrm Pearl** → Destiny Spark ×1 (the only renewable spark source, once/year) |

## 7. Processing

| Step | Where | Input → Output |
| --- | --- | --- |
| Cook | Kitchen (04) | fish → dishes (Grilled Perch atk+, Eel Stew sharpness, Salmon Bowl stamina regen) |
| Fillet | Workbench "Fish Knife" (free, 0.5 s, uses FS16-like 3-frame cut set FS19, below) | 1 fish → 1–2 Fillet + Fish Bone + Fish Scale (grade by fish tier) |
| Fish Oil | Steam Press (02) | 3 Fillet or 1 Oily fish (Eel/Catfish/Koi) → Fish Oil |
| Smoked fish | Fish Smoker (new, 2 Iron Bar + 20 Hardwood + 5 Coal) | fish → Smoked Fish (×2 value, portable buff) |
| Aged Roe | Preserves Kiln | Roe (from pots/catch 10%) → Aged Roe (gift/price) |
| Bait | Workbench | Chum (2 Fillet) → Fish Bait ×5 |
| **Gear uses** (04) | Forge | Fish Scale → armor plating (Riverscale set, Low/High rank); Fish Bone → bow limbs/hilts; Fish Oil → Coating base + Whetstone oil; legendary drops → §6 |

Add character set **FS19 Fillet cut** (3 frames, 32×32, 0.08 s, front-facing at the workbench).

## 8. Minigame (mechanics)

1. **Cast**: hold to charge (FU7 power 0–100%). Release → distance = power × rod range. Energy is spent at release.
2. **Wait**: bite time 1.5–8 s (faster with Spinner, bait, rich spot). Nibbles (0–3) are fake-outs. Pressing during a nibble spooks the fish (+3 s).
3. **Bite**: "!" for 0.6 s (0.9 s with the Trap Bobber). Press in the window → hook. Otherwise the fish escapes (FS13).
4. **Reel**: Stardew-style vertical bar. Hold to raise the catch zone; gravity pulls it down. The fish marker moves per behavior (dart/smooth/sinker/floater/mixed) and difficulty. Progress fills while the fish is in the zone and drains outside. Treasure appears 15% of the time (Treasure Lens ×2); keep it in the zone to capture.
5. **Result**: fill → catch (FS9 + FS10). Empty → fail (FS12). **Perfect** (never left the zone) → +1 quality step and the FU6 stamp.
6. **Accessibility (13)**: "Relaxed fishing" assist: larger zone +32 px, drain ×0.5, optional auto-hook.

Quality: base Normal. Rod tier ≥ Brass → 20% Silver. Perfect +1 step. Destiny only via Perfect + Destiny Rod.

## 9. Hooks

- **Forge (04)**: scales/bones/oil components; legendary uniques gate specific weapon branches and the forge Lv3/Lv4 upgrades; Skywyrm Pearl → Destiny Spark.
- **Hunting (05/06)**: Fish Oil coating base and whetstone oil; Monster Bait uses monster parts (hunt → fish); river large monster **Tidecaller Kraken-Eel** (06) emerges at the Reservoir if you fish there with Monster Bait at Evening in fall: a hunt-trigger. Fish dishes give hunt buffs.
- **Mining (01)**: mine lakes; Magnet Bait uses copper; crab pots need bars.
- **Farming (02)**: Steam Press/Kiln process fish; worms from compost; Roe → fertilizer (Deluxe).
- **Animals (07)**: Fish Pond building (post-launch) breeds fish; cats (pet) like fish.
- **Social (08)**: Harbormaster Wren's quests (catch X), the fishing festival (Summer 20 "Gearfin Derby"), many loved gifts.
- **UI (13)**: Field Guide fish tab.

## 10. Energy / time / risk

Cast energy 4→2. Each cast takes ~5–15 s real, which spends the phase clock. Risk: low (cozy). Lava Pool B35: a failed catch has a 10% chance of an "ember spit" (5 dmg). Cloud Shoals storms: lightning near the dock (telegraphed) 10 dmg. Line snap only loses the tackle durability tick.

## 11. Progression / gating

Bamboo Rod at the "Old Dock" quest day 2 → pond + stream. Copper Rod → bait. Iron → tackle + ice. Brass → Ruin Aqueduct difficulty. Aether → lava/sky. Destiny → Sky Wyrmling. Fishing XP feeds the player level (09). Fish spots unlock with zones (10).

## 12. UI

Minigame overlay (FU1–FU8); cast meter; catch card; tackle/bait slots on the rod tooltip (inventory: rod item shows 2 sub-slots); crab pot hover (ready/bait); Field Guide fish page (silhouette until caught; shows location/season/time/weather once caught 1×, size record); tackle durability bar.

## 13. Audio

cast_whoosh, reel_click loop (pitch with speed), line_tension loop (rises when the fish is outside the zone), bobber_plop, nibble_tick, bite_splash + "!" sting, catch_splash, catch_jingle (quality variants), perfect_fanfare, legendary_theme (short orchestral sting), line_snap, treasure_chime, crab_pot_haul, water ambience per spot (pond frogs, stream babble, dam machinery hum, cave drips, lava bubbles, sky wind). Music: fishing uses zone music. Optional "Derby" festival track.

## 14. Save data

`fishing: FishingProgress { caught: HashMap<FishId, {count, max_size, best_quality}>, legendaries_caught: bitmask, dock_repaired: bool }`; `crab_pots: Vec<{zone, tile, bait, ready_day, catch: Option<..>}>`; rod item with `tackle: [Option<(TackleId, durability)>; 2]`, `bait: Option<(BaitId, count)>`. Profile version bump.

## 15. Balance levers

Bite time range; nibble count; hook window; catch zone px per rod/tackle; fish difficulty and behavior curves; progress fill/drain rates; treasure chance; quality chances; energy per cast; fish sell prices; processing ratios; legendary conditions; crab pot catch tables.

## 16. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Up/down cast facing? | Side-only with flip (matches the dwarf). Water placement puts castable edges on the left/right of the player paths. |
| Q2 | Minigame style (Stardew bar vs timing ring)? | Stardew-style bar. It is proven, and the accessibility assist is easy. |
| Q3 | Do rods use per-rod frames? | Yes, baked by the script compositing rod layers onto shared dwarf frames. These are new PNGs, not tints. |
| Q4 | Fish quality? | Yes, same `ItemQuality` as crops (FRM-08). |
| Q5 | Where does the first rod come from? | The "Old Dock" quest from Harbormaster Wren on day 2. Fallback before 08 ships: a starter-chest item. |
| Q6 | Legendary repeatable? | Once per save, plus a "Legendary Echo" post-game respawn each year for farming its material. |
| Q7 | Crab pots require Wren's hearts? | No. Forge Lv1 recipe. |

## 17. Ticket breakdown

- **FSH-01 [ART]** Homestead pond water autotiles (FW4 pond) + ripple FW3 + broken/repaired dock FW5. deps: none.
- **FSH-02 [GAME]** Homestead pond + fishing dock landmark (EPICS #63), water tiles flagged castable, `FishingSpot`. deps: FSH-01.
- **FSH-03 [ART]** Bamboo rod dwarf sets FS1–FS3, FS5, FS9, FS12, FS13 + bobber FB1–FB5. deps: none.
- **FSH-04 [GAME]** Cast (charge/release) → bobber flight → wait → bite → hook window with the FSH-03 animations; no minigame yet (hook = catch). deps: FSH-02, FSH-03.
- **FSH-05 [ART]** Minigame UI sprites FU1–FU8 + reel loops FS6–FS8 + nibble FS4. deps: FSH-03.
- **FSH-06 [GAME]** Reel minigame (zone, fish behaviors, progress, perfect) driving FS6/FS7/FS8. deps: FSH-04, FSH-05.
- **FSH-07 [DATA]** `fish.ron` + loot table by spot/season/time/weather + 6 pond/canal fish. deps: FSH-06, CYC-02 (soft: treat as spring).
- **FSH-08 [ART]** Fish icons batch 1 (pond, stream, canal: 10) + trash icons + catch hold-up FS10 (shared FRM F12 if merged). deps: none.
- **FSH-09 [GAME]** Catch result card + hold-up + inventory insert with quality. deps: FSH-06, FSH-08, FRM-08.
- **FSH-10 [ART]** Fish shadows FW1 + jump splash FW2. deps: none.
- **FSH-11 [GAME]** Fish shadows and rich-spot tells affecting bite time. deps: FSH-07, FSH-10.
- **FSH-12 [ART]** Rod skins Copper/Iron/Brass/Aether/Destiny baked into all FS sets + rod icons. deps: FSH-03, FSH-05.
- **FSH-13 [GAME]** Rod tiers (range, zone, slots) + forge recipes. deps: FSH-12, FRG-46.
- **FSH-14 [ART]** Bait attach FS16 + bait/tackle icons + bobber skins FB3. deps: FSH-03.
- **FSH-15 [GAME]** Bait + tackle slots and effects + durability. deps: FSH-13, FSH-14.
- **FSH-16 [ART]** Crab pot FB7 + haul FS15 + crab icons. deps: none.
- **FSH-17 [GAME]** Crab pots (overnight passive catch). deps: FSH-16, FSH-07.
- **FSH-18 [ART]** Treasure FU5 + treasure open FS18. deps: FSH-05.
- **FSH-19 [GAME]** Treasure in the minigame + treasure tables. deps: FSH-06, FSH-18.
- **FSH-20 [ART]** Stream + canal water autotiles. deps: FSH-01.
- **FSH-21 [GAME]** Mistbrook Stream (forest) + Hearthspire Canal spots + their fish. deps: FSH-20, FSH-07, WLD-03.
- **FSH-22 [ART]** Ice hole FW6 + winter fish icons. deps: none.
- **FSH-23 [GAME]** Winter ice fishing (pickaxe chip opens the hole). deps: FSH-22, CYC-02, MIN-23.
- **FSH-24 [ART]** Reservoir/spillway/cave/aqueduct/cloud water + lava/sky bobbers FB8 + cloud FW8 + remaining fish icons. deps: FSH-20.
- **FSH-25 [GAME]** Riverlands, Mine B20/B35, Aqueduct, Cloud Shoals spots + fish. deps: FSH-24, WLD-13/WLD-17/WLD-21, MIN-40/MIN-42.
- **FSH-26 [ART]** Fillet cut FS19 + processed icons (fillet, oil, scales, bone, roe, smoked). deps: none.
- **FSH-27 [GAME]** Fish Knife filleting at the workbench + Steam Press fish oil + Kiln roe. deps: FSH-26, FRM-47, FRM-49.
- **FSH-28 [ART]** Fish Smoker FW7. deps: FRM-40.
- **FSH-29 [GAME]** Fish Smoker machine. deps: FSH-28, FRM-41.
- **FSH-30 [DATA]** Forge recipes using scales/bones/oil (Riverscale armor Low, bow limbs). deps: FSH-27, FRG-21.
- **FSH-31 [ART]** Legendary hold-up FS11 + 5 legendary icons + legendary minigame markers + legendary shadows. deps: FSH-10.
- **FSH-32 [GAME]** Legendary fish (conditions, Monster Bait requirement, unique drops). deps: FSH-15, FSH-25, FSH-31.
- **FSH-33 [GAME]** Monster Bait + Slime Bait crafting (hunt→fish hook). deps: FSH-15.
- **FSH-34 [GAME]** Kraken-Eel hunt trigger at the Reservoir (fish with Monster Bait → large monster hunt). deps: FSH-33, MON-16.
- **FSH-35 [UI]** Field Guide fish tab (silhouette → filled). deps: FSH-09, UI-12.
- **FSH-36 [UI]** Relaxed fishing assist + remappable fishing keys. deps: FSH-06, UI-16.
- **FSH-37 [AUD]** Fishing sfx set + water ambience beds. deps: FSH-06.
- **FSH-38 [SAVE]** Persist fishing progress, crab pots, rod slots. deps: FSH-17, FSH-15.
- **FSH-39 [DATA]** Fishing balance pass. deps: FSH-25.
- **FSH-40 [GAME]** Gearfin Derby festival hook (timed catch contest). deps: SOC-29, FSH-07.

**Ticket count: 40.**

**Build first (top 3):** FSH-01 → FSH-02 (pond + dock), FSH-03 → FSH-04 (bamboo rod cast/hook sprites + cast loop), FSH-05 → FSH-06 (minigame UI art + reel minigame).
