# 02 — Farming (crops, quality, seasons, fruit trees, magitech processing)

Standards: the homestead is steam-assisted, with crop plots, fruit trees, animals and magitech processing machines. Farming supplies food for stamina and buffs, and materials for potions, coatings and forge components. Crops have quality tiers Normal, Silver, Gold and Destiny, and they are seasonal. Some rare crops grow only with certain forge upgrades or relationship levels. Farming and hunting feed each other: a perfect crop makes a potion that helps break a monster part, and a rare monster part upgrades the forge, which unlocks better farm tools. Animals are in 07; cooking stations are in 04.

Build priority: **2 of 16**.

## 0. Standing rules

- Every player action (till, water, plant, harvest, chop, scythe, shake, load machine, fertilize) gets **new in-house dwarf frames** from a `tools/draw_*.py` script. **Today the farm tools have no action frames.** Feedback is plot color layers only, so every existing farming action needs an [ART] ticket here.
- Crop growth must use drawn stage sprites. The current `PlantSpriteKind {Seed, Foliage}` color layers on reused tiles are placeholders to replace.
- Clean Code, no code comments, zero warnings/errors. Dungeon art frozen (farming never touches it).
- Sprite/animation work is in separate **[ART]** tickets. Gameplay tickets list them as deps.

## 1. What already exists (do not redo)

| Exists | Where |
| --- | --- |
| `HomesteadTool {Hoe, WateringCan, Seeds, Hand}`, keys 1–4, energy 5/3/1/0 | `src/farming/tools.rs` |
| `PlotStage {Soil, Tilled, Growing{crop, days, watered}, Ready{crop}}` and `FarmActionResult` | `src/farming/crops.rs` |
| `CropKind {Turnip (2 days), Potato (3 days)}`, seeds and harvests as `MaterialId` | `src/farming/crops.rs`, `material.rs` |
| Plot spawning + color-layer visuals (`soil_visual`, `growing_plant_visual`) | `src/farming/plots.rs` |
| Plot persistence `crop_plots: Vec<SavedCropPlot>` | `src/core/memory/crop_save.rs`, `profile.rs` |
| Farm HUD + tool select | `src/farming/hud.rs`, `select_tool.rs` |
| Sleep advances growth; tool energy 100 | `day_cycle.rs` |
| Cooking Roast Turnip / Potato Stew + food buffs (attack, defense, carve speed) + eat frames (4 @ 0.18 s, order 0,1,2,3,2,3) | PR #96 |
| Inventory 32 slots, stack 99, categories Tool/Seed/Crop/Food/Ore/Monster | PR #96 |

## 2. Where it happens

- **Homestead farm field** (top-down, `GameState::Overworld`). Start: a 6×4 clearable plot area. Expandable to 3 fields (North Field via the "Clear the North Field" debris cleanup, East Terrace via the carpenter (08) for 2,000 g + 100 Wood + 50 Stone). Debris covers ~60% of new fields at start.
- **Greenhouse** (restored steam glasshouse, 12×10 plots, all-season). Unlocked by the "Rekindle the Glasshouse" community bundle (09) or Forge Lv3 (story). Fruit trees can grow on its edge tiles (8).
- **Riverlands allotment** (optional, 10): wet soil, never needs watering, only rice/water crops (Sunrice, Reedmint).
- **Floating islet garden** (endgame, 10): sky soil where Starbloom grows.
- **Wild forage** in all zones (spring onions, berries, mushrooms) uses the gathering node rules from 01 §2.

## 3. Entities and objects

| Entity | Notes |
| --- | --- |
| `CropPlot` (exists) | gains `fertilizer: Option<FertilizerKind>`, `quality_seed: u32`, `withered: bool`, `regrow_days: Option<u8>` |
| `Crop` | `CropKind` grows from 2 to 30+ kinds; data moves to `assets/data/crops.ron` |
| `Trellis` | trellis crops block movement |
| `GiantCrop` (3×3) | 1% per day for Cauliflower/Melon/Pumpkin fully grown in a 3×3 block; axe to harvest |
| `FruitTree` | `{kind, age_days, stage, fruit_on_tree: 0..3, quality}` |
| `Debris` | Weeds, Twig, Small Stone, Stump, Log, Boulder (mine T2) |
| `Sprinkler` | Steam / Brass / Aether |
| `Scarecrow` | Clockwork Crowbot, radius 8 |
| `Crow` | eats one unprotected crop on 5% of mornings (Q6) |
| `Well` / `WaterTrough` | refill watering can (also the pond/river) |
| `ShippingBin` | sells items overnight (09) |
| Machines (§7) | `Machine {kind, input, output, ready_at}` |
| `CompostBin` | |
| `LightningRod` | storm → Stormcell |
| `FarmSign` | decor, optional |

## 4. Tools and tiers

All upgrades are crafted at the Destiny Forge (04) and need a Forge level. Each tier gets new swing frames (see §5). Charge = hold the use key. Each charge level adds 0.35 s and 1× energy.

| Tool | T1 (start) | T2 Copper | T3 Iron | T4 Brass | T5 Aethersteel | Energy (T1→T5) |
| --- | --- | --- | --- | --- | --- | --- |
| Hoe (exists) | 1 tile | charge → 3 line | 5 line | 3×3 | 6×3 | 5 → 4 → 3.5 → 3 → 2 per charge level |
| Watering Can (exists) | 1 tile, 40 water | 3 line, 55 | 5 line, 70 | 3×3, 85 | 6×3, 100 | 3 → 2.5 → 2 → 1.5 → 1 |
| Axe (new) | twigs, stumps (5 hits) | stumps 3, logs 8 | logs 5 | hardwood 3, giant crops | 1-hit stumps | 4 → 3 → 2.5 → 2 → 1.5 |
| Scythe (new) | 1×3 sweep | — | — | Golden Scythe 1×5 + 50% more fiber | — | 0 (no energy, Stardew-like) |
| Seed Bag (exists as `Seeds`) | 1 tile | — | — | Sowing Drill (planter, 3 line) | — | 1 |
| Hand (exists) | harvest / pick / shake | — | — | — | — | 0 |
| Fertilizer Spreader (new item) | 1 tile | — | — | — | — | 0 |

