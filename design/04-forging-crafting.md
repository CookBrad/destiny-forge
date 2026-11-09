# 04 — Forging & Crafting (The Destiny Forge)

Standards: forging is the central progression system. Gear is built from natural materials and monster parts, in ranks Low, High and Master, with skills, decoration and jewel slots, and elemental or status properties set by the materials used. Destiny Binding, at higher forge levels, spends rare Destiny Sparks to lock in or enhance specific skills. Forge upgrades need village reputation and rare materials, and unlock better base stats, more skill slots and unique recipes. A rare part upgrades the forge, which unlocks better farm tools. Social bonds grant unique weapon trees. Power comes from gear, not levels. Forging is must-have; decorations are nice-to-have.

Build priority: **4 of 16**.

## 0. Standing rules

Every forge/craft action plays new in-house dwarf frames: hammering, bellows, quench, workbench, jeweler, cooking, and the binding ritual. The current instant craft from the UI button is a violation, so it gets an art ticket. Station entities animate with drawn frames. No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

| Exists | Where |
| --- | --- |
| Forge station on the homestead: `forge_furnace.png` 80×80, `forge_workbench.png` 96×160, `forge_anvil.png` 96×80 | `assets/overworld/` |
| Forge window: recipe cycle, cost text, requirement text, set-bonus hint, craft button, status | `src/ui/forge_window.rs` |
| `Recipe {name, costs, output: Weapon/Armor/Item/Food, requires_weapon}` in `assets/data/recipes.ron` (RON hot-edit) | `src/forging/recipes.rs` |
| Weapon line: Rusty Sword, Rusty Spear, Iron Sword (needs Iron), Slime Blade (needs Iron Sword + Royal Slime Core) | `combat/weapon.rs`, recipes |
| Slime armor 4 pieces (Helm/Mail/Gauntlets/Greaves) + set bonus scaling attack | `player/loadout.rs`, `combat/attack.rs` |
| `GearStash` (multi-weapon stash) | `player/loadout.rs` (PR #96) |
| Cooking recipes Roast Turnip / Potato Stew live in the forge list | PR #96 |
| Pickaxe recipes from ore | PR #95 |
| Carve loot tables (`carve_loot.ron`) with bonus rolls | `dungeon/carve_loot.rs` |

## 2. Where it happens

| Station | Location | Purpose |
| --- | --- | --- |
| **Destiny Forge** (anvil + furnace + workbench group, exists) | Homestead forge yard | weapons, armor, tool upgrades, forge upgrades, Destiny Binding |
| Smelter (01) | next to the furnace | ore → bars |
| **Workbench** (reuse `forge_workbench.png` as its own interactable, or a new carpenter bench) | Homestead | machines, bombs, traps, torches, fences, whetstones, ammo/coatings assembly |
| **Jeweler's Bench** | Forge yard, Forge Lv2 | cut gems, craft decorations, socket/unsocket |
| **Kitchen** | Farmhouse interior (exists: house/bed) | cooking (PR #96 recipes move here, Q4 in 02) |
| Alchemy Still / Essence Extractor (02) | farm | potions, coatings, essences |
| Hearthspire Smithy (Ottilie) | Village | geode cracking (01), NPC-only weapon trees after hearts (08), repairs |
| Field Camp Box (hunt camp, 05) | hunting grounds | swap gear/consumables only, no forging |

## 3. Entities and data model

- `ForgeStation {level: ForgeLevel}` with components per sub-station (Anvil, Furnace, Bellows, Core).
- `ForgeLevel` 0–5 (§9).
- `GearPiece {base: GearId, rank, rarity 1–10, attack|defense, element: Option<(Element, value)>, status: Option<(Status, value)>, affinity, sharpness, skills: Vec<(SkillId, level)>, slots: [SlotSize; ≤3], decorations, binding: Option<Binding>, upgrade_level}`. This replaces the plain `WeaponKind`/`ArmorKind` enums with data rows in `gear.ron`. Q1 covers the migration.
- `Recipe` (exists) gains `kind: Craft|Upgrade{from}`, `station`, `forge_level`, `rank`, `components: {frame, core, polish, lining, infusion}`, `reputation`, `story_flag`, `heart_gate: Option<(VillagerId, hearts)>`.
- `Decoration {skill, level, size}`.
- `DestinySpark` currency (09) and a `Binding` record.

### 3.1 Material roles (frames, cores, polish)

| Role | Typical source | Determines |
| --- | --- | --- |
| Frame | bars (01), Hardwood (02), Fish Bone (03) | base attack/defense, rank tier |
| Core | monster parts: hearts, cores, plates (05/06) | skills, set identity |
| Polish | gems, gem dust, Brass Sap, Fish Oil | affinity, sharpness, rarity bump |
| Lining (armor) | Ironroot Fiber, wool/silk (07), fish scale, hide | resistances, defense bonus |
| **Infusion** (optional chosen slot) | Essences (02), elemental monster parts, Stormglass/Frost Crystal (01) | **element/status type and value**. This is how "properties determined by materials used" works: the same recipe with a Fire Essence vs a Paralyshroom gives a fire or a paralysis weapon |

## 4. Gear content (launch)

### 4.1 Weapons (launch subset: GS, LS, SnS, DB, Hammer, Bow; existing Sword→SnS family, Spear = legacy, Q2)

Each weapon type has 4 trees. Each tree has Low (R1–4), High (R5–7), and Master (R8–10) steps. A tree has 6–9 nodes. That gives about 6 types × 4 trees × 7 nodes ≈ 170 weapon rows. Launch target: ~90 rows (3 trees per type, 5 nodes each).

| Tree (all types) | Source | Element |
| --- | --- | --- |
| Ore tree (Copper → Iron → Steel → Aethersteel → Mythril → Starsteel → Destinium) | mine bars (01) | raw, takes any Infusion |
| Monster trees (one per large monster, 06) | parts | monster's element/status |
| Villager trees (Ottilie: Brasswork; Hask: Hunter's; Mirela: Hedge-mage; Brunwald: Miner's Drill-weapons) | heart 6+ unlock (08) | special skills |
| Legendary fish/Elder trees | 03/06 endgame | unique |

### 4.2 Armor

Slots: Helm, Mail, Gauntlets, Greaves (exist) + **Coil (waist)** (Q3: proposed adding a 5th slot to match MH 5-piece sets). Sets (Low/High/Master variants each): Slime (exists, Low), Leather Hunter (Low), Miner (Low, 01), Riverscale (Low/High, 03), Briar (High, Thornback), Ironhide (High), Tidecaller (High), Gearwork (High, villager), Crystal (High), Stormwing (Master), Magma (Master), Elder Aether (Master), Destiny (Master endgame). Each set: 2-piece and 4-piece set bonus (extends the existing set bonus).

### 4.3 Skills (~36, max levels in parentheses)

Offense: Attack Boost (5), Critical Eye (5), Weakness Exploit (3), Critical Boost (3), Partbreaker (3), Slugger/KO (3), Focus (charge speed, 3), Artillery (bombs/explosive, 3), Fire/Water/Thunder/Ice/Aether Attack (5 each), Poison/Paralysis/Sleep Attack (3 each), Handicraft (sharpness, 5), Razor Sharp (3), Agitator (enraged monster bonus, 3).
Defense: Defense Boost (5), Health Boost (3), Guard (3), Guard Up (1), Evade Window (3), Evade Extender (3), Recovery Up (3), Recovery Speed (3), Fire/Water/Thunder/Ice/Aether Resist (3), Status Resist (3), Heat/Cold Guard (1), Tremor Resist (2), Windproof (2), Earplugs (2).
Utility/hunt: Carving Pro (1), Carving Master (extra carve, 1), Capture Master (1), Trap Master (1), Bombardier (1), Tracker (track-clue radius, 2), Marathon (stamina, 3), Hunger Resist (food duration, 3), Wide-Range (items affect companion, 2), Speed Sharpening (3).
**Homestead crossover** (the dual-fantasy pillar): Green Thumb (crop quality +, 3), Geologist (+1 ore chance, 3), Angler (larger catch zone, 3), Steam Stride (homestead move speed, 2), Stout Heart (tool energy −10%/lvl, 3), Animal Whisperer (07 affection, 2), Silver Tongue (heart gain, 2).
Set skills (from 2/4-piece bonuses): Royal Gel (exists as the Slime bonus), Ironhide Resolve, Tidal Flow, Briar Retaliation, Gearwork Overdrive, Elder Ascension, Destiny's Favor (Spark-boosted).

### 4.4 Decoration slots and decorations

- Slots per piece: 0–3, size 1/2/3 (Lv1 fits only size-1 decorations, Lv3 fits any). The base row defines the slots. Forge level adds +1 slot cap (§9).
- Decorations (~40) are made at the Jeweler's Bench from 1 cut gem + 1 monster part + 1 polish. Size depends on gem tier (Quartz/Amber = 1, Topaz–Emerald = 2, Ruby/Diamond/Prismatic = 3). Each grants 1 level of a skill (size-3 = 2 levels or a 2-skill combo).
- Unsocket is free; decorations return to the pouch.

### 4.5 Destiny Binding (Forge Lv4+)

Spend Destiny Sparks at the Destiny Core:
| Binding | Cost | Effect |
| --- | --- | --- |
| **Lock** | 1 Spark | A skill on a piece stays through upgrades and is transferred to the upgraded weapon/armor even if that row lacks it |
| **Enhance** | 2 Sparks | +1 level of an existing skill above its normal max (once per piece) |
| **Awaken** | 3 Sparks + Destiny Nectar (02) | adds 1 size-3 slot (Lv5 only) |
| **Rebind** | 1 Spark | move a Lock to another skill |
Max 1 binding effect of each type per piece. Spark sources (09): story milestones ×6, Elder/tempered monsters (≈5%), Skywyrm Pearl (03), Festival of Lanterns (08) 1/yr, Destiny Vault chest (01). About 18 in a full main run.

### 4.6 Crafting (non-gear)

Workbench recipes: machines (02), sprinklers, torches, bombs (01), **traps** (Pitfall Trap: Net + Trap Tool; Shock Trap: Trap Tool + Thunder Essence; Tranq Bomb: Sleepcap + Slime Gel), Whetstone (Stone + Fish Oil), Lubricant, arrow coatings assembly, Barrel Bomb (Coal + Sulfur + Wood), Flash Pod (Glowcap + Wisp Mote), Smoke Pod, Paintball/Tracker Moth (Glowcap + Ember Dust, 05 tracking), Trap Tool (Iron Bar + Gear Spring), Net (Fiber ×10), fences/gates (07), crab pots (03), Ore Satchel (01), cosmetic furniture.
Kitchen: PR #96 foods + multi-ingredient meals (2-buff), **Hearth Meal** (pre-hunt meal, consumed at the village hall or camp, gives 1 skill-like "Daily" buff such as Felyne-style "Carving Luck"), drinks from the Steam Press, smoked/dried field food.

## 5. ANIMATIONS (most detailed section)

Dwarf 16×28. Forge actions use a **64×60 canvas** (dwarf at (24,16)), facing **right toward the anvil** (the anvil is placed to the player's right; flip otherwise). Station entities face front.

### 5.1 Character sets

| # | Set | Frames | Canvas | Facing | Timing | Loop | Trigger | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| G1 | Anvil hammer strike | 4 | 64×60 | R | 0.10 raise, 0.06, 0.04 strike, 0.12 recover = 0.32 s | yes ×3 per craft | f2 strike → spark V-G1 | **replaces the instant craft** |
| G2 | Bellows pump | 4 | 48×60 | R | 0.15 ×4 | ×2 | f2 furnace flare | before G1 on metal recipes |
| G3 | Pull from furnace (tongs) | 4 | 64×60 | R | 0.1 ×4 | no | f3 | |
| G4 | Quench in barrel | 5 | 48×60 | R | 0.08, 0.08, 0.15 (steam), 0.15, 0.1 | no | f2 → steam V-G3 | |
| G5 | Reveal / hold-up gear | 2 + weapon-type pose | 32×48 | front | 0.4 ×2 | no | — | one hold-up per weapon type (6) + armor + tool = 8 variants, item sprite at the hand anchor |
| G6 | Masterwork strike (timing success) | 3 | 64×60 | R | 0.05 ×3 | no | — | optional minigame Q5 |
| G7 | Fumble (timing miss; cosmetic only) | 3 | 64×60 | R | 0.08 ×3 | no | — | |
| G8 | Workbench assemble (saw/wrench) | 4 | 48×48 | R | 0.1 ×4 | ×2 | — | crafting non-gear |
| G9 | Jeweler polish/cut | 4 | 48×48 | R | 0.12 ×4 | ×2 | — | |
| G10 | Socket decoration | 3 | 32×36 | front | 0.1 ×3 | no | f2 click | |
| G11 | Cook stir (pot) | 4 | 48×48 | R | 0.15 ×4 | ×2 | — | Kitchen |
| G12 | Cook chop (knife) | 3 | 48×48 | R | 0.08 ×3 | ×2 | — | Kitchen; FSH fillet FS19 can share |
| G13 | Taste / plate up | 3 | 32×36 | front | 0.15 ×3 | no | — | |
| G14 | Destiny Binding ritual: kneel, raise spark, channel loop, release | 8 (2+2+2 loop+2) | 64×64 | front | 0.15, 0.15, 0.12, 0.12, 0.1 ×2 loop ×4, 0.08, 0.2 | partial | f6 bind | |
| G15 | Forge upgrade install (heave part into the machine) | 5 | 64×60 | R | 0.12 ×5 | no | f4 | then the station upgrade anim plays |
| G16 | Whetstone sharpen (homestead, preview of 05) | 4 | 32×36 | R | 0.12 ×4 | ×3 | — | the combat-side sharpen is the knight set (05) |

### 5.2 Station / entity sets (per forge level the station art changes)

| # | Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| GE1 | Furnace idle fire per level L0–L5 | 4 ×6 | 80×80 | 0.12 loop | L0 cold embers; L5 prismatic flame. Replaces the static `forge_furnace.png` |
| GE2 | Furnace flare (bellows) | 3 | 80×80 | 0.06 | |
| GE3 | Anvil idle + glowing workpiece | 1 + 3 | 96×80 | 0.15 | |
| GE4 | Workbench idle + working | 1 + 4 | 96×160 | 0.1 | replaces the static image while working |
| GE5 | Bellows (new prop) idle/pump | 1 + 4 | 48×48 | 0.15 | Lv2 "Brass Lung" |
| GE6 | Gear tower (Lv2+) turning | 6 | 48×96 | 0.1 loop | |
| GE7 | Aether Core (Lv3+) pulse | 6 | 64×64 | 0.12 loop | |
| GE8 | Mythril Bellows chimneys (Lv4) steam | 6 | 64×96 | 0.1 | |
| GE9 | Destiny Crown (Lv5) halo + Binding Core | 8 | 96×96 | 0.1 | used by G14 |
| GE10 | Forge upgrade transformation | 10 | 192×160 | 0.1 ×10 | one per level step (5 sequences) |
| GE11 | Quench barrel idle/steam | 1 + 4 | 32×48 | 0.1 | |
| GE12 | Jeweler's bench idle/working | 1 + 4 | 64×48 | 0.12 | |
| GE13 | Kitchen stove idle/cooking | 1 + 4 | 64×64 | 0.12 | farmhouse interior |
| GE14 | Smithy (village) exterior chimney smoke | 6 | 128×128 | 0.15 | 10-world |

### 5.3 VFX

| # | Set | Frames | Cell | Timing |
| --- | --- | --- | --- | --- |
| VG1 | Anvil sparks | 5 | 32×32 | 0.04 ×5 |
| VG2 | Molten glow on the workpiece | 3 | 16×8 | 0.1 loop |
| VG3 | Quench steam burst | 6 | 32×48 | 0.07 ×6 |
| VG4 | Item reveal shine (rarity-colored, 4 rarity bands) | 6 ×4 | 48×48 | 0.06 |
| VG5 | Decoration socket click glint | 4 | 16×16 | 0.05 |
| VG6 | Destiny Spark orb (inventory + ritual) | 6 | 16×16 | 0.1 loop |
| VG7 | Binding beam + rune circle | 8 | 128×128 | 0.08 |
| VG8 | Cooking steam/sizzle | 4 | 24×24 | 0.1 |
| VG9 | Element infusion swirl (Fire/Water/Thunder/Ice/Aether/Poison/Para/Sleep) | 6 ×8 | 32×32 | 0.07 |

### 5.4 UI icons

- Weapon icons: per weapon type × tree node (~90 launch rows) at 16×16, plus a 32×32 preview per row for the forge window. Rarity color frame ×10.
- Armor icons: 5 slots × 13 sets × ranks (L/H/M variants recolored **by the draw script**) ≈ 150 at 16×16, plus 32×32 previews.
- Skill icons ×36 + 7 set skills (16×16), element/status icons ×8 (shared with 05), slot-size glyphs ×3 (empty/filled), decoration jewels ×40 (by size and color), Destiny Spark icon, binding glyphs (Lock/Enhance/Awaken/Rebind).
- Station UI: forge level medallion ×6, rank badges (Low/High/Master), material role glyphs (Frame/Core/Polish/Lining/Infusion).
- Kitchen: dish icons (~40), meal buff icons.

### 5.5 Equipped-look dependency

Weapons and armor must appear on the **side-scroll hunter** (knight, 343×160 cells) during hunts. That art lives in 05-combat (CMB weapon movesets) and 15-art (armor look via a per-set palette-baked sheet + helm overlay). Forging tickets do not block on it. A crafted item works with the placeholder look until the CMB/ART tickets ship.

## 6. Inventory items

| Item | Category | Stack | Price | Source |
| --- | --- | --- | --- | --- |
| Weapons / armor | Gear (new category; lives in the GearStash, not the 32-slot bag) | 1 | sell 30% of material value | forge |
| Decorations | Decoration (pouch, separate) | 99 | not sellable | Jeweler |
| Cut gems, Gem Dust | Gem | 99 | 1.5× raw gem | Jeweler |
| Destiny Spark | Currency (wallet, not inventory) | — | — | §4.5 |
| Trap Tool, Net, traps, bombs, Whetstone, coatings, pods | Hunt item (new category) | 10–20 (hunt items capped like MH) | 20–200 g | Workbench |
| Meals | Food (exists) | 20 | per recipe | Kitchen |
| Forge upgrade parts (Copper Heart Coil, Brass Lung, Aether Core, Mythril Bellows, Destiny Crown) | Key item | 1 | — | story/boss/craft |

## 7. Processing steps (forging a weapon)

1. Open the forge (KeyE, exists) → choose **Craft** or **Upgrade** tab → pick a type → tree → node.
2. The UI lists components with **source hints** (13), owned/needed, rank, skills preview, slots, and a compare vs. equipped.
3. Choose the **Infusion** (optional; only rows with an infusion slot) → preview element/status.
4. Confirm → pay gold fee (09; 50–5,000 g) → the cinematic: G2 bellows ×2 → G3 tongs → G1 hammer ×3 (optional timing Q5) → G4 quench → G5 hold-up with VG4. About 2.5 s total, skippable after first view with a held cancel.
5. Gear goes to the GearStash. Upgrading keeps decorations and bindings, then refunds or transfers them.

## 8. Hooks

- **Mining**: bars, gems → decorations; smelt recipes gated by forge level; tool upgrades.
- **Farming**: essences (infusion), linings (Ironroot), Destiny Nectar (binding), potions/coatings (Still) routed to hunts; farm tool upgrades; rare crops gated by forge level.
- **Fishing**: scales/bones/oil; legendary uniques; Skywyrm Pearl → Spark.
- **Hunting/Monsters**: all core parts; part-break-only parts (gems, horns) gate the best rows; capture rewards; rank gating by monster class.
- **Animals**: wool/silk/leather linings; Royal Jelly.
- **Social**: villager weapon trees (6♥), unique forge techniques (Ottilie 8♥: "Folded Steel" = +1 slot on Ore tree items; Mirela 8♥: "Hedge Infusion" = 2 infusions), reputation gates forge upgrades.
- **Progression**: forge level = rank gate = story gate; gold fees; Sparks.
- **Saves**: §12.

## 9. Forge upgrades (levels)

| Level | Name | Requirements | Unlocks |
| --- | --- | --- | --- |
| L0 | Dormant | start | Rusty/Iron/Slime rows (exist), cooking moves to Kitchen |
| L1 | Copper Heart | Copper Heart Coil (mine F10 chest) + 20 Copper Bar + Reputation 50 | Low rank complete, smelter bars, copper tools, Workbench |
| L2 | Brass Lung | 10 Brass Bar + Royal Slime Core + Thornback Briar Heart (06) + Rep 150 | Jeweler (size-1 decos), Steel, iron tools, Still/Extractor, Emberbloom, +1 slot cap |
| L3 | Aether Core | 5 Aethersteel + Tidecaller Pearl + Brassjaw Gear-Scale (03) + Rep 350 + Act 2 | **High rank**, size-2 decos, Aethersteel smelt, brass tools, greenhouse, Thunder Gourd |
| L4 | Mythril Bellows | 5 Mythril Bar + Cinder Heart (01) + Stormwing Plume (06) + Rep 600 | Starsteel smelt, size-3 decos, **Destiny Binding Lock/Enhance**, aether tools, +1 base stat tier (+5% raw/def) |
| L5 | Destiny Crown | 3 Starsteel + Elder Aether Core + 3 Sparks + Rep 1000 + story finale | **Master rank**, Awaken, Destinium, endgame rows |

## 10. Energy / time / risk

Forging costs no energy (it is the reward step). Each craft advances the clock by its cinematic time only. Risk: none. The cost is materials and gold. Optional timing minigame: never a penalty, Masterwork +3% raw or +1 slot-size step on Low rank only (Q5).

## 11. Progression gating

Recipes gate on `forge_level`, `rank`, `story_flag`, `heart_gate`, `requires_weapon` (exists → generalize to `requires_gear`), and whether the material has been obtained once (recipe visibility: unknown rows show silhouettes).

## 12. UI (detailed in 13)

Forge window v2: tabs (Craft/Upgrade/Armor/Tools/Bind), a tree view per weapon type (nodes + lines), component list with role glyphs + source hints ("Briar Heart — Thornback, break tail / carve 8%"), infusion picker, stat preview + compare vs. equipped (green/red deltas), skills list with levels, slot glyphs, fee, Craft button. Gear Stash window: filter by type/rank/skill, sort, equip. Jeweler: piece list → slots → decoration list. Binding UI: piece → skill → binding type → Spark cost confirm. Kitchen window. Workbench window (category tabs).

## 13. Audio

hammer_strike ×3 (bright anvil), masterwork_ring, fumble_clank, bellows_whoosh, furnace_roar loop (louder per level), tongs_clink, quench_hiss, reveal_fanfare by rarity band (4), socket_click, gem_grind loop, binding_choir + magic_swell, forge_upgrade_sequence stingers (5), workbench_saw / wrench ratchet, cook_sizzle / stir / chop / plate_ding, forge yard ambience (steam, gears) per level.

## 14. Save data

`forge: {level, upgrades_installed}`; `gear_stash: Vec<GearPiece>` (generalizes the current GearStash), `loadout` → gear ids (+ Coil slot); `decorations: HashMap<DecoId, u32>`; `destiny_sparks: u32`, `sparks_earned_flags`; `known_recipes`, `seen_materials` (source hints + silhouettes); migration from `WeaponKind`/`ArmorKind` to gear rows (Q1).

## 15. Balance levers

`gear.ron` stats by rank/rarity; component counts; gold fees; infusion values per essence quality (Normal 10 / Silver 13 / Gold 16 / Destiny 22 element); skill caps; slot counts per row; deco recipe costs; spark costs; forge upgrade requirements (materials + reputation); set bonus thresholds; masterwork bonus; sharpness bars.

## 16. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Keep `WeaponKind`/`ArmorKind` enums or data rows? | Move to data rows (`gear.ron`) with a stable string id. Migrate saves by mapping the 4 weapon + 4 armor enums. Existing tests keep their names. |
| Q2 | What happens to Sword and Spear? | Sword becomes the **Sword & Shield** tree (Rusty→Iron→Slime Blade keep their names). Spear is kept as a legacy family until Lance post-launch, still forgeable, and is not expanded. |
| Q3 | Add a 5th armor slot (Coil)? | Yes, Waist "Coil". This gives MH-style 5-piece sets and room for 2/4-piece bonuses. |
| Q4 | Decorations are nice-to-have. Ship at launch? | Ship slots + size-1 decorations at Forge L2 (small scope). Size 2/3 come later. |
| Q5 | Forging minigame? | Optional 3-strike timing on the G1 strike frame, with no penalty and a small Masterwork bonus. Toggle in settings. Default ON, skippable. |
| Q6 | Gold fee on crafting? | Yes, a modest gold sink (09). |
| Q7 | Cooking at the forge (PR #96) → Kitchen? | Move it. Keep the forge recipe entries working until the Kitchen ticket lands, then migrate. |
| Q8 | Infusion on every row? | Only Ore-tree rows and select villager rows. Monster rows have a fixed element. |
| Q9 | Sharpness (MH) yes/no? | Yes for melee, simplified to 4 colors (Red/Yellow/Green/Blue + White at Master) with the Whetstone item. Decided in 05. |

## 17. Ticket breakdown

- **FRG-01 [ART]** Dwarf anvil hammer G1 + bellows G2 + tongs G3 + quench G4 + generic hold-up G5 (weapon, armor, tool variants) + VG1/VG2/VG3. deps: none.
- **FRG-02 [ART]** Furnace L0/L1 animated fire GE1 + flare GE2 + anvil glow GE3 + quench barrel GE11. deps: none.
- **FRG-03 [GAME]** Craft cinematic: the forge craft button plays G2→G3→G1×3→G4→G5 with station anims; skippable; inventory change at the end. deps: FRG-01, FRG-02.
- **FRG-04 [GAME]** `gear.ron` data rows + `GearPiece` model + migration from `WeaponKind`/`ArmorKind` (tests unchanged). deps: none.
- **FRG-05 [SAVE]** Gear stash/loadout save migration + version fixture. deps: FRG-04.
- **FRG-06 [GAME]** Recipe model v2 (`kind` Craft/Upgrade, station, forge_level, rank, gates, components by role, gold fee). deps: FRG-04.
- **FRG-07 [GAME]** `ForgeLevel` resource + L0/L1 upgrade flow (requirements check, install) + gating of recipes. deps: FRG-06, MIN-21 (Copper Heart Coil, soft), PRG-12 (soft).
- **FRG-08 [ART]** Forge upgrade install G15 + L0→L1 transformation GE10. deps: FRG-02.
- **FRG-09 [GAME]** Upgrade cinematic L0→L1. deps: FRG-07, FRG-08.
- **FRG-10 [UI]** Forge window v2: tabs, tree view, component list with role glyphs and source hints. deps: FRG-06, UI-06, UI-08.
- **FRG-11 [ART]** UI icon batch: weapon/armor icons for existing rows (Rusty/Iron/Slime Blade/Spear, Slime set, pickaxes) + rarity frames + role glyphs + rank badges + forge level medallions. deps: none.
- **FRG-12 [UI]** Stat preview + compare vs. equipped (deltas). deps: FRG-10, UI-14.
- **FRG-13 [GAME]** Skills system: `skills.ron`, skill sum over equipped gear, set skills generalized from the existing Slime bonus. deps: FRG-04.
- **FRG-14 [ART]** Skill icons (36 + 7 set) + element/status icons (8). deps: none.
- **FRG-15 [GAME]** Infusion slot: essence/part chooses element/status + value by quality. deps: FRG-06, FRM-57 (essences), CMB-34.
- **FRG-16 [ART]** Element infusion swirl VFX VG9 (8). deps: none.
- **FRG-17 [DATA]** Ore tree rows (Copper→Mythril) for SnS and the existing Spear + migrate the Iron Sword etc. deps: FRG-06, MIN-09.
- **FRG-18 [GAME]** Upgrade path (Upgrade tab) keeping decorations/bindings. deps: FRG-06, FRG-12.
- **FRG-19 [GAME]** Coil (waist) armor slot + loadout UI. deps: FRG-04.
- **FRG-20 [ART]** Coil icons for existing sets. deps: FRG-11.
- **FRG-21 [DATA]** Leather Hunter + Miner armor sets (Low) incl. Coil. deps: FRG-19, MIN-59 Rock Crab drops (soft).
- **FRG-22 [ART]** Workbench idle/working GE4 + assemble G8. deps: none.
- **FRG-23 [GAME]** Workbench station + recipe categories (machines, bombs, traps, torches, whetstones). deps: FRG-06, FRG-22.
- **FRG-24 [ART]** Kitchen stove GE13 + cook stir/chop/plate G11–G13 + VG8 + dish icons for existing foods. deps: none.
- **FRG-25 [GAME]** Kitchen station in the farmhouse; migrate the PR #96 recipes; multi-buff meals. deps: FRG-24, FRG-06.
- **FRG-26 [DATA]** Kitchen recipes batch (crops/fish/animal products, ~25 dishes). deps: FRG-25, FRM-22, FSH-07.
- **FRG-27 [ART]** Jeweler bench GE12 + polish G9 + socket G10 + VG5 + decoration icons (size 1) + slot glyphs. deps: none.
- **FRG-28 [GAME]** Decoration slots on gear + size-1 decorations + socket/unsocket. deps: FRG-13, FRG-27, FRG-07 (L2).
- **FRG-29 [GAME]** Gem cutting (cut gems, gem dust). deps: FRG-27, MIN-48.
- **FRG-30 [ART]** Forge L2–L5 station art (GE5–GE9) + transformations GE10 ×4. deps: FRG-08.
- **FRG-31 [GAME]** Forge levels L2–L5 (requirements, unlock effects incl. stat tier, slot cap). deps: FRG-07, FRG-30, MON-13/MON-16 boss parts (soft).
- **FRG-32 [ART]** Destiny Binding ritual G14 + spark orb VG6 + binding beam VG7 + binding glyphs. deps: FRG-30.
- **FRG-33 [GAME]** Destiny Binding: Lock/Enhance/Rebind (L4), Awaken (L5) + Spark wallet spend. deps: FRG-32, FRG-31, PRG-16.
- **FRG-34 [GAME]** Optional forging timing minigame + Masterwork result (G6/G7). deps: FRG-03, FRG-35.
- **FRG-35 [ART]** Masterwork G6 + fumble G7 frames. deps: FRG-01.
- **FRG-36 [DATA]** Weapon trees per launch type (GS/LS/DB/Hammer/Bow: Ore + 2 monster trees each, Low rank). deps: FRG-17, CMB-47/CMB-49/CMB-51/CMB-53/CMB-55.
- **FRG-37 [DATA]** High rank rows + sets. deps: FRG-31, MON-19/MON-22/MON-25/MON-28.
- **FRG-38 [DATA]** Master rank rows + Destiny set. deps: FRG-37.
- **FRG-39 [GAME]** Villager weapon trees + unique forge techniques (heart gates). deps: SOC-08, FRG-36.
- **FRG-40 [GAME]** Recipe visibility: silhouettes until the material is seen; seen-material tracking. deps: FRG-10.
- **FRG-41 [AUD]** Forge sfx set + furnace loops per level + reveal fanfares. deps: FRG-03.
- **FRG-42 [ART]** Size-2/3 decoration icons + higher decoration recipes. deps: FRG-27.
- **FRG-43 [GAME]** Size-2/3 decorations (L3/L4). deps: FRG-28, FRG-42.
- **FRG-44 [DATA]** Gold fees + forge balance pass. deps: FRG-36, PRG-03.
- **FRG-46 [DATA]** Tool upgrade recipes (hoe/can/axe tiers, pickaxe T3–T6 rows, rod tiers, shears) in `recipes.ron` with forge-level gates. deps: FRG-06, MIN-09.
- **FRG-45 [GAME]** Ottilie smithy (village) services: geode crack, villager tree vendor. deps: SOC-12, MIN-50.

**Ticket count: 46.**

**Build first (top 3):** FRG-04 (gear data rows + migration; everything forge sits on it), FRG-01 + FRG-02 → FRG-03 (craft animations replacing the instant craft), FRG-06 → FRG-07 (recipe v2 + forge level L0/L1 gating).
