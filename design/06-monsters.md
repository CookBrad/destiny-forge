# 06 — Monsters (large + small, behaviors)

Authority: GDD §4.5 (large magical creatures; tells, part breaks, status, traps; phases, elemental shifts, enrage), §4.2 (small monsters in the mine), §5 difficulty curve "Village monsters → Forest/Mine threats → Large territorial monsters → Elder-class threats", §6 (seasons affect creature behavior), §8 ("distinctive silhouettes, readable tells, satisfying part-break animations… blend organic and crystalline or elemental traits"), §10 (Must-have **8–12 unique large monsters**; "define 2 monster behavior sets" first).

Build priority: **6 of 16**. Combat rules are in 05. Mine monsters are listed in 01 §5.4/§8 and not repeated here.

## 0. Standing rules

Every monster action is a drawn frame strip: telegraphs, attacks, flinches, part-break states, trapped/KO/sleep/limp/death. Color-flash damage feedback must be a drawn silhouette frame. (The current `BOSS_COLOR_*` white values are fine as no-ops, but no tint-only feedback.) **King Slime and Floor 1 enemies are frozen dungeon art. Do not redraw them.** New monsters live under `assets/monsters/<id>/`. No code comments; zero warnings. [ART] tickets are separate.

## 1. Already exists

Dungeon enemies (frozen art, 4-frame sheets): slime 64×64, bat 96×48, goblin 64×112, skeleton 64×144, zombie 96×128. King Slime boss 192×192 ×4, phases + ground slam (96×24 hurtbox, 1.35 s). Enemy AI/movement/stats/anim (`dungeon/enemy*.rs`, frame times 0.10–0.18 s). Carve loot per monster (`carve_loot.ron`). Forest has trees only (no monsters). Overworld has wandering quadruped animals (`overworld/animals.rs`, 16×16 cells).

## 2. Where they live

See 05 §2 for the hunting grounds. Small monsters fill grounds and zones. Large monsters have a **home ground**, a **nest area**, and a seasonal schedule (Q4).

## 3. Monster framework (entities/components)

`MonsterDef` (data `monsters.ron`): id, class (Small/Large/Elder), rank availability, HP by rank, size, speed, element, weaknesses (per hitzone × damage type × element), parts (id, hitbox, HP, breakable/severable, reward table), status thresholds, KO/exhaust, moves (per phase), phase thresholds, enrage rule, fatigue, capture threshold, tells, habitat/season/weather/time, drops (carve/break/capture/quest).
Runtime: `Monster {def}`, `MonsterBrain` (state machine: Idle, Roam, Alert, Engage, Telegraph(move), Attack(move), Recover, Flinch, Topple, Stunned, Trapped, Sleeping, Fatigued, Enraged overlay, AreaChange, Limp, Nest, Dead, Captured), `MoveSet`, `Aggro`, `PartState`, `StatusBuildup` (05), `MonsterAnimator` (layered parts: base + broken overlays).

**Two behavior archetypes first** (GDD §11): *Charger* (Bristleboar: rush/turn/charge, readable straight-line threats) and *Caster/Zoner* (Glowmoth Matriarch: ranged patterns, ground AoE, aerial). Every other monster combines these with one signature mechanic.

## 4. Large monster roster (12 + existing King Slime)