Upgrade recipes (data, 04): T2 = tool + 5 Copper Bar; T3 = 5 Iron Bar; T4 = 5 Brass Bar + 2 Gear Spring; T5 = 5 Aethersteel Bar + 1 Monster Core (Tidal/Briar boss part) + Forge Lv3. This is the "rare monster part → forge upgrade → better farm tools" chain.

Action bar: today tools are keys 1–4 with no drag-in bar. Adding Axe/Scythe/Fertilizer needs the 8-slot action bar (FRM-05: keys 1–8 and the mouse wheel, plus drag-in from the inventory window).

## 5. ANIMATIONS (most detailed section)

Conventions: dwarf 16×28 body. Action canvas **48×60, dwarf at (16,16)** (same as pickaxe). Authored **facing right**, `flip_x` for left (current convention, Q1 in 01). Frame timings per frame; "hit" = the frame that applies the gameplay effect. Every tier is a separate drawn set (new head/can shape and palette baked by the script, never a runtime tint).

### 5.1 Character (dwarf) sets

| # | Set | Files | Frames | Canvas | Facing | Timing | Loop | Hit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F1 | Hoe till T1 (wood/rusty) | `dwarf_m_hoe_t1_f0..3` | 4 | 48×60 | R | 0.08, 0.06, 0.06, 0.10 = 0.30 s | no | f2 | **covers the existing Hoe action** |
| F2 | Hoe till T2–T5 | `dwarf_m_hoe_t{2..5}_f0..3` | 4 ×4 | 48×60 | R | T2 0.29 s … T5 0.24 s (−0.015 s per tier) | no | f2 | copper/iron/brass/aether heads |
| F3 | Hoe charge hold (T2+) | `dwarf_m_hoe_charge_t{2..5}_f0..2` | 3 ×4 | 48×60 | R | f0 0.15, f1↔f2 0.12 loop | yes | — | each charge level flashes a drawn level-pip frame on the tool |
| F4 | Watering pour T1 | `dwarf_m_water_t1_f0..5` | 6 | 48×60 | R | tilt 0.06 ×2, pour 0.08 ×3, return 0.06 | no | f3 | **covers the existing Watering Can action**; the water stream is drawn in frames |
| F5 | Watering pour T2–T5 | `dwarf_m_water_t{2..5}_f0..5` | 6 ×4 | 48×60 | R | same | no | f3 | can shapes per tier (copper can, iron can, brass steam-sprayer, aether mister) |
| F6 | Watering charge (T2+) | `dwarf_m_water_charge_t{2..5}_f0..1` | 2 ×4 | 48×60 | R | 0.15 loop | yes | — | |
| F7 | Watering can empty shake | `dwarf_m_water_empty_f0..2` | 3 | 48×60 | R | 0.1 ×3 | no | — | when the can is out of water |
| F8 | Refill can at well/pond | `dwarf_m_refill_f0..3` | 4 | 48×60 | R | 0.12 ×4 | no | f3 | |
| F9 | Seed scatter / plant | `dwarf_m_sow_f0..3` | 4 | 32×36 | R | 0.07 ×4 = 0.28 s | no | f2 | **covers the existing Seeds action**; seeds drawn falling |
| F10 | Sowing Drill (T4 seed tool) | `dwarf_m_drill_sow_f0..3` | 4 | 48×60 | R | 0.07 ×4 | no | f2 | |
| F11 | Harvest pull (small crop) | `dwarf_m_harvest_f0..3` | 4 | 24×36 | R | 0.06, 0.08, 0.06, 0.10 | no | f2 | **covers the existing Hand harvest** |
| F12 | Harvest hold-up (item over head) | `dwarf_m_holdup_f0..1` | 2 | 24×40 | front | 0.25 ×2 (once per new item kind/quality ≥ Gold) | no | — | item icon is drawn at the hand anchor |
| F13 | Harvest heavy lift (melon/pumpkin) | `dwarf_m_lift_f0..4` | 5 | 32×40 | R | 0.08 ×5 | no | f3 | |
| F14 | Axe chop T1–T5 | `dwarf_m_axe_t{1..5}_f0..3` | 4 ×5 | 48×60 | R | 0.075 ×4 → T5 0.055 ×4 | no | f2 | horizontal swing, distinct from the pickaxe |
| F15 | Axe charge (T2+) | `dwarf_m_axe_charge_t{2..5}_f0..2` | 3 ×4 | 48×60 | R | like F3 | yes | — | |
| F16 | Scythe sweep T1 / Golden | `dwarf_m_scythe_t{1,2}_f0..4` | 5 ×2 | 64×60 | R | 0.05 ×5 = 0.25 s | no | f2–f3 (hit window) | 64-wide for the arc; the arc is drawn into the frames, not a tinted shape |
| F17 | Tree shake | `dwarf_m_shake_f0..3` | 4 | 32×36 | R | 0.06 ×4 | no | f1, f3 | |
| F18 | Fruit pick (reach up) | `dwarf_m_pick_up_f0..2` | 3 | 24×36 | R | 0.08 ×3 | no | f2 | |
| F19 | Fertilizer sprinkle | `dwarf_m_fertilize_f0..3` | 4 | 32×36 | R | 0.07 ×4 | no | f2 | |
| F20 | Place object (sprinkler/machine/sapling/scarecrow) | shared `dwarf_m_place_f0..2` (01 C17) | 3 | 24×28 | R | 0.1 ×3 | no | f2 | ticket owned by MIN-28. If farming ships first, FRM owns it |
| F21 | Load machine | shared `dwarf_m_load_f0..3` (01 C21) | 4 | 32×32 | R | 0.09 ×4 | no | f2 | ticket owned by MIN-06 |
| F22 | Collect from machine | `dwarf_m_collect_f0..2` | 3 | 24×28 | R | 0.08 ×3 | no | f1 | |
| F23 | Plant sapling | `dwarf_m_plant_sapling_f0..3` | 4 | 32×36 | R | 0.1 ×4 | no | f3 | |
| F24 | Carry seed sack walk (optional idle flavor) | — | — | — | — | — | — | — | deferred (not an action) |

