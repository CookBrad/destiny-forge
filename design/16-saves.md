# 16 — Saves (profiles, persistence, migration, autosave)

Standards: sleeping advances the day and the homestead persists. Saves and profiles already ship. This doc covers how every new system persists safely.

Build priority: **16 of 16** as a system, but **SAV-01 and SAV-02 should land early**. Nearly every system ticket adds save fields, and the current migration and error handling (§1) will not scale.

## 0. Standing rules

Saving/loading actions visible to the player get drawn frames: the sleep-save (11 D1–D3), the autosave gear spinner (13 U18), and the title profile cards. No code comments; zero warnings. Dungeon art untouched. [ART] tickets are separate.

## 1. Already exists (and findings)

| Exists | Where |
| --- | --- |
| 3 profiles (`PROFILE_COUNT = 3`), RON files under `ProjectDirs(..., "destiny_forge").data_local_dir()` | `core/memory/storage.rs`, `profile.rs` |
| `PlayerProfile {version, name, inventory, loadout, progress, calendar_day, day_phase, tool_energy, crop_plots, settings}` | `profile.rs` |
| `PROFILE_VERSION = 6` on `develop`. PR #96 bumps it to 7 (24→32 slots, food). PR #95 adds `#[serde(default)] depleted_ore_node_ids` without a bump (convention: additive fields use serde default and do not bump) | `profile.rs` on each branch |
| Migration: `if self.version < PROFILE_VERSION { self.version = PROFILE_VERSION }` + fixture `profile_v6_24_slots.ron` test | `profile.rs`, `fixtures/` |
| Atomic write (tmp + rename), pretty RON depth 4 | `storage.rs::write_ron` |
| Debounced autosave 0.75 s on `ProfileDirty`, flush on exit | `autosave.rs` |
| Settings file (`SETTINGS_VERSION = 2`, last active profile, audio, skill bindings per profile) | `settings.rs` |
| Hydrate runtime from memory | `sync.rs` |
| Title profile picker + rename | `ui/profile_picker.rs`, `title_profiles.rs` |

**Findings to fix:**
1. **Concurrent schema edits**: PR #95 and PR #96 both change `PlayerProfile` (one additive field; one v7 bump with a fixture). Today this works only because additive fields use `serde(default)`. As 16 systems add fields, the merge-order risk grows, and renames/re-typings (e.g. `WeaponKind` → gear rows) cannot be handled by defaults. SAV-01 adds a mechanical migration path and namespaced sub-structs (SAV-05) to stop branches colliding.
2. **Silent reset risk**: `load_ron` returns `None` on any parse error, and the caller falls back to `Default`. A save that fails to parse (schema mistake, partial write from an older build) **loads as a brand-new profile, and the next autosave overwrites the old file**. That is data loss (SAV-02).
3. Migration is a single version bump with `#[serde(default)]`-style tolerance. There is no per-version transform hook for renames (needed for `WeaponKind` → gear rows, `DayPhase` growth, quality on slots).

## 2. Where it happens

Autosave (debounced on dirty), sleep (authoritative daily save), zone transitions (debounced), quit/exit flush (exists), expedition tent (05), and never mid-hunt or mid-mine-floor (01 Q5, 05 Q7).

## 3. Entities / data model (target)

`SaveFile {header: SaveHeader {version, game_build, saved_at_utc, playtime_secs, calendar summary, location, portrait/thumb}, body: PlayerProfile}`.
`PlayerProfile` grows **namespaced sub-structs** (one per system, each `#[serde(default)]`): `inventory`, `loadout`/`gear` (04), `farm` (02), `mine` (01), `fishing` (03), `animals` (07), `social` (08), `economy` (09), `world` (10), `time` (11), `story` (12), `ui_prefs` (13), `audio` (14), `hunter` (05), `field_guide` (06).
`Migrations`: an ordered list `fn(v_n RON Value) -> v_{n+1} RON Value`, applied before typed deserialize, each with a fixture test.

## 4. Tools

`tools/save_inspect.py` (pretty-print + validate a profile), a debug "export save" button (copies the file to `exports/`), and the backup rotation (3 most recent: `profile_N.ron.bak1..3`).

## 5. ANIMATIONS (most detailed section)

