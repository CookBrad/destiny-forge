# 11 — Day / Season Cycle (time, sleep, calendar, seasons, weather)

Authority: GDD §3 ("day/night and seasonal cycle similar to Stardew Valley; hunting expeditions span multiple in-game days or a single outing… Rest & Advance: sleep advances the day, restores stamina, progresses seasons and story events"), §4.1/§4.2/§6 (seasonal crops, rare nodes, creature behavior, weather-only materials), §10 (Must-have: seasons).

Build priority: **11 of 16**. Many systems depend on **CYC-02 (seasons)** and **CYC-05 (weather)**, so those two tickets should be pulled forward once farming starts (they are listed early here for that reason).

## 0. Standing rules

Sleep, waking, passing out, and the clock-driven player states (yawn at late evening, tired walk) get drawn dwarf frames. Day/night lighting uses drawn overlay sprites/palettes (the existing `ambient_clear_color` tint is fine for the sky clear color; actors and props need drawn night variants or a drawn light-mask overlay, not a per-sprite tint, Q4). No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists

| Exists | Where |
| --- | --- |
| `DayPhase {Morning, Afternoon, Evening}` with `PHASE_DURATION_SECS = 90` real seconds; Evening holds until sleep | `core/day_cycle.rs` |
| `DayClock {calendar_day, phase, phase_elapsed_secs}`; HUD label | `day_cycle.rs`, `ui/day_hud.rs` |
| Sleep at the house bed (hold E) → advances the day, grows crops, respawns ore (PR #95) | `overworld/interaction.rs::try_sleep_at_bed`, `perform_sleep` |
| Hunt day cost (2 phase steps) | `core/hunt_day_cost.rs` |
| Tool energy 100, restored on sleep | `day_cycle.rs`, `ui/energy_hud.rs` |
| Ambient clear color per phase | `DayPhase::ambient_clear_color` |
| Saves `calendar_day`, `day_phase`, `tool_energy` | `profile.rs` |

## 2. Where it happens

Everywhere. Time runs in all zones except menus, cutscenes, and dialogue (Q1). Sleep happens at the bed, the Base Camp tent (expeditions), the Inn (paid, 100 g), or by passing out.

## 3. Entities / data

- `DayClock` gains `season: Season {Spring, Summer, Fall, Winter}`, `day_of_season 1–28`, `year`, `weekday` (7-day week: Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday).
- `DayPhase` extends to **Morning, Midday, Afternoon, Evening, Night** (Q2: proposal 5 phases × 120 s = 10 real minutes per day, with Night ending at the 2 AM pass-out).
- `Weather {Sunny, Cloudy, Rain, Storm, Snow, Fog, Windy}` + `tomorrow_forecast` (TV/newspaper; Mayor's notice board).
- `Calendar` data (`calendar.ron`): festivals (08), birthdays, story events, seasonal shop stock, rare-node windows.
- `SleepOutcome` (energy restore %, pass-out penalties, overnight events).
- `OvernightEvents` queue (crop growth, animal products, machines, shipping summary, mail, story/heart triggers, random events: meteor on the homestead → Starmetal, fairy-like "Spark Wisp" grows a crop, storm damage).

## 4. Tools / equipment

Clock/Watch HUD (no item). "Pocket Chronometer" accessory (Forge L2): shows the exact minute + forecast. Inn bed, tent (expeditions), Sleeping Bag (craftable: sleep in the mine at rest floors, 5n, waking there; Act 2).

## 5. ANIMATIONS (most detailed section)

Dwarf 16×28; bed actions on a 48×40 canvas; right-authored + `flip_x` unless noted.

### 5.1 Character

| # | Set | Frames | Canvas | Facing | Timing | Loop | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| D1 | Get into bed | 5 | 48×40 | side | 0.1 ×5 | no | **the existing hold-E sleep has no frames** |
| D2 | Sleeping in bed | 2 | 48×40 | top | 0.6 | yes | + Zzz overlay |
| D3 | Wake up + stretch | 6 | 48×40 → 24×32 | front | 0.12 ×6 | no | morning start |
| D4 | Yawn (late Evening / Night idle) | 4 | 16×28 | front | 0.15 ×4 | no | plays when idle ≥ 3 s after 10 PM |
| D5 | Tired walk | shares MIN C20 | | | | | ≤ 10% energy or after midnight |
| D6 | Pass out (2 AM or 0 energy) | shares MIN C19 (5) | | | | | |
| D7 | Tent sleep (camp) | 4 in + 2 loop | 48×40 | side | 0.1 / 0.6 | | expeditions |
| D8 | Umbrella walk (rain, optional cosmetic) | 4 | 24×32 | R | 0.1 | yes | only with the Umbrella cosmetic |
| D9 | Shiver idle (winter, outdoors, no coat) | 2 | 16×28 | front | 0.2 | yes | flavor |
| D10 | Read forecast (newspaper) | 3 | 24×28 | front | 0.15 | no | |

### 5.2 Entities / environment

| Set | Frames | Cell | Timing | Notes |
| --- | --- | --- | --- | --- |
| Bed (made/unmade + blanket rise) | 2 + 2 | 48×40 | 0.6 | |
| Window light shafts per phase (interiors) | 5 | 64×64 | static per phase | |
| Lamp posts on/off + flicker | 2 + 3 | 16×48 | 0.12 | lit Evening–Night |
| Night light-mask overlays (radial masks for lamps/windows) | 3 sizes | 128×128 | static | full-screen night darkness layer + light cutouts (shared tech with MIN E19) |
| Phase sky gradient overlays (5) | 5 | 1×256 stretched | static | replace the flat clear color |
| Meteor event (falls on the homestead) | 8 + crater 1 | 64×128 | 0.06 | overnight event |
| Spark Wisp event | 6 | 16×16 | 0.1 | |
| Newspaper/forecast board | 1 + flutter 3 | 32×32 | 0.15 | |

### 5.3 VFX

Zzz bubbles 3 @0.4; sunrise sparkle transition 6 (full-screen wipe drawn); day-end fade with a calendar page flip 8 (UI, 128×96) @0.08; season-change banner unfurl 8 (drawn); weather VFX are owned by 10 (WLD-24).

### 5.4 UI

Clock dial (brass, 48×48) with hand positions per phase (5) + weather icon slot; season badge ×4 (02); calendar page frame; day-start card (day name, weather icon); forecast icons ×7; energy and night warnings (moon icon).

## 6. Inventory items

Sleeping Bag (Tool, 1), Umbrella (cosmetic, 1), Pocket Chronometer (accessory, 1), Newspaper (daily, not kept).

## 7. Processing (sleep pipeline)

On sleep, in order (pure functions, testable): end-of-day shipping → crops (02) → fruit trees → animals (07) → machines → smelter (01) → node respawns (01, exists) → forage spawns → weather roll for tomorrow → season rollover (wither out-of-season crops, swap tilesets) → festival/story/heart triggers (08/12) → mail → autosave (16) → wake. Energy restore: 100% (bed before midnight), 75% (after midnight), 50% (pass out), plus food/Inn bonuses.

## 8. Hooks

Farming (growth, seasons, rain watering), Mining (respawn, rare nodes, time in the mine), Fishing (time/season/weather tables), Animals (products, outdoors), Social (schedules, festivals, birthdays), Combat (hunt time cost, expeditions, night monster behavior), Monsters (seasonal variants), World (dressing, weather), Narrative (dated events), Economy (shipping on sleep, shop hours), Saves (autosave on sleep).

## 9. Energy / time / risk

Day = 5 phases × 120 s (Q2). Passing out at 2 AM (end of Night) costs 10% gold (cap 1,000) and wakes at home with 50% energy. Hunts consume phases by quest (05). Expeditions skip nights through the tent.

## 10. Progression / gating

Year 1 Spring start. Seasons gate crops/fish/forage/festivals/monsters. Story acts are gated by events, not dates, with soft "by Fall Y1" pacing targets (09 §3.3).

## 11. UI

Day HUD (exists) → clock dial + weather + season + day-of-season; the night warning toast at 12 AM; sleep confirm dialog ("Go to bed?"); day summary on wake (shipping, products ready, mail); calendar screen (C key).

## 12. Audio

Rooster/morning chime, clock tower bell per phase (village), evening crickets, night owl, sleep lullaby sting, wake-up chime, rain/storm/snow/wind ambience layers (10), season-change stinger, pass-out "whomp".

## 13. Save data

`calendar_day` (exists) → `{year, season, day_of_season}` derived or stored; `day_phase` (exists, enum grows: map Morning/Afternoon/Evening → new ids); `weather_today`, `weather_tomorrow`; `overnight_events_pending`; `last_sleep_kind`. Migration fixture from the v6+ profile.

## 14. Balance levers

Phase count and seconds; energy restore by bedtime; pass-out penalty; weather probabilities per season (Spring rain 25%, Summer storm 10%, Fall rain 20%/wind 15%, Winter snow 35%); season length (28); hunt phase costs; Inn price; overnight event chances.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Does time pass during menus/dialogue? | No (Stardew-like): paused in menus, dialogue, cutscenes, and the forge cinematic. |
| Q2 | 3 × 90 s days (4.5 min) feel short | **5 phases × 120 s = 10 min/day.** Keep the `DayPhase` names, add Midday + Night. The existing hunt cost of 2 steps stays (~40%). |
| Q3 | Show clock minutes or phases only? | Phases + an analog dial. Exact minutes only with the Pocket Chronometer. |
| Q4 | Night lighting approach | Full-screen drawn darkness overlay with light-mask cutouts (lamps, windows, lantern), shared with the mine (MIN-27). No per-sprite tint. |
| Q5 | Season length | 28 days (Stardew familiarity). Configurable in data. |
| Q6 | Weekdays | Yes, 7 named days, for schedules and Kit's Friday-equivalent "Brassday" merchant. |

## 16. Ticket breakdown

- **CYC-01 [GAME]** Extend `DayPhase` to 5 phases (120 s) + profile migration mapping old phases; keep the hunt cost semantics. deps: none.
- **CYC-02 [GAME]** Seasons + day-of-season + year + weekday derived from `calendar_day`; `Season` resource + events on rollover. deps: none.
- **CYC-03 [ART]** Get into bed D1 + sleeping D2 + wake D3 + bed states + Zzz. deps: none.
- **CYC-04 [GAME]** Sleep sequence plays D1→D2→fade→D3, replacing the instant hold-E. deps: CYC-03.
- **CYC-05 [GAME]** Weather system: daily roll per season, tomorrow forecast, `Weather` resource and events. deps: CYC-02.
- **CYC-06 [GAME]** Overnight pipeline as an ordered list of systems (refactor the existing sleep side effects into it) + tests. deps: CYC-02.
- **CYC-07 [GAME]** Night phase + 2 AM pass-out + energy restore by bedtime (shared with MIN-45). deps: CYC-01, MIN-43 (faint frames).
- **CYC-08 [ART]** Yawn D4 + shiver D9 + read forecast D10 + newspaper board. deps: none.
- **CYC-09 [GAME]** Late-night idle yawn/tired walk triggers + forecast board. deps: CYC-08, CYC-05.
- **CYC-10 [ART]** Night light-mask overlays + lamp posts + phase sky gradients + interior window shafts. deps: none.
- **CYC-11 [GAME]** Day/night lighting layer (overlay + light cutouts), shared with the mine darkness. deps: CYC-10, CYC-01.
- **CYC-12 [ART]** Clock dial + season badges + forecast icons + calendar page + day-start card + season banner. deps: none.
- **CYC-13 [UI]** Day HUD v2 (dial, weather, season) + wake summary + calendar screen (C). deps: CYC-12, CYC-02, CYC-05.
- **CYC-14 [DATA]** `calendar.ron` (festivals, birthdays, story events, shop stock windows, rare nodes). deps: CYC-02.
- **CYC-15 [ART]** Meteor + Spark Wisp overnight event sprites. deps: none.
- **CYC-16 [GAME]** Random overnight events (meteor → Starmetal, Spark Wisp crop, storm damage). deps: CYC-06, CYC-15.
- **CYC-17 [ART]** Tent sleep D7 + sleeping bag. deps: CYC-03.
- **CYC-18 [GAME]** Alternative sleep spots (Inn paid, camp tent for expeditions, mine sleeping bag). deps: CYC-17, CYC-04, CMB-28.
- **CYC-19 [AUD]** Time-of-day sfx (rooster, bells, crickets, lullaby, wake chime, season sting). deps: CYC-04.
- **CYC-20 [SAVE]** Persist season/weather/forecast/overnight queue + migration fixture. deps: CYC-02, CYC-05.
- **CYC-21 [DATA]** Day-length and weather-probability balance pass. deps: CYC-05, FRM-72.

**Ticket count: 21.**

**Build first (top 3):** CYC-02 (seasons, which farming/fishing/monsters depend on), CYC-03 → CYC-04 (sleep animation for the existing sleep action), CYC-05 (weather).
