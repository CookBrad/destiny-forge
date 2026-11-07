# Destiny Forge — System Design Breakdowns (index)

Designer breakdowns for every system in the game. These documents are the standard for each system, and the engine is **Rust + Bevy 0.15**. Repo `CookBrad/destiny-forge`, default branch `develop`. `docs/GAME_DESIGN.md` and `docs/EPICS.md` were read only to learn what exists; they are not authority.

Each file is written so the PM agent (Forge) can split it into tickets. Every doc has the same sections: where it happens · entities/objects · tools & tiers · **animations (the most detailed section: every sprite set with frames, size, facing, and timing)** · inventory items (icon/stack/price/source) · processing · hooks (esp. forge + hunting) · energy/time/risk · progression/gating · UI · audio · save data · balance levers · open questions with proposed answers · **ticket breakdown**.

## Standing rules (baked into every doc)

1. **Every player action needs new in-house sprite frames.** UI banners, color tints, and tinted arcs do not count. Tier variants are separate baked PNGs from `tools/draw_*.py` scripts.
2. **Clean Code, no code comments.**
3. **Zero warnings/errors** (`cargo build`, `clippy --all-targets`, `test`).
4. **Dungeon platform/art churn is frozen** (`assets/dungeon/**`, Floor 1 layout, King Slime). Taste-signed hunter cells never change. Other zones' art is fine to change.
5. **Sprite/animation work is in its own [ART] tickets.** Gameplay tickets list the ART tickets they depend on (*soft* = placeholder allowed until the art merges).

## Ticket legend

`[ART]` sprites/animation (PNG + draw script + loader constants) · `[GAME]` gameplay logic · `[DATA]` RON/balance · `[UI]` · `[AUD]` · `[SAVE]`. IDs: MIN, FRM, FSH, FRG, CMB, MON, ANM, SOC, PRG, WLD, CYC, NAR, UI, AUD, ART, SAV. All cross-references between docs use these IDs, and every referenced ID exists.

## Art conventions (summary; canonical in 15)

| Kind | Size | Facing | Notes |
| --- | --- | --- | --- |
| Dwarf (homestead/mine/zones) | 16×28 body; action canvas 48×60 (dwarf at 16,16), 64×60 for long tools | right-authored + `flip_x`; back/front only for vertical actions (ART-02) | matches the shipped pickaxe swing (4 f, 0.3 s) and eat (4 f, 0.18 s) |
| Side-scroll hunter + companions | 343×160 cells | right + flip | grit-neon, signed cells untouchable |
| Large monsters | 384×256 / 512×320 / 640×384 | right + flip | modular rigs baked to PNG (MON-03) |
| Tiles / props | 32×32 (TILE = 32) | — | 4 season skins baked |
| Item icons | 16×16 shown ×3 | — | atlas + quality star overlays |

## Files (build-priority order)

