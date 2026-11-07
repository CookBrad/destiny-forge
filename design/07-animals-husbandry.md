# 07 — Animals & Husbandry

Authority: GDD §4.1 ("Animals (chickens, cows, bees, etc.) produce materials used in both cooking and armor linings/upgrades"), §10 (Should-have: animal husbandry).

Build priority: **7 of 16**.

## 0. Standing rules

Each player action with animals (pet, feed, milk, shear, collect eggs, brush, harvest honey, lead/carry, ride) gets new dwarf frames. **Each animal state the player can trigger** (petted, milked, sheared, eating, sleeping, happy/sad) gets drawn animal frames. No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

- Decorative farm animals wandering in a homestead pen `PEN_MIN (33,7)` → `PEN_MAX (48,17)` tiles, `WANDER_SPEED 22`, graze timer 2–5 s (`src/overworld/animals.rs`).
- Sprites: **third-party** Dawnlike `quadraped.png` (16×16 cells, 4 frames per creature row, CC-BY 4.0).
- No interaction, products, or save data for animals.

## 2. Where it happens

Homestead: **Coop** (chickens, ducks, clockwork hens), **Barn** (cows, goats, sheep, steam-oxen), **Apiary** (beehives, placed anywhere; best near flowers), **Pasture** (the existing pen area, fenced; animals go out in fair weather in spring–fall), **Stable** (Brass Strider mount, optional), **Pet bed** (cat/dog), **Silo** (hay storage). Buildings come from the Carpenter (Hearthspire). Animals come from the Ranch (Marla, 08).

## 3. Entities and objects

`Animal {id, species, name, age_days, friendship 0–1000, mood 0–255, fed_today, petted_today, product_ready, product_quality, outside, sick}`, `Building {kind, level, capacity, tiles}`, `FeedTrough {hay_slots}`, `Silo {hay}`, `Fence/Gate` (placeable), `Beehive {honey_days, flower_kind}`, `Incubator {egg, days}`, `AutoFeeder` (magitech, Lv2 barn), `SteamMilker` (magitech auto-collect, late), `Pet {species, friendship}`, `Mount` (Brass Strider).

## 4. Species, products, tiers

| Species | Building | Cost | Product (days) | Upgraded product (friendship ≥ 600 or building Lv2) | Uses |
| --- | --- | --- | --- | --- | --- |
| Chicken | Coop | 800 g | Egg (1) | Large Egg | cooking; **Egg Tempera** polish (04) |
| Duck | Coop Lv2 | 1,200 g | Duck Egg (2), Feather (rand) | Down Feather | cooking; **Down lining** (cold resist armor) |
| Clockwork Hen (magitech) | Coop Lv3 | 3,000 g + 2 Brass Bar | Brass Egg (3) | Gear Egg | **Gear Spring** source, machine parts |
| Cow | Barn | 1,500 g | Milk (1) | Large Milk | cooking, Cheese (Press) |
| Goat | Barn Lv2 | 2,000 g | Goat Milk (2) | Large Goat Milk | cheese; **Goat Horn** (shed once a year after age 1) |
| Sheep | Barn Lv2 | 4,000 g | Wool (3, shear) | Fine Wool | **Wool lining** → Loom → Cloth (armor linings, all Low/High sets) |
| Steam-Ox (magitech) | Barn Lv3 | 6,000 g + 1 Aether Crystal | Ox Hide shed (7), Steam Milk (2) | Tempered Hide | **leather armor linings High rank**, Hammer grips |
| Silk Moth (insect) | Coop Lv3 "Silkhouse" | 2,500 g | Silk Cocoon (4) | Aether Silk | **Silk lining** (status resist), Bow strings |
| Bee | Beehive (placeable) | 40 Wood + 8 Fiber + 1 Copper Bar + 1 Sap | Honey (4; flower-flavored by nearest crop/flower) | Royal Jelly (rare 5% Gold+ flowers) | Might Draught (02), Deluxe fertilizer, gifts |
| Cat / Dog (pet) | — | free (story day 5) | — | finds forage daily at 800 friendship | pet companion; Animal Whisperer skill |
| Brass Strider (mount, optional) | Stable | Act 2 quest | — | — | homestead/zone travel speed ×1.6 |

