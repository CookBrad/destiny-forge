# 12 — Narrative & Story Events

Authority: GDD §7 ("light narrative focused on restoring the Destiny Forge, discovering why magical creatures are stirring, and the player's personal destiny. Villagers include steam engineers, hedge mages, retired hunters, and magitech smiths whose stories intertwine with the rising threat. Tone: hopeful and cozy with steampunk wonder, moments of magical spectacle, high-stakes hunts. Not grimdark"), §6 (inherit a rundown homestead + dormant forge; valley threatened), §3 (sleep progresses story events), §4.6 (Sparks from story).

Build priority: **12 of 16**.

## 0. Standing rules

Story cutscenes use drawn character/NPC/monster frames. A text box over a static scene is not enough for key beats (the 10 "spectacle" beats below need bespoke frames). No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

`WorldProgress { boss_defeated_floor_1 }` is the only story state. There is no intro, cutscene system, or dialogue (08 builds dialogue). Title screen + profiles exist (`ui/title_profiles.rs`).

## 2. Story outline (acts → beats)

| Act | Beat | Where | Trigger | Outcome / unlock | Spark |
| --- | --- | --- | --- | --- | --- |
| Prologue | P1 "The Letter": the player arrives on a steam-rail cart with the deed + forge keys from Grandmother Edda (late forge-keeper) | Homestead | new game | homestead, tools, Mayor intro | — |
| | P2 "Cold Forge": the forge sputters with a vision of the Destiny Wyrm's eye | Forge yard | first forge use | forge window, King Slime hint | — |
| | P3 "Slime Season": King Slime threatens the crops (existing dungeon) | Dungeon F1 | day 2–3 | Floor 1 quest; Hask notices you | — |
| | P4 "Tusks at the Gate": Bristleboar attack on the village | Outskirts | after P3 | GS unlock, Lodge, HR 1 | **1** |
| Act 1 "Copper Heart" | A1 Brunwald's mine and the Copper Heart Coil (F10) | Mine | Rep 50 | Forge **L1** | — |
| | A2 The Briar Blight: thorn growths spread from the forest; Mirela links it to stirring magic | Forest | L1 | Briarwolf hunt; alchemy | — |
| | A3 Glowmoth Night: the village lamps fail; Glowmoth swarm | Village/Forest | Summer | Glowmoth hunt; lantern tech | — |
| | A4 "The Dam Speaks": the Kraken-Eel at the Clockwork Dam; Kit's airship mast restored | Riverlands | Brass Lung (L2) | Riverlands open; Kit; Forge **L2** done | **1** |
| Act 2 "Aether Core" | B1 The Pass: repair the mountain road (Tobin + Brunwald) | Ruins pass | Rep 350 | Highland Ruins | — |
| | B2 "Rune Archive": Quillon deciphers that the old magitech city sealed an Elder | Ruins | arrival | Golem/Gryphon hunts | — |
| | B3 Rook appears: the drifter knows Edda | Inn | B2 | Rook bonds; Binding lore | — |
| | B4 Aether Core: forge the core from Tidecaller Pearl + Brassjaw Gear-Scale | Forge | parts | Forge **L3**, High rank | **1** |
| | B5 "Caldera Wakes": Magmaw erupts; Edda's journal found | Caldera | HR 16 | Forge L4 path | **1** |
| Act 3 "Destiny Crown" | C1 Airship to the Islets | Islets | L4 + Kit | Islets | — |
| | C2 Prism Ray / the sky cracks | Islets | arrival | Prism Ray hunt | — |
| | C3 The Aether Drake: an Elder test, not a villain | Islets | Act 3 HR 21 | Elder Aether Core | **1** |
| | C4 Destiny Crown: the forge is fully restored, and Rook reveals he was Edda's apprentice who fled | Forge | L5 | Master rank, Awaken | — |
| Finale | F1 "Forge Your Destiny": the Destiny Wyrm (the valley's sleeping magic) rises; not evil, just a wild force; the player binds a pact rather than destroying it (hopeful tone) | Destiny Crater | Master gear | credits; Destiny Vault; endgame | **1 + 3 (fight)** |
| Post-game | Epilogue letters, tempered threats, the Wyrm's echoes (Destiny Trials) | — | — | endgame (09) | renewable (rare) |

Main-story Sparks = 6 milestones + finale fight 3 (= 9). The rest of the ≈ 18 comes from other sources (09).

**Theme**: "destiny" is something you forge and do not inherit. Edda's legacy, the village bonds, and the player's choices of gear and pact express it. Optional light choice at the finale (pact wording: Guardian / Kindred / Free) changes the epilogue text and a cosmetic forge glow color.

## 3. Entities / systems

`StoryFlags` (generalizes `WorldProgress`; string/enum flags), `StoryBeat {id, requirements, trigger: OnEnterZone|OnSleep|OnInteract|OnQuestComplete|OnDate, script}`, `Cutscene {steps: [Move, Animate(set), Dialogue, Camera(pan/zoom/shake), Fade, Spawn, Wait, Sfx, Music]}`, `Journal` (story log + Edda's journal pages ×12 collectible), `Letter` (mail, 08).

## 4. Tools / equipment

Story key items: Deed, Forge Keys, Edda's Journal pages ×12, Copper Heart Coil, Brass Lung, Aether Core, Mythril Bellows, Destiny Crown (04), Rook's Compass (Act 3 islet navigation).

## 5. ANIMATIONS (most detailed section)

Key "spectacle" beats get bespoke frames. All other beats reuse SOC NPC sets + emotes. Conventions follow the zone: top-down dwarf/NPCs (16×28, 48×60 canvas), side-scroll monster/hunter cells for hunt-ground beats.

| # | Beat | Sets | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| N1 | P1 arrival | steam-rail cart arrive/stop/depart; dwarf hop-off 5; look-around 4 | 8 + 5 + 4 | 128×64 / 24×32 | 0.1 | opening shot |
| N2 | P2 forge vision | forge sputter 6, Wyrm-eye vision overlay 10 (full-screen 320×180), dwarf stagger 4 | 20 | various | 0.08 | spectacle 1 |
| N3 | P4 Bristleboar village charge | boar crash through a fence 8 (side-scroll), villagers flee (reuse walk), Hask arrives 6 | 14 | 384×256 / 343×160 | 0.08 | spectacle 2 |
| N4 | A1 Copper Heart install | uses FRG G15 + GE10 L0→L1 | — | — | — | owned by FRG |
| N5 | A2 thorn blight spread | thorn growth tiles 6 (32×32), Mirela cast 6 | 12 | | 0.1 | |
| N6 | A3 lamps fail + moth swarm | lamp die 3, swarm overlay 8 (full-screen), villagers huddle 4 | 15 | | 0.1 | spectacle 3 |
| N7 | A4 dam awakening | dam shake 6, Kraken-Eel emerge (MON set) + airship mast restore 8 | 14 | 192×192 | 0.08 | spectacle 4 |
| N8 | B1 pass repair montage | Tobin/Brunwald work loops (SOC) + rocks clear 6 | 6 | 64×64 | 0.1 | |
| N9 | B2 rune archive | rune wall decipher glow 8, hologram of the old city 10 (192×128) | 18 | | 0.1 | spectacle 5 |
| N10 | B3 Rook intro | Rook cloak swirl 6, hat tip 4 | 10 | 24×32 | 0.1 | |
| N11 | B5 eruption | caldera eruption 12 (320×180), ash cloud overlay 6 | 18 | | 0.08 | spectacle 6 |
| N12 | C1 airship launch | airship lift-off 10 (192×128), deck wind (WLD W5) | 10 | | 0.1 | spectacle 7 |
| N13 | C3 Drake trial greeting | Drake landing 10 (640×384, MON skin), bow/acknowledge 6 | 16 | | 0.09 | spectacle 8 |
| N14 | C4 Destiny Crown | forge full-restoration 16 (192×160), Rook reveal emote set 6 | 22 | | 0.08 | spectacle 9 |
| N15 | F1 pact | Wyrm rise 16 (640×384), pact binding beam 12 (full-screen), dwarf raises hand 6, 3 glow variants by choice | 34 + 3 | | 0.08 | spectacle 10 |
| N16 | Credits | village montage stills ×12 (320×180, drawn) + parade loop 8 | 20 | | 0.15 | |
| N17 | Edda's ghost-memory vignettes (journal pages) | 12 sepia vignettes ×3 frames | 36 | 160×96 | 0.3 | collectible |

Shared VFX: cutscene letterbox bars (UI), fade-to-white flash, camera shake (code, OK), vision vignette overlay.

## 6. Inventory items

Key items above (stack 1, not sellable, category Key); Journal pages (collection, not inventory).

## 7. Processing

Beat requirements → trigger → cutscene → flags set → unlocks (zones, recipes, NPCs) → optional letter next morning.

## 8. Hooks

Gates the forge levels (04), zones (10), monsters (06), companions/villagers (08), sparks (09), the Destiny Vault (01), the Greenhouse (02), and the calendar (11). Heart events (08) are side stories that intertwine with the main beats (Hask/A2, Mirela/A3, Wren/A4, Quillon/B2, Rook/B3–C4).

## 9. Energy / time / risk

Cutscenes pause time. Story hunts carry normal risk but can be retried indefinitely (no lockout).

## 10. Progression / gating

See §2 requirements. Soft pacing: Act 1 within Spring–Summer Y1, Act 2 Fall–Winter Y1, Act 3 + finale Spring–Summer Y2. Never time-locked; there are no fail states.

## 11. UI

Cutscene letterbox + skip (hold Esc 1 s, after the first view); Journal (main quest log tab: current objective + hint); story toast "New area unlocked" (drawn banner); finale choice dialog; credits roll.

## 12. Audio

Main theme (title + key beats, leitmotif: forge hammer rhythm), Edda's theme (music box), Rook's theme, Wyrm theme, per-spectacle stingers, credits song (instrumental).

## 13. Save data

`story_flags: BTreeSet<StoryFlag>` (migrates `boss_defeated_floor_1`), `journal_pages: bitmask`, `cutscenes_seen` (for skip), `finale_choice: Option<Pact>`, `current_objective`.

## 14. Balance levers

Beat requirements (rep/HR/forge), pacing targets, spark counts per beat, skip rules.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Is the Destiny Wyrm a villain? | No. It is a force of the valley's magic. The finale is a pact (hopeful tone, GDD "not grimdark"). |
| Q2 | Player choice? | One light finale choice (epilogue text + cosmetic glow) only. |
| Q3 | Cutscene tech | Data-driven step list (RON) run by a `Cutscene` system that reuses the NPC/actor animators. No timeline editor. |
| Q4 | How is King Slime (existing) tied in? | It becomes P3 "Slime Season", the first hunt, so the existing content keeps its role. |
| Q5 | Voice acting | None. Text + voice blips (08). |

## 16. Ticket breakdown

- **NAR-01 [GAME]** `StoryFlags` (migrate `WorldProgress`) + beat requirements/trigger data (`story.ron`). deps: none.
- **NAR-02 [GAME]** Cutscene system (steps: move, animate, dialogue, camera, fade, spawn, wait, sfx, music) + skip. deps: SOC-04 (dialogue UI), NAR-01.
- **NAR-03 [ART]** P1 arrival set N1. deps: none.
- **NAR-04 [GAME]** Prologue P1 new-game intro (replaces spawning straight onto the homestead). deps: NAR-02, NAR-03.
- **NAR-05 [ART]** P2 forge vision N2. deps: none.
- **NAR-06 [GAME]** P2 beat on first forge use. deps: NAR-05, NAR-02.
- **NAR-07 [GAME]** P3 "Slime Season" wrap of the existing Floor 1 as a story quest (no dungeon art changes). deps: CMB-15, NAR-01.
- **NAR-08 [ART]** P4 Bristleboar village charge N3. deps: MON-06.
- **NAR-09 [GAME]** P4 beat + GS unlock + first Spark. deps: NAR-08, MON-07, PRG-16.
- **NAR-10 [GAME]** Act 1 beats A1–A4 wiring (flags, unlocks). deps: NAR-02, MIN-21, FRG-07, MON-13, MON-10, MON-16.
- **NAR-11 [ART]** Act 1 spectacle sets N5–N7. deps: none.
- **NAR-12 [GAME]** Act 2 beats B1–B5 wiring. deps: NAR-10, WLD-17, WLD-19, FRG-31.
- **NAR-13 [ART]** Act 2 spectacle sets N8–N11. deps: none.
- **NAR-14 [GAME]** Act 3 beats C1–C4 wiring. deps: NAR-12, WLD-21, MON-37.
- **NAR-15 [ART]** Act 3 spectacle sets N12–N14. deps: none.
- **NAR-16 [GAME]** Finale F1 + pact choice + epilogue + Destiny Vault unlock. deps: NAR-14, MON-40, MIN-82.
- **NAR-17 [ART]** Finale N15 + credits N16. deps: MON-39.
- **NAR-18 [ART]** Edda journal vignettes N17. deps: none.
- **NAR-19 [GAME]** Journal pages collectible + Journal UI (objective + hints). deps: NAR-18, NAR-01.
- **NAR-20 [AUD]** Main/Edda/Rook/Wyrm themes + spectacle stingers. deps: NAR-04.
- **NAR-21 [SAVE]** Persist story flags, journal, cutscenes seen, choice (migration from WorldProgress). deps: NAR-01.
- **NAR-22 [DATA]** Story dialogue scripts for all beats. deps: NAR-02, SOC-05.

**Ticket count: 22.**

**Build first (top 3):** NAR-01 (story flags + migration), NAR-02 (cutscene system), NAR-07 (wrap the existing Floor 1 as the "Slime Season" story quest).