Character total: **~24 sets, ~170 frames.**

### 5.2 Tool / held-item sprites

| # | Set | Frames | Size | Notes |
| --- | --- | --- | --- | --- |
| FT1 | Tool icons per tier: Hoe ×5, Can ×5, Axe ×5, Scythe ×2, Seed Bag, Sowing Drill, Spreader | 1 each = 20 | 16×16 | inventory + action bar |
| FT2 | Water meter overlay on the can icon | 5 fill states | 16×16 | action bar |
| FT3 | Watering can water droplet splash | 4 | 16×16 | 0.05 ×4 at the F4 hit tile |

### 5.3 Entity / world sprites

| # | Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| FE1 | Tilled soil autotile, dry | 16 | 32×32 | static | replaces the `floor_ground` tint for `Tilled` |
| FE2 | Tilled soil autotile, watered | 16 | 32×32 | static | dark wet variant, drawn |
| FE3 | Till dirt-turn VFX | 4 | 32×32 | 0.05 ×4 | on F1/F2 hit |
| FE4 | Crop growth stages per crop (31 crops, §6) | seed, sprout, young, mature, ready (+ regrow-bare where regrowable) = 5–6 | 32×32 (trellis 32×64; giant 96×96) | static per stage | **~165 frames**. Turnip and Potato are first, replacing color layers |
| FE5 | Crop ready idle sway | 2 per crop | 32×32 | 0.6 s loop | readability: ready crops sway |
| FE6 | Withered crop | 1 per crop family (leafy/root/vine/trellis/tree) | 32×32 | — | end-of-season death |
| FE7 | Giant crop ×3 (Cauliflower/Melon/Pumpkin) + chop crack | 1 + 3 crack | 96×96 | — | |
| FE8 | Crop harvest pop (leaf burst) | 4 | 32×32 | 0.05 ×4 | |
| FE9 | Fruit tree sapling stages | 4 (sapling, bush, young, mature-bare) | 48×64 | static | |
| FE10 | Fruit tree mature per season per kind (6 kinds × 4 seasons) | 24 | 48×64 | static | spring blossom, summer leaf, fall color, winter bare |
| FE11 | Fruit overlay 1/2/3 fruits per kind | 3 ×6 | 48×64 | static | |
| FE12 | Tree shake wobble (shared fruit and forest trees) | 4 | 48×64 | 0.05 ×4 | forest trees use their own skin (10) |
| FE13 | Fruit drop bounce | 3 | 16×16 | 0.06 ×3 | |
| FE14 | Tree chop fall | 6 | 64×64 | 0.08 ×6 | + stump 1 frame |
| FE15 | Debris: weeds 2 vars, twig, small stone, stump, log, boulder; each with break 4 | 7 × (1 + 4) | 32×32 (log 64×32, boulder 64×64) | break 0.05 ×4 | |
| FE16 | Steam Sprinkler (3 tiers): idle 1 + spray 6 | 7 ×3 | 32×32 (+ spray 96×96 overlay) | 0.07 ×6 at 6:00 game time | spray drawn as droplets |
| FE17 | Clockwork Crowbot scarecrow idle | 4 | 32×48 | 0.25 loop | head turn |
| FE18 | Crow: fly 4, land 2, peck 3, flee 4 | 13 | 24×24 | fly 0.08, peck 0.12 | |
| FE19 | Shipping bin lid open/close | 4 | 48×32 | 0.06 ×4 | (09) |
| FE20 | Well / trough idle water shimmer | 3 | 32×48 | 0.25 loop | |
| FE21 | Compost bin: idle 1, working bubble 4, ready lid 1 | 6 | 32×32 | 0.15 | |
| FE22 | Machines (9 kinds, §7): idle 1, working 4–6, ready indicator 2 | ~8 each = 72 | 32×48 | working 0.1 loop; ready bob 0.4 | per-machine steam pipes + gauges |
| FE23 | Ready indicator bubble with output icon | 2 | 16×16 | 0.4 loop | above any ready machine |
| FE24 | Lightning rod strike + charged | 4 + 2 | 16×48 | 0.05 | |
| FE25 | Greenhouse exterior broken / restored + door | 2 + 4 | 192×128 | door 0.07 | |
| FE26 | Fertilized soil marker per kind (3 fert × 2 quality) | 6 | 32×32 | static | speckle overlay drawn |
| FE27 | Quality sparkle on harvest (Silver/Gold/Destiny) | 4 ×3 | 16×16 | 0.06 ×4 | Destiny = prismatic |

### 5.4 VFX

