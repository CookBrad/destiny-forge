# 08 — Social & Villagers (schedules, gifts, hearts, quests, festivals, companions)

Standards: there are about 12 to 16 unique villagers with schedules, likes and dislikes, and personal stories, many with hunter or smithing backgrounds, such as steam engineers, hedge mages, retired hunters and magitech smiths. Gifts, dialogue and quests raise heart levels. High bonds unlock unique forge recipes, weapon skill trees, temporary hunt companions, farm helpers and story events. Festivals and community events give rare materials, temporary buffs and relationship chances. The village is a living community. Basic social is must-have; the full relationship system and festivals are should-have.

Build priority: **8 of 16**. Nothing exists yet. There is no village zone, NPC, or dialogue.

## 0. Standing rules

Every player social action gets dwarf frames: talk, give gift, receive, accept a quest, festival activities. Every NPC has drawn walk/idle/talk/emote/job frames. Portrait expressions are drawn sprites. A dialogue box alone does not satisfy the rule. No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

Nothing social. Reuse: `interaction_prompt.rs` (KeyE), top-down movement, day phases, inventory, PR #96 popup pattern (`eat_feedback.rs`) for toast-style UI.

## 2. Where it happens

**Hearthspire village** (new zone, 10): Town Square (bell tower, quest board), Hunter's Lodge, Ottilie's Smithy, General Store, Inn & Kitchen ("The Copper Kettle"), Carpenter's Workshop, Ranch, Harbor/Canal dock, Museum & Archive, Clinic/Apothecary (Mirela), Mayor's House, Airship Mast (courier), residences. Villagers also visit the homestead (deliveries, heart events), the forest, the mine entrance (Brunwald), and hunting grounds as companions.

## 3. Cast (14 villagers)

| # | Villager | Role / background | Home / work | Loves (examples) | Hates | Unlocks (hearts) |
| --- | --- | --- | --- | --- | --- | --- |
| V1 | **Mayor Aldous Penhallow** | retired steam engineer, town elder | Mayor's House / Town Hall | Brass Bar, Pumpkin, Old Cog | Slime Gel | 2♥ community bundles board; 6♥ Greenhouse plans; 8♥ "Penhallow Valve" decoration |
| V2 | **Ottilie Brassbeard** | magitech smith (forge tutor) | Smithy | Iron/Steel Bar, Ruby, Firepepper | Seaweed | 3♥ geode discount; **6♥ Brasswork weapon tree**; **8♥ "Folded Steel" forge technique** |
| V3 | **Hask Thornwood** | retired hunter, Great Sword master | Hunter's Lodge porch | Boar Hide, Smoked Fish, Tusk | Flowers | GS unlock quest (0♥ story); **4♥ hunt companion**; 6♥ Hunter's tree; 10♥ "Old Hunter's Oath" set skill |
| V4 | **Mirela Fen** | hedge mage, healer, alchemist | Clinic/Apothecary | Moonpetal, Glowcap, Amethyst | Coal | 3♥ Alchemy Still recipe; 6♥ Moonpetal seeds; **8♥ "Hedge Infusion" (2 infusions)**; companion (healer) 5♥ |
| V5 | **Brunwald Ironsoot** | mine foreman | Mine entrance hut | Coal, Beer-free Cordial, Topaz | Fish | elevator repair quest; cogs questline (01); 6♥ Miner's Drill-weapon tree |
| V6 | **Wren Calloway** | harbormaster, ex-ranger archer | Harbor | Fish (any Gold+), Driftwood, Sapphire | Sulfur | Old Dock quest (day 2); 3♥ Bow unlock; **4♥ companion (bow, status)**; legendary fish hints |
| V7 | **Marla Hayes** | rancher | Ranch | Corn, Wool, Apple | Bones | animals shop; 3♥ Lv2 buildings; **8♥ farm helper (pets/feeds animals daily)** |
| V8 | **Captain Vell Arden** | Hunter's Lodge master | Lodge | Monster parts rarity ≥ 4, Steel Bar | Turnip | quest board; HR promotions; 6♥ Lodge tree; 8♥ tempered intel |
| V9 | **Tobin Gearwright** | carpenter & steam engineer | Workshop | Hardwood, Gear Spring, Brass Egg | Clay | buildings; 4♥ cheaper builds; **8♥ farm helper (clockwork bot waters 20 tiles)** |
| V10 | **Juniper Hollis** | general store owner | General Store | Strawberry, Honey, Jam | Rotten anything | seeds/supplies; 5♥ seasonal rare seed stock |
| V11 | **Sela Marrow** | innkeeper & cook | The Copper Kettle | Cheese, Melon, Large Egg | Ore | Hearth Meals; **recipes by mail at 2/4/6/8♥** |
| V12 | **Professor Quillon Ashby** | museum curator, monster scholar | Museum & Archive | Fossils, Monster parts (new kinds), Prismatic | Weeds | Field Guide research quests; 6♥ reveal weaknesses; 8♥ Elder lore |
| V13 | **Kit Sparrow** | airship courier & tinkerer | Airship Mast | Gear Spring, Skybloom, Stormglass | Mud | 3♥ fast travel by airship; **5♥ companion (scout: tracking +50%)**; 8♥ islet route (Act 3 shortcut) |
| V14 | **Rook** | mysterious drifter (Destiny lore) | wanders; Inn at night | Destiny Nectar, Starmetal, Moonplum | gifts on most days (aloof) | story only; 6♥ Destiny Binding tutorial; 10♥ final heart event (Destiny reveal) |

