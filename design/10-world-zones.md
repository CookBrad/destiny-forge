# 10 — World & Zones (Hearthspire, forests, riverlands, crystal mines, highland ruins, floating islets, seasons/weather in the world)

Authority: GDD §6 ("steampunk world where clockwork industry and living magic coexist… airships, steam engines… the player inherits a rundown steam-powered homestead and the keys to a dormant Destiny Forge… Hub: player's farm + brass-and-timber village of Hearthspire. Zones: misty forests with glowing flora, riverlands with clockwork dams, crystal-veined mines, highland ruins of old magitech cities, floating islets. Seasons affect crops and creature behavior; certain magic-infused materials only appear in specific weather or seasons"), §8 art direction.

Build priority: **10 of 16**. Calendar/weather *rules* are in 11. This doc covers the places, their art, transitions, and world objects.

## 0. Standing rules

Zone travel actions (enter door, climb, board airship, ride lift, cross a bridge, swim/wade if any) get dwarf frames. Every zone gets in-house tilesets and animated props. **The dungeon Floor 1 art/platforms are frozen**: the dungeon gate stays where it is and looks as it does. Other zones' art can change (including the homestead 3× scale work, branch `fix/homestead-3x-scale`). No code comments; zero warnings. [ART] tickets are separate.

## 1. Already exists

