# Descent: Null

An original 2D survival-exploration simulation. You are a test subject
in a hostile underground. The design is systemic: physics, body, hunger,
inventory, and environment should all create consequences.

Inspired by the *gameplay philosophy* of games like Casualties: Unknown —
not a copy. Original name, world, items, creatures, and code.

## Honest status

**Milestone 1 (player + physics), milestone 2's core loop, and a first
slice of milestone 3.**

What is real right now:

- Bevy 0.17 + Avian 2D physics
- Gravity, collision, slopes, jumping, falling, landing velocity
- Coyote time and jump buffering
- Camera follow
- Pushable debris
- Landing-impact events wired into a real body sim: a hard landing wounds
  both legs (bruise / laceration / fracture depending on impact speed),
  and each wound contributes ongoing pain and — above laceration severity
  — bleeding.
- A real cardiovascular consequence: bleeding drains blood volume every
  frame (with a small passive regen when nothing is actively bleeding).
  Drop below 40% and you're unconscious — no movement or jump input, just
  gravity and collision. Drop below 15% and you're dead.
- Hunger and thirst drain over real time (placeholder pace: minutes, not
  days — there's no food/water items to restore them yet). Once either
  hits zero it also drains blood volume, feeding the same KO/death path
  as bleeding — starve or dehydrate long enough and you black out or die
  exactly the way bleeding out does.
- `R` fully resets: position, velocity, wounds, blood volume, hunger, and
  thirst — the respawn side of a die → respawn loop.
- Unit tests for jump assist, landing severity, the wound/body model, the
  cardio model, and the hunger/thirst model.

What is **not** in this build yet: infection, treatment/healing/food-and-
water items, stamina/fatigue, temperature, inventory, procedural caves,
crafting, enemies, save/load, any UI beyond the plain-text HUD.

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
wounds your legs — watch pain, bleeding, and blood volume on the HUD.
Bleed out, or just wait out your hunger and thirst, and you'll pass out
and then die. `R` respawns you fully healed and fully fed.