Bonds: friendship-only at launch (Q1). Hearts 0–10. Rook is capped at 6♥ until Act 3.

## 4. Systems

### 4.1 Schedules
- Data `schedules.ron`: per villager, per day-of-week (7-day week), per phase (Morning/Afternoon/Evening; + Night indoors), a location + activity anchor. Overrides by season, weather (rain → indoors), festival, and story flags. Pathing: tile waypoints along village paths (no full navmesh; Q3).
- Activities have animations (§5.2 job loops).

### 4.2 Dialogue
- `dialogue/<villager>.ron`: lines keyed by hearts tier (0/2/4/6/8/10), season, weather, location, recent events (hunted X, caught legendary, festival tomorrow). First daily talk +20 pts. Typed text with a portrait + expression tag.

### 4.3 Gifts
- 2 gifts/week per villager + birthday (×8). Tastes: Loved +80, Liked +45, Neutral +20, Disliked −20, Hated −40. Quality multiplier: Silver ×1.1, Gold ×1.25, Destiny ×1.5. Universal loves: Prismatic Shard, Destiny Nectar. Universal hates: trash.
- Reaction: portrait expression + NPC emote anim.

### 4.4 Hearts
- 250 points per heart, decay −2/day if not talked to (stops at heart thresholds 2/4/6/8). Heart events (cutscenes) at 2/4/6/8/10 hearts, triggered by entering a location at a time.

### 4.5 Quests
- **Request board** (Town Square): daily 1–3 requests ("Bring 5 Copper Bar", "Catch a Rain Eel", "Slay 3 Mosslings"); reward gold + hearts + reputation.
- **Personal questlines**: 3–4 per villager at 2/4/6/8♥, multi-step, using all systems (e.g. Hask: "The Last Hunt": track Bristleboar, carve its tusk, forge the "Hask's Memento" GS).
- **Hunt requests** from the Lodge (05) count for Vell/Hask hearts.
- **Community bundles** (Mayor; 09): restore town amenities (Greenhouse, Airship Mast, Canal locks, Museum wing).

### 4.6 Festivals (8; 2 per season; data `festivals.ron`)