| Zone | State | Where |
| --- | --- | --- |
| Homestead (top-down) | layout, house + bed, forge yard, crop plots, animal pen, dungeon gate, forest transition, mine entrance (PR #95) | `src/overworld/*` |
| Misty Forest (top-down) | 44×32 tiles, trees sheet (`forest/trees.png` 128×96), return trail | `src/forest/*` |
| Dungeon Floor 1 (side-scroll) | procedural platforms, King Slime | `src/dungeon/*` (frozen art) |
| Mine layer 1 (top-down) | PR #95 | `src/mining/*` |
| Shared exploration helpers | `build_map_border`, `spawn_grid_overlay`, `tile_checker_shade`, `tint_shade` | `src/exploration/*` |
| House interior / bed | PR #68 | |
| Environment tiles reuse `floor_ground`/`floor_platform`/`wall` 32×32 with tints | `assets/overworld/environment/` | placeholder look |

## 2. World map (zones and connections)

```
                    [Floating Islets] ←airship (Kit 3♥ / bundle)
                           ↑
 [Highland Ruins] ← mountain pass (Act 2) ← [Hearthspire Village] → canal → [Riverlands + Clockwork Dam]
         ↓                                          ↑ road                         ↓ spillway
  [Ember Caldera]                            [Homestead] ── trail ── [Misty Forest]
                                               │    │
                                         mine entrance  dungeon gate (Floor 1, frozen)
                                               ↓
                                   [Hearthspire Crystal Mines B1–B40 + Vault]
```

Top-down zones (exploration/life sim): Homestead, Village, Misty Forest, Riverlands, Highland Ruins (surface), Floating Islets (surface), Mines.
Side-scroll Hunting Grounds (05) are entered from a **Hunting Ground gate** inside each zone (forest grove, dam works, ruin plaza, caldera rim, islet spire).

## 3. Zones in detail

| Zone | Size (tiles) | Sub-areas | Points of interest | Gathering (01) | Fishing (03) | Monsters | Unlock |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Homestead** | 64×48 (3× scale branch) | farmhouse, fields ×3, forge yard, coop/barn lots, pond/dock, greenhouse ruin | Destiny Forge, Smelter, shipping bin, mailbox, mine entrance, dungeon gate | boulders, copper outcrops, weeds, twigs | pond | pests (moles, crows) | start |
| **Hearthspire Village** | 80×60 | square, lodge, smithy, store, inn, workshop, ranch, harbor/canal, museum, clinic, hall, airship mast, residences | quest board, bundles, festivals | forage (flowers, berries) | canal | none | day 1 |
| **Misty Forest** | 44×32 → 72×56 | glowcap glade, old shrine, mushroom hollow, hot spring (winter fishing), hunt grove gate | herbs, fossils, hardwood logs | Mistbrook stream | smalls (Mossling, Puffcap) in grounds | exists; expand Act 1 |
| **Riverlands** | 72×48 | clockwork dam, sluice gates, reed marsh, allotment, spillway, otter banks | dam works hunt gate, canal locks (bundle) | zinc, clay, river stones, reedmint, pearls | reservoir, spillway | river smalls | Act 1 (road) |
| **Crystal Mines** | per floor (01) | strata | elevator | ores, gems | B20/B35 | mine monsters | homestead entrance |
| **Highland Ruins** | 80×60 | magitech city ruins, rune plaza, aqueduct, sky-tram station (broken), caldera path | ruin plaza hunt gate, archive vault (Quillon) | silver, relic rubble, sunstone | aqueduct | rune sprites | Act 2 (mountain pass repair: Tobin + Brunwald) |
| **Ember Caldera** | 48×40 | lava fields, obsidian flats, vent towers | caldera hunt gate | obsidian, sulfur, ember herbs | lava pool (Aether rod) | ember lizards | Act 2 end |
| **Floating Islets** | 5 islets × 32×24 linked by wind bridges | sky garden, meteor crater, prism spires, Destiny Crater (finale) | cloud shoals, islet hunt gate | starmetal meteorite, skybloom | cloud shoals | sky jelly | Act 3 (airship) |

## 4. World objects / entities

`ZoneTransition {from, to, rect, arrival}` (exists as rects; generalize), `Door` (interiors), `HuntGate`, `AirshipMast`, `SkyTram` (Ruins → Islets shortcut post-bundle), `Bridge` (repairable), `RepairSite` (quest-gated obstacles: fallen tree, broken bridge, collapsed pass, rusted canal lock), `Signpost`, `FastTravelPoint` (airship stops), `WeatherEmitter` (rain/snow/fog/storm per zone), `AmbientCritter` (butterflies, fireflies, gears-birds), `LightSource` (lamps on at Evening), `Season dressing` (per-season tile skins).

## 5. ANIMATIONS (most detailed section)

Conventions: top-down tiles 32×32 (TILE = 32), autotiles 16-tile blob sets (or 47-tile where corners matter), props in multiples of 32. Season skins: each outdoor tileset gets **4 season variants** baked by the draw script (spring/summer/fall/winter: palette + snow cover drawn as separate tiles). Player traversal sets use the dwarf (16×28; 48×60 canvas for wide actions), right-authored + `flip_x`, with back/front facings where the action is vertical (ART-02).

### 5.1 Player traversal sets

| # | Set | Frames | Canvas | Facing | Timing | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| W1 | Door enter (open + step in) | 4 | 24×32 | back | 0.08 ×4 | then fade |
| W2 | Door exit | 3 | 24×32 | front | 0.08 ×3 | |
| W3 | Climb ladder/vines (zone) | 4 | 16×32 | back | 0.12 loop | shares MIN C11/C12 art if the same ladder; vines need a new set |
| W4 | Board airship (step up + wave) | 5 | 32×40 | R | 0.1 ×5 | |
| W5 | Airship ride (deck idle, wind) | 4 | 32×40 | R | 0.15 loop | travel cutscene |
| W6 | Sky-tram ride | 2 | 32×40 | front | 0.3 loop | |
| W7 | Wade shallow water (walk variant) | 4 | 16×28 | R | 0.12 loop | riverlands marsh |
| W8 | Wind bridge gust brace | 3 | 24×28 | R | 0.1 | islets |
| W9 | Repair site interaction (hammer/wrench) | 4 | 48×60 | R | 0.1 loop ×3 | e.g. fixing the bridge |
| W10 | Read signpost | 2 | 16×28 | back | 0.3 | |
| W11 | Walk up / walk down (if ART-02 approves) | 4 + 4 | 16×28 | back/front | 0.1 loop | shared with SOC-40 |
| W12 | Snow walk (winter, footprint VFX) | uses the walk + footprint VFX | | | | |
| W13 | Rain idle (shake off water) | 4 | 16×28 | front | 0.1 | entering interiors on rainy days |

### 5.2 Tilesets and props (per zone)

| Zone | Tilesets (×4 seasons) | Animated props (frames @ timing) |
| --- | --- | --- |
| Homestead | grass, dirt path, cobble, water edge, fence, farmhouse exterior | windmill-turbine 6 @0.12, chimney smoke 6 @0.15, steam valve puff 5 @0.08, pond lily sway 2 @0.6, lamp post flicker 3 @0.12 (Evening) |
| Village | cobblestone, brass plaza tiles, timber walls, canal edge, roofs | clock tower gears 8 @0.1, bell swing 6 @0.08 (on the hour), fountain 4 @0.12, canal boats bob 2 @0.5, shop signs swing 4 @0.2, airship mast balloon 4 @0.25, street lamps 3 |
| Misty Forest | moss ground, roots, glowcap patches, fog overlay | glowcap pulse 4 @0.2, fireflies 4 @0.1, fog drift 6 layers (parallax sprite), shrine rune glow 4 @0.2, waterfall 4 @0.08, swaying tall trees 3 @0.4 (new trees beyond `trees.png`) |
| Riverlands | riverbank, reeds, marsh water, dam brickwork, sluice metal | dam turbine spin 8 @0.08, sluice gate open/close 6 @0.1, water spill 4 @0.08, reed sway 3 @0.3, otters (critter) 4 |
| Highland Ruins | ruined magitech plating, rune tiles, overgrowth, aqueduct | rune pylon pulse 6 @0.12, broken sky-tram sway 4 @0.3, floating debris bob 4 @0.4, aqueduct water 4 @0.1, crystal growths shimmer 3 |
| Ember Caldera | basalt, obsidian, lava (shared MIN E16 style, separate top-down set), ash | vent tower steam 6 @0.1, lava bubble 4 @0.15, ash fall overlay 6, heat shimmer (shader-free drawn ripple overlay 4) |
| Floating Islets | sky grass, cloud edges, prism crystal, wind bridge planks | cloud drift 6 @0.2, prism spire glints 4 @0.15, wind ribbon 6 @0.08, meteor crater glow 4, island bob (whole-tile parallax 2 @0.8) |
| Interiors | wood floor, brass trim, rugs, furniture | hearth fire 4 @0.12, kettle steam 4, clock pendulum 4 @0.25 |

### 5.3 Weather / season VFX (world)

Rain 4-frame drop sheet (full-screen tiled 128×128) @0.05 + puddle ripple 4 + splash 3; storm lightning flash (2 drawn overlay frames) + bolt 5 (64×256); snow 6 @0.1 + accumulation tile variants; fog bank 6 layers; spring blossom drift 6; summer heat shimmer 4; fall leaf drift 6; windy grass sway overrides 3; rainbow overlay (post-rain) 1; aurora (winter night, islets) 6 @0.2.

### 5.4 UI icons

Zone map (parchment brass map 320×240) with zone vignettes ×8, fast-travel icons, location markers (shops, gates), weather icons ×6, season icons ×4, zone title cards (drawn banners ×8, 160×32).

## 6. Inventory items

None new beyond gathering (01) and forage: Spring Onion, Salmonberry-like "Brassberry", Wild Leek, Chanterelle, Blue Mushroom, Hazelnut, Snow Root, Crystal Fruit, Skybloom (01), Ember Pepper (caldera). Each: stack 99, sell 20–150, sources = zone forage spawns by season.

## 7. Processing

N/A. Zones are sources. Repair sites take materials (bridge: 50 Wood + 10 Iron Bar; pass: 20 Steel + Brunwald quest; canal locks: Waters bundle).

## 8. Hooks

All gathering/fishing/hunting locations; social schedules use zones; story acts open zones; weather/season dressing from 11; field guide habitat links; audio ambience per zone.

## 9. Energy / time / risk

Walking costs no energy. Zone transitions take 0.5 s fades. Airship rides cost 1 phase (the islets) or 0 (village ↔ homestead post-bundle). Risk: only in hunting grounds and mines. Overworld zones are safe except pests.

## 10. Progression / gating

Forest (start) → Village (day 1) → Riverlands (Act 1: repair the bridge) → Ruins (Act 2: mountain pass) → Caldera (Act 2 end) → Islets (Act 3: airship) → Destiny Crater (finale). Rare nodes per season/weather (01). Hunting Ground gates open when the HR/forge gate is met (05).

## 11. UI

Zone title card on first entry; world map (M key) with unlocked zones, current location, villagers' current locations (after 4♥, toggle), fast-travel nodes; minimap optional (13 setting); signpost text.

## 12. Audio

Per-zone music (day/evening variants) and ambience beds: homestead (birds, windmill), village (crowd murmur, clock tower, canal), forest (fog wind, insects, owls at Evening), riverlands (water rush, dam turbines), ruins (wind, rune hum), caldera (lava rumble), islets (high wind, chimes). Weather layers: rain, storm thunder, snow muffling (low-pass), wind.

## 13. Save data

`zones_unlocked`, `repairs_completed`, `last_zone` (loads still spawn at the bed, as today), `fast_travel_unlocked`, per-zone `forage_spawns: Vec<{zone, tile, item, day}>`, `world_flags` (generalize `WorldProgress`).

## 14. Balance levers

Zone sizes, forage density per season, transition costs, airship time cost, repair costs, gate requirements.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Village as a separate zone or part of the homestead map? | Separate top-down zone connected by a road from the homestead's east edge. |
| Q2 | Hunting grounds inside zones or a separate menu? | Physical gate inside each zone that leads into the side-scroll ground (05), keeping the world feel. |
| Q3 | Season tile skins: 4 sets per tileset is a lot of art | Bake through the draw script (palette remaps + hand-drawn snow overlay tiles). These count as in-house drawn tiles. |
| Q4 | Reuse `floor_ground`/`wall` tinted placeholders? | Replace them per zone with in-house tilesets (tints are placeholder only). |
| Q5 | Map scale | Follow the `fix/homestead-3x-scale` direction if merged. Other zones share the same TILE world units. |

## 16. Ticket breakdown

- **WLD-01 [GAME]** Hearthspire Village zone shell (layout, transitions from the homestead east road, buildings as door rects). deps: none.
- **WLD-02 [ART]** Village tilesets (spring) + buildings exteriors + clock tower/fountain/lamps props. deps: none.
- **WLD-03 [GAME]** Village art pass using WLD-02 + animated props. deps: WLD-01, WLD-02.
- **WLD-04 [ART]** Door enter/exit W1/W2 + signpost W10. deps: none.
- **WLD-05 [GAME]** Generic `Door`/interior transition system + 3 interiors (store, smithy, inn). deps: WLD-04, WLD-01.
- **WLD-06 [ART]** Interior tilesets + furniture + hearth/kettle/clock props. deps: none.
- **WLD-07 [ART]** Homestead in-house tilesets (replace tinted placeholders) + homestead props (windmill, chimney, valve, lamps). deps: none (coordinate with the 3×-scale branch).
- **WLD-08 [GAME]** Homestead art pass. deps: WLD-07.
- **WLD-09 [ART]** Misty Forest expanded tileset + new trees + glowcaps/fireflies/fog/shrine/waterfall. deps: none.
- **WLD-10 [GAME]** Forest expansion (72×56, sub-areas, hunt grove gate, forage spawns). deps: WLD-09, CMB-19 (gate soft).
- **WLD-11 [GAME]** Zone transition framework generalization (ZoneTransition data, arrival points, fades) + world flags (generalize WorldProgress). deps: none.
- **WLD-12 [ART]** Riverlands tileset + dam/turbine/sluice + wade W7. deps: none.
- **WLD-13 [GAME]** Riverlands zone + bridge repair site + dam hunt gate. deps: WLD-11, WLD-12, WLD-14.
- **WLD-14 [ART]** Repair site W9 + broken/repaired bridge, pass, canal lock props. deps: none.
- **WLD-15 [GAME]** Repair site system (materials + quest flag → state change). deps: WLD-14, WLD-11.
- **WLD-16 [ART]** Highland Ruins tileset + pylons/aqueduct/sky-tram. deps: none.
- **WLD-17 [GAME]** Highland Ruins zone + pass repair gate (Act 2). deps: WLD-16, WLD-15, NAR-01 (Act 2 flag).
- **WLD-18 [ART]** Ember Caldera tileset + vents/lava/ash. deps: none.
- **WLD-19 [GAME]** Ember Caldera zone. deps: WLD-18, WLD-17.
- **WLD-20 [ART]** Floating Islets tileset + clouds/prisms/wind bridges + W4/W5/W8 + airship. deps: none.
- **WLD-21 [GAME]** Floating Islets zone + airship travel. deps: WLD-20, SOC-35 (Kit 3♥) or PRG-18.
- **WLD-22 [ART]** Season skins for homestead/village/forest/riverlands (summer/fall/winter variants + snow overlays). deps: WLD-02, WLD-07, WLD-09, WLD-12.
- **WLD-23 [GAME]** Season dressing swap per season. deps: WLD-22, CYC-02.
- **WLD-24 [ART]** Weather VFX set (§5.3) + rain idle W13. deps: none.
- **WLD-25 [GAME]** Weather rendering per zone (rain/snow/fog/storm/wind). deps: WLD-24, CYC-05.
- **WLD-26 [ART]** World map parchment + vignettes + markers + title cards. deps: none.
- **WLD-27 [UI]** World map (M) + zone title cards + fast-travel. deps: WLD-26, WLD-21.
- **WLD-28 [ART]** Ambient critters (butterflies, fireflies, gear-birds, otters). deps: none.
- **WLD-29 [GAME]** Ambient critters + forage spawn system per zone/season. deps: WLD-28, CYC-02.
- **WLD-30 [AUD]** Zone music + ambience beds + weather layers. deps: WLD-03, WLD-25.
- **WLD-31 [SAVE]** Persist zones unlocked, repairs, fast travel, forage spawns, world flags. deps: WLD-11, WLD-15.

**Ticket count: 31.**

**Build first (top 3):** WLD-11 (transition framework + world flags), WLD-01 + WLD-02 → WLD-03 (Hearthspire Village, which is needed by social/economy), WLD-04 → WLD-05 (doors/interiors with door frames).
