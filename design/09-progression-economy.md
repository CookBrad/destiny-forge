# 09 — Progression & Economy (gold, reputation, sparks, player level, difficulty curve, endgame)

Standards: progression is mainly horizontal and gear-based. A modest level system unlocks basic tools and stamina, and real power comes from forged equipment. Currencies are Gold (from selling crops, fish and parts), Reputation (with the village) and Destiny Sparks (rare). Difficulty climbs from village monsters to forest and mine threats, then large territorial monsters, then elder-class threats. The endgame is endless optional hunts, rare material farming and perfecting gear sets for matchups. Forge upgrades need reputation.

Build priority: **9 of 16**.

## 0. Standing rules

Economy actions that are player actions get dwarf frames: ship an item (bin), buy/sell at a counter, donate to the museum, level-up pose. Currency changes need **drawn** feedback sprites (coin pop, rep crest, spark orb), not only a number. No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

- Inventory footer shows three coin displays (gold/silver/copper colors) all hard-coded "0" (`ui/inventory_window.rs:523–525`). There is no gold resource.
- `WorldProgress { boss_defeated_floor_1 }` (`player/progress.rs`).
- No shops, prices, reputation, sparks, or player level.

## 2. Where it happens

Shipping Bin (homestead, sells overnight); General Store (Juniper: seeds, basic supplies); Smithy (Ottilie: bars/tools, geodes); Ranch (Marla: animals, hay); Carpenter (Tobin: buildings); Inn (Sela: meals, recipes); Hunter's Lodge (Vell: hunt items, quest rewards, HR); Museum (Quillon: donations); Town Hall (Mayor: community bundles, reputation board); Airship Mast (Kit: traveling merchant Fridays with rare stock).

## 3. Entities / data

`Wallet {gold: u64, sparks: u32}`, `Reputation {points: u32, rank: RepRank}`, `PlayerLevel {xp, level}` with `SkillXp {farming, mining, fishing, foraging, husbandry, hunting, forging}`, `HunterRank {hr, points}` (05), `PriceTable` (`prices.ron`: base sell per MaterialId; buy multipliers per shop), `Shop {id, stock, schedule, restock}`, `ShippingBin {items}`, `Bundle {id, slots, reward}`, `Museum {donated}`, `Milestone` (one-time rewards).

### 3.1 Currencies

| Currency | Earn | Spend | Notes |
| --- | --- | --- | --- |
| **Gold** | shipping bin, shop sell, quest rewards, festival prizes | seeds, animals, buildings, forge fees (04), tools, recipes, geode cracking, travel | single integer (Q1) |
| **Reputation** (village) | requests (+5–15), personal quests (+20–50), community bundles (+50–100), festivals (+10–30), museum donations (+5 each), hunts on village-threat monsters (+10–40), first mine strata (+10/20/30/50) | not spent: **thresholds** gate forge upgrades (04 §9), shop tiers, buildings, story acts | ranks: Newcomer 0, Neighbor 50, Trusted 150, Pillar 350, Hero 600, Legend 1000 |
| **Destiny Sparks** | story milestones (6), Elders (L11 30%, L12 3), tempered ~5%, Skywyrm Pearl (03), Festival of Lanterns (1/yr), Destiny Vault chests (01) | Destiny Binding (04), Forge L5 (3), Starbloom seed (02), Destiny Pick/Rod | ≈ 18 in a main run |

### 3.2 Player level (modest)

Level 1–20 from total skill XP. Each level: +5 max tool energy (100 → 195), +3 max HP, +2 max stamina. Levels unlock basics only: L2 Axe charge, L3 Seed Maker/Steam Press recipes, L5 Kiln/Dehydrator, L6 Torch bundles, L8 Sprinkler blueprint discount, L10 Golden Scythe recipe, L15 Ore Satchel ×2. **No damage scaling from level** (power comes from gear). Per-skill XP shows "proficiency" (cosmetic stars + small perks: Farming → quality bonus +0.2%/lvl; Mining → +1% geode chance; Fishing → +2 px zone; Hunting → carve speed +1%; Forging → fee −1%).

XP sources: till 1, water 1, harvest by crop value, ore break by hardness ×2, fish caught by difficulty/10, monster hunt (small 5, large 100–500), forge craft by rarity ×10, animal product 5, quests.

