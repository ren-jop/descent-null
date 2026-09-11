# Descent: Null

An original 2D survival-exploration simulation. You are
a test subject descending through a procedurally generated underground
environment, managing per-limb injuries, a real cardiovascular system,
hunger/thirst/stamina, a weight-limited inventory, and basic crafting.
Built independently, inspired by the systemic design of games like
*Casualties: Unknown* — not a copy of it. Original name, world, items,
creature, and code throughout.

## Honest status

This is a **vertical slice**, not the full game. What's real and verified:

- Full body simulation: 6 independently injurable regions (head, torso,
  4 limbs), each tracking condition/bleeding/fracture/infection/pain
- A real cardiovascular model: blood volume, heart rate, shock, and
  consciousness all interact — bleeding doesn't just subtract a number,
  it cascades into weakness and eventually death if untreated
- Hunger/thirst/stamina/temperature/fatigue/mood, feeding back into the
  body sim (starvation adds cardiovascular shock; extreme temperature
  damages condition over time)
- Procedurally generated caves (cellular automata + smoothing), with a
  flood-fill guarantee that every generated layer is actually solvable —
  checked against 100 seeds in the test suite
- A weight-and-stack-constrained inventory and a transactional crafting
  system (a failed craft never partially consumes ingredients)
- One original enemy with sense/chase/patrol behavior
- Fall damage based on actual drop distance before landing
- **34 automated tests**, all passing, covering the above — this isn't
  just code that compiles, it's code whose actual behavior (cascading
  injury, starvation, fair map generation, inventory limits, crafting
  transactions) is checked.

What's *not* here yet: the 11 layers, ~300 items, three playable
characters, real continuous physics (the GUI is still grid-step), audio,
and save/load — see `HANDOFF.md` for the plan to keep building those.

A **graphical window** (macroquad) is now the default front end. The
original terminal UI is still available as a debug binary. The library
modules are unchanged and still have no rendering/input code.

## Run it

```
cargo test --release                 # confirms all 34 tests still pass
cargo run --release                  # graphical window
cargo run --release --bin descent_null_tui   # original terminal UI
```

## Controls

- `w a s d` or arrows — move (careful near ledges: falling several tiles
  and landing causes real injury)
- `e` — eat a ration    `f` — drink purified water
- `c` — craft a bandage from fiber (if you have 2+ fiber)
- `t` — open the treatment menu, pick an item then a body region to apply
  it to (or use the on-screen Eat / Drink / Craft / Treat buttons)
- `r` — start a new run (after extraction or death)
- `q` / `Esc` — quit

## Reading the map

In the window: yellow circle is you, red circle is a hostile creature,
grey blocks are walls, gold is extraction. Green / cyan / magenta /
yellow tiles are food, water, medical, and scrap. Dark blue is untreated
water; bright red is a hidden hazard.

Terminal glyphs (debug UI): `@` you · `#` wall · `~` water · `^` hazard ·
`*` food · `o` water · `+` medical · `%` scrap/fiber · `E` extraction ·
`x` hostile creature

## Project layout

```
src/
  lib.rs        -- module declarations, no logic itself
  body.rs        -- per-region injury + cardiovascular simulation
  survival.rs     -- hunger/thirst/stamina/temperature/mood, feeds into body.rs
  world.rs       -- procedural cave generation, solvability-checked
  inventory.rs    -- weight/stack-constrained item storage
  crafting.rs     -- data-driven, transactional recipes
  entities.rs     -- enemy AI
  player.rs       -- ties body/survival/inventory/attributes together
  game.rs        -- top-level tick loop, the thing a front end drives
  main.rs        -- graphical window (macroquad)
  bin/tui.rs     -- original terminal UI (crossterm + ratatui)
```

The lib/bin split is deliberate: `cargo test --release` runs the entire
simulation without ever touching a window or the terminal, so the game
logic is fully verifiable independent of any rendering choice. Neither
front end lives in the library modules. A later Bevy/physics rewrite
(see HANDOFF milestone D) can still replace only the front end plus
`try_move`.

## Continuing this

See `HANDOFF.md` for a milestone-based prompt written for another AI
coding agent (or a future session with me) to pick this up and keep
building — more layers, more items, real physics, a GUI, the works.