| # | File | System | Tickets | Build first (top 3) |
| --- | --- | --- | --- | --- |
| 01 | [01-mining.md](01-mining.md) | Mining & gathering | 84 | MIN-01 NodeKind data table · MIN-06→MIN-07 Smelter art + station (bars) · MIN-11/MIN-12→MIN-13 multi-floor mine + ladder art |
| 02 | [02-farming.md](02-farming.md) | Farming (quality, seasons, fruit trees, magitech machines) | 73 | FRM-01→02 hoe frames · FRM-03→04 watering frames (+FRM-06→07 sow/harvest) · FRM-08 quality on inventory slots |
| 03 | [03-fishing.md](03-fishing.md) | Fishing | 40 | FSH-01→02 pond + dock · FSH-03→04 cast/hook sprites + loop · FSH-05→06 minigame art + reel |
| 04 | [04-forging-crafting.md](04-forging-crafting.md) | Forging & crafting (ranks, skills, decorations, Destiny Binding, forge levels, workbench, kitchen) | 46 | FRG-04 gear data rows · FRG-01/02→03 craft animation cinematic · FRG-06→07 recipe v2 + forge L0/L1 |
| 05 | [05-combat-hunting.md](05-combat-hunting.md) | Combat & hunting (GS/LS/SnS/DB/Hammer/Bow, tracking, parts, phases, status, traps, capture, carting) | 64 | CMB-01/02→03 frames for existing block/parry/charge/spin/hurt/idle · CMB-09→10 carve anim · CMB-15 Quest Board |
| 06 | [06-monsters.md](06-monsters.md) | Monsters (12 large + small, behaviors) | 58 | MON-01/02 model + layered animator · MON-03 rig bake tool · MON-06→07 Bristleboar |
| 07 | [07-animals-husbandry.md](07-animals-husbandry.md) | Animals & husbandry | 42 | ANM-01→02 in-house chicken/cow + model · ANM-03→04 petting/friendship · ANM-05→06 coop/barn |
| 08 | [08-social-villagers.md](08-social-villagers.md) | Social & villagers (schedules, gifts, hearts, quests, festivals, companions) | 40 | SOC-01/02 villager model + first NPC art · SOC-03→05 dialogue · SOC-06→08 gifts + hearts |
| 09 | [09-progression-economy.md](09-progression-economy.md) | Progression & economy | 27 | PRG-01 Wallet · PRG-02→03 prices + buy/sell · PRG-04→07 shipping bin |
| 10 | [10-world-zones.md](10-world-zones.md) | World & zones | 31 | WLD-11 transitions/world flags · WLD-01/02→03 Hearthspire village · WLD-04→05 doors/interiors |
| 11 | [11-day-season-cycle.md](11-day-season-cycle.md) | Day/season cycle & weather | 21 | CYC-02 seasons · CYC-03→04 sleep animation · CYC-05 weather |
| 12 | [12-narrative-story.md](12-narrative-story.md) | Narrative & story events | 22 | NAR-01 story flags · NAR-02 cutscene system · NAR-07 Floor 1 as "Slime Season" quest |
| 13 | [13-ui-ux.md](13-ui-ux.md) | UI/UX (inventory, sources, field guide, compare, remap, assists) | 28 | UI-01 InputMap · UI-02→03/04 theme + icons · UI-06→07 material sources + tooltip |
| 14 | [14-audio.md](14-audio.md) | Audio | 18 | AUD-01 bus mixer · AUD-02→03 cue registry + frame-synced cues · AUD-04 music director |
| 15 | [15-art-presentation.md](15-art-presentation.md) | Art & presentation (pipeline + whole-game sprite roll-up ≈ 11k frames) | 13 | ART-01 draw helpers · ART-03 sprite registry · ART-02 dwarf back/front facing |
| 16 | [16-saves.md](16-saves.md) | Saves | 12 | SAV-02 stop silent profile resets · SAV-01 migration framework · SAV-05 namespaced sub-structs |
| | **Total** | | **619** | |

## Cross-system notes for Forge (PM)

- **Pull forward** (small, unblock many): ART-01, ART-03, CYC-02 (seasons), CYC-05 (weather), SAV-01/SAV-02/SAV-05, UI-01 (InputMap), FRG-04 (gear rows), FRM-08 (item quality), PRG-01 (wallet), WLD-11.
- **Standing-rule debt**: these existing actions have no frames today. Hoe, watering, sow, harvest, crop growth, bed sleep, forge craft, carve, block/parry/charge/spin/hurt/idle, Iron pickaxe (uses T1 frames), ore crack states, and zone transitions. The list with ticket IDs is in 15 §5.1.
- **Bug found while reading saves**: `storage::load_profile` falls back to `Default` on any parse error, so the next autosave can overwrite a real save with a fresh profile. Fix: SAV-02.
- **Schema churn**: PR #95 (additive `depleted_ore_node_ids`) and PR #96 (v7 bump) both edit `PlayerProfile`. This is fine today through `serde(default)`. SAV-01/SAV-05 keep 16 systems from colliding.
- **Art is the critical path** (≈ 11,000 frames estimated; 15 §5). Launch scope levers: 8 large monsters first, 3 trees per weapon, 8 villagers first, weapon order SnS → Hammer → GS → Bow → LS → DB.
- Already shipped / in flight, not re-ticketed: hunt→carve→forge→re-hunt loop, sword/spear, slime set, Floor 1 + King Slime, day/sleep, tool energy, till/plant/water/harvest, homestead/forest/dungeon zones, saves/profiles, PR #95 mining slice, PR #96 food buffs + inventory UX + eat frames.
