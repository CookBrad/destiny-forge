# Destiny Forge — Epics & Tickets

Index of the GDD v1.0 work breakdown on GitHub.  
**Issues:** https://github.com/CookBrad/destiny-forge/issues

Priority order: **#1 → #2 → #3 / #4 (parallel after energy) → #5**.

Labels: `epic`, `ticket`, `phase-1`…`phase-5`, `combat`, `homestead`, `forge`.

---

## Epic #1 — Weapon Mastery (Phase 1, current priority)

https://github.com/CookBrad/destiny-forge/issues/1

| # | Ticket |
| - | ------ |
| 6 | Combo state machine foundation |
| 7 | Sword 2–3 hit combo string |
| 8 | Spear poke / lunge moveset |
| 9 | Special attack cooldowns + skill bar UI |
| 10 | Weapon-owned specials (data-driven) |
| 11 | Block mastery / parry window prototype |
| 12 | King Slime multi-phase hunt polish |
| 13 | Combat armor set skills v1 |
| 14 | Combat juice pass (hit stop, telegraphs, SFX) |

**Suggested order:** 6 → 7 & 8 (parallel) → 9 → 10; 11, 12, 13, 14 can parallel after basics.

---

## Epic #2 — Homestead Spine (Phase 2)

https://github.com/CookBrad/destiny-forge/issues/2

| # | Ticket |
| - | ------ |
| 15 | Soft day cycle + sleep |
| 16 | Hunt consumes large share of the day |
| 17 | Tool energy pool + UI meter |
| 18 | Homestead tool equip + use input |
| 19 | Crop plots: till, plant, water, harvest |
| 20 | Starter crops (2–3) + growth data |
| 21 | Cooking + pre-hunt food buffs |
| 22 | Inventory UX for crops / food / ore stacks |

**Suggested order:** 15 → 16 & 17 → 18 → 19 → 20 → 21; 22 can start early with 19.

---

## Epic #3 — Mining (Phase 3)

https://github.com/CookBrad/destiny-forge/issues/3

| # | Ticket |
| - | ------ |
| 23 | Mine zone layout + homestead entrance |
| 24 | Ore nodes + hardness + drops |
| 25 | Pickaxe tool + tiers (energy cost) |
| 26 | Forge metal tiers require mined ore |

**Depends on:** #17 tool energy (from Epic #2).

---

## Epic #4 — Fishing (Phase 4)

https://github.com/CookBrad/destiny-forge/issues/4

| # | Ticket |
| - | ------ |
| 27 | Fishing spot on homestead |
| 28 | Fishing cast + timing minigame |
| 29 | Rod tiers + fish loot tables (3–5 fish) |
| 30 | Fish food recipe + optional forge reagent |

**Depends on:** #17 tool energy; benefits from #21 cooking.

---

## Epic #5 — Content & Balance (Phase 5)

https://github.com/CookBrad/destiny-forge/issues/5

| # | Ticket |
| - | ------ |
| 31 | Second armor set + source monster |
| 32 | Second boss hunt |
| 33 | Iron Spear + monster spear weapon branch |
| 34 | Hammer weapon family prototype |
| 35 | Dungeon Floor 2 |
| 36 | Balance pass: day, energy, hunt length, damage |
| 37 | UI polish: set bonus preview + inventory sorting |
| 38 | Audio pass: hub, hunt, carve, craft |

**Depends on:** Epic #1 strongly; lifestyle epics for full day-loop balance (#36).

---

## Totals

| Kind | Count |
| ---- | ----- |
| Epics | 5 |
| Tickets | 33 |
| **All issues** | **38** |

See also: [`GAME_DESIGN.md`](GAME_DESIGN.md) (product design + PR plan).