| # | Monster | Class | Ground | Rank | Element / Weak | Signature | Breakable parts |
| --- | --- | --- | --- | --- | --- | --- | --- |
| L0 | King Slime (exists) | Village | Dungeon F1 | Low | Slime / Fire | ground slam | (none; carve only) |
| L1 | **Bristleboar** | Village threat | Outskirts | Low | — / Fire, Thunder | charging tusks, mud-roll | tusks (×2), back bristles |
| L2 | **Thornback Briarwolf** | Forest | Misty Forest | Low/High | Poison / Fire | thorn-volley tail, pounce combos | head, thorn back, tail (sever) |
| L3 | **Glowmoth Matriarch** | Forest | Misty Forest | Low/High | Sleep spores / Thunder, Ice | spore clouds, flying dives, light-burst | wings (×2), antennae, abdomen |
| L4 | **Tidecaller Kraken-Eel** | Large territorial | Riverlands Dam | Low/High | Water / Thunder | water beams, submerge/emerge, tentacle sweeps | head fin, tentacles (×2), tail (sever) |
| L5 | **Ironhide Behemoth** | Large territorial | Riverlands Dam works | High | — (metal) / Thunder, blunt | armored plates crack, boulder throw, quake stomp | plates (×3), horn, forelegs |
| L6 | **Gearjaw Drillwyrm** | Large territorial | Galleries Deep | High | Fire (steam) / Water | burrows, drill-snout charge, steam vents | drill snout, gear spine, tail (sever) |
| L7 | **Stormwing Gryphon** | Large territorial | Highland Ruins | High/Master | Thunder / Ice | aerial dives, lightning feathers, wind gusts | head crest, wings (×2), tail |
| L8 | **Runeback Golem** | Large territorial | Highland Ruins | High | Aether / Thunder (cores) | rune pylons, laser sweeps, shield phases | rune cores (×3), arms (×2), head |
| L9 | **Magmaw Salamander** | Large territorial | Ember Caldera | High/Master | Fire / Water, Ice | lava swim, magma spit, molten-armor cooling | head, back crust, tail (sever) |
| L10 | **Prism Leviathan Ray** | Large territorial | Floating Islets | Master | Aether / Dragon-like "Void" (none; Ice) | glides, prism beams, refracting crystal spires | crystal fins (×2), tail stinger (sever), core |
| L11 | **Aether Drake** | Elder | Floating Islets | Master | Aether, elemental shift (Fire↔Ice↔Thunder) | element shifts per phase, nova | horns, wings (×2), tail (sever), chest core |
| L12 | **Elder Destiny Wyrm** (finale) | Elder | Destiny Crater | Master | all (rotating) / weak point shifts | 3 phases, arena changes, Destiny nova | horns, wings, tail, heart-forge core |

Tempered (High/Master post-game) variants of L2–L9: +HP, faster tells, extra move, ~5% Destiny Spark.
Seasonal/weather variants: Frost Briarwolf (winter), Storm-charged Kraken-Eel (thunderstorm), Bloom Glowmoth (spring; spores give status cure if broken).

### 4.1 Per-monster move tables (telegraph = readable windup; times in seconds)

**L1 Bristleboar** (HP Low 1,800). Phases: P1 100–50%, P2 <50% (adds mud roll). Enrage: after 600 dmg in 20 s, or on a tusk break. Tells:
| Move | Telegraph | Active | Recover | Dmg | Notes |
| --- | --- | --- | --- | --- | --- |
| Paw & Charge | 0.8 (paws ground ×2, dust) | 1.2 straight | 0.9 skid (punish) | 22 | turns at walls |
| Tusk Toss | 0.5 head dip | 0.3 | 0.6 | 18 + launch | |
| Mud Roll (P2) | 0.7 rolls on its back | 1.0 | 1.0 | 14 + Slimed | |
| Bristle Shake (enraged) | 0.6 bristles rise | 0.4 radial | 0.5 | 10 | breaking the bristles disables it |
| Roar | 0.4 | 1.0 stun-lock (Earplugs) | — | 0 | |

**L2 Thornback Briarwolf** (HP Low 2,600 / High 9,000). P2 at 60% (the tail volley gains 2 extra thorns; poison). Severing the tail removes the volley.
| Move | Telegraph | Active | Recover | Notes |
| --- | --- | --- | --- | --- |
| Pounce | 0.6 crouch + growl | 0.4 arc | 0.7 | |
| Triple Pounce (P2) | 0.6 | 3× 0.35 | 1.0 | |
| Thorn Volley | 0.9 tail curls, back glows | 5 thorns 0.5 | 0.8 | poison projectiles |
| Bite Combo | 0.4 | 2 bites | 0.6 | |
| Briar Burrow | 0.8 digs | emerges under the player 0.5 | 1.2 | P2 only |
| Howl (summons 2 Mosslings at High) | 0.5 | — | — | |

