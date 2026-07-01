# Destiny Forge — Coding Standards

All Rust code in this project follows the principles from *Clean Code* (Robert C. Martin). The ECS architecture already encourages separation of concerns; these rules keep that discipline explicit.

---

## Meaningful names

- Types, functions, and variables reveal intent without comments: `CarveLootTable`, `try_craft_recipe`, `equipped_armor_pieces`
- Avoid abbreviations and noise words (`data`, `info`, `manager`, `handle_thing`)
- Names are pronounceable and searchable; one word per concept (`craft` not `make`/`build`/`create` interchangeably)

---

## Small functions

- Functions do **one thing**, do it well, and do it only
- Prefer ~5–15 lines per function; extract when a function mixes concerns (e.g. input + validation + mutation)
- One level of abstraction per function — a system orchestrates, helpers perform single steps

---

## SOLID (adapted for Rust + Bevy)

| Principle                 | How we apply it                                                                        |
| ------------------------- | -------------------------------------------------------------------------------------- |
| **S**ingle Responsibility | One plugin/module per domain (`dungeon`, `forging`, `combat`); one system per behavior |
| **O**pen/Closed           | New weapons, armor sets, and recipes extend via data/types — not by editing core loops |
| **L**iskov Substitution   | Shared traits (`Item`, `Recipe`) behave consistently across all implementors           |
| **I**nterface Segregation | Small traits and marker components instead of fat "god" components                     |
| **D**ependency Inversion  | Systems depend on resources/traits; content is injected via plugins and data           |

---

## Comments

- Code explains **what**; comments explain **why** (non-obvious rules, balance notes, deferred work)
- No commented-out code in commits; no journal-style change logs in source files
- TODOs include context: `// TODO(phase-2): persist inventory when save system lands`

---

## Formatting & structure

- Consistent module layout: `plugin.rs` registers systems; behavior lives in focused files
- `main.rs` only wires plugins — no game logic
- Related code stays vertically close; variables declared near first use
- Boy Scout Rule: leave every file slightly cleaner than you found it

---

## Error handling

- Recoverable game logic returns `Option` or `Result` — not panics
- `expect`/`unwrap` only for programmer invariants (e.g. "player exists in dungeon state")
- Invalid player actions fail silently or with UI feedback — never crash the app

---

## Tests

- Pure logic (recipes, set bonuses, inventory math) gets unit tests
- Systems that glue Bevy together stay thin so logic remains testable without a full app

---

## What we avoid

- Monolithic `update` functions and thousand-line `main.rs`
- Boolean flag parameters that change behavior (`craft(item, true, false)`)
- Duplicated logic across dungeon and overworld — shared behavior lives in `core` or `items`
- Premature abstraction; extract only after the second real use case