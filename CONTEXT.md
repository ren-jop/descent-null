# Context for anyone (human or AI) picking this project up

This file exists because most of this codebase was written by an AI
assistant (Claude) working from a sandboxed environment with **no
working Rust/Bevy toolchain** — every change described below was
written by careful reading and pattern-matching against the existing
code, not verified by compiling. If you're picking this up, read
"Known risk areas" before you read anything else.

## What this project is

An original 2D survival-exploration game: descend through a
procedurally generated cave, manage injuries/hunger/thirst/stamina,
scavenge and craft, recover an objective (cargo) from the bottom, and
get out (currently: reach it — return trip isn't implemented).

It's inspired by the *gameplay philosophy* of games like Casualties:
Unknown — systemic consequences, real injuries, resource pressure — but
is **not a copy**. Original name, world, items, code, and art (the
sprites in `assets/sprites/` were generated for this project
specifically; no assets from any other game are used anywhere).

Stack: Bevy 0.17 (`bevy = { version = "0.17", default-features = false,
features = [...] }`, see Cargo.toml for the exact feature set),
Avian2D 0.4 for physics, `rand` 0.8 for procedural generation.

## Module map

```
src/
  physics/    character controller, jump assist, landing detection.
              engine-agnostic logic (jump.rs, landing.rs) separated
              from Bevy wiring (controller.rs). no dependency on
              anything else in the crate.
  body/       wounds + cardiovascular model. region.rs/wound.rs/
              cardio.rs/state.rs are pure logic with unit tests;
              integration.rs is the Bevy Component/Plugin/systems.
              depends on physics (reads LandingImpact messages).
  survival/   hunger/thirst/stamina. same pure/wiring split. depends
              on body (starvation/dehydration drain blood volume) and
              physics (reads CharacterController, ButtonInput).
  items/      item kinds, weight-limited inventory, crafting recipes,
              pickup/use/craft systems, and LastEvent (a fading toast
              resource — pickups, item use, crafting, combat all report
              through it). depends on player (Player marker), body,
              survival.
  enemy/      one enemy type (cave crawler): idle/chase/attack, pure
              distance→state logic in state.rs with tests, Bevy wiring
              in integration.rs. bites call body::landing_wound directly
              (reuses the fall-damage wound pipeline instead of a
              separate damage system). depends on body, player, items
              (LastEvent for combat text).
  player/     spawns the player entity, camera follow, nametag,
              spawn-hint toast timer, R-to-reset. depends on body,
              survival, items, physics, world (RunStats).
  world/      procedural cave generation (cave.rs), depth tracking, run
              stats, the extraction objective, enemy spawn placement.
              depends on items (spawns Pickups), player (tracking),
              enemy (spawn_enemy).
  ui/         all HUD/overlay rendering. depends on nearly everything
              else (it displays their state) — nothing depends on ui.
  game/       GamePlugin — wires every other plugin into one App.
```

Note `items` <-> `player` and `player` <-> `world` are mutual module
references (e.g. `items` reads `player::Player`, `player` reads
`items::PlayerInventory`). This is fine — Rust only requires *crate*
dependencies to be acyclic, not module references within one crate —
but it's easy to mistake for a mistake if you're used to other
languages' module systems. It isn't one.

Every gameplay module with non-trivial math (body, survival, items)
keeps that math in plain Rust structs with `#[cfg(test)] mod tests`,
separate from the Bevy `Component`/`Plugin`/`System` wiring. That
split is deliberate and has held up well — keep it if you extend these
systems. It's also the reason the pure-logic files are the
*lowest*-risk code in this repo (they're genuinely unit-tested) while
the Bevy wiring files are the highest-risk (never executed by anyone
but you).

## Current status (phase-by-phase, per docs/REBUILD_PLAN.md's original
milestone numbering plus the later survival-loop design doc's phases)

**Done:**
- Physics: gravity, collision, slopes, jump buffering/coyote time,
  landing-impact detection.
- Body: six regions, wounds (bruise/laceration/fracture by impact
  severity), pain, bleeding, a blood-volume/consciousness/death model,
  treatment (bandage/splint/medkit via `F`).
- Survival: hunger/thirst drain over real time and feed the same
  blood-volume pipeline once empty; stamina drains from movement and
  caps speed when exhausted.
- Procedural cave: 4 depth layers, randomized platform gaps, side
  ledges, per-layer loot tables, a guaranteed-reachable catch-floor at
  the bottom.
- Items: weight-limited inventory, proximity auto-pickup, 3 crafting
  recipes (`C`).
- An actual win condition: a Cargo objective on the bottom floor.
  Reaching it sets `RunStats.extracted` and shows a win overlay. This
  exists specifically because early testing found runs had no way to
  *finish* — every run was guaranteed to reach the bottom floor
  eventually (that's what it's for), it just had nothing there before.
- Original pixel-art sprites for the player, per-depth rock tiles,
  every item, the enemy, and HUD vital icons, generated with Pillow —
  see `assets/sprites/`. The generator script is a throwaway Python
  file in the authoring sandbox, not part of this repo; if you want
  more sprites in the same style, ask for them regenerated.
- One enemy (cave crawler): idle/chase/attack driven by a pure
  distance-threshold function (`enemy::state::state_for_distance`,
  unit-tested), spawned probabilistically on layer-1+ platforms during
  cave generation (not a separate showcase room). Bites call
  `body::landing_wound` directly — a bite is mechanically the same kind
  of wound a bad fall causes, just triggered from a different place.
  Player fights back with `E` (proximity-based melee, no animation).
- A redesigned HUD: bottom-left icon+bar vitals cluster (green health,
  smaller hunger/thirst stacked above it, stamina only while moving),
  bottom-right inventory icon grid (9 slots, quantity numbers), a small
  persistent depth label, and fading toasts (spawn instructions, depth
  changes, pickup/craft/combat events via `items::LastEvent`).
- A developer debug overlay (`F3`, off by default) holding the old raw
  numbers (velocity, wound/pain/bleed figures, raw inventory list) that
  used to be permanently on screen. A single `HideOnDeath`-tagged,
  chain-ordered system forces every normal HUD element *and* the debug
  panel to `Display::None` during death/win, regardless of the debug
  toggle's own state — added specifically so debug text and normal HUD
  elements can't show through the (96%, not 100%, opaque) death/win
  overlay backgrounds.
- A floating "Player One" nametag above the character.

**Not done (roughly Phase 2+):**
- Environmental hazards beyond falling and the one enemy (spikes,
  traps, toxic areas).
- More than one enemy type; ranged or varied enemy behavior.
- Light/darkness as an actual visibility-gating mechanic (there's a
  purely cosmetic vignette, but it doesn't hide anything).
- Random events.
- NPCs.
- A return trip / real extraction (right now "extraction" is just
  reaching the cargo, not carrying it back out).
- Regenerating the cave layout on `R` (currently `R` resets the
  player but the cave itself is fixed for the life of the process).
- Any audio.
- Actual CC0 asset integration (Kenney/OpenGameArt) — reviewed for
  license/style per project direction, but the authoring sandbox's
  network egress is allowlisted and doesn't include kenney.nl or
  opengameart.org, so nothing could actually be downloaded. Everything
  visual is original art generated for this project instead.

## Key tunable numbers (all in the relevant `state.rs`/`cardio.rs`,
search for the constant name if you want to retune)

- `KO_THRESHOLD` 40% / `DEATH_THRESHOLD` 15% blood volume
  (`body/cardio.rs`).
- **Bleed rates were catastrophically miscalibrated for multiple
  rounds** (`body/wound.rs::LACERATION_BLEED_PER_SEVERITY`/
  `FRACTURE_BLEED_PER_SEVERITY`). `blood_volume` is a 0..1 fraction and
  `bleed_rate` drains it directly per second — the original constants
  (0.8 / 1.6) meant a single severe landing could put two fractured
  legs at a combined 3.2/s, i.e. total blood loss in under a third of a
  second. Reported as "I die for no reason." Fixed to 0.025 / 0.05, so
  a worst-case wound takes on the order of 10-30 real seconds to become
  fatal if ignored — still urgent, but reactable. If you ever touch
  these again, sanity-check the actual seconds-to-death by hand before
  committing; the failure mode (a number that's fine in isolation but
  implies near-instant death against a 0..1 pool) is not obvious from
  reading the constant alone.
- Hunger empties in 3 min, thirst in 2.5 min
  (`survival/state.rs::HUNGER_DRAIN_PER_SEC`/`THIRST_DRAIN_PER_SEC`) —
  sped up from 5/4 min because a short session never saw any effect
  from them ("hunger and thirst don't really do anything"). They also
  now have a graduated effect (`hunger_hardship`/`thirst_hardship`,
  ramping from 0 to 1 below 40%) that softens stamina regen smoothly
  instead of only doing anything at an exact-zero cliff.
- Stamina: was originally tuned far too aggressively (emptied from
  ordinary walking in ~4s, and regen nearly stalled once thirst hit
  zero) — this made the game nearly unplayable and was reported and
  fixed. Current numbers (`STAMINA_DRAIN_PER_SEC` 0.05,
  `STAMINA_REGEN_PER_SEC` 0.25) are deliberately gentle. If you retune
  this, playtest walking normally for 30+ seconds before committing —
  that exact regression is easy to reintroduce by "balancing" in the
  wrong direction.
- Cave layout constants (`LAYER_COUNT`, `LAYER_HEIGHT`,
  `PLATFORMS_PER_LAYER`, gap ranges) are in `world/cave.rs`. The cave
  now regenerates on every `R` restart (`regenerate_on_restart`,
  despawns everything tagged `CaveObject` and calls `generate_cave`
  again) — it's no longer fixed for the life of the process.

## A pattern worth noticing

Several of the bugs reported so far (stamina lockup, catastrophic bleed
rate) share a shape: a constant that looks reasonable in isolation but
implies an absurd real-world rate once you actually compute "how many
seconds does this take." Both were caught by the person playing, not by
review. If you're adding a new drain/regen/damage constant, do the
seconds-to-empty or seconds-to-death arithmetic by hand before
committing, the same way this file now asks you to for bleed rates
specifically. It's cheap and would have caught both bugs immediately.

## Known risk areas (things that have never been compiled)

Every commit in this repo's history past the initial two (the ones
already on `origin/master` before the AI started) was written without
a working Bevy-capable Rust toolchain in the authoring sandbox (only
an old apt-provided rustc, far below what Bevy 0.17 needs; rustup's
install domains were network-blocked). The person on the other end
has been compiling locally, but **as of this handoff, no build/test
output has actually been reported back yet across several rounds of
changes** — don't assume any of this compiles, let alone plays well,
until you've actually run `cargo test --release && cargo build
--release` yourself and looked at the result.

Highest-risk code, roughly in order:
1. `ImageNode::new(handle)` / `.image` field (used for the HUD's vital
   icons and inventory-slot icons in `ui/mod.rs`) — this is a
   reasonable-confidence guess at Bevy 0.17's UI-image API, not a
   certainty. If the HUD doesn't compile, check this first.
2. `ui/mod.rs` generally — six uses of `.with_children()` for nested UI
   (toasts and overlays), the icon+bar absolute-positioning layout, and
   the `HideOnDeath` + chained-system-order trick for forcing every HUD
   element off during death/win. None of it has ever been rendered.
3. `enemy/integration.rs` — a second `RigidBody::Dynamic` archetype
   (the first being the player's) with its own collider/physics
   tuning. Chase speed, attack range, and bite damage are first-guess
   numbers, not playtested.
4. `world/cave.rs` — procedural generation math, now also placing
   enemies. Could compile fine and still produce a bad run (unreachable
   platforms, enemies clustered unfairly, an unreasonable cargo
   position) without ever throwing a Rust error. Playtest, don't just
   build-check.
5. `player/mod.rs` — `Text2d` for the nametag is a newer-ish Bevy API
   surface, used here for the first time.
6. Anything using `Sprite { image: ..., ..default() }` (textured
   sprites) — the `image` field name on `Sprite` is a reasonable-
   confidence guess based on Bevy's post-0.14 Sprite unification, not
   a certainty for 0.17 specifically.
7. `game::spawn_ambience` — `AudioPlayer::new(handle)` +
   `PlaybackSettings::LOOP` for the background drone. First (and only)
   use of `bevy_audio` in this repo; deliberately did NOT also call
   `.with_volume(...)` since I was unsure of that API's exact shape in
   this version — the track's volume is instead baked quiet at
   generation time. If audio doesn't play, check that the `wav` and
   `bevy_audio` Cargo features are both present (Cargo.toml).
8. `player::apply_camera_shake` — the camera's actual Transform is now
   written *only* by this one system, computed fresh each frame as
   `CameraFollow.position + jitter`. If you add another system that
   writes the camera's Transform directly (instead of going through
   `CameraFollow`), you will reintroduce drift — this was the whole
   reason the resource exists instead of jittering the Transform
   in place.

One mistake already caught and fixed twice in this history: writing
`const SOME_COLOR: Color = Color::srgb(...)` at module scope. Bevy's
`Color::srgb` may or may not be a `const fn` in this version — rather
than gamble, every `Color` value in this codebase is built inside a
function body (a `let` binding or inline literal) at runtime instead.
Keep doing that; don't reintroduce a module-level `const … : Color`.

## Workflow notes for continuing this collaboration

- The person is on a Mac, comfortable with basic terminal commands but
  not a fluent Rust/git user — spell out exact commands, don't assume
  familiarity with `git am`, path expansion, etc.
- Git patch files (`git format-patch`) turned out to be unreliable —
  they got mangled somewhere in the browser download/save round-trip
  and failed to apply. **Zip snapshots of the whole repo are the
  reliable handoff mechanism** — the person unzips and copies
  `src/`, `assets/` (when it changes), `Cargo.toml` (when it changes),
  and `README.md` over their real local clone, preserving their `.git`
  folder. Don't hand over a zip and expect them to `git am` it.
- Every round has ended with: commit locally (in the sandbox clone),
  zip the repo minus `.git`, hand it over with copy-paste-ready `cp`
  commands plus `cargo test --release && cargo build --release`, and
  ask for the build output before assuming anything works.
- README.md's "Honest status" section is the living source of truth
  for what's actually implemented vs. planned — keep it updated
  alongside code changes, the same way it's been kept updated every
  round so far. Don't let it drift into aspirational claims.