**L3 Glowmoth Matriarch** (Caster archetype; HP Low 2,400 / High 8,500). Breaking a wing grounds it for 8 s. P2 60%: light-burst. Enrage: wings glow brighter.
| Move | Telegraph | Active | Notes |
| --- | --- | --- | --- |
| Spore Cloud | 0.8 wing fold | cloud 4 s (sleep buildup) | stand outside |
| Dive | 0.7 hovers high + shriek | 0.5 diagonal | |
| Scale Dust Rain | 1.0 rises | 3 s columns | |
| Light Burst (P2) | 1.2 abdomen pulse | 0.3 screen-wide (flash: blind 1 s unless facing away) | |
| Antenna Beam | 0.6 antennae spark | 1.0 sweep | breaking the antennae removes it |

**L4 Tidecaller Kraken-Eel** (HP Low 3,000 / High 11,000; trigger: fishing with Monster Bait, 03). Submerges between phases. Tentacles severable.
| Move | Telegraph | Active | Notes |
| --- | --- | --- | --- |
| Water Beam | 1.0 head rears, mouth glow | 1.2 sweep | |
| Tentacle Sweep | 0.6 tentacle raises | 0.4 | |
| Submerge-Ambush | 0.8 bubbles at the target | 0.5 | |
| Tidal Surge (P2) | 1.4 roar + dam sluices open | wave 1.5 | use high ledges; env trap: close the sluice to stun it |
| Coil Grab | 0.5 | grab 2 s (mash) | |

**L5 Ironhide Behemoth** (HP High 14,000). Plates must break before the flesh hitzones open (blunt is best). P2 at 50%: plates broken → faster.
Moves: Quake Stomp (0.9 forefoot lift, ground shockwave line, Tremor), Boulder Throw (1.0 rips rock, arc), Horn Gore (0.6), Plate Roll (0.8 tucks, rolling 2 s), Rage Charge (P2). Env trap: dam crane drop.

**L6 Gearjaw Drillwyrm** (HP High 13,000). Burrows and surfaces. Its steam vents leave burn zones. Water element staggers the steam.
Moves: Drill Charge (0.8 snout spins up), Burrow Pop (0.7 rumble marker), Steam Vent Spine (1.0 spine gears turn, vents fire ×3), Tail Grind (0.5), Overheat (enrage: glows red, then fatigue "cool-down" 6 s of vulnerability).

**L7 Stormwing Gryphon** (HP High 12,000 / Master 30,000). Aerial. Wing breaks shorten flight time. P2 at 60%: lightning feathers, storm field.
Moves: Talon Dive (0.8 screech + hover), Feather Volley (0.7 wings spread, 6 bolts), Gust (0.9 backflap, pushback), Thunder Crest (1.1 crest charge, lightning column ×3), Ground Peck combo (0.4).

**L8 Runeback Golem** (HP High 15,000). It is shielded until its rune pylons (env objects) are destroyed. The rune cores are the weak points and appear in sequence.
Moves: Rune Laser Sweep (1.2 core charge), Arm Slam (0.8), Pylon Summon (1.0), Shield Phase (invuln. until 2 pylons are broken), Self-Repair (P2: cores re-form unless hit within 5 s).

**L9 Magmaw Salamander** (HP High 14,000 / Master 34,000). Molten crust: water/ice "cools" it (hardens the crust; then a blunt hit breaks it for big damage).
Moves: Lava Swim (0.8 dives into lava), Magma Spit (0.6 throat glow, 3 globs), Tail Lash (0.5), Crust Eruption (1.1 back cracks glow, radial), Lava Wave (P2, 1.5).