| # | Set | Frames | Size | Timing | Notes |
| --- | --- | --- | --- | --- | --- |
| SV1 | Autosave gear spinner (HUD corner) | 8 | 16×16 | 0.06 loop | shown ≥ 0.5 s per save (13 U18) |
| SV2 | Sleep save sequence | reuse CYC D1–D3 + a calendar page flip | — | — | the day-save moment |
| SV3 | Title profile card: idle portrait of the dwarf (front idle AP1) + equipped weapon icon | 4 | 32×40 | 0.2 loop | per profile card |
| SV4 | Profile card select/hover | 3 | 9-slice | 0.08 | |
| SV5 | New profile "deed signing" (dwarf signs the deed) | 6 | 48×40 | 0.1 | new game action (pairs with NAR P1) |
| SV6 | Delete profile confirm (deed burns, cozy) | 6 | 48×48 | 0.08 | |
| SV7 | Load error card ("Your save needs attention") with a broken-gear icon | 1 + 3 | 160×96 / 16×16 | 0.15 | SAV-02 UX |
| SV8 | Save thumbnail capture frame | 1 | 96×54 | — | header thumb (optional) |

## 6. Inventory items

None.

## 7. Processing

Save: gather sub-structs → serialize → tmp write → rotate backups → rename (exists) → clear dirty. Load: read → parse the header → migrate the RON value up to the current version → typed deserialize → validate (clamp ranges, drop unknown ids with a warning log) → hydrate (exists). On failure, **never overwrite**: show SV7 with options Restore backup / Open folder / Start fresh (moves the bad file to `corrupt/`).

## 8. Hooks

Every system declares its sub-struct + default + migration entry + a fixture. Per-doc "Save data" sections list the fields.

## 9. Energy / time / risk

Risk: data loss (finding 2). Target: zero silent resets.

## 10. Progression / gating

N/A.

## 11. UI

Title profile cards (portrait, name, day/season/year, playtime, location, gold), rename (exists), delete (new, confirm), backup restore dialog, "Saving…" spinner, load error card.

## 12. Audio

save_tick (subtle), profile_select, deed_sign quill, delete_whoosh, error_soft.

## 13. Save data

(this document) Settings file: add video options, global accessibility defaults; keep per-profile bindings.

## 14. Balance levers

Autosave debounce (0.75 s), backup count (3), when to save (sleep/transition/dirty).

## 15. Open questions

| # | Question | Proposal |
| --- | --- | --- |
| Q1 | RON vs binary? | Keep RON (debuggable, existing) with a size check. Profiles stay < 1 MB with the projected systems. |
| Q2 | More than 3 profiles? | Keep 3 (exists). |
| Q3 | Cloud saves | Steam Auto-Cloud on the save directory (no code). Document it at ship time. |
| Q4 | Mid-hunt saves | No, except the expedition tent (05 Q7). |
| Q5 | Strict deserialization (deny unknown fields)? | Tolerant on load (unknown fields ignored with a log), strict in tests (fixtures round-trip exactly). |

## 16. Ticket breakdown

- **SAV-01 [SAVE]** Migration framework: ordered RON-value migrations + per-version fixture tests; add a fixture covering the merged PR #95 + #96 schema. deps: none.
- **SAV-02 [SAVE]** Load failure safety: never overwrite unparseable saves, 3-backup rotation, `corrupt/` quarantine, error surfaced to the UI state. deps: none.
- **SAV-03 [ART]** Load error card SV7 + autosave spinner SV1. deps: UI-02 (soft).
- **SAV-04 [UI]** Load error dialog (restore backup / open folder / start fresh) + saving spinner. deps: SAV-02, SAV-03.
- **SAV-05 [SAVE]** `SaveHeader` (playtime, calendar, location, gold) + namespaced sub-structs refactor (behavior unchanged). deps: SAV-01.
- **SAV-06 [ART]** Profile card portrait SV3 + hover SV4 + deed signing SV5 + delete SV6. deps: ART-02.
- **SAV-07 [UI]** Title profile cards v2 (header data, portrait, delete with confirm). deps: SAV-05, SAV-06.
- **SAV-08 [SAVE]** Save points policy: sleep authoritative save, transition saves, block mid-hunt/mid-floor saves, expedition tent save. deps: SAV-05, CMB-28.
- **SAV-09 [SAVE]** `tools/save_inspect.py` + debug export. deps: SAV-05.
- **SAV-10 [SAVE]** Load validation/clamping (unknown ids dropped + logged; ranges clamped). deps: SAV-05.
- **SAV-11 [AUD]** Save/profile sfx. deps: SAV-07.
- **SAV-12 [SAVE]** Settings v3 (video, global accessibility defaults) + migration. deps: SAV-01.

**Ticket count: 12.**

**Build first (top 3):** SAV-02 (stop silent profile resets, a data-loss bug), SAV-01 (migration framework + merged PR #95/#96 fixture), SAV-05 (namespaced sub-structs so each system adds saves without conflicts).
