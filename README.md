# Descent: Null

An original 2D survival-exploration game about descending into a hostile underground, managing injuries and supplies, recovering mission cargo, and making it back out alive.

## Current build

The project is in a **gameplay clarity / core-loop refinement pass**. The goal of the current work is to make the first five minutes understandable without filling the screen with instructions.

What is implemented now:

- Bevy 0.17 + Avian 2D physics with jumping, falling, landing damage, coyote time, jump buffering and camera follow.
- A procedurally generated four-layer cave that regenerates on every restart.
- A two-part mission: **reach the cargo at the bottom, then return it to the EXIT at the surface**.
- Catastrophic falls are fatal, so dropping straight to the bottom is no longer a shortcut.
- Cave crawlers appear from layer 1 onward, notice the player from farther away, and the cargo chamber is guarded.
- Spike hazards on deeper layers create real wounds and bleeding.
- Rare **Null Surge** pickups temporarily increase movement acceleration.
- A body simulation with bruises, lacerations, fractures, pain, bleeding, blood volume, unconsciousness and death.
- Hunger, thirst and stamina affect survival without silently draining health.
- A weight-limited backpack with a selectable 1-9 hotbar. The hotbar is slots; the `12` limit is explicitly labelled as **pack weight**, not slot count.
- `F` uses the currently selected item instead of automatically choosing one for you.
- `C` opens a dedicated crafting overlay. Recipes are selected with `1`, `2`, or `3` and state exactly what they make and why the item is useful.
- A compact normal HUD: short mission text, depth, compact vitals and hotbar only.
- Contextual warnings appear only when something needs attention, such as bleeding or a fracture.
- First-run onboarding teaches movement, jumping, pickups and the hotbar progressively, then gets out of the way.
- `Esc` opens a pause/help menu with Resume, Restart, Quit and control instructions.
- Death and mission-complete screens show run information and restore cleanly after restart.
- Pixel art uses nearest-neighbour sampling so scaled sprites remain crisp.
- The floating player nameplate was removed and the player presentation was made less stretched.
- The existing original ambience is layered at several playback speeds and volumes for a less static soundscape.

## Still to improve

The next art milestone should replace the placeholder-quality character and cave source sprites with a unified original pixel-art set rather than continuing to compensate through scaling alone. Additional enemy archetypes, stronger sound effects, lighting/flashlight gameplay, richer powerups and handcrafted landmark rooms are also planned.

See `docs/REBUILD_PLAN.md` for the gameplay-first roadmap.

## Run it

```bash
cargo test --release
cargo run --release
```

## Controls

- `A` / `D` or arrows - move
- `Space` - jump
- `1`-`9` - select a hotbar slot
- `F` - use the selected item
- `C` - open/close crafting
- `1` / `2` / `3` while crafting - craft that recipe
- `E` - melee attack
- `Esc` - pause / help
- `R` - restart the run and generate a new cave
- `Q` while paused - quit to desktop

## Core loop

Explore downward, collect only what you can carry, avoid or fight threats, treat injuries deliberately, craft supplies when needed, recover the cargo, then survive the climb back to the marked EXIT. Reaching the bottom is now the midpoint of the run rather than the win condition.