**L10 Prism Leviathan Ray** (HP Master 32,000). Glides between islets. Wind currents (env) let the player reach it.
Moves: Prism Beam (1.0 fins align, refracted by crystal spires: beams bounce), Glide Bomb (0.8), Stinger Strike (0.6, severable), Refraction Field (P2, 1.3, beam grid).

**L11 Aether Drake** (Elder, HP Master 45,000). Elemental shift every phase (P1 Fire → P2 Ice at 66% → P3 Thunder at 33%). Its weakness inverts each phase (GDD "elemental shifts"). Spark drop 30%.
Moves per element: breath (1.0), claw combo (0.5), wing nova (1.4), element-specific signature (Fire meteor rain / Ice spike field / Thunder chain).

**L12 Elder Destiny Wyrm** (finale; HP 60,000). 3 phases with arena changes (crater floor cracks → floating debris platforms → forge-light core). Guaranteed 3 Sparks on first clear.

### 4.2 Large monster rewards (examples; full tables in `monsters.ron`)

| Monster | Carve (3) | Break-only | Capture-only | Quest | Feeds |
| --- | --- | --- | --- | --- | --- |
| Bristleboar | Boar Hide, Bristle, Tusk | Tusk Chip (tusk break) | Boar Fat | Gold, rep | Leather Hunter set (04), GS unlock |
| Briarwolf | Briar Pelt, Thorn, Fang+ | Briar Heart 8% carve / 25% back break | Briar Heart 20% | | **Forge L2**, poison DB/Bow trees |
| Glowmoth | Glow Scale, Moth Silk, Antenna | Luminous Wing (wing break) | Spore Sac+ | | Sleep weapons, lantern upgrade |
| Kraken-Eel | Eel Hide, Tentacle, Fin | Tidecaller Pearl (head) | | | **Forge L3**, Water weapons, Riverscale+ |
| Ironhide | Iron Plate, Behemoth Horn | Ironhide Core (plates) | | | Ironhide set, Hammer tree |
| Drillwyrm | Gear Scale, Drill Shard | Gearjaw Snout (snout) | Steam Gland | | Drill-Pick (01), fire GS |
| Gryphon | Storm Feather, Talon | **Stormwing Plume** (wing break) | | | **Forge L4**, Thunder Bow |
| Golem | Rune Stone, Golem Arm | Rune Core (cores) | | | Aether weapons, Jeweler size-3 |
| Magmaw | Magma Crust, Salamander Hide | Magmaw Gem (head) | | | Magma set |
| Prism Ray | Prism Scale, Ray Fin | Prism Core | | | Aether Master rows |
| Aether Drake | Drake Horn, Elder Scale | **Elder Aether Core** (chest) | — (cannot capture) | Destiny Spark | **Forge L5**, Elder set |
| Destiny Wyrm | Destiny Scale | Heart-Forge Shard | — | 3 Sparks | Destiny set |

## 5. Small monsters (hunting grounds + overworld)

| Small | Where | Behavior | Drops | Role |
| --- | --- | --- | --- | --- |
| Puffcap | Misty Forest | waddles, puffs sleep spores when hit | Puffcap Spore | Sleep coating ingredient |
| Mossling | Misty Forest | packs of 3, flank | Moss Pelt, Fang | early leather |
| Glowbeetle (passive) | Forest, night | flees; light source | Glow Shell | Lantern Oil |
| Riverling Crab | Riverlands | sideways scuttle, shell block (front) | Crab Shell | Riverscale armor |
| Dam Otter (herbivore) | Riverlands | passive, flees | Otter Pelt, Fish | carving practice |
| Clockwork Scarab | Galleries Deep | rolls into a ball, bounces | Gear Spring, Brass Scrap | machine parts |
| Rune Sprite | Highland Ruins | blinks, zaps | Rune Dust | Aether polish |
| Ember Lizard | Caldera | lava hop, fire spit | Ember Scale, Fire sac | fire coating |
| Sky Jelly | Islets | floats, shock on touch | Jelly Membrane | Thunder coating |
| Village pests: Garden Mole, Crow (02) | Homestead | dig up crops / eat crops | Mole Claw | farm pest (field combat-lite) |

