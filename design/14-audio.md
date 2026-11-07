# 14 — Audio

Authority: GDD §8 ("Audio: warm acoustic and light steampunk instrumentation for farm life; dynamic orchestral tracks with magical and mechanical accents during hunts. Satisfying weapon impacts, steam vents, and forge hammers").

Build priority: **14 of 16**. Per-system cue lists are in each doc's Audio section. This doc owns the audio engine, mixing, the music system, asset standards, and the master cue registry.

## 0. Standing rules

Audio never replaces animation: every cue is tied to a **drawn frame index** (hit frames, footfalls) so timing matches the sprite. The visual accessibility aids (sound indicators/captions) are drawn sprites. All new audio is in-house or properly licensed with `assets/ATTRIBUTION.txt` entries. No code comments; zero warnings. Dungeon audio may be added to (new music layers) but the dungeon platform/art is frozen. [ART]/[AUD] tickets are separate.

## 1. Already exists

| Exists | Where |
| --- | --- |
| `dungeon_music.mp3` + start/stop/pause/resume, base volume 0.38 | `audio/music.rs` |
| 14 sfx (block, boss_charge, charge, enemy_melee, enemy_shoot, ground_slam, heavy_hit, parry, player_hurt, slime_burst, slime_shoot, spin, sword_hit, sword_swing) | `assets/audio/sfx/`, `audio/sfx.rs` |
| Audio settings: music on/volume, sfx on/volume per profile | `audio/settings.rs`, `memory/settings.rs` |
| Pause audio handling | `ui/pause_audio.rs` |
| **No homestead/forest/mine music; no ambience; no UI sounds** | — |

## 2. Where it happens

All states. Buses: Master → Music, Ambience, SFX (World, Combat, UI), Voice-blips.

## 3. Entities / systems

`AudioBus` volumes, `MusicDirector` (zone track + layers + crossfades + stingers), `AmbienceBed` (per zone/phase/weather layers), `SfxRegistry` (`audio_cues.ron`: cue id → files (variants), volume, pitch jitter, cooldown, bus, max instances), `AnimationSfxBinding` (sprite set + frame index → cue), `SpatialSfx` (pan/attenuation by distance from the player/camera), `Captions` (accessibility).

### 3.1 Music map (tracks; 60–180 s loops, OGG Vorbis, 44.1 kHz)

| Context | Tracks | Style | Dynamic layers |
| --- | --- | --- | --- |
| Title | Main Theme | forge-hammer leitmotif, orchestral + music box | — |
| Homestead | Spring/Summer/Fall/Winter (day) + Evening variant + Rain variant | warm acoustic guitar, accordion, light clockwork percussion | Evening = fewer instruments |
| Village | Day / Evening / Festival ×8 | brass band light, hand percussion | crowd layer |
| Forest | Day / Evening | flute, harp, glow-chime pads | |
| Riverlands, Ruins, Caldera, Islets | Day / Evening each | per-zone motifs | |
| Mine | per stratum ×5 | steampunk ambient, drips as rhythm | depth layer |
| Hunting grounds | Explore / Combat / Enraged / Victory per ground (or shared per rank) | dynamic orchestral + mechanical accents | 3 layers crossfaded by monster state |
| Elder/Finale | Aether Drake, Destiny Wyrm 3-phase | full orchestral + choir | phase transitions |
| Dungeon F1 (exists) | dungeon_music | — | optional combat layer (audio only) |
| Characters | Edda, Rook, Wyrm themes | | |
| Credits | Credits suite | | |

### 3.2 SFX families and counts (target)

Movement/footsteps by surface (grass, dirt, wood, stone, metal, snow, water, crystal) × 4 variants; tools (01/02/03); forge (04); combat (05; per weapon ×6 families); monsters (06; ~12 large × ~15 cues + small ×5); animals (07); social (08 voice blips ×14 timbres); economy (09); world ambience beds (10); time (11); UI (13). **≈ 650 cue ids, ≈ 1,500 files** with variants.