Processing (machines from 02's generic machine system): Cheese Press (milk → Cheese, 2 phases), Mayo Churn (egg → Mayonnaise, 1 phase), **Loom** (Wool → Cloth; Silk → Silk Cloth; Down → Quilted Pad, overnight), **Tanning Rack** (Ox Hide / monster hides → Leather, overnight. It also tans hunt hides: Boar Hide → Tough Leather, the hunt hook), Honey Extractor (built into the hive).

Tools: **Milk Pail**, **Shears** (T1 Copper, T2 Brass: +1 wool chance), **Brush** (+friendship), **Animal Bell** (call animals in), **Hay Fork** (fill troughs ×5). Each is a crafted tool (Workbench), with tiers only on Shears.

Feed: Hay (Scythe on grass → Silo), Corn Meal (Mill, 02), Treats (Apple; animals love them, +mood). Outside grazing replaces hay on grass days.

## 5. ANIMATIONS (most detailed section)

Conventions: in-house redraws replace the Dawnlike sheet (Q1). Top-down homestead, **side-facing (right-authored, `flip_x`)** like the dwarf, plus **front (down) facing for idle/sleep/eat** so animals read when facing the camera. Cells: Chicken/Duck/Hen 16×16, Cow/Goat/Sheep 32×32, Steam-Ox 48×40, Silk Moth 16×16, Bee 8×8, Cat/Dog 24×20, Strider 48×48. The dwarf player sets use the 48×60 canvas (dwarf at (16,16)).

### 5.1 Player (dwarf) sets

| # | Set | Frames | Canvas | Facing | Timing | Loop | Trigger | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A1 | Pet animal (crouch + stroke) | 4 | 32×32 | R | 0.12 ×4 | ×2 | f3 heart VFX | small animals: kneel; large: stand |
| A2 | Pet large animal (stand) | 4 | 32×36 | R | 0.12 ×4 | ×2 | f3 | |
| A3 | Milk (sit stool + pull) | 2 + 4 loop + 2 | 48×36 | R | 0.1 / 0.1 / 0.1 | loop ×3 | end → Milk | |
| A4 | Shear (clip) | 6 | 48×40 | R | 0.08 ×6 | no | f5 → Wool | shears T1/T2 baked variants (2) |
| A5 | Brush | 4 | 32×36 | R | 0.1 ×4 | ×2 | f3 | |
| A6 | Collect egg (bend + lift) | 3 | 24×28 | R | 0.08 ×3 | no | f2 | the pickup bend MIN C10 is not enough: an egg-cradle frame is new |
| A7 | Fill trough (hay fork) | 5 | 48×48 | R | 0.09 ×5 | no | f3 | |
| A8 | Carry hay bale walk | 4 | 24×32 | R | 0.1 ×4 | yes | — | from silo to trough |
| A9 | Harvest honey (smoke + lift frame) | 6 | 32×36 | R | 0.12 ×6 | no | f5 | |
| A10 | Ring animal bell | 3 | 24×32 | R | 0.1 ×3 | no | f1 | |
| A11 | Give treat (hold out) | 3 | 24×28 | R | 0.12 ×3 | no | f2 | |
| A12 | Mount / dismount Strider | 4 / 4 | 48×56 | R | 0.08 | no | — | |
| A13 | Ride idle / ride run | 4 / 6 | 48×56 | R | 0.15 / 0.07 | yes | — | the dwarf rider is baked into the Strider frames |
| A14 | Place incubator egg | shared `place` (01 C17) | | | | | | |
| A15 | Load Loom/Tanning Rack/Cheese Press | shared `load` (01 C21) | | | | | | |

### 5.2 Animal sets (per species; frames @ timing)

| Species | Idle (front) | Idle (side) | Walk (side) | Eat | Sleep | Happy (petted) | Product state | Sad/sick | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Chicken | 2 @0.4 | 2 @0.4 | 4 @0.1 | 3 @0.15 peck | 2 @0.5 | 3 @0.1 hop | egg-laid nest sprite 1 | 2 @0.4 | baby chick set: idle 2, walk 4 |
| Duck | 2 | 2 | 4 @0.11 waddle | 3 | 2 | 3 | feather drop 1 | 2 | swim 4 @0.15 (pond) |
| Clockwork Hen | 2 + gear spin 4 @0.08 | 2 | 4 | 3 (oil sip) | 2 (powered down) | 3 (whistle steam) | brass egg 1 | 2 (rust) | wind-up 4 |
| Cow | 2 @0.5 | 2 | 4 @0.13 | 4 @0.2 graze | 2 @0.6 | 3 | udder full overlay 1 | 2 | calf set: idle 2, walk 4 |
| Goat | 2 | 2 | 4 @0.11 | 4 | 2 | 3 hop | — | 2 | headbutt play 4 |
| Sheep | 2 woolly + 2 sheared | 2 + 2 | 4 + 4 | 4 + 4 | 2 + 2 | 3 + 3 | wool-ready (fluffy) vs sheared skin = 2 full skins | 2 | lamb set |
| Steam-Ox | 2 + steam puff 3 | 2 | 6 @0.14 | 4 | 2 | 3 | hide shed overlay 1 | 2 | stomp 4 |
| Silk Moth | 2 flutter | — | fly 4 @0.08 | — | 2 | 2 | cocoon 3-stage | — | |
| Bee | — | — | fly 2 @0.05 | — | — | — | — | — | swarm overlay 4 @0.1 around hives |
| Cat | 2 | 2 | 4 | 3 | 2 curled | 3 purr | — | — | sit 2, stretch 4 |
| Dog | 2 | 2 | 6 | 3 | 2 | 3 tail wag | — | — | sit 2, fetch run 6 |
| Brass Strider | 4 @0.15 | — | 6 @0.07 run | 2 | 2 | 3 | — | — | rider baked (A13) |

Animal frame totals ≈ 30–45 per species × 11 ≈ **420 frames** (small cells).

### 5.3 Buildings / objects

| Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- |
| Coop Lv1/2/3 exterior + door open | 3 + 4 | 96×96 | door 0.07 | |
| Barn Lv1/2/3 exterior + door | 3 + 4 | 128×112 | 0.07 | |
| Coop/Barn interior tilesets | 16-tile + props | 32×32 | — | |
| Silo | 1 + fill gauge 5 | 48×96 | — | |
| Trough (empty/filled ×hay/meal) | 3 | 32×16 | — | |
| Fence/gate (wood, brass) autotile + gate open 3 | 16 + 3 per material | 32×32 | 0.06 | |
| Beehive idle + bees + honey-ready drip | 1 + 4 + 2 | 32×32 | 0.1 | |
| Incubator idle/working | 1 + 3 | 32×32 | 0.2 | |
| Auto-feeder / Steam Milker | 1 + 4 each | 32×48 | 0.1 | |
| Cheese Press / Mayo Churn / Loom / Tanning Rack | 1 + 4 + ready 2 each | 32×48 | 0.1 | 02 machine system |
| Stable + pet bed | 1 + 1 | 96×80 / 32×16 | — | |

### 5.4 VFX

Heart pop (friendship up) 4 @0.08 16×16; anger puff 3; "!" product-ready bubble 2 @0.4 (shared FRM FE23); wool tuft burst 4; milk splash 3; feather float 4; honey drip 3; hay scatter 4; sparkle on a Gold/Destiny product 4 (shared FE27).

### 5.5 UI icons

Products ×20 (egg variants, milks, wool, cloth, silk, down, hide, leather, honey ×4 flavors, royal jelly, cheeses, mayo, feather, horn), tools ×6, feed ×3, buildings for the carpenter menu ×10 portraits, animal portraits ×11 (32×32) for the animal panel, mood/heart icons.

## 6. Inventory items

| Item | Stack | Sell (N) | Source |
| --- | --- | --- | --- |
| Egg / Large Egg | 99 | 50 / 95 | Chicken |
| Duck Egg / Feather / Down Feather | 99 | 95 / 120 / 250 | Duck |
| Brass Egg / Gear Egg | 99 | 160 / 300 | Clockwork Hen |
| Milk / Large Milk / Goat Milk / Large Goat Milk / Steam Milk | 99 | 125 / 190 / 225 / 345 / 300 | Cow / Goat / Ox |
| Wool / Fine Wool | 99 | 340 / 500 | Sheep |
| Ox Hide / Tempered Hide | 99 | 300 / 600 | Steam-Ox |
| Silk Cocoon / Aether Silk | 99 | 200 / 700 | Silk Moth |
| Honey (Wild/Flower/Ember/Moon) / Royal Jelly | 99 | 100–300 / 800 | Beehive |
| Cheese, Goat Cheese, Mayonnaise | 99 | 230 / 400 / 190 | machines |
| Cloth, Silk Cloth, Quilted Pad, Leather, Tough Leather | 99 | 470 / 900 / 600 / 450 / 650 | Loom / Tanning Rack |
| Hay | 999 (Silo) | 0 | Scythe |
| Treat (Apple) | 99 | — | 02 |

Quality: products take the animal's quality (friendship + mood) → `ItemQuality` (FRM-08).

## 7. Processing summary

Collect → (optional) machine → Kitchen (cooking) or Forge (linings). Lining chain: Wool → Cloth → **Lining slot of armor** (04 material role "Lining"). Example: Leather Hunter Mail = 2 Boar Hide + 1 Tough Leather (Tanning Rack from Boar Hide) + 1 Cloth. **Armor upgrades**: High rank sets require Silk Cloth/Tempered Hide/Quilted Pad (GDD "armor linings/upgrades").

## 8. Hooks

- **Forge**: linings for every armor set from Low+; Gear Egg → Gear Spring; Egg Tempera polish; Goat Horn for Hammer.
- **Hunting**: hunt hides → Tanning Rack; Might Draught (honey); pets do not hunt (companions are villagers, 08).
- **Farming**: manure → compost; Royal Jelly → Deluxe fertilizer; beehive honey flavor from nearby crops; hay from the scythe; Corn Meal from the Mill.
- **Social**: Marla the rancher sells animals and upgrades; animal products are common gifts; the "Fair Day" festival judges animal products.
- **Day cycle**: products on sleep; animals go out on fair days, stay in on rain/winter; mood drops if left outside at night.
- **Saves**: §12.

## 9. Energy / time / risk

Pet/milk/shear/brush: 0 energy, short animation time. Filling troughs with the hay fork: 1 energy. Risk: neglect lowers mood → products stop at mood < 50 and friendship decays −10/day without petting. Animals never die (cozy, Q3). Sickness only from 3 unfed days in a row; a Medicine (Healroot + Milk) cures it.

## 10. Progression / gating

Coop at Ranch unlock (Marla 0♥, day 7) → Barn (after Coop) → Lv2 upgrades (Marla 3♥ + gold/wood/stone) → Lv3 magitech (Forge Lv2 + Brass bars). Beehive recipe at Farming level 3. Silk Moth at Forge Lv3 (Act 2). Strider at Act 2 quest "Iron Hooves". Pet on story day 5.

## 11. UI

Animal panel (hover/interact): name, portrait, hearts (friendship, 5 hearts), mood face, product status, age. Purchase menu (Ranch): species cards with building requirement. Naming dialog. Carpenter build menu (buildings, upgrades, costs, days to build: 2). Silo hay count in the coop/barn HUD.

## 12. Audio

Species calls (chicken cluck/bawk, duck quack, clockwork hen whistle-tick, cow moo, goat bleat, sheep baa, ox snort-steam, moth flutter, bee buzz loop, cat meow/purr, dog bark/pant, strider clank-step); interactions: pet (soft rustle + happy call), milk squirts, shear snips, brush strokes, egg pick, hay fork swish, bell ring, honey drip, door creaks; ranch music cue (warm acoustic).

## 13. Save data

`animals: Vec<Animal>` (§3 fields), `buildings: Vec<{kind, level, tile}>`, `silo_hay`, `beehives: Vec<{tile, days, flower}>`, `pet: Option<Pet>`, `mount_owned`, `incubators`, `fences: Vec<tile+kind>`. Profile version bump.

## 14. Balance levers

Animal prices; product days; friendship gains (pet +15, treat +25, brush +10, fed +5) and decay; quality thresholds (Silver 400, Gold 700, Destiny 950 + mood 200); building costs/capacities (4/8/12); hay per animal/day; machine times; product prices; sickness rule; honey flavor multipliers.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Keep the third-party Dawnlike animals? | **Replace them with in-house sprites.** Interaction frames must be new anyway, and mixing art sources breaks the cozy steampunk look. Keep the CC-BY entry until they are removed. |
| Q2 | Top-down facings for animals? | Side (flip) + front for idle/eat/sleep. No back facing. |
| Q3 | Can animals die? | No. Neglect only stops production and lowers friendship. |
| Q4 | Breeding / incubator? | Incubator for eggs only at launch. Barn pregnancy is post-launch. |
| Q5 | Mount scope | Optional Strider at Act 2, only on homestead + world zones (not hunts). |

## 16. Ticket breakdown

- **ANM-01 [ART]** In-house chicken + cow sets (§5.2) replacing the Dawnlike rows for those species. deps: none.
- **ANM-02 [GAME]** `Animal` model + species data (`animals.ron`) + existing wander driven per species/state. deps: ANM-01.
- **ANM-03 [ART]** Dwarf pet A1/A2 + heart VFX. deps: none.
- **ANM-04 [GAME]** Pet interaction + friendship + mood + daily decay. deps: ANM-02, ANM-03.
- **ANM-05 [ART]** Coop + barn exteriors + doors + interiors (Lv1). deps: none.
- **ANM-06 [GAME]** Coop/Barn buildings: enter/exit, capacity, animals sleep inside, pasture time by weather. deps: ANM-05, ANM-02.
- **ANM-07 [ART]** Trough/silo/hay sprites + fill trough A7 + carry hay A8 + Hay icon. deps: none.
- **ANM-08 [GAME]** Feeding (hay from silo/scythe, troughs, grass grazing). deps: ANM-07, ANM-06, FRM-33.
- **ANM-09 [ART]** Egg collect A6 + nest egg sprites + product icons (eggs). deps: none.
- **ANM-10 [GAME]** Daily products on sleep + egg collection + product quality. deps: ANM-09, ANM-08, FRM-08.
- **ANM-11 [ART]** Milk A3 + pail icon + milk splash + udder overlay. deps: ANM-01.
- **ANM-12 [GAME]** Milking with the Milk Pail. deps: ANM-11, ANM-10.
- **ANM-13 [ART]** Ranch shop portraits + animal purchase UI art. deps: none.
- **ANM-14 [GAME]** Ranch purchase + naming (Marla stub vendor until 08). deps: ANM-13, ANM-06, PRG-03.
- **ANM-15 [ART]** Sheep (woolly/sheared skins) + shear A4 (T1/T2) + wool VFX. deps: none.
- **ANM-16 [GAME]** Sheep + shearing + Shears tiers. deps: ANM-15, ANM-10.
- **ANM-17 [ART]** Goat + Duck sets (incl. duck swim). deps: none.
- **ANM-18 [GAME]** Goat + Duck species + Lv2 building upgrade. deps: ANM-17, ANM-06.
- **ANM-19 [ART]** Brush A5 + treat A11 + bell A10. deps: none.
- **ANM-20 [GAME]** Brush, treats, animal bell. deps: ANM-19, ANM-04.
- **ANM-21 [ART]** Cheese Press, Mayo Churn, Loom, Tanning Rack (idle/working/ready). deps: FRM-40.
- **ANM-22 [GAME]** Animal machines on the FRM-41 system incl. hunt hides at the Tanning Rack. deps: ANM-21, FRM-41.
- **ANM-23 [DATA]** Linings into forge recipes (Cloth/Leather/Silk/Down) for Low/High sets. deps: ANM-22, FRG-21.
- **ANM-24 [ART]** Beehive + bees + harvest honey A9 + honey icons. deps: none.
- **ANM-25 [GAME]** Beehives with flower-flavored honey + Royal Jelly. deps: ANM-24, FRM-10.
- **ANM-26 [ART]** Clockwork Hen + Steam-Ox + Silk Moth sets + Lv3 building skins. deps: none.
- **ANM-27 [GAME]** Magitech species + Lv3 buildings (Forge-gated). deps: ANM-26, ANM-18, FRG-31.
- **ANM-28 [ART]** Cat + Dog sets + pet bed. deps: none.
- **ANM-29 [GAME]** Pet adoption + daily forage find. deps: ANM-28, NAR-01 day-5 flag (soft).
- **ANM-30 [ART]** Fences/gates autotiles. deps: none.
- **ANM-31 [GAME]** Placeable fences/gates (shared with farm decor). deps: ANM-30.
- **ANM-32 [ART]** Incubator + chick/calf/lamb baby sets. deps: ANM-01.
- **ANM-33 [GAME]** Incubator + baby growth. deps: ANM-32, ANM-10.
- **ANM-34 [ART]** Brass Strider + mount/ride A12/A13. deps: none.
- **ANM-35 [GAME]** Strider mount (homestead + zones). deps: ANM-34, NAR-12.
- **ANM-36 [ART]** Auto-feeder + Steam Milker. deps: none.
- **ANM-37 [GAME]** Automation (auto-feed, auto-milk). deps: ANM-36, ANM-27.
- **ANM-38 [UI]** Animal panel (hearts, mood, product) + carpenter build menu. deps: ANM-04, ANM-06.
- **ANM-39 [AUD]** Animal calls + interaction sfx. deps: ANM-04.
- **ANM-40 [SAVE]** Persist animals, buildings, silo, hives, pet, fences. deps: ANM-10.
- **ANM-41 [DATA]** Husbandry balance pass. deps: ANM-27.
- **ANM-42 [GAME]** Remove the Dawnlike sheet after all species are in-house + attribution cleanup. deps: ANM-01, ANM-15, ANM-17.

**Ticket count: 42.**

**Build first (top 3):** ANM-01 → ANM-02 (in-house chicken/cow + animal model), ANM-03 → ANM-04 (petting + friendship), ANM-05 → ANM-06 (coop/barn buildings).