Small monsters in the side-scroll grounds use the hunter combat. On top-down zones (homestead pests, forest roamers) they use CMB-04 field combat-lite with **top-down sprite variants** (a separate small set, Q3).

## 6. ANIMATIONS (most detailed section)

Conventions: side-scroll monsters are **right-authored with `flip_x`**, horizontal strips per state, `assets/monsters/<id>/<id>_<state>.png`. Cell sizes: small 64×64 (or 96×64 for long bodies); large 384×256 (L1–L4, L9), 512×320 (L5–L8, L10), 640×384 (Elders L11–L12). The ground line sits at the cell bottom −1 px, matching the hunter sole convention. Part-break states are **drawn overlay layers** per part (intact/broken) composited at runtime as child sprites, so breaks show immediately without redrawing every frame twice (Q1). Timing is per frame.

### 6.1 Large monster standard state set (each L1–L12)

| State | Frames | Timing | Loop | Notes |
| --- | --- | --- | --- | --- |
| Idle | 6 | 0.15 | yes | breathing |
| Walk | 8 | 0.11 | yes | |
| Run / charge locomotion | 8 | 0.07 | yes | |
| Turn | 4 | 0.08 | no | readable pivot |
| Alert (notice) | 4 | 0.1 | no | head snap |
| Roar | 8 | 0.1 | no | roar-shock frames f3–f6 |
| Attack moves (per move in §4.1) | telegraph 4–8 + active 3–6 + recover 3–6 | tele 0.1–0.15, active 0.04–0.06, recover 0.08 | no | **telegraph frames must exaggerate the pose** (readable mastery) |
| Flinch (light) | 3 | 0.06 | no | silhouette flash frame f0 drawn |
| Flinch (part break) | 6 | 0.07 | no | + CV4 burst |
| Topple (knockdown) + struggle + get-up | 4 + 4 loop + 4 | 0.08 / 0.12 / 0.08 | | |
| KO stun | 4 | 0.12 | yes | stars CV5 |
| Exhaust / fatigue (drool) idle | 6 | 0.18 | yes | slower variant |
| Enraged idle | 6 | 0.1 | yes | steam puffs CV18 |
| Limp walk | 8 | 0.14 | yes | capture tell |
| Sleep (nest) | 4 | 0.3 | yes | Z's drawn as an overlay sprite 3 frames |
| Pitfall trapped | 6 | 0.1 | yes | |
| Shock trapped | 4 | 0.06 | yes | |
| Paralyzed | 2 | 0.05 | yes | |
| Poisoned (overlay bubbles) | 4 | 0.12 | yes | overlay sprite |
| Death | 12 | 0.09 | no | final frame = corpse (carve) |
| Captured (net + tranq) | 4 | 0.15 | no | |
| Area-change exit (fly/burrow/swim) | 6 | 0.08 | no | per monster type |
| Part overlays: broken versions of each breakable part | 1 per frame-set group (drawn per state cluster: locomotion, attack, down) ≈ 3 per part | — | — | e.g. Briarwolf back-thorns broken × 3 clusters |
| Severed tail (ground prop, carvable) | 1 + 3 twitch | 0.1 | | |