| # | Set | Frames | Cell | Timing |
| --- | --- | --- | --- | --- |
| FV1 | Hoe charge ring (level 1–4 pips) | 4 | 32×32 | per level |
| FV2 | Water charge mist | 4 | 32×32 | 0.08 loop |
| FV3 | Scythe grass clippings | 5 | 32×32 | 0.05 ×5 |
| FV4 | Wood chips (axe hit) | 4 | 16×16 | 0.04 ×4 |
| FV5 | Machine steam puff | 5 | 24×24 | 0.08 ×5 |
| FV6 | Essence Extractor glow (per element: fire, water, thunder, ice, dragon/aether, poison) | 4 ×6 | 16×16 | 0.1 loop |
| FV7 | Season-change leaf/blossom/snow drift (ambient) | 6 ×4 | 16×16 | 0.12 |

### 5.5 UI icons (16×16 native, 3× in slots)

- Seeds: 31 seed packets (one per crop) + 6 saplings.
- Crops: 31 crop icons + 6 fruits. **Quality overlay**: Silver/Gold/Destiny 5×5 corner star (3 icons) drawn once and composited by UI. Normal has no star.
- Artisan goods (§7): ~45 icons (juices, oils, jams, pickles, flour, dried goods, essences ×6, potions ×10, coatings ×6, fertilizers ×6, seed packets via the Seed Maker reuse).
- Wood, Hardwood, Fiber, Sap, Mixed Seeds: 5.
- Machines (as placeable items): 12 icons.
- HUD: water meter, season badge (4), fertilizer cursor, action bar frame and slots (8 slots, in-house art).

## 6. Crops (data in `assets/data/crops.ron`)

Days = days to first harvest; R = regrow days; season(s); seed buy price / crop base sell (Normal). Quality multipliers: Silver ×1.25, Gold ×1.5, Destiny ×2.

| Crop | Season | Days | R | Seed g | Sell g | Primary use |
| --- | --- | --- | --- | --- | --- | --- |
| Turnip (exists) | Spring | 2 (keep) | — | 10 | 18 | Roast Turnip (atk) |
| Potato (exists) | Spring | 3 (keep) | — | 25 | 40 | Potato Stew (def/carve) |
| Healroot | Spring | 4 | — | 30 | 45 | Potions (HP) |
| Brass Bean (trellis) | Spring | 8 | 3 | 40 | 25 | food; Steam Press → Bean Oil (Lubricant: weapon sharpness item) |
| Strawberry | Spring | 7 | 4 | 60 | 35 | jam; buff drink |
| Cauliflower (giant) | Spring | 10 | — | 70 | 150 | food |
| Moonpetal (rare) | Spring | 6 | — | gated: Mirela 6♥ | 220 | Moon Draught (status resist), forge reagent |
| Tomato | Summer | 9 | 4 | 40 | 40 | food |
| Firepepper | Summer | 5 | 3 | 45 | 50 | **Fire Essence**, Blaze Coating (bow), Hot Meal (atk) |
| Blueberry | Summer | 12 | 4 | 80 | 30 ×3 | food |
| Sunwheat | Summer, Fall | 4 | — | 10 | 15 | Mill → Flour |
| Melon (giant) | Summer | 12 | — | 80 | 250 | juice |
| Corn | Summer, Fall | 14 | 4 | 50 | 50 | food, Mill → Meal (animal feed 07) |
| Emberbloom (rare) | Summer | 8 | — | gated: Forge Lv2 | 300 | **Ember Core** forge reagent; Mega Potion |
| Pumpkin (giant) | Fall | 13 | — | 100 | 320 | food |
| Cranberry | Fall | 7 | 5 | 80 | 40 ×2 | jam |
| Ironroot | Fall | 9 | — | 60 | 70 | **Ironroot Fiber** → armor linings (04), Hardening Tonic (def) |
| Grape (trellis) | Fall | 10 | 3 | 60 | 30 | Steam Press → juice/wine-free "Cordial" |
| Yam | Fall | 10 | — | 60 | 160 | food |
| Thunder Gourd (rare) | Fall | 10 | — | gated: Forge Lv3; growth day needs a storm | 400 | **Thunder Essence**, Shock Trap bait |
| Frostleaf | Winter | 6 | — | 40 | 60 | **Ice Essence**, Cool Drink (heat resist for Molten Deep / volcano) |
| Snow Yam | Winter | 7 | — | 50 | 90 | food |
| Winter Kale | Winter | 5 | — | 30 | 45 | food |
| Starbloom (rare) | Greenhouse/islets | 20 | — | gated: Story Act 3 + Destiny Spark seed | 1500 | **Destiny Nectar** (Destiny Binding catalyst, 04) |
| Sunrice | Riverlands/Summer | 6 | — | 20 | 30 | food |
| Reedmint (river) | Spring–Fall | 5 | 5 | 25 | 25 | Antidote |
| Bitterroot | Fall | 6 | — | 30 | 35 | Antidote/poison coating |
| Sleepcap mushroom (shade log) | any (Mushroom Log machine) | 7 | 7 | — | 60 | **Sleep Coating**, Tranq Bomb (capture, 05) |
| Paralyshroom | any (Mushroom Log) | 7 | 7 | — | 70 | Paralysis Coating, Shock Trap |
| Glowcap (forage → farmable) | Spring/Fall | 6 | — | 50 | 40 | Lantern Oil |
| Mixed Seeds | season-random | — | — | from Scythe/weeds | — | — |

Fruit trees (sapling 28 days to mature, 3×3 clear. They yield 1 fruit/day in season and hold up to 3 on the tree. Quality rises 1 step per year of age: Silver at year 1, Gold at 2, Destiny at 3).

