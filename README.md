# Descent: Null

An original 2D survival-exploration simulation. You are a test subject in a hostile underground. The design is systemic: physics, body, hunger, inventory, enemies, and environment should all create understandable consequences and decisions.

Inspired by the *gameplay philosophy* of systemic survival games, not a content clone. Original name, world, items, creatures, code, and art.

## Current status

The project is now beyond the engine-rebuild stage and is in a **gameplay clarity / core-loop refinement pass**.

What is real right now:

- Bevy 0.17 + Avian 2D physics: gravity, collision, slopes, jumping, falling, coyote time, jump buffering, camera follow.
- A procedurally generated four-layer cave that regenerates on every restart.
- A clear run objective: **descend to the bottom and recover the cargo**.
- A cave crawler enemy that chases and causes wounds through the same body pipeline as fall damage.
- A body simulation with bruises, lacerations, fractures, pain, bleeding, blood volume, unconsciousness, and death.
- Hunger, thirst, and stamina. Hunger/thirst reduce stamina recovery and survival capability; they no longer silently drain blood/health.
- A weight-limited inventory with fixed visible slots.
- Treatment with bandages, splints, food, water, and medkits.
- Crafting recipes that show both their ingredients and why the crafted item is useful.
- A larger, labelled HUD with percentages for health, hunger, thirst, and stamina.
- Explicit warnings for bleeding, fractures, starvation, dehydration, and low supplies.
- A persistent objective label and depth indicator.
- Larger player presentation, pickups, cargo, inventory slots, and HUD text for readability.
- Death and win overlays with run stats and a death cause summary.
- HUD state that restores correctly after restarting a dead/completed run.
- Ambient audio and camera shake on hard landings.
- Original pixel-art sprites and cave tiles.
- Unit tests across the simulation modules.

## Important current limitation

The player and cave art are still placeholder-quality relative to the intended final presentation. The next visual pass should redraw the character and standardize the pixel density / scale of player, terrain, items, enemies, and UI rather than continuing to enlarge mismatched source art indefinitely.

The inventory is currently a fixed visible grid, but not yet a fully selectable hotbar. `F` still uses the most relevant supply automatically and `C` still crafts the first affordable recipe. The next interaction milestone is explicit hotbar and recipe selection.

See `docs/REBUILD_PLAN.md` for the current gameplay-first roadmap.

## Run it

```bash
cargo test --release
cargo run --release
```

## Controls

- `A` / `D` or arrows — move
- `Space` — jump
- `E` — melee attack
- `F` — use the most relevant supply
- `C` — craft the first affordable recipe
- `R` — restart the run and generate a new cave
- `F3` — toggle developer debug overlay

## Current loop

Explore downward, collect supplies, read the HUD, treat injuries, craft what you need, decide how aggressively to descend, and recover the cargo at the bottom. Bleeding is the primary source of health loss and is called out directly on the HUD so the player knows both **why health is falling** and **what action stops it**.