| Festival | Day | Activity | Rewards |
| --- | --- | --- | --- |
| Gearbloom Fair | Spring 13 | flower-gear dance + seed stalls | rare seeds (Moonpetal starter), hearts |
| Hunter's Oath | Spring 24 | sparring exhibition with Hask (training dummy hits) | HR points, "Oath" temporary buff 3 days (+5% atk) |
| Steam Regatta | Summer 11 | canal boat race minigame | Brass Bar ×10, Regatta trophy decoration |
| Gearfin Derby | Summer 20 | timed fishing contest (03) | Tackle, legendary hint |
| Harvest Grading | Fall 16 | submit the best crop/animal product (quality-judged) | gold, Deluxe fertilizer, rep |
| Hunter's Moon | Fall 27 | night hunt tournament (timed small-monster hunt) | rare parts, Hunter's Moon decoration |
| Festival of Lanterns | Winter 8 | lantern release + Rook event | **Destiny Spark ×1 per year** (04), Moonplum sapling |
| Forgefire Night | Winter 25 | gift exchange + forge demo with Ottilie | random villager gift, forge discount |

### 4.7 Companions (temporary hunt companions)
- 1 companion per hunt (unless Wide-Range…). Hired at the Lodge after the hearts gate; costs a "Hearth Meal" ticket (no gold).
- Hask (GS, draws aggro, flinch), Wren (Bow, status coatings), Mirela (support: heals 30 HP per 45 s, cleanses), Kit (scout: tracking, paintball, flash pods).
- AI: follow → engage target → evade telegraphs (reads the same telegraph data) → support actions. Faints: the companion is downed for 30 s (no faint count).

### 4.8 Farm helpers (8♥)
Marla: animal care daily. Tobin's Clockwork Helper Bot: waters 20 tiles daily. Juniper: sells your shipping bin at +5%. Each helper is visible on the homestead with its own animation (§5.2).

## 5. ANIMATIONS (most detailed section)

Conventions: village/homestead NPCs are top-down like the dwarf: **body 16×28 (adults) / 16×24 (Kit)**, action canvas 48×60 when holding tools, **side-facing right-authored with `flip_x`** + **front facing** (talk, idle) + **back facing** (walk away) because schedules move NPCs up/down paths (Q2). Hunt companions use **side-scroll hunter cells 343×160** (05 conventions). Portraits are 64×64 busts.

### 5.1 Player (dwarf) social sets

| # | Set | Frames | Canvas | Facing | Timing | Trigger | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S1 | Talk idle (listening nod) | 2 | 16×28 | R | 0.4 loop | while the dialogue is open | |
| S2 | Give gift (hand over) | 4 | 32×32 | R | 0.1 ×4 | f3 → gift applied | gift icon at the hand anchor |
| S3 | Receive item (from NPC) | 3 | 32×32 | R | 0.1 ×3 | f2 | then the hold-up FRM F12 |
| S4 | Accept quest (salute/note) | 3 | 24×28 | R | 0.12 ×3 | f2 | |
| S5 | Read board/letter | 3 | 24×28 | back | 0.15 ×3 | — | back-facing; the Q2 exception |
| S6 | Festival dance | 8 | 24×32 | front | 0.12 loop | — | Gearbloom Fair |
| S7 | Lantern release | 5 | 24×40 | front | 0.15 ×5 | — | Festival of Lanterns |
| S8 | Cheer/clap | 4 | 24×28 | front | 0.1 loop | festival results | |
| S9 | Regatta row (boat) | 4 | 48×32 | R | 0.1 loop | — | with the boat sprite |
| S10 | Sparring swing (village dummy, top-down) | uses CMB-05 field strike | | | | | |
| S11 | Hug/pat (heart events) | 4 | 32×32 | R | 0.15 ×4 | cutscene | friendship-only tone |
| S12 | Surprised / laugh / sad emotes (cutscenes) | 2 each ×3 | 16×28 | front | 0.2 | cutscene | |

### 5.2 Per-villager NPC sets (each V1–V14)

