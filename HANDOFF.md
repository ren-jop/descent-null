# Handoff prompt: continuing Descent: Null

Paste everything below this line to another AI (or a fresh session with
me) along with the project files, to continue development from exactly
where this session left off.

---

You are an expert Rust game programmer and systems designer. You are
continuing development of an existing, working project called
**Descent: Null** — an original 2D survival-exploration simulation
inspired by the systemic design of games like *Casualties: Unknown*
(per-limb injury, cardiovascular simulation, procedural descent,
crafting, permadeath). It is an original game: original name, world,
items, creatures, and code. Do not introduce any copied names, assets,
characters, dialogue, maps, or proprietary content from any existing
commercial game.

## What already exists and works

The attached project has a `descent_null` library crate with **34
passing automated tests** (`cargo test --release`) covering:

- `src/body.rs` — six independently injurable body regions (head, torso,
  4 limbs) with condition/bleeding/fracture/infection/pain, plus a
  cardiovascular model (blood volume, heart rate, shock, consciousness)
  that cascades toward death if untreated
- `src/survival.rs` — hunger/thirst/stamina/temperature/fatigue/mood,
  feeding back into the body sim
- `src/world.rs` — procedural cave generation (cellular automata +
  smoothing) with a flood-fill guarantee that every layer is solvable
- `src/inventory.rs` — weight-and-stack-constrained item storage
- `src/crafting.rs` — data-driven, transactional recipes
- `src/entities.rs` — one enemy with sense/chase/patrol AI
- `src/player.rs`, `src/game.rs` — tie the above into a single tick loop
- `src/main.rs` — a terminal (crossterm + ratatui) front end; this is the
  ONLY file that knows the game is currently text-based

The lib/bin split is deliberate and must be preserved: all gameplay
logic lives in library modules with zero rendering/input code, so it's
independently testable and portable to a different front end later.

**Before changing anything, run `cargo test --release` and confirm all
34 tests still pass.** Do not modify existing test assertions to make
them pass — if a test starts failing, that's a real regression to fix
in the logic, not the test (unless you find the test's premise was
actually wrong, as happened once already with the fall-damage test
during initial development — document any such case clearly if it
comes up again).

## Development rule — follow this strictly

1. Before adding a feature, run `cargo test --release` — confirm green.
2. Implement ONE system or one meaningful slice of a system.
3. Add tests for it BEFORE or alongside the implementation, not after
   the fact as an afterthought.
4. Run `cargo build --release` — fix all warnings, not just errors.
5. Run `cargo test --release` — confirm still green, including the
   pre-existing 34.
6. Only then move to the next item below.

Never generate thousands of lines of untested code in one pass. If a
feature is large, split it into smaller milestones and stop to verify
between them. At the end of your response, always state plainly: what
files changed, what was implemented, how to run it, how to test it,
known limitations, and what's next.

## Milestones, in priority order

### Milestone A — more layers, real depth progression
- Extend `world.rs` so a `Layer` carries a `depth: u32` and per-depth
  generation parameters (wall density, hazard density, ambient temp)
  instead of the single hard-coded config in `game.rs::for_depth`.
- When the player reaches `Tile::Extraction`, instead of ending the run,
  generate the next layer at `depth + 1` and drop the player at its
  entrance, carrying over their body/survival/inventory state.
- Target 5 distinct depth "bands" with visibly different parameters
  before attempting anything like the original spec's 11.
- Tests: generation determinism and solvability must hold at every
  depth band, not just depth 1.

### Milestone B — more items and a real crafting tree
- Expand `inventory.rs`'s `ItemId`/`item_def` and `crafting.rs`'s
  `RecipeId`/`recipe` — these are already data-driven, so this should be
  mostly additive, not structural.
- Add at least one multi-tier recipe (an output of one recipe is an
  input to another) to prove the data-driven design scales.
- Consider moving item/recipe definitions to an external data file
  (RON, JSON, or TOML via `serde`) rather than Rust match arms, per the
  original spec's "data-driven design" goal — this is a good moment to
  make that change since the item count is still small.

### Milestone C — more enemies and real combat
- `entities.rs` currently has one enemy archetype. Add at least 2 more
  with genuinely different behavior (not just different stats) — e.g.
  one that flees at low health, one that only attacks from range.
- Give the player a way to fight back (a simple weapon item that, when
  used against an adjacent enemy, reduces its `health`), not just be
  attacked.
- Tests: verify each behavior variant actually behaves differently
  given the same inputs (this is the same style as the existing
  `entities::tests`).

### Milestone D — real physics and a graphical front end
This is the biggest lift and should come after A–C are solid.
- Recommended: migrate the front end (only — library stays engine-
  agnostic) to **Bevy**, using its ECS and physics (avian2d or
  bevy_rapier2d) for actual continuous movement, gravity, and collision,
  replacing the current grid-step movement in `game.rs::try_move`.
  This will require restructuring `try_move`'s discrete-tile logic into
  something a continuous physics step can drive — expect this to touch
  `game.rs` and `player.rs` meaningfully, not just `main.rs`.
- Do this on a machine with a current Rust toolchain (1.80+). Bevy and
  its ecosystem will not build on old/frozen Rust installs — confirm
  `rustc --version` is current before starting.
- Keep the existing terminal front end working as a lightweight/debug
  mode if practical — it's a fast way to test logic changes without
  waiting on a full graphical build.

### Milestone E — save/load, difficulty settings, polish
- Add `serde` derives to the core state structs and implement
  save/load, versioned so a future format change doesn't break old
  saves.
- Expose the difficulty-relevant constants currently hard-coded in
  `game.rs` (fall damage scaling, hunger/thirst decay rates, enemy
  aggression) as a `DifficultySettings` struct with a few presets.
- Audio, a real UI, and further content come after this, once the
  systems are all in place and tested.

## Constraints that apply to every milestone

- Original content only — no names, characters, items, or assets from
  any existing commercial game.
- Every new system needs tests before you consider it done. "It looks
  right when I ran it once" is not sufficient — that's exactly the kind
  of thing this project's test suite exists to catch.
- Keep the library crate free of any rendering/input dependency. If
  you're tempted to add a `use bevy::...` or `use macroquad::...` inside
  a file under `src/` other than `main.rs`, stop and reconsider the
  boundary.
- Compile and test after every milestone, not just at the end.