## 4. Tools / equipment

Tooling: `tools/audio_lint.py` (loudness check −16 LUFS music / −14 LUFS short sfx peak −1 dBTP, sample-rate check, unused/missing cue report), `audio_cues.ron` schema validation in tests.

## 5. ANIMATIONS (most detailed section)

Audio-facing visuals: frame-synced cue bindings plus drawn accessibility/visual assets.

### 5.1 Frame-sync bindings (cue ← sprite set frame)

| Sprite set (doc) | Frame | Cue |
| --- | --- | --- |
| Dwarf walk/run (exists, 4 frames) | f1, f3 | footstep by surface |
| Hunter run (exists, 8 frames cells 1–8) | cells 2, 6 | footstep by surface |
| Pickaxe swing C1–C6 (01) | f2 | pick_hit_* or pick_clank |
| Hoe F1/F2 (02) | f2 | hoe_till |
| Watering F4/F5 (02) | f3–f5 | water_pour loop |
| Cast FS2 (03) | f2 | cast_whoosh |
| Anvil G1 (04) | f2 | hammer_strike |
| Quench G4 (04) | f2 | quench_hiss |
| Every attack move (05) | first active frame | swing; on hit → impact by damage type |
| Monster telegraph (06) | telegraph f0 | telegraph cue (required) |
| Monster footsteps (06) | contact frames | weight-class stomp |
| Animal idle/happy (07) | f0 | species call (cooldown 4 s) |
| NPC talk (08) | each typed character | voice blip |
| Eat (PR #96, 4 frames order 0,1,2,3,2,3) | f2 | munch |
| Sleep D1 (11) | f3 | blanket rustle |

### 5.2 Drawn audio visuals

| # | Set | Frames | Size | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| AU1 | Sound direction indicators (accessibility): arc pip around the player for off-screen roars/telegraphs | 4 (pulse) × 8 directions | 16×16 | 0.1 | toggle in assists |
| AU2 | Caption box frame + speaker icon | 1 + 2 | 9-slice / 12×12 | 0.2 | captions "[Bristleboar roars]" |
| AU3 | Music note emote for NPC humming (shared SOC bubble) | 3 | 16×16 | 0.12 | |
| AU4 | Gramophone (homestead decor/jukebox: replay unlocked tracks) idle + playing | 1 + 6 | 32×48 | 0.1 | horn wobble + notes |
| AU5 | Dwarf wind gramophone | 4 | 32×36 | 0.1 | player action when using the jukebox |
| AU6 | Audio settings icons (master/music/ambience/sfx/UI/voice; mute states) | 2 each ×6 | 16×16 | — | |
| AU7 | Volume slider knob (brass) + track | 2 | 16×16 / 9-slice | — | |
| AU8 | Village bell tower bell swing | 6 | 32×32 | 0.08 | sync with the bell sfx (shared WLD) |

## 6. Inventory items

Gramophone (decor, Workbench: 2 Brass Bar + 20 Hardwood), Music Rolls (collectibles from festivals/chests that unlock jukebox tracks; key items, ×20).

## 7. Processing

Cue pipeline: source → normalized OGG (`assets/audio/{music,ambience,sfx/<family>}/`) → registered in `audio_cues.ron` → bound to frames or events → lint/test.

## 8. Hooks

Every system. Music state reads zone (10), phase/weather/season (11), hunt monster state (05/06), festivals (08), story (12).

## 9. Energy / time / risk

N/A.

## 10. Progression / gating

Jukebox tracks unlock by visiting zones/hunting monsters/collecting Music Rolls.

## 11. UI

Audio settings (bus sliders: Master, Music, Ambience, SFX, UI, Voice; mute toggles; existing music/sfx toggles migrate), Captions on/off + size, Sound indicators on/off, Jukebox menu.

## 12. Audio

(this document)

## 13. Save data

Profile `audio` settings (exists: music/sfx enabled + volume) → add ambience/UI/voice/master volumes, captions, indicators. `jukebox_unlocked` set.

## 14. Balance levers

Bus volumes, crossfade times (1.5 s zone, 0.8 s combat layer), stinger priority, cue cooldowns, max simultaneous instances, distance attenuation, pitch jitter.

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | Bevy built-in audio vs `bevy_kira_audio` for layered music? | Use **bevy_kira_audio** (or Bevy 0.15 native, if crossfade/tweens suffice in a spike). Decide in AUD-01's spike, with a 1-day timebox. |
| Q2 | Source of music | Commissioned or in-house composition, credited in ATTRIBUTION. Placeholder CC0 is allowed only in dev branches. |
| Q3 | Per-ground hunt music or per-rank? | Per rank (3 sets) at launch + Elder/finale themes. Per-ground tracks come later. |
| Q4 | Spatial audio | Simple stereo pan + distance falloff (2D). |

## 16. Ticket breakdown

- **AUD-01 [AUD]** Audio engine spike + `AudioBus` mixer (Master/Music/Ambience/SFX/UI/Voice), migrating the existing settings. deps: none.
- **AUD-02 [AUD]** `SfxRegistry` + `audio_cues.ron` (variants, pitch jitter, cooldown, bus) migrating the 14 existing sfx + tests. deps: AUD-01.
- **AUD-03 [AUD]** `AnimationSfxBinding` (sprite set + frame → cue) and bind the existing walk/run/attack/eat frames. deps: AUD-02.
- **AUD-04 [AUD]** `MusicDirector` (zone tracks, crossfades, stingers) + homestead spring track + forest track (first non-dungeon music). deps: AUD-01.
- **AUD-05 [AUD]** Ambience beds (homestead, forest, mine upper) + weather layers. deps: AUD-01, CYC-05 (soft).
- **AUD-06 [AUD]** Footstep surfaces (tile → surface tag) for dwarf + hunter. deps: AUD-03.
- **AUD-07 [AUD]** Dynamic hunt music layers (explore/combat/enraged/victory) for Low rank. deps: AUD-04, CMB-61 (same work, merge if done there).
- **AUD-08 [AUD]** Seasonal homestead tracks (summer/fall/winter + evening/rain variants). deps: AUD-04, CYC-02.
- **AUD-09 [AUD]** Village + festival tracks. deps: AUD-04, WLD-03.
- **AUD-10 [AUD]** Zone tracks Riverlands/Ruins/Caldera/Islets + mine strata. deps: AUD-04.
- **AUD-11 [AUD]** High/Master hunt layers + Elder/finale themes + character themes + title theme + credits. deps: AUD-07.
- **AUD-12 [ART]** Accessibility visuals AU1/AU2 + audio settings icons/slider AU6/AU7. deps: UI-02.
- **AUD-13 [UI]** Audio settings v2 + captions + sound direction indicators. deps: AUD-01, AUD-12.
- **AUD-14 [ART]** Gramophone AU4 + wind gramophone AU5 + Music Roll icons. deps: none.
- **AUD-15 [GAME]** Jukebox (gramophone decor) + Music Roll unlocks. deps: AUD-14, AUD-04.
- **AUD-16 [AUD]** `tools/audio_lint.py` + CI test that every cue id in the registry has files and every bound frame has a valid cue. deps: AUD-02.
- **AUD-17 [SAVE]** Persist new audio settings + jukebox unlocks (migration). deps: AUD-13, AUD-15.
- **AUD-18 [AUD]** Final mix/loudness pass. deps: AUD-11.

**Ticket count: 18.**

**Build first (top 3):** AUD-01 (bus mixer + engine decision), AUD-02 → AUD-03 (cue registry + frame-synced bindings, which every system's audio tickets need), AUD-04 (music director + first homestead/forest tracks).