| Set | Frames | Facing | Timing | Notes |
| --- | --- | --- | --- | --- |
| Idle | 4 | front + side | 0.2 loop | |
| Walk | 4 side, 4 front, 4 back | 3 facings | 0.12 loop | |
| Talk | 2 (mouth/gesture) | front + side | 0.15 loop | |
| Emotes: happy, love (gift loved), annoyed, sad, surprised, thinking | 2–3 each = ~15 | front | 0.15 | + emote bubble VFX (§5.4) |
| Receive gift (take + react) | 3 | side | 0.12 | |
| Job loop (unique per villager) | 4–6 | side | 0.12–0.2 | Ottilie hammering, Hask whittling, Mirela grinding herbs, Brunwald pickaxe, Wren fishing, Marla brushing, Vell posting notices, Tobin sawing, Juniper sweeping, Sela stirring, Quillon reading, Kit wrench, Rook leaning, Mayor winding the clock |
| Sit (bench/inn) | 2 | side/front | 0.5 | |
| Sleep (indoors) | 2 | front | 0.6 | |
| Festival pose/dance | 6–8 | front | 0.12 | shared choreography, per-NPC body |
| Heart event specials | 2–4 per event × 5 events | — | — | authored per event (≈ 15 per villager) |

≈ 75–90 frames per villager × 14 ≈ **1,150 small frames**.

### 5.3 Companion hunt sheets (Hask, Wren, Mirela, Kit; side-scroll 343×160)

| Set | Frames | Timing | Notes |
| --- | --- | --- | --- |
| Idle / run / jump / fall | 4 / 8 / 1 / 1 | 0.18 / 0.08 | |
| Evade | 6 | 0.05 | |
| Attack combo (weapon) | 3 moves × 5–6 | 0.05 | Hask GS (reuses the GS timing table), Wren Bow, Mirela staff-light, Kit wrench + gadget |
| Special | 8 | 0.06 | Hask Tackle-Slash, Wren Paralysis Volley, Mirela Heal Circle (VFX), Kit Flash/Paintball throw |
| Hurt / knockdown / get-up | 3 / 4 / 4 | 0.06–0.08 | |
| Downed (30 s) + revive | 2 loop + 4 | 0.3 / 0.1 | |
| Cheer (victory) | 4 | 0.15 | |
| Callout (point at monster) | 3 | 0.1 | |

≈ 70 frames × 4 companions ≈ 280 large frames.

### 5.4 VFX and UI

Emote bubbles (heart, music note, anger vein, "!", "?", sweat, Zzz, ellipsis) 3 frames each @0.12, 16×16. Heart-gain burst 5 frames. Gift wrap sparkle 4. Festival fireworks 8 (64×64) + lantern float 6 + confetti 6. Portraits 64×64 × 14 villagers × 6 expressions (neutral, happy, love, sad, angry, surprised) = 84. Dialogue box frame (9-slice brass), name plate, heart meter icons (empty/half/full), gift-taste icons, quest board card frames, calendar icons (birthday, festival).

## 6. Inventory items

Gifts are existing items. New: Letters (mail, key item), Quest items (Hask's Old Whetstone, Brunwald's Crew Cogs ×3, Mirela's Moon Herb, etc.; stack 1), Festival tokens (Regatta trophy, Hunter's Moon badge → decorations), Recipe cards (consumed to learn).

## 7. Processing / flows

Talk → (dialogue) → gift (S2 + NPC receive anim + emote) → points. Quest: board/letter → accept (S4) → objectives tracked → turn in (S3 + rewards). Heart threshold reached → next day a letter → heart event on location entry. Companion: Lodge → choose → in hunt.

## 8. Hooks

