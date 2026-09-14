# Descent: Null

An original 2D survival-exploration simulation. You are a test subject
in a hostile underground. The design is systemic: physics, body, hunger,
inventory, and environment should all create consequences.

Inspired by the *gameplay philosophy* of games like Casualties: Unknown —
not a copy. Original name, world, items, creatures, and code.

## Honest status

**Milestone 1 (player + physics), plus a first slice of milestone 2.**

What is real right now:

- Bevy 0.17 + Avian 2D physics
- Gravity, collision, slopes, jumping, falling, landing velocity
- Coyote time and jump buffering
- Camera follow
- Pushable debris
- Landing-impact events, now wired into a real body sim: a hard landing
  wounds both legs (bruise / laceration / fracture depending on impact
  speed), and each wound contributes ongoing pain and — above laceration
  severity — bleeding. Totals show on the HUD. `R` clears wounds along
  with the position/velocity reset.
- Unit tests for jump assist, landing severity, and the wound/body model

What is **not** in this build yet: cardiovascular simulation (blood
volume, heart rate, shock, consciousness, death), infection, treatment or
healing, hunger/thirst, inventory, procedural caves, crafting, enemies,
save/load.

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
prints severity on the HUD, and hard enough landings actually wound your
legs — watch the pain and bleeding totals climb. Cardiovascular
consequences (shock, KO, death) aren't wired up yet.