### 3.3 Difficulty curve (target timeline, 28-day seasons)

| Stage | Calendar | HR | Forge | Monsters | Mine | Gear |
| --- | --- | --- | --- | --- | --- | --- |
| Village threats | Spring Y1 d1–14 | 1–2 | L0→L1 | King Slime, Bristleboar | F1–10 | Rusty/Iron/Slime, Leather Hunter |
| Forest/Mine threats | Spring d15 – Summer | 3–6 | L1→L2 | Briarwolf, Glowmoth, Kraken-Eel (Low) | F11–20 | Copper/Iron trees, Briar/Riverscale |
| Large territorial | Fall Y1 – Winter Y1 | 9–16 (High) | L3 | Ironhide, Drillwyrm, Gryphon, Golem, Magmaw | F21–40 | High rank sets |
| Elder threats | Spring–Summer Y2 | 21+ (Master) | L4→L5 | Prism Ray, Aether Drake, Destiny Wyrm | Vault | Master rank, Binding |
| Endgame | Y2+ | ∞ | L5 | tempered, Vault elites, legendary echoes | Vault 41+ | perfecting sets |

### 3.4 Endgame
Infinite optional hunts: tempered variants, "Destiny Trials" (rotating weekly modifiers: double enrage, no items, element-locked), Destiny Vault endless floors with depth leaderboards (local), Legendary Echo fish, Awakened gear (Binding), a **matchup set builder** (13 loadouts per monster, gear compare), collection completion (museum, field guide, fish, cooking), perfect-quality farm (Destiny crops/animals), optional "Hearthspire Prestige" cosmetics.

### 3.5 Community bundles (Town Hall)
6 bundle rooms (Fields, Mines, Waters, Wilds, Forge, Hunt). Each room has 4–6 bundles of 4–8 items (e.g., "Spring Crops": Turnip, Potato, Healroot, Strawberry). Rewards: Greenhouse (Fields), Elevator express to F30 (Mines), Canal locks → Riverlands shortcut (Waters), Airship Mast → Islets fast travel (Wilds), Forge L3 component discount (Forge), Hunter's Lodge Arena (Hunt).

### 3.6 Prices and sinks (economy targets)
- Daily income targets: Spring Y1 300–800 g, Summer 1–3 k, Fall 3–6 k, Y2 8–15 k.
- Sinks: buildings (800 → 20 k), forge fees (50 → 5,000), animals, sprinklers/machines (via materials), seeds, Kit's rare stock (5–50 k), cosmetics, field expansion (2 k), greenhouse upgrades.
- Monster parts sell at 30% of their forge value so selling them is not the best path (players should forge).

## 4. Tools / equipment

N/A for currencies. Related: Wallet upgrade "Brass Coin Purse" cosmetic only. Mailbox (homestead) for quest rewards.

## 5. ANIMATIONS (most detailed section)

Conventions: dwarf 16×28, canvas sizes noted, right-authored with `flip_x`; counters face front.

### 5.1 Character (dwarf)

| # | Set | Frames | Canvas | Facing | Timing | Trigger | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P1 | Ship item (lift + drop into bin) | 5 | 32×40 | R | 0.08 ×5 | f4 → bin | with the bin lid FE19 (02) |
| P2 | Counter buy/sell (coin hand-over) | 4 | 24×32 | back (facing the counter) | 0.1 ×4 | f2 | back-facing (ART-02 decision) |
| P3 | Donate to museum (place on pedestal) | 4 | 32×36 | R | 0.1 ×4 | f3 | |
| P4 | Bundle offering (place in a brass cabinet) | 4 | 32×36 | back | 0.1 ×4 | f3 | |
| P5 | Level-up flourish | 6 | 32×40 | front | 0.08 ×6 | on level up | plays when free (not mid-action) |
| P6 | Reputation rank-up salute | 4 | 24×32 | front | 0.12 ×4 | on rank | |
| P7 | Spark received (orb absorb) | 6 | 32×40 | front | 0.1 ×6 | on spark | with VG6 |
| P8 | Read mail | shared SOC S5 | | | | | |

### 5.2 Entities

| Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- |
| Shipping bin (lid) | 4 | 48×32 | 0.06 | FE19 in 02; drawn in PRG-04 |
| Overnight shipping tally cart (sleep transition) | 6 | 64×48 | 0.1 | |
| Shop counters ×7 + shopkeeper idle (from SOC sets) | 1 each | 96×48 | — | |
| Museum pedestals (empty/filled) + display cases | 2 + per item uses icon | 32×48 | — | |
| Community bundle cabinet (closed/open/glowing complete) | 3 + glow 4 | 48×64 | 0.1 | |
| Bundle completion restoration sequence (per amenity) | 8 | 192×128 | 0.1 | ×6 rooms |
| Mailbox flag up/down | 2 + bounce 3 | 16×32 | 0.1 | |
| Kit's airship merchant stall | 1 + balloon sway 4 | 128×96 | 0.2 | |

### 5.3 VFX / UI sprites

Coin pop (+g) 5 @0.06 16×16; coin shower (big sale) 8; reputation crest pulse 5 (32×32); spark orb (VG6, 04); XP sparkle 4; level-up burst 8 (64×64); rank badges ×6 (32×32); currency icons gold/rep/spark (16×16 + 32×32); shop UI frames; price tag sprites; bundle slot frames (empty/filled); shipping summary screen background (overnight).

## 6. Inventory items

No new carried items except: Bundle reward items, Museum rewards (Ancient Seed, decorations), Kit's rare stock (catalog items), Recipe cards. Prices: `prices.ron` lists sell values for every MaterialId (01–07 tables are the source).

## 7. Processing

Shipping bin: drop items any time → sold at sleep (summary screen with per-category totals). Shops: buy/sell immediately at 100% (sell) / 250% (buy) of base. Bundles: deposit items once → reward on completion.

## 8. Hooks

Every system feeds gold. Reputation gates the forge (04). Sparks feed binding (04). Level → tool energy (02/01), HP/stamina (05). HR gates quests (05). Bundles unlock world shortcuts (10) and greenhouse (02).

## 9. Energy / time / risk

Shipping/shopping cost no energy. Shops keep hours (Morning–Afternoon; Inn open in the Evening). Risk: none, but the faint penalties (01, 05) cost gold.

## 10. Progression / gating

| Gate | Requirement |
| --- | --- |
| Forge L1/L2/L3/L4/L5 | Rep 50/150/350/600/1000 + materials (04) |
| Story acts | Act 1 Rep 50 + Forge L1; Act 2 Rep 350 + L3 + HR 9; Act 3 Rep 600 + L4 + HR 21 |
| Shop tiers | Juniper tier 2 at Neighbor, tier 3 at Pillar; Ottilie steel stock at Trusted |
| Buildings | Coop (Neighbor), Barn Lv2 (Trusted), Stable (Pillar) |

## 11. UI

Wallet in the HUD (gold) + inventory footer (replace the "0" placeholders with Gold / Reputation / Sparks per Q1); Reputation meter on the pause/status page; Sparks counter; Level/XP page with skill proficiencies; Shop window (buy/sell tabs, quantity, price with quality); Shipping summary (overnight); Bundles UI; Museum collection UI; Milestone toasts (drawn frames).

## 12. Audio

coin_pickup, sell_ka-ching, buy_bell, ship_drop thunk, shipping summary tally ticks + total chime, rep_rankup fanfare, level_up fanfare, spark_choir (shared 04), bundle_complete orchestral swell, museum_donate chime, mail_arrive.

## 13. Save data

`wallet {gold, sparks}`, `reputation {points}`, `player_level {xp per skill}`, `hunter_rank` (05), `shipping_bin: Vec<(MaterialId, quality, count)>`, `shipped_totals` (stats), `bundles: HashMap<BundleId, slots_filled bitmask>`, `museum` (01), `milestones_claimed`, `shop_states` (stock, restock day).

## 14. Balance levers

`prices.ron`, buy multiplier, shop stock/restock, rep values per source, rep thresholds, XP per action, level curve (XP to L20 ≈ 120 k), level perks, spark sources/rates, bundle contents, forge fees, building costs.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | The UI shows 3 coin colors (gold/silver/copper). Three currencies or denominations? | **One gold integer.** Do not split it into g/s/c denominations. Repurpose the 3 footer chips as **Gold / Reputation / Sparks**. |
| Q2 | Can reputation be lost? | No (cozy). It only rises. |
| Q3 | Player level scope | Small: energy/HP/stamina and recipe unlocks. No damage. |
| Q4 | Selling monster parts | Allowed at 30% so players are not soft-locked, while forging stays better. |
| Q5 | Multiple saves economy separation | Per profile (exists: profiles). |