Forge (weapon trees, techniques, Binding tutorial), Hunting (companions, Lodge quests, Hunter's Oath/Moon), Farming (rare seeds, helpers, Harvest Grading), Fishing (Wren, Derby), Mining (Brunwald), Animals (Marla), Progression (reputation from quests/bundles/festivals; Sparks at Lanterns), Narrative (heart events and story events overlap; Rook = main plot), UI (calendar, relationship page), Save (§12).

## 9. Energy / time / risk

Talking/gifting cost no energy. Festivals take a full phase (time skip on exit). Quests have deadlines (requests: 2 days). Risk: none, except hearts decay if you neglect villagers.

## 10. Progression / gating

Village unlock day 1 (Mayor intro). Villager introductions as zones open (Brunwald at the mine, Kit at Act 1 end when the airship mast is restored, Rook at Act 2). Companions at the listed hearts. Heart caps: 8♥ for most until Act 2, 10♥ after (forces story pacing).

## 11. UI

Dialogue box with portrait/expression + typewriter + choice list; gift prompt (hold the item, KeyE on NPC); Relationship page (hearts, gifts given this week, known likes, birthday); Calendar (festivals, birthdays); Quest log (main, personal, requests, hunts); Request board UI; Mail/letters; Companion select at the Lodge; Festival minigame UIs (Regatta, Derby results, Grading).

## 12. Audio

Per villager voice "blip" timbre (gibberish syllables, pitch per villager) for typing; emote stingers; gift loved/hated jingles; heart up chime; quest accept/complete; letter arrive; festival music themes ×8; village day/night music; ambient crowd for festivals; companion barks (short callouts: "Monster's limping!", "Healing!").

## 13. Save data

`villagers: HashMap<VillagerId, {points, hearts_events_seen, gifts_this_week, talked_today, birthday_gift_given, quest_states}>`, `requests_active`, `quest_log`, `mail_queue`, `festival_history`, `companion_unlocked`, `helpers_active`, `known_tastes: HashMap<VillagerId, set<MaterialId>>`.

## 14. Balance levers

Points per action, decay, gift multipliers, weekly limits, heart caps by act, quest rewards, companion damage/heal values, helper coverage, festival rewards, request frequency.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Romance/marriage? | Not in scope. **Friendship bonds only** at launch, with 10♥ "Kindred" events. Revisit post-launch. |
| Q2 | Do NPCs need front/back facing, unlike the side-only dwarf? | Yes: NPCs walk up/down village paths for long stretches. Give the **dwarf back-facing walk** too (S5 + a walk set) at the same time, for consistency (15-art decision). |
| Q3 | Pathfinding | Waypoint graphs per zone (authored), not a full navmesh. |
| Q4 | Count | 14 villagers (inside the 12–16 standard). Launch with 8 (V1–V8), then V9–V14 in content updates. |
| Q5 | Companions in co-op? | Single-player only. Co-op is post-launch. |
| Q6 | Dialogue authoring format | RON per villager with a small condition DSL (hearts ≥, season, flag). No scripting language. |

## 16. Ticket breakdown

- **SOC-01 [GAME]** Villager data model + `villagers.ron` + spawn in a zone + KeyE talk hook. deps: WLD-01 (village zone) (soft: spawn on the homestead first).
- **SOC-02 [ART]** Ottilie + Mayor sprite sets (idle/walk 3-facing/talk/job) + portraits ×6 each. deps: none.
- **SOC-03 [ART]** Dialogue box 9-slice + name plate + emote bubbles + heart icons. deps: none.
- **SOC-04 [UI]** Dialogue UI (portrait, expression, typewriter, choices). deps: SOC-03, SOC-01.
- **SOC-05 [GAME]** Dialogue data + condition matching (hearts/season/weather/flags). deps: SOC-04.
- **SOC-06 [ART]** Player talk S1 + give gift S2 + receive S3. deps: none.
- **SOC-07 [GAME]** Gifts (tastes, weekly limit, quality mult, reactions). deps: SOC-05, SOC-06, FRM-08.
- **SOC-08 [GAME]** Hearts (points, thresholds, decay, caps by act). deps: SOC-07.
- **SOC-09 [GAME]** Schedules (`schedules.ron`, waypoint pathing, phase/season/weather overrides). deps: SOC-01, CYC-02, CYC-05.
- **SOC-10 [ART]** Hask + Mirela + Brunwald + Wren NPC sets + portraits. deps: none.
- **SOC-11 [ART]** Marla + Vell + Tobin + Juniper NPC sets + portraits. deps: none.
- **SOC-12 [DATA]** First 8 villagers: tastes, schedules, dialogue tier 0–4. deps: SOC-09, SOC-02, SOC-10, SOC-11.
- **SOC-13 [ART]** Request board + quest card frames + accept S4 + read S5. deps: none.
- **SOC-14 [GAME]** Request board + quest log (requests). deps: SOC-13, SOC-08.
- **SOC-15 [UI]** Quest log + relationship page + calendar. deps: SOC-14.
- **SOC-16 [GAME]** Mail/letters system. deps: SOC-08.
- **SOC-17 [GAME]** Heart event cutscene system (trigger, camera, scripted moves, emotes). deps: SOC-09, SOC-05.
- **SOC-18 [ART]** Heart event special frames batch 1 (8 villagers × 2♥/4♥ events) + player S11/S12. deps: SOC-02, SOC-10, SOC-11.
- **SOC-19 [DATA]** Heart events 2♥/4♥ for 8 villagers. deps: SOC-17, SOC-18.
- **SOC-20 [GAME]** Personal questlines framework + Hask "Last Hunt" + Brunwald cogs (01). deps: SOC-14, CMB-15, MIN-40.
- **SOC-21 [GAME]** Unlock hooks: villager weapon trees/techniques/rare seeds/recipes by hearts. deps: SOC-08, FRG-39, FRM-23.
- **SOC-22 [ART]** Companion hunt sheets: Hask. deps: CMB-48 (GS timings).
- **SOC-23 [GAME]** Companion AI framework (follow, engage, evade telegraphs, downed/revive) + Hask. deps: SOC-22, MON-04, CMB-19.
- **SOC-24 [ART]** Companion sheets Wren + Mirela + Kit. deps: SOC-22.
- **SOC-25 [GAME]** Wren/Mirela/Kit companion behaviors. deps: SOC-23, SOC-24.
- **SOC-26 [UI]** Lodge companion select. deps: SOC-23.
- **SOC-27 [ART]** Farm helper sprites (Marla on the homestead, Tobin's Helper Bot) + job loops. deps: none.
- **SOC-28 [GAME]** Farm helpers at 8♥. deps: SOC-27, ANM-10, FRM-04.
- **SOC-29 [GAME]** Festival framework (day takeover, map dressing, scripted activity, rewards). deps: SOC-09, CYC-02.
- **SOC-30 [ART]** Festival dressing (stalls, banners, lanterns, fireworks) + S6/S7/S8 + NPC festival poses. deps: none.
- **SOC-31 [GAME]** Festivals batch 1: Gearbloom Fair, Hunter's Oath, Harvest Grading, Festival of Lanterns (Spark). deps: SOC-29, SOC-30.
- **SOC-32 [ART]** Regatta boat + S9 + Derby props. deps: none.
- **SOC-33 [GAME]** Festivals batch 2: Steam Regatta, Gearfin Derby (FSH-40), Hunter's Moon, Forgefire Night. deps: SOC-31, SOC-32, FSH-40.
- **SOC-34 [ART]** V12–V14 (Quillon, Kit, Rook) + Sela (V11) NPC sets + portraits. deps: none.
- **SOC-35 [DATA]** V9–V14 data (tastes, schedules, dialogue, events). deps: SOC-34, SOC-12.
- **SOC-36 [DATA]** Heart events 6♥/8♥/10♥ + art batch 2 (split as needed). deps: SOC-19.
- **SOC-37 [AUD]** Voice blips, gift/heart jingles, festival themes. deps: SOC-04, SOC-31.
- **SOC-38 [SAVE]** Persist villagers, quests, mail, festivals, companions, helpers. deps: SOC-14, SOC-16.
- **SOC-39 [DATA]** Social balance pass. deps: SOC-35.
- **SOC-40 [ART]** Dwarf back-facing walk + front idle (per Q2) for consistency with NPCs. deps: none (15-art decision ART-02).

**Ticket count: 40.**

**Build first (top 3):** SOC-01 + SOC-02 (villager model + Ottilie/Mayor art), SOC-03 → SOC-04 → SOC-05 (dialogue UI + data), SOC-06 → SOC-07 → SOC-08 (gifts + hearts with player gift frames).
