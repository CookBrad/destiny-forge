# Destiny Forge — Coding Standards

All code in this project follows a strict reading of *Clean Code* (Robert C. Martin). Every rule below is numbered so a reviewer can cite it by number. A pull request that breaks any rule is blocked until it is fixed.

---

## 1. Comments

1.1. No comments of any kind anywhere in the repository. This covers `//`, `///`, `//!`, and `/* */` in Rust; `#` lines and docstrings in Python; and comments in `.ron` files and every other data or config file.

1.2. Names and structure carry the intent. If code seems to need a comment, rename or restructure it until it does not.

1.3. Any comment in a diff blocks the pull request. Commented-out code and TODO notes are comments, so they are not allowed either.

---

## 2. Warnings and the build gate

2.1. No warning suppression. No `#[allow(...)]`, no `#[expect(...)]`, no clippy allow attributes, no crate-level allows, and no `-A` flags in `RUSTFLAGS` or anywhere else. Fix the warning in the code.

2.2. CI runs with plain `RUSTFLAGS=-D warnings` and includes a `cargo clippy` step that must pass with zero warnings.

2.3. Zero warnings and zero errors are required to merge. A warning blocks a merge exactly like a failing test.

---

## 3. Reads like a book

3.1. Stepdown rule: each file opens with its highest-level function or system. The helpers it calls come directly below it, in the order they are called, so the file reads top-down.

3.2. One level of abstraction per function. A function either tells the story in game terms or does the detail work (math, ECS plumbing), never both.

3.3. Related code stays vertically close, and variables are declared next to their first use.

---

## 4. Functions

4.1. Each function does one thing, and its name says exactly what that thing is. If describing a function needs the word "and", split it.

4.2. Functions are small. Extract a helper as soon as a function mixes concerns such as input, validation, and mutation.

4.3. No boolean flag parameters that change behavior. Write `craft_masterwork(item)` and `craft_standard(item)` instead of `craft(item, true)`.

4.4. No hidden side effects. A function changes only what its name says it changes.

---

## 5. Names

5.1. Names are full words that read as a sentence at the call site, for example `if hunter.can_afford(recipe)`.

5.2. No abbreviations, and no vague names such as `data`, `info`, `tmp`, `manager`, or `handle_stuff`. A reader must get the intent from the name alone.

5.3. Names must make sense on their own in plain English to a reader who has never opened any doc. If a reader would need the glossary or any other doc to understand a name, that name breaks this rule.

---

## 6. Domain words

6.1. `docs/GLOSSARY.md` exists only for writers and reviewers, to keep each game concept spelled with one name everywhere. For example, never `ore` in one file and `mineral` in another for the same thing.

6.2. Readers of the code never need the glossary, and code must never depend on it to be understood. Rule 5.3 still applies to every name.

6.3. A new game concept gets its single name added to `docs/GLOSSARY.md` in the same pull request that introduces it.

---

## 7. Architecture (ECS)

7.1. One plugin or module per domain (`dungeon`, `forging`, `combat`).

7.2. One system per behavior.

7.3. Each module's `plugin.rs` registers its systems. Behavior lives in focused files next to it.

7.4. `main.rs` only wires plugins. It contains no game logic.

7.5. New weapons, armor sets, and recipes extend the game through data and types, not by editing core loops.

7.6. Shared traits (`Item`, `Recipe`) behave the same way for every implementor.

7.7. Prefer small traits and marker components over large "god" components.

7.8. Systems depend on resources and traits. Content is injected through plugins and data.

7.9. Behavior shared by the dungeon and the overworld lives once, in `core` or `items`. It is never duplicated.

7.10. No monolithic `update` functions.

7.11. No premature abstraction. Extract shared code only after the second real use case exists.

---

## 8. Error handling

8.1. Recoverable game logic returns `Option` or `Result`, not a panic.

8.2. `unwrap` and `expect` are used only for true programmer invariants, such as the player entity existing while the dungeon state is active.

8.3. An invalid player action never crashes the game. It fails quietly or shows feedback in the UI.

---

## 9. Player feedback

9.1. Every player action has its own sprite animation. No instant state flips for tool use, mining swings, eating, crafting feedback, or similar actions.

---

## 10. Tests

10.1. Every new or changed function has unit tests for the normal case, the edge cases, and the failure case.

10.2. Every system has a test that builds a small Bevy `App` with only the plugins it needs, runs `update()`, and checks the exact components, resources, or events that come out.

10.3. Every bug fix comes with a regression test that fails without the fix.

10.4. Every `.ron` data file in the repository has a test that loads and parses it.

10.5. Every player action has a test showing that a bad input fails safely and does not crash the game (see rule 8.3).

10.6. Tests call code the way the game does and check exact results, not internal details. A test that only checks that nothing panicked does not count. Test names read as sentences, for example `hunter_cannot_afford_recipe_without_enough_sparks`.

10.7. A pull request that changes behavior without a test that would catch the change is blocked.

10.8. Every save migration step has a test that loads a save from the previous version (see rule 12.2).

10.9. Every plugin has a smoke test: an `App` with that plugin registers its systems and resources, and one update runs without a panic.

10.10. Every id that a `.ron` file or the code uses to refer to an entry in another data file has a test proving that it resolves to a real entry.

10.11. Tests are deterministic. They use fixed random seeds and fixed time steps, never the wall clock, and pass the same way on every run.

10.12. Bevy glue stays thin, so the logic can be tested without an `App`.

10.13. A file's tests live in that file's own `#[cfg(test)]` module, so they ship in the same one-file pull request as the code they test (see rule 11.6).

---

## 11. Pull requests

11.1. The pull request test plan names the exact head commit SHA that was tested.

11.2. The test plan states that `cargo test --bin destiny_forge` passed with zero warnings and that a real `cargo run` passed with zero warnings. A docs-only pull request leaves out the `cargo run` result (see rule 11.8).

11.3. The test plan states that CI is green on that commit.

11.4. A missing or incomplete test plan blocks the pull request. A docs-only test plan without a `cargo run` result is complete (see rule 11.8).

11.5. Leave every file you touch cleaner than you found it, within the scope of the ticket.

11.6. Each pull request changes one file. The only exceptions are the uses of a renamed name in other files and the `.ron` data files that the rename touches. The pull request body lists each of those extra files.

11.7. A rename that touches `.ron` data files in the repository updates those files in the same pull request, and the game must still load in a real `cargo run`.

11.8. Every pull request gets a real `cargo run` before review, except docs-only pull requests, meaning pull requests that touch only `.md` files.

---

## 12. Saves

12.1. Any change to what is stored in a save, including a rename, gets its own step in the save migration chain.

12.2. Each migration step comes with a test that loads an old save and checks the migrated result.

12.3. `#[serde(rename)]` and `#[serde(default)]` are never used in place of a migration step. Any `serde(rename)` in a diff blocks the pull request.