**Per-monster unique additions:**
- L1 Bristleboar: paw-scrape 4, skid 4, mud-roll 8, bristle shake 6.
- L2 Briarwolf: crouch-pounce 6, triple pounce 14, thorn volley 10 + thorn projectile 3 (24×8), burrow 8 / emerge 6, howl 6.
- L3 Glowmoth: hover idle 6 (replaces walk), fly 8, grounded (wing broken) idle 4, spore cloud VFX 6 (128×128), dive 8, scale rain 6 + column VFX 4, light burst 8 + screen flash UI 3, antenna beam 6 + beam 4.
- L4 Kraken-Eel: submerge 6 / emerge 6, water beam 10 + beam VFX 6, tentacle sweep 8 (per tentacle overlay), coil grab 6 + player-grabbed H-set (hunter 4 frames), tidal surge 12 + wave VFX 8.
- L5 Ironhide: plate-crack overlays ×3 plates × 3 crack stages, boulder rip 8 + boulder 4, quake stomp 8 + shockwave 6, plate roll 8.
- L6 Drillwyrm: burrow 8 / surface 8, drill spin-up 6, steam vents 6 ×3, overheat glow 4, cool-down slump 6.
- L7 Gryphon: hover 6, fly 8, dive 8, feather volley 8 + feather 3, gust 8 + wind VFX 6, thunder crest 10 + lightning column 5.
- L8 Golem: rune core open/close 4 ×3, laser charge 6 + laser 4, shield on/off 6, pylon summon 8 + pylon idle 4 / break 6, self-repair 8.
- L9 Magmaw: lava swim dive/emerge 6/6, spit 6 + glob 3, crust eruption 10, cooled-crust overlay 3 stages, lava wave 12.
- L10 Prism Ray: glide 8, prism beam 8 + refract VFX 6, stinger 6, refraction field 12.
- L11 Aether Drake: 3 elemental skins (Fire/Ice/Thunder) baked by the draw script per state = ×3 of the standard set (largest single art item), element shift transformation 12, signature moves 10 ×3.
- L12 Destiny Wyrm: 3 phase skins, arena transition cinematics 3 × 12, Destiny nova 16.

**Per large monster ≈ 160–220 frames** (×3 for Elders). Small monster standard set: idle 4, move 6, telegraph 3, attack 4, hurt 2, death 6, + unique 2–6 ≈ 30 frames each × 10 ≈ 300 frames. Top-down variants for homestead pests: idle 4, move 4, attack 3, hurt 2, death 4 (16×16 or 24×24).

### 6.2 Monster VFX and projectiles

Thorns (3), spore cloud (6), scale dust column (4), light burst (8), water beam (6), wave (8), boulder (4), shockwave (6), steam vent (6), feathers (3), lightning column (5), rune laser (4), magma glob (3), lava wave (8), prism beam/refract (6/6), element breaths ×3 (8 each), Destiny nova (16). Cells 64×64–256×128 at 0.04–0.08 s/frame.

### 6.3 UI icons

Monster portraits (64×64) for the Field Guide/quest cards ×13 large + 10 small (+ variants), monster-part icons for drops (~80 at 16×16), weakness chart glyphs (shared element icons), and a "tracks known" silhouette per monster.

## 7. Inventory items

Monster parts: stack 99, category Monster (exists), sell 10–2,000 g by rarity (rarity 1–10 matches the monster rank). Key items: Tidecaller Pearl, Stormwing Plume, Elder Aether Core, Heart-Forge Shard (stack 99 but quest-relevant). Sources in §4.2 / §5.

## 8. Hooks

Forge (every part → rows, forge-level materials); Fishing (Kraken-Eel trigger, Monster Bait uses parts); Farming (Puffcap/Sleep spores, pests); Mining (Drillwyrm, Ironhide; mine monsters in 01); Social (Captain Vell's quests, Hask's companion tips, monster lore by Mirela); Progression (difficulty curve, HR, Sparks); Day/season (nocturnal behavior: Glowmoth only at Evening; winter Frost Briarwolf; storms charge the Kraken-Eel/Gryphon); Narrative ("why creatures stir": the Destiny Wyrm awakening); UI (Field Guide fills by research/breaks).