| Tree | Season | Sapling g | Fruit sell | Use |
| --- | --- | --- | --- | --- |
| Cherry | Spring | 3,400 | 80 | jam, Cherry Tonic |
| Peach | Summer | 6,000 | 140 | juice |
| Emberfruit | Summer | quest (Hask 4♥) | 200 | Fire Essence ×2 |
| Apple | Fall | 4,000 | 100 | animal treat |
| Brasspear | Fall | 6,000 | 160 | **Brass Sap** (machine lubricant; forge polish) |
| Moonplum | Winter | quest (Festival of Lanterns) | 220 | Moon Draught |

## 7. Processing steps (magitech machines)

Machines are crafted at the Workbench (04 crafting) from bars/wood/stone. Times use in-game phases (90 s each; 3 per day) or "overnight". Machines take one input at a time. **Output quality = input quality** unless noted.

| Machine | Recipe | Input → Output | Time | Feeds |
| --- | --- | --- | --- | --- |
| Compost Bin | 20 Wood + 10 Stone | 10 Weeds/Fiber/crop scraps → Basic Fertilizer ×2 | overnight | farming quality |
| Seed Maker | 25 Wood + 2 Copper Bar + 1 Gear Spring | 1 crop → 1–3 seeds (2% Mixed/rare-gated) | 2 phases | farming |
| Steam Press | 2 Copper Bar + 1 Brass Bar + 20 Wood | fruit/berry → Juice; bean/seed → Oil; grape → Cordial | 2 phases | food buffs (drinks), Lubricant/sharpness (05) |
| Preserves Kiln | 50 Stone + 1 Copper Bar + 8 Coal | fruit → Jam; veg → Pickles; uses 1 Coal | overnight | food, gifts (08), price ×2 + 50 |
| Dehydrator | 30 Wood + 2 Iron Bar | 5 fruit/mushroom → Dried pack (keeps quality, stacks) | overnight | travel food (portable buffs, hunts) |
| Mill (farm building) | carpenter 2,500 g + 50 Stone + 150 Wood | Sunwheat → Flour; Corn → Meal; Sunrice → Rice Flour | overnight | cooking (04), animal feed (07) |
| **Alchemy Still** | 2 Iron Bar + 1 Aether Crystal + 1 Sapphire (Forge Lv2) | herbs/crops → Potions (§7.1); essences → Coatings | 1–2 phases | **hunting**: potions, coatings, status ammo |
| **Essence Extractor** | 2 Brass Bar + 2 Aether Crystal + 1 Wisp Mote (Forge Lv2) | element crop → Essence (Fire/Water/Thunder/Ice/Aether/Poison); Gold+ = Pure Essence | 2 phases | **forge**: element infusion on weapons (04) |
| Mushroom Log | 10 Hardwood + 10 Fiber | grows Sleepcap/Paralyshroom/Glowcap weekly | 7 days | coatings |
| Beehive | (07) | — | — | — |
| Lightning Rod | 1 Iron Bar + 1 Copper Bar + 5 Stone | storm → Stormcell (battery: powers the Aether Sprinkler, Thunder coating) | on storm | 04/05 |
| Steam Sprinklers | Steam: 1 Copper Bar + 1 Iron Bar; Brass: 1 Brass Bar + 1 Iron Bar + 1 Gear Spring; Aether: 1 Aethersteel Bar + 1 Aether Crystal + 1 Stormcell | auto-water 4-adjacent / 3×3 / 5×5 at 6:00 | daily | farming |
| Clockwork Crowbot | 50 Wood + 1 Copper Bar + 1 Gear Spring | prevents crows radius 8 | — | farming |

### 7.1 Potions/coatings chain (the farm→hunt coupling)

| Product | Inputs (Still) | Effect in hunts | Quality effect |
| --- | --- | --- | --- |
| Potion | 1 Healroot + 1 Blue Mushroom (forage) | heal 40 | +10% heal per quality step |
| Mega Potion | Potion + Emberbloom | heal 90 | |
| Antidote | Reedmint ×2 | cure poison | |
| Hardening Tonic | Ironroot + Bitterroot | +15 def for 3 min | duration +30 s per step |
| Might Draught | Firepepper ×2 + Honey (07) | +10% atk 3 min | |
| Cool Drink / Hot Drink | Frostleaf / Firepepper + Water | heat/cold zone immunity | |
| Moon Draught | Moonpetal | status resist +50% | |
| **Breaker Tonic** | Gold+ Ironroot + Gold+ Healroot + Bone Shard | **+25% part-break damage 3 min** (perfect crop → superior potion → break the head) | Normal not allowed; Destiny = +40% |
| Coatings (Bow, 05): Blaze, Frost, Shock, Poison, Sleep, Paralysis, Power, Close-range | essence/shroom + Fiber + 1 Sap | arrow coating ×20 | stack +5 per step |
| Lantern Oil | Glowcap + Bean Oil | lantern fuel (mine 01) | |
| Lubricant (Whetstone+) | Bean Oil + Brass Sap | sharpness restore +25% (05) | |

## 8. Crop quality

