# Descent: Null

An original 2D survival-exploration game about descending into a hostile underground, managing injuries and supplies, recovering mission cargo, and delivering it to extraction.

## Current build

The current build is deliberately optimized for a **first-time player**. The game should explain itself through play instead of assuming the player already understands the inventory, crafting, injuries, or objective.

What is implemented now:

- Bevy 0.17 + Avian 2D physics with jumping, falling, landing damage, coyote time, jump buffering and camera follow.
- A four-layer cave built around a **guaranteed downward descent spine**. Random side ledges add loot/risk but cannot block the main route.
- Falling is part of progression: ordinary controlled drops between wide ledges are expected, while reckless multi-layer falls can be fatal.
- A two-step mission shown clearly in the top-left:
  1. **Go down and recover the lost cargo.**
  2. **Cross the bottom chamber and deliver it to the green extraction pad.**
- The bottom chamber is guarded, so touching the cargo is not an instant win.
- Cave crawlers become more common deeper down.
- Spike hazards create real wounds and bleeding.
- Rare **Null Surge** pickups temporarily increase movement acceleration.
- A body simulation with bruises, lacerations, fractures, pain, bleeding, blood volume, unconsciousness and death.
- Hunger, thirst and stamina affect survival without silently draining health.
- Every ordinary world pickup has its **name displayed above it** before pickup.
- Pickup messages explain what an item is for (for example WATER restores thirst; BANDAGE stops bleeding; SCRAP is a crafting material).
- A selectable 1-9 hotbar. Pressing a number visibly changes the highlighted slot and shows the selected item, quantity, action and purpose above the hotbar.
- `F` uses the currently selected item. Crafting materials explicitly tell you to press `C` instead.
- The `12` inventory limit is **pack weight**, not a number of hotbar slots; pack weight is shown separately in the top-right.
- `C` opens a dedicated crafting overlay. Recipes are selected with `1`, `2`, or `3` and state what they make, what they need and why they are useful.
- Progressive first-run onboarding teaches mission direction, movement, jumping/falling, pickups, hotbar selection and crafting.
- `Esc` opens a pause/help menu with Resume, Restart, Quit and control instructions.
- Contextual warnings say what is wrong and what item/action fixes it.
- Pixel art uses nearest-neighbour filtering for crisp scaling.
- The old static ambience playback has been replaced with an **original procedurally generated soundtrack** created at launch: evolving cave harmony, low pulses, sparse melody, metallic accents and a tension swell.

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

Follow the wide ledges downward, scavenge labelled supplies, use the hotbar deliberately, manage wounds and survival pressure, choose optional side branches for extra resources, reach the guarded bottom chamber, recover the cargo, and deliver it across the chamber to extraction.
