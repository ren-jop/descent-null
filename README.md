# Descent: Null

An original 2D survival-exploration simulation. You are a test subject
in a hostile underground. The design is systemic: physics, body, hunger,
inventory, and environment should all create consequences.

Inspired by the *gameplay philosophy* of games like Casualties: Unknown —
not a copy. Original name, world, items, creatures, and code.

## Honest status

**Milestone 1 only: player + physics.**

What is real right now:

- Bevy 0.17 + Avian 2D physics
- Gravity, collision, slopes, jumping, falling, landing velocity
- Coyote time and jump buffering
- Camera follow
- Pushable debris
- Landing-impact events (not yet wired into a body sim)
- Unit tests for jump assist and landing severity

What is **not** in this build: body/limb wounds, cardiovascular play,
hunger/thirst, inventory, procedural caves, crafting, enemies, save/load.

See `docs/REBUILD_PLAN.md` for the full overhaul plan.

## Run it

```
cargo test --release
cargo run --release
```

## Controls

- `A` `D` or arrows — move (acceleration, not grid steps)
- `Space` — jump
- `R` — reset to the starting ledge

Walk the ramps, push crates, jump from the high perch. A hard landing
prints severity on the HUD. Injuries start in milestone 2.
