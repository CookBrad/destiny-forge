# Destiny Forge — Full Game Epics & Tickets

Complete map of **all game functionality** from GDD v1.0.  
**Issues:** https://github.com/CookBrad/destiny-forge/issues  
**Design:** [`GAME_DESIGN.md`](GAME_DESIGN.md)

## How to read this

| Mark | Meaning |
| ---- | ------- |
| ✅ | Shipped (ticket closed / `status:shipped`) |
| 🔧 | In progress / largely done, polish remaining |
| ⬜ | Not started |

**Implementation order (remaining work):**  
`#1 Weapon Mastery` → `#40 Loot` / `#41 Forge` polish → `#2 Homestead` → `#3 Mining` ∥ `#4 Fishing` → `#5 Content` → world polish.

---

## Epic map (all pillars)

| Epic | GitHub | Pillar | Status |
| ---- | ------ | ------ | ------ |
| **Foundation — Core Loop** | [#39](https://github.com/CookBrad/destiny-forge/issues/39) | Hunt→carve→forge shell | ✅ mostly shipped |
| **Weapon Mastery** | [#1](https://github.com/CookBrad/destiny-forge/issues/1) | Combat skill ceiling | ✅ shipped |
| **Hunt Loot & Carving** | [#40](https://github.com/CookBrad/destiny-forge/issues/40) | MH materials | ✅ baseline; ⬜ depth |
| **Forging & Gear Progression** | [#41](https://github.com/CookBrad/destiny-forge/issues/41) | MH gear trees | ✅ baseline; ⬜ expand |
| **World, Saves & Meta** | [#42](https://github.com/CookBrad/destiny-forge/issues/42) | Hub, zones, profiles | ✅ mostly; ⬜ sleep/landmarks |
| **Homestead Spine** | [#2](https://github.com/CookBrad/destiny-forge/issues/2) | Day, energy, farm, food | 🔧 day/sleep started |
| **Mining** | [#3](https://github.com/CookBrad/destiny-forge/issues/3) | Ore → metal spine | ⬜ |
| **Fishing** | [#4](https://github.com/CookBrad/destiny-forge/issues/4) | Minigame, fish food/reagents | ⬜ |
| **Content & Balance** | [#5](https://github.com/CookBrad/destiny-forge/issues/5) | Floor 2, sets, polish | ⬜ |

---

## #39 Foundation — Core Loop

Hub → dungeon → fight → **carve** → **forge** → re-hunt. Vertical slice.

| # | Ticket | Status |
| - | ------ | ------ |
| 43 | App shell: Bevy plugins, title, game states | ✅ |
| 44 | Dungeon Floor 1: platforms, player, enemies | ✅ |
| 45 | Combat baseline: attack, block, health, death | ✅ |
| 46 | Material inventory + loadout slots | ✅ |
| 47 | End-to-end vertical slice regression | ✅ |

---

## #1 Weapon Mastery

Deeper combat than Stardew: combos, CDs, parry, boss phases, set skills, juice.

| # | Ticket | Status |
| - | ------ | ------ |
| 6–14 | Combo, spear/sword, specials, parry, boss, set skills, juice | ✅ shipped (`80f88a6`) |

---

## #40 Hunt Loot & Carving

**This is the carve/loot epic.** Unique set materials come from monsters.

| # | Ticket | Status |
| - | ------ | ------ |
| 48 | Hold-to-carve corpses (interruptible) | ✅ |
| 49 | Per-monster carve loot tables (MVP) | ✅ [PR #66](https://github.com/CookBrad/destiny-forge/pull/66) |
| 50 | Boss carve: rare parts for set unlocks | ✅ [PR #66](https://github.com/CookBrad/destiny-forge/pull/66) |
| 51 | Carve UX: progress feedback | ✅ [PR #65](https://github.com/CookBrad/destiny-forge/pull/65) |
| 52 | Expanded loot tables + rarity tiers | 🔧 partial (bonus chance rolls in tables) |

---

## #41 Forging & Gear Progression

**This is the forge epic.** Weapon trees, armor sets, tools, data files.

| # | Ticket | Status |
| - | ------ | ------ |
| 53 | Forge station UI: recipe cycle + craft | ✅ |
| 54 | Weapon tree MVP (Rusty→Iron→Slime + spear) | ✅ |
| 55 | Sli me armor set (4pc) + set bonuses | ✅ |
| 56 | Migrate recipes/loot to RON data | 🔧 [PR #67](https://github.com/CookBrad/destiny-forge/pull/67) |
| 57 | Forge tool recipes (hoe, pickaxe, rod…) | ⬜ |
| 58 | Gear storage / multi-weapon stash | ⬜ |

*(Also: #26 metal ore recipes, #33 spear branches, #31 second set — under Mining/Content but forge-facing.)*

---

## #42 World, Saves & Meta

Zones, transitions, persistence, menus.

| # | Ticket | Status |
| - | ------ | ------ |
| 59 | Homestead hub + forge + dungeon gate | ✅ |
| 60 | Forest zone transition | ✅ |
| 61 | Profile save/load + autosave + settings | ✅ |
| 62 | House interior / bed for sleep | 🔧 [PR #68](https://github.com/CookBrad/destiny-forge/pull/68) |
| 63 | Landmarks: mine entrance + fishing dock | ⬜ |
| 64 | Pause menu + key rebinding polish | ⬜ |

---

## #2 Homestead Spine (Stardew feel)

| # | Ticket | Status |
| - | ------ | ------ |
| 15 | Soft day cycle + sleep | 🔧 [PR #68](https://github.com/CookBrad/destiny-forge/pull/68) |
| 16 | Hunt consumes large share of the day | ⬜ |
| 17 | Tool energy pool + UI meter | ⬜ |
| 18 | Homestead tool equip + use | ⬜ |
| 19 | Crop plots: till/plant/water/harvest | ⬜ |
| 20 | Starter crops (2–3) | ⬜ |
| 21 | Cooking + pre-hunt food buffs | ⬜ |
| 22 | Inventory UX for crops/food/ore | ⬜ |

---

## #3 Mining

| # | Ticket | Status |
| - | ------ | ------ |
| 23 | Mine zone + homestead entrance | ⬜ |
| 24 | Ore nodes + hardness + drops | ⬜ |
| 25 | Pickaxe tiers (energy) | ⬜ |
| 26 | Forge metal tiers require mined ore | ⬜ |

---

## #4 Fishing

| # | Ticket | Status |
| - | ------ | ------ |
| 27 | Fishing spot on homestead | ⬜ |
| 28 | Cast + timing minigame | ⬜ |
| 29 | Rod tiers + fish tables | ⬜ |
| 30 | Fish food + optional forge reagent | ⬜ |

---

## #5 Content & Balance

| # | Ticket | Status |
| - | ------ | ------ |
| 31 | Second armor set + monster monster | ⬜ |
| 32 | Second boss hunt | ⬜ |
| 33 | Iron Spear + monster spear branch | ⬜ |
| 34 | Hammer weapon family prototype | ⬜ |
| 35 | Dungeon Floor 2 | ⬜ |
| 36 | Balance pass (day/energy/hunt/damage) | ⬜ |
| 37 | UI polish: set preview + inventory sort | ⬜ |
| 38 | Audio pass: hub, hunt, carve, craft | ⬜ |

---

## Functionality checklist (GDD coverage)

| GDD system | Epic | Covered? |
| ---------- | ---- | -------- |
| Dual perspective (top-down / side-scroll) | #42, #39 | ✅ |
| Hub / forge / dungeon gate | #42, #39 | ✅ |
| Forest link | #42 | ✅ |
| Saves / profiles | #42 | ✅ |
| Floor hunts + procedural dungeon | #39 | ✅ |
| Boss hunts | #1, #5 | 🔧 / ⬜ |
| Weapon mastery combat | #1 | 🔧 |
| Carve corpses | #40 | ✅ + ⬜ depth |
| Material inventory | #39 | ✅ |
| Forge craft weapons/armor | #41 | ✅ + ⬜ expand |
| Armor set skills | #1, #41 | 🔧 |
| Day cycle / energy | #2 | ⬜ |
| Farming | #2 | ⬜ |
| Food buffs | #2 | ⬜ |
| Mining / ore | #3 | ⬜ |
| Fishing | #4 | ⬜ |
| Floor 2 / more sets | #5 | ⬜ |
| Audio / UI polish | #5, #42 | ⬜ |

---

## Labels

`epic`, `ticket`, `phase-0`…`phase-5`, `combat`, `homestead`, `forge`, `loot`, `world`, `status:shipped`

---

## Totals (approx.)

| Kind | Count |
| ---- | ----- |
| Epics | **9** |
| Tickets | **~55** |
| Shipped tickets closed | **13** (#43–49, #53–55, #59–61) |

Filter shipped: https://github.com/CookBrad/destiny-forge/issues?q=label%3Astatus%3Ashipped  
Filter open work: https://github.com/CookBrad/destiny-forge/issues?q=is%3Aopen+label%3Aticket