## 16. Ticket breakdown

- **PRG-01 [GAME]** `Wallet` resource (gold, sparks) + save field + inventory footer shows the real wallet (replacing the hard-coded "0"s per Q1). deps: none.
- **PRG-02 [DATA]** `prices.ron` with sell values for all existing MaterialIds + loader + tests. deps: none.
- **PRG-03 [GAME]** Economy core: sell/buy API on Wallet + PriceTable (quality multipliers). deps: PRG-01, PRG-02, FRM-08 (soft).
- **PRG-04 [ART]** Shipping bin FE19 + ship frames P1 + overnight tally cart + coin pop/shower VFX + currency icons. deps: none.
- **PRG-05 [GAME]** Player level + skill XP + level perks (energy/HP/stamina). deps: PRG-01.
- **PRG-06 [ART]** Level-up P5 + burst + XP sparkle + rank badges. deps: none.
- **PRG-07 [GAME]** Shipping bin (deposit, sell on sleep) + shipping summary screen. deps: PRG-03, PRG-04.
- **PRG-08 [UI]** Shipping summary screen + level/XP page. deps: PRG-07, PRG-05.
- **PRG-09 [ART]** Counter buy/sell P2 + shop counter sprites + shop UI frames. deps: none.
- **PRG-10 [GAME]** Shop framework (stock, hours, restock, tiers) + General Store (seeds by season). deps: PRG-03, PRG-09, WLD-01 (soft: homestead catalog).
- **PRG-11 [UI]** Shop window (buy/sell, quantity). deps: PRG-10.
- **PRG-12 [GAME]** Reputation (sources, ranks, gates API used by FRG-07/31 and story). deps: PRG-01.
- **PRG-13 [ART]** Rep crest pulse + rank-up P6 + rep icon. deps: none.
- **PRG-14 [UI]** Reputation meter + status page (gold/rep/sparks/level/HR). deps: PRG-12, PRG-13.
- **PRG-15 [ART]** Spark received P7 (+ VG6 from FRG-32 if not merged). deps: none.
- **PRG-16 [GAME]** Spark sources wiring (story, elders, tempered, festival, fish, vault) + spend API. deps: PRG-01, PRG-15.
- **PRG-17 [ART]** Bundle cabinet + restoration sequences ×6 + bundle offering P4. deps: none.
- **PRG-18 [GAME]** Community bundles + rewards (greenhouse, elevator express, canal locks, airship, forge discount, arena). deps: PRG-17, PRG-12.
- **PRG-19 [ART]** Museum pedestals/cases + donate P3. deps: none.
- **PRG-20 [GAME]** Museum donations (fossils, gems, artifacts) + rewards. deps: PRG-19, MIN-48, PRG-12.
- **PRG-21 [GAME]** Shops: Smithy, Ranch, Carpenter, Inn, Lodge, Kit's Friday merchant. deps: PRG-10, SOC-12, SOC-35.
- **PRG-22 [GAME]** Difficulty curve config: act gates (rep/forge/HR) as one `progression.ron`. deps: PRG-12, CMB-64, FRG-31.
- **PRG-23 [GAME]** Endgame: Destiny Trials (weekly modifiers) + tempered rotation. deps: MON-54, CMB-15.
- **PRG-24 [GAME]** Milestones (one-time rewards: first Gold crop, first legendary, F40, etc.) + toast. deps: PRG-03.
- **PRG-25 [AUD]** Economy sfx set. deps: PRG-07.
- **PRG-26 [SAVE]** Persist wallet, rep, level, bin, bundles, museum, milestones, shop states. deps: PRG-07, PRG-18.
- **PRG-27 [DATA]** Economy balance pass vs income targets (§3.6). deps: PRG-21, FRM-72, MIN-81, FSH-39.

**Ticket count: 27.**

**Build first (top 3):** PRG-01 (Wallet + real coin display), PRG-02 → PRG-03 (prices + sell/buy core), PRG-04 → PRG-07 (shipping bin with ship frames; this is the first gold source).