- Roll at harvest: `p(Destiny) = clamp(0, 0.01 * fert_tier² + 0.002 * farming_level, 0.15)`, `p(Gold) = 0.05 + 0.08 * fert_tier + 0.01 * level`, `p(Silver) = 0.15 + 0.1 * fert_tier + 0.015 * level`, else Normal. Watered every day = +5% to Silver/Gold (the "tending streak").
- Fertilizers: Basic (tier 1), Quality (tier 2: compost + 1 Iron Ore), Deluxe (tier 3: compost + 1 Aether Crystal + Royal Jelly 07). Speed-Gro variants: −10/−25/−33% days.
- **Quality affects**: sell price, food buff magnitude/duration (extends PR #96 `FoodBuff` with a quality multiplier), potion potency, essence purity (Gold+ → Pure), gift value (08), Destiny-only forge recipes (04, e.g. Destiny Nectar).
- Inventory: stacks split by quality. `InventorySlot {material, quality, count}` (save migration).

## 9. Seasons in farming

- 4 seasons × 28 days (11-day-season). Out-of-season crops wither on day 1 of the next season (FE6). Crops on 2 seasons (Corn/Sunwheat) carry over.
- Seeds are sold only in season (shop 09). Planting an out-of-season seed is blocked with a prompt.
- Greenhouse ignores season. Rain auto-waters outdoor plots. Storms: 10% chance a lightning strike withers one unprotected crop (Lightning Rod catches it).
- Winter: outdoor soil only takes winter crops. Tilled soil stays tilled.

## 10. Hooks into other systems

- **Forge (04)**: tool upgrades; Essences → weapon element infusion; Ironroot Fiber/Brass Sap → armor linings/polish; Destiny Nectar → Destiny Binding; Forge level gates rare crops (Emberbloom Lv2, Thunder Gourd Lv3); machines need mine bars.
- **Hunting (05)**: potions, coatings, tonics, Breaker Tonic part-break buff, Tranq bombs (Sleepcap) for capture, Shock Trap (Paralyshroom + Thunder Essence), food buffs (exists, extended with quality), Dehydrated packs as field food.
- **Mining (01)**: bars for machines/sprinklers; Lantern Oil; boulders need T2 pickaxe.
- **Animals (07)**: Meal/Corn/Hay feed; Honey/Royal Jelly → tonics/fertilizer; Manure → compost.
- **Social (08)**: rare crops gated by hearts (Moonpetal, Emberfruit); gift likes; Harvest Festival grading uses crop quality.
- **Progression (09)**: shipping bin gold; community bundles; Farming XP.
- **Day cycle (11)**: growth on sleep (exists), seasons, rain/storm.

## 11. Energy, time, risk

- Energy (§4): till 5→2, water 3→1, plant 1, chop 4→1.5, scythe 0. Charges multiply.
- Time: each action takes its animation time only. Real time runs the phase clock.
- Risk: crows (unprotected 5%/morning), storms, season-end wither, missed watering (no growth that day; 3 dry days in a row → wither). Q5 proposes a softer rule.

## 12. Progression and gating

Start: Hoe T1, Can T1, Seeds, Hand, 15 Turnip seeds, 6×4 field. Day 3: Axe T1 (from the carpenter quest "Clear the Way"). Scythe from the Mayor (first visit). Copper tools need Forge Lv1 (story Act 1 start). Iron tools Lv2. Brass Lv3. Aethersteel Lv4. Greenhouse at Act 2. Rare crops: §6. Machines unlock by recipe: Compost/Crowbot at start, Seed Maker/Steam Press at Farming level 3, Kiln/Dehydrator at 5, Still/Extractor with Forge Lv2 + Mirela 3♥ (hedge mage teaches alchemy), Aether Sprinkler at Lv4.

## 13. UI

- **Action bar** (8 slots) replaces the 4-key tool select HUD and keeps those bindings. The bar starts empty, and slots fill only when the player drags items into them from the inventory. Keys 1–8 and the mouse wheel change the selected slot; clicking a slot selects it. The bar is one row, centered at the bottom of the screen, and moves to the top when the player stands near the bottom. The frame is a warm brown wooden bevel with light parchment slot backgrounds, a red-orange highlight on the selected slot, slot numbers in the top-left of each slot, stack counts in the bottom-right for seeds, and a tooltip that names the hovered item. Frame and slot art are drawn in-house and wired in the same pull request. In the inventory window, click or drag to pick any item up and drop it into any of the 8 bar slots, swapping if that slot is already full. No slot is reserved for tools, and no item type is excluded. The bar is a view of inventory slots, so items are not duplicated. Using the selected slot on the tile in front of the player uses that slot's item; seeds plant from it. The bar shows the water meter and the selected tool's tier icon.
- **Tile cursor** shows the area of effect (charge preview) as a drawn grid sprite.
- **Plot inspector** (hover): crop, days left, watered, fertilizer, season-valid.
- **Machine UI**: insert by using the item on the machine. The status bubble shows the output icon. Hover shows time left.
- **Seed shop window** (09) with season filters.
- **Quality stars** on all item icons. The tooltip shows the buff with the quality multiplier.
- **Crafting window** (04) lists machine recipes with material sources.
- **Season badge** on the day HUD (`ui/day_hud.rs`).

## 14. Audio

hoe_till (3 variants), water_pour loop + splash, can_empty, refill_gurgle, seed_scatter, harvest_pop (3), harvest_heavy_grunt, holdup_jingle (quality-specific: silver ting, gold chime, destiny choir), axe_chop wood ×3, tree_fall, scythe_swish ×2, grass_cut, tree_shake rustle, fruit_thump, sprinkler_hiss, machine_load clunk, machine_working loops (one per machine: press thump, kiln roar, still bubble, extractor hum, mill creak), machine_ready ding, crow caw/flap, crowbot tick, lightning_crack, season-change stinger. Music: `homestead_spring/summer/fall/winter` (warm acoustic + light steampunk), rain variant.

## 15. Save data

- `crop_plots` (exists): add `fertilizer`, `quality_bias`, `withered`, `regrow`, `crop` expanding to the new `CropKind` set. Bump version + fixture.
- `fruit_trees: Vec<{tile, kind, age_days, fruit_count}>`
- `debris: Vec<{tile, kind}>`, `fields_unlocked: u8`, `greenhouse_restored: bool`
- `machines: Vec<{tile, kind, input: Option<(MaterialId, Quality)>, ready_at: (day, phase)}>`
- `placed_objects` (sprinklers, crowbots, rods)
- `inventory` slots gain `quality` (migration: all existing = Normal)
- `tool_tiers` (if tools become upgradable items instead of separate MaterialIds; Q3)
- `farming_xp`

## 16. Balance levers

`crops.ron` (days, regrow, prices, seasons, gates); quality formula coefficients; fertilizer effects; tool energy by tier and charge areas; can capacity; machine times and ratios; artisan price multipliers; crow chance; storm wither chance; giant crop chance; fruit tree maturity and yields; seed shop prices; field expansion costs.

## 17. Open questions (with proposals)

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Quality on stacks needs an inventory migration. Is it worth it now? | Yes. Do it in FRM-08 before more items exist; it is cheaper now. |
| Q2 | Day length 3 × 90 s is very short for Stardew-scale farming. | Keep for now. Balance via tool areas, and revisit in 11 (proposal: 4 phases × 120 s). |
| Q3 | Tool tiers as separate `MaterialId`s (like `PickaxeTier1/2`) or one item with a tier field? | **One `ToolItem {kind, tier}`** with a migration from `PickaxeTier1/2`, `Hoe`, `WateringCan`. It avoids enum explosion. |
| Q4 | Is cooking a farm or a forge station? | A separate **Kitchen** station in the farmhouse (04). The forge keeps metalwork. The PR #96 recipes move there. |
| Q5 | Missed watering penalty | No growth that day. Never wither from thirst (cozy). Season end is the only wither. |
| Q6 | Crows | Keep at 5% with Crowbot unlocked day 1, for light stakes. |
| Q7 | Machine power grid (steam pipes)? | Not for launch. Only the Aether Sprinkler uses a Stormcell. A pipes system is a post-launch "steam network" feature. |
| Q8 | Rare crop gates (forge vs hearts) | Mix as listed: 2 forge-gated, 2 heart-gated, 1 story-gated. |

## 18. Ticket breakdown

Art first for the existing actions (they currently have no frames):
- **FRM-01 [ART]** Hoe till T1 frames F1 + till VFX FE3 + tilled soil autotiles FE1/FE2. deps: none.
- **FRM-02 [GAME]** Play F1 on Hoe use (hit on f2), swap tilled/watered visuals to FE1/FE2. deps: FRM-01.
- **FRM-03 [ART]** Watering T1 frames F4 + empty shake F7 + refill F8 + splash FT3. deps: none.
- **FRM-04 [GAME]** Play F4 on water, can capacity 40 + refill at well/pond + water meter. deps: FRM-03, FRM-05 (soft).
- **FRM-05 [UI]** 8-slot action bar (keys 1–8 + wheel, drag-in from the inventory window, in-house frame and slot art), replacing the 4-key HUD and keeping existing bindings. deps: none.
- **FRM-06 [ART]** Sow frames F9 + harvest pull F11 + hold-up F12 + harvest pop FE8. deps: none.
- **FRM-07 [GAME]** Wire sow/harvest/hold-up animations to existing Seeds/Hand actions. deps: FRM-06.
- **FRM-08 [SAVE]** `ItemQuality` on inventory slots + profile migration (all existing = Normal) + split stacks by quality. deps: none.
- **FRM-09 [ART]** Turnip + Potato growth stage sprites FE4 + ready sway FE5 + withered FE6 (root family). deps: none.
- **FRM-10 [GAME]** Crop data to `crops.ron` (days, regrow, seasons, gates, prices); plot renders stage sprites; Turnip/Potato behavior unchanged. deps: FRM-09.
- **FRM-11 [GAME]** Quality roll at harvest + quality multiplier on PR #96 food buffs. deps: FRM-08, FRM-10.
- **FRM-12 [ART]** Quality star overlays + quality harvest sparkle FE27. deps: none.
- **FRM-13 [UI]** Quality stars on inventory icons + tooltip multipliers. deps: FRM-08, FRM-12, MIN-05.

Seasons and crops:
- **FRM-14 [GAME]** Seasonal planting rules + end-of-season wither + rain auto-water. deps: FRM-10, CYC-02 (seasons), CYC-05 (weather, soft).
- **FRM-15 [ART]** Spring crops (Healroot, Brass Bean trellis, Strawberry, Cauliflower, Moonpetal) stages + icons + seed packets. deps: FRM-09.
- **FRM-16 [DATA]** Spring crops in data + regrow + trellis blocking. deps: FRM-14, FRM-15.
- **FRM-17 [ART]** Summer crop set (Tomato, Firepepper, Blueberry, Sunwheat, Melon, Corn, Emberbloom, Sunrice). deps: FRM-09.
- **FRM-18 [DATA]** Summer crops. deps: FRM-16, FRM-17.
- **FRM-19 [ART]** Fall crop set (Pumpkin, Cranberry, Ironroot, Grape, Yam, Thunder Gourd, Bitterroot). deps: FRM-09.
- **FRM-20 [DATA]** Fall crops. deps: FRM-18, FRM-19.
- **FRM-21 [ART]** Winter crop set (Frostleaf, Snow Yam, Winter Kale) + Starbloom. deps: FRM-09.
- **FRM-22 [DATA]** Winter crops. deps: FRM-20, FRM-21.
- **FRM-23 [GAME]** Rare crop gates (forge level, hearts, story, storm-day growth). deps: FRM-20, FRG-07/FRG-31 forge level, SOC-08 hearts (soft: config flags).
- **FRM-24 [ART]** Giant crops FE7 + heavy lift F13. deps: FRM-15, FRM-17, FRM-19.
- **FRM-25 [GAME]** Giant crop roll + axe harvest. deps: FRM-24, FRM-30.

Tools:
- **FRM-26 [ART]** Hoe T2–T5 frames F2 + charge F3 + FV1 + hoe icons. deps: FRM-01.
- **FRM-27 [GAME]** Hoe tiers with charge areas + forge upgrade recipes. deps: FRM-26, FRM-05, FRG-46.
- **FRM-28 [ART]** Watering T2–T5 F5 + charge F6 + FV2 + can icons + meter FT2. deps: FRM-03.
- **FRM-29 [GAME]** Can tiers (capacity, areas, charge). deps: FRM-28, FRM-04.
- **FRM-30 [ART]** Axe T1–T5 F14 + charge F15 + wood chips FV4 + tree fall FE14 + axe icons. deps: none.
- **FRM-31 [GAME]** Axe tool: chop debris/stumps/logs/homestead trees → Wood/Hardwood/Sap; tiers. deps: FRM-30, FRM-05.
- **FRM-32 [ART]** Scythe T1/Golden F16 + clippings FV3. deps: none.
- **FRM-33 [GAME]** Scythe: weeds → Fiber/Mixed Seeds; 1×3 / 1×5. deps: FRM-32, FRM-05.
- **FRM-34 [ART]** Debris sprites + break anims FE15. deps: none.
- **FRM-35 [GAME]** Debris on farm fields + field expansion (North Field clear, East Terrace purchase). deps: FRM-31, FRM-33, FRM-34, MIN-23 (boulders).

Fertilizer, sprinklers, pests:
- **FRM-36 [ART]** Fertilize frames F19 + fertilized soil markers FE26 + fertilizer icons. deps: FRM-01.
- **FRM-37 [GAME]** Fertilizers (quality + speed) + compost output. deps: FRM-36, FRM-11, FRM-40.
- **FRM-38 [ART]** Steam/Brass/Aether sprinkler FE16 + place frames (if MIN-28 not merged). deps: none.
- **FRM-39 [GAME]** Sprinklers auto-water at day start + placement rules. deps: FRM-38, MIN-07 (bars).
- **FRM-40 [ART]** Compost bin FE21 + load/collect frames F21/F22 (+ ready bubble FE23). deps: MIN-06 (load frames).
- **FRM-41 [GAME]** Generic `Machine` system (input, timer in phases, ready bubble, collect) + Compost Bin as the first machine. deps: FRM-40.
- **FRM-42 [ART]** Crowbot FE17 + Crow FE18. deps: none.
- **FRM-43 [GAME]** Crows + Crowbot protection. deps: FRM-42, FRM-10.

Machines (each = 1 ART + 1 GAME on top of FRM-41):
- **FRM-44/45** Seed Maker. **FRM-46/47** Steam Press (juices, oils). **FRM-48/49** Preserves Kiln. **FRM-50/51** Dehydrator. **FRM-52/53** Mill building (carpenter). **FRM-54/55** Alchemy Still (+ potion/tonic recipes §7.1; GAME deps CMB-12). **FRM-56/57** Essence Extractor (+ FV6; GAME deps FRG-15). **FRM-58/59** Mushroom Log. **FRM-60/61** Lightning Rod + Stormcell (FE24; deps CYC-05 storms).
- **FRM-62 [DATA]** Artisan goods prices + buff tables (`artisan.ron`). deps: FRM-47, FRM-49, FRM-51.
- **FRM-63 [GAME]** Breaker Tonic + coatings recipes routed to hunts. deps: FRM-55, CMB-29, CMB-51.

Fruit trees and greenhouse:
- **FRM-64 [ART]** Fruit tree saplings FE9 + 6 kinds × 4 seasons FE10 + fruit overlays FE11 + shake FE12 + drop FE13 + plant sapling F23 + shake F17 + pick F18. deps: none (largest art ticket; may split per tree).
- **FRM-65 [GAME]** Fruit trees: plant, mature 28 days, seasonal fruiting, shake/pick, age-quality. deps: FRM-64, CYC-02.
- **FRM-66 [ART]** Greenhouse exterior broken/restored + door + interior tileset. deps: none.
- **FRM-67 [GAME]** Greenhouse zone (all-season plots), unlock via bundle/story. deps: FRM-66, PRG-18 or NAR-12 (soft flag).

Polish:
- **FRM-68 [AUD]** Farm sfx (till, water, sow, harvest, chop, scythe, machines). deps: FRM-02, FRM-04, FRM-07.
- **FRM-69 [AUD]** Seasonal homestead music ×4 + rain variant. deps: CYC-02.
- **FRM-70 [UI]** Area-of-effect cursor + plot inspector tooltip. deps: FRM-27.
- **FRM-71 [SAVE]** Persist fruit trees, debris, machines, placed objects, fields, greenhouse. deps: FRM-35, FRM-41, FRM-65.
- **FRM-72 [DATA]** Farming balance pass (energy, growth days, prices vs economy). deps: FRM-22, PRG-03 economy.
- **FRM-73 [GAME]** Farming XP + level hooks (quality bonus, recipe unlocks). deps: PRG-05 player level.

**Ticket count: 73.**

**Build first (top 3):** FRM-01 → FRM-02 (hoe frames + wiring: the existing action has no sprites, a standing-rule violation), FRM-03 → FRM-04 + FRM-06 → FRM-07 (water/sow/harvest frames for the existing actions), FRM-08 (quality on inventory slots, which unblocks quality everywhere).