## 9. Energy / time / risk

Covered in 05. Monster difficulty curve: Village (L0–L1) → Forest/Mine (L2–L4) → Large territorial (L5–L10) → Elder (L11–L12). Target kill times (solo, appropriate gear): Low 6–9 min, High 10–14, Master 14–20, Elder 18–25.

## 10. Progression / gating

| Story act | Monsters available |
| --- | --- |
| Prologue | King Slime (exists), Bristleboar |
| Act 1 (Forge L1) | Briarwolf, Glowmoth (Low) |
| Act 1 end | Kraken-Eel (Low) via fishing trigger |
| Act 2 (L3, High rank) | Ironhide, Drillwyrm, Gryphon, Golem; High versions of L2–L4 |
| Act 2 end | Magmaw |
| Act 3 (L4–L5, Master) | Prism Ray, Aether Drake |
| Finale | Destiny Wyrm |
| Post-game | Tempered variants, Destiny Vault elites, Legendary Echo fish (03) |

## 11. UI

Field Guide entries (13): portrait, habitat, weaknesses (revealed after hunting 1× or a Mirela research quest), breakable parts (revealed per part when broken: this is the GDD's "fills in as you research or break parts"), drops by source (revealed when obtained), size records, kill/capture counts, tempered status. Quest cards show the monster portrait. The hunt HUD shows a monster name plate and an "enraged"/"limping" icon (drawn).

## 12. Audio

Per large monster: idle breath, footsteps (by weight class), roar (signature), telegraph cues per move (**required**, distinct, the audio half of readable tells), attack impacts, flinch cry, part-break crack (material-specific), topple thud, KO groan, enrage roar, limp whimper, sleep snore, trapped struggle, death cry. Per small monster: 4–5 cues. Elders: unique combat themes. The Destiny Wyrm has a 3-phase theme.

## 13. Save data

`field_guide: HashMap<MonsterId, {seen, hunted, captured, parts_broken: bitmask, weaknesses_known, drops_seen: set, max_size, min_size, tempered_kills}>`. No world monster state persists across days (hunts are instanced). Quest progress is in 05.

## 14. Balance levers

HP per rank, damage, move frequencies/weights per phase, telegraph durations (the assist scales them ×1.2), part HP, hitzone values, status thresholds, enrage triggers/durations, fatigue timing, capture thresholds, reward rates, tempered multipliers, spawn schedules.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Fully hand-drawn frames vs modular rigs baked to PNG? | **Modular part rigs** (body/head/limbs/tail as layers) posed in the draw script and **baked to PNG frames**. They count as in-house frames and make broken-part overlays and Elder skins tractable. |
| Q2 | Ship 8 or 12 large monsters at launch? | 8 at launch (L1–L8), with L9–L12 in a content update. That meets the GDD's 8–12. |
| Q3 | Small monsters in top-down zones? | Only homestead pests + mine monsters (01). Forest roamers stay in the side-scroll grounds. |
| Q4 | Monster schedules (seasonal) | Each monster has a favored season with +spawn and a "rare" season with none: GDD "seasons affect creature behavior". |
| Q5 | Monster HP display | Hidden by default (05 Q5). |
| Q6 | Reuse King Slime as High rank? | No new art (frozen). It stays a Low-rank tutorial monster. |

## 16. Ticket breakdown

Framework:
- **MON-01 [GAME]** `MonsterDef` data model + `monsters.ron` loader + brain state machine skeleton (no art). deps: none.
- **MON-02 [GAME]** Layered monster animator (base strip + part overlay children + per-state playback). deps: MON-01.
- **MON-03 [ART]** Modular rig draw tooling `tools/draw_monster_rig.py` (bake layered poses to strips) + a test monster. deps: none.
- **MON-04 [GAME]** Telegraph → attack → recover move execution with hitboxes from data. deps: MON-01, CMB-31.
- **MON-05 [GAME]** Aggro/roam/area-change/nest/limp/sleep behaviors. deps: MON-01, CMB-19, CMB-26.

Bristleboar (Charger archetype):
- **MON-06 [ART]** Bristleboar standard set + unique (paw, skid, mud-roll, bristle shake) + tusk/bristle broken overlays + portrait + part icons. deps: MON-03.
- **MON-07 [GAME]** Bristleboar behavior (charge/toss/roll/shake/roar), phases, enrage, parts, rewards. deps: MON-06, MON-04, CMB-29, CMB-37.
- **MON-08 [AUD]** Bristleboar sound set. deps: MON-07.

Glowmoth Matriarch (Caster archetype):
- **MON-09 [ART]** Glowmoth set + VFX (spores, dust column, light burst, beam) + wing/antenna overlays. deps: MON-03.
- **MON-10 [GAME]** Glowmoth behavior incl. flight + grounding on wing break + sleep spores. deps: MON-09, MON-04, CMB-32.
- **MON-11 [AUD]** Glowmoth sounds. deps: MON-10.

Remaining large monsters (each ART + GAME + AUD):
- **MON-12/13/14** Thornback Briarwolf (+ tail sever, burrow). GAME deps CMB-29 sever.
- **MON-15/16/17** Tidecaller Kraken-Eel (+ submerge, sluice env trap; fishing trigger FSH-34).
- **MON-18/19/20** Ironhide Behemoth (+ plate crack stages).
- **MON-21/22/23** Gearjaw Drillwyrm (+ burrow, steam vents, overheat).
- **MON-24/25/26** Stormwing Gryphon (+ aerial states).
- **MON-27/28/29** Runeback Golem (+ pylons, shields, cores).
- **MON-30/31/32** Magmaw Salamander (+ crust cooling).
- **MON-33/34/35** Prism Leviathan Ray (+ refraction).
- **MON-36/37/38** Aether Drake (3 elemental skins + shift).
- **MON-39/40/41** Elder Destiny Wyrm (3 phases + arena transitions).

Small monsters:
- **MON-42 [ART]** Forest smalls: Puffcap, Mossling, Glowbeetle. **MON-43 [GAME]** their behaviors + drops.
- **MON-44 [ART]** River smalls: Riverling Crab, Dam Otter. **MON-45 [GAME]**.
- **MON-46 [ART]** Galleries/Ruins smalls: Clockwork Scarab, Rune Sprite. **MON-47 [GAME]**.
- **MON-48 [ART]** Caldera/Islets smalls: Ember Lizard, Sky Jelly. **MON-49 [GAME]**.
- **MON-50 [ART]** Homestead pests top-down (Garden Mole) . **MON-51 [GAME]** mole pest on the farm (field combat-lite). deps: CMB-04.

Variants and data:
- **MON-52 [ART]** Seasonal/weather variant skins (Frost Briarwolf, Storm Kraken-Eel, Bloom Glowmoth). **MON-53 [GAME]** variant spawn rules. deps: CYC-02/05.
- **MON-54 [GAME]** Tempered variants (stat multipliers, extra move, Spark chance). deps: MON-07, CMB-37, PRG-16.
- **MON-55 [DATA]** Reward tables for all monsters + forge material links. deps: FRG-36.
- **MON-56 [UI]** Field Guide monster pages (data from MON-01 + save). deps: UI-12.
- **MON-57 [SAVE]** Field Guide persistence. deps: MON-56.
- **MON-58 [DATA]** Monster balance pass (HP, telegraph times vs kill-time targets). deps: MON-25.

**Ticket count: 58.**

**Build first (top 3):** MON-01 + MON-02 (data model + layered animator), MON-03 (rig bake tooling, which unblocks every monster art ticket), MON-06 → MON-07 (Bristleboar: the first charger archetype and first new hunt).
