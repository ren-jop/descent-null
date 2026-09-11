# Descent: Null — rebuild plan

Internal architecture assessment after a full-repo audit. Original identity only; systemic inspiration, not a content clone.

## Current architecture

The prototype is a **discrete grid simulation** with two front ends, not a physics game.

- **Engine:** none. Library crate (`src/*.rs`) plus `macroquad` window (`src/main.rs`) and a leftover `ratatui` terminal binary (`src/bin/tui.rs`).
- **Loop:** front end polls WASD, calls `GameState::try_move(dx, dy)` one tile at a time, then `GameState::tick()` every 2 seconds for survival/body/AI.
- **World:** 40×24 cellular-automata cave of enum tiles (`Empty` / `Wall` / `Water` / `Hazard` / `Resource` / `Extraction`). Flood-fill checks one start→extraction path.
- **Player:** integer `(x, y)` plus `BodyState`, `SurvivalState`, `Inventory`, three float attributes, integer `fall_distance`.
- **Physics:** none. Gravity, acceleration, collision, slopes, and landing velocity do not exist. “Fall damage” counts consecutive down-tile steps.
- **Entities:** one `Enemy` struct with Manhattan chase, contact injury, and a separate `health` float.
- **Inventory:** `Vec<ItemStack>` with weight + stack caps. No hands, no nested containers, no world objects.
- **Items / crafting:** match-arm `ItemId` / `RecipeId`. Two recipes. No liquids, no properties, no duration.
- **Body:** six regions with floats (`condition`, `bleeding`, `pain`, `infection`) plus a boolean fracture. No wound objects. Cardio is four floats ticked monolithically inside `BodyState::tick`.
- **UI:** colored grid + status bars. No body inspection panel, no drag inventory.
- **Assets / data files:** none. Everything is hard-coded Rust.
- **Save:** none.
- **Tests:** 34 unit tests that prove the *grid* prototype’s own rules. They are not the target design.

## Current problems

The game drifted into “Roguelike ASCII / colored tiles with status bars.” That is the opposite of a physical survival simulation.

1. Movement is grid navigation. There is no weight, inertia, coyote time, slopes, or landing velocity.
2. Injury is “subtract condition on this enum.” There are no wounds, no impact position, no force.
3. Cardiovascular state exists but is a fake HP cousin: one tick function, no events, no visual/audio coupling, no location-specific impacts.
4. Survival bars decay on a 2s cadence and barely change movement (a speed multiplier on grid steps).
5. Inventory is a bag of IDs, not a physical carrying problem.
6. World generation is a tiny 2D maze. No shafts, chambers, lighting, or continuous space.
7. Enemies are “step toward player, overlap = damage.” Separate enemy HP vs player body.
8. Extraction tile *wins the run*. The intended loop is descend / retreat / die / learn.
9. Macroquad GUI is a skin on the same grid loop. It cannot host real physics.
10. One giant `GameState` update path instead of events between systems.

## Systems worth keeping (ideas, not files)

Preserve these *intents* and some numeric instincts. Do **not** keep the grid types as the runtime.

| Intent | Why it is still valid |
| --- | --- |
| Six named body regions | Matches the target physiology |
| Cardio fields (volume, HR, shock, consciousness) | Right *categories*; need a real resolver |
| Survival categories (hunger, thirst, stamina, temp, fatigue, mood) | Right *categories*; must actually couple to physics |
| Strength / resilience / intelligence | Keep as the three attributes |
| Deterministic seed + connectivity check | Required for procedural caves (rewrite the generator) |
| Transactional craft (all-or-nothing consume) | Keep the rule when recipes become data |
| Original name, world, items, creatures | Non-negotiable originality |
| Test-the-simulation culture | Keep; rewrite tests to match new architecture |

## Systems that should be rewritten

- `world.rs` cave generator — keep CA/flood-fill *ideas*, output colliders + layers, not 40×24 glyphs.
- `body.rs` — replace region floats with wound lists + evented cardio.
- `survival.rs` — same fields, driven by dt and events, feeding movement/physics.
- `inventory.rs` / `crafting.rs` — physical slots, nested containers, data-driven defs.
- `entities.rs` — perception FSM, no fake HP.
- `player.rs` / `game.rs` — ECS entities + events, not one struct.
- Front ends — Bevy 2D + Avian physics. Terminal UI is not the product.

## Systems that should be removed

- Tile-step `try_move` and integer `fall_distance`.
- Macroquad and ratatui/crossterm binaries as the game.
- Extraction-as-win condition.
- Enemy `health` as a second HP system.
- Hard-coded `ItemId` match arms as the long-term item architecture.
- The 34 grid tests as a compatibility target (replace with tests for the new sim).

## Proposed architecture

Engine: **Bevy 0.17** + **avian2d 0.4** (ECS, fixed physics step, events/messages).

```
src/
  main.rs                 -- app entry
  lib.rs
  game/                   -- plugins, run state, schedules
  player/                 -- controller wiring, camera, spawn
  physics/                -- jump assist, landing, character controller
  world/                  -- arenas now; procedural caves later
  generation/             -- (M5)
  entities/ ai/ combat/   -- (M7)
  items/ inventory/       -- (M4)
  crafting/ medical/      -- (M6)
  survival/               -- (M3)
  ui/ audio/ save/ debug/ -- progressive
```

Communication is **event-driven**. Example for later milestones:

`LandingImpact` → damage resolver → `WoundCreated` → bleeding → `BloodLost` → cardiovascular → `ConsciousnessChanged` → movement penalties.

Schedules stay split: physics (fixed), survival (dt), render, UI.

## Implementation order

| Milestone | Playable when | Depends on |
| --- | --- | --- |
| **1. Player + physics** | Walk, jump, fall, land, camera, push objects, slopes | Engine swap |
| **2. Body simulation** | Location-specific wounds, bleed, pain, fracture, cardio, KO, death | Landing/combat damage events from physics |
| **3. Survival** | Hunger/thirst/fatigue/stamina/temp change movement and recovery | Body + dt |
| **4. Inventory** | Hands, equipment, nested containers, weight | Physics objects in world |
| **5. World** | Seeded caves, chambers, shafts, loot, 5 layers | Physics colliders + inventory spawn |
| **6. Crafting + medical** | Timed treatment on a body part; recipes consume real items | Body + inventory |
| **7. Enemies + hazards** | Perception AI, location damage, avoid-or-fight | Body + world + combat events |
| **8. Polish** | HUD/panels, feedback architecture, death recap, save | All of the above |

Do not start the next milestone until the previous one is actually playable.

## Dependencies between systems

```
physics ──────────────► body (impact, force, location)
physics ──────────────► inventory (world items, throw, drop)
body ─────────────────► physics (limp, climb fail, KO)
survival ─────────────► body (shock, infection, temperature)
survival ─────────────► physics (stamina / hydration movement)
inventory ────────────► physics (mass, throwables)
inventory ────────────► medical / crafting
world/generation ─────► physics colliders, loot, hazards
ai/combat ────────────► damage events ─► body
ui/audio ─────────────► reads all; never owns sim truth
save ─────────────────► serializes player, world seed, containers, wounds
```
