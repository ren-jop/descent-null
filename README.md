# Descent: Null

An original 2D survival-exploration game about descending into a hostile underground, managing injuries and supplies, recovering mission cargo, and delivering it to extraction.

## Current build

The current build is optimized around **first-time-player UX**. The game should teach the player at the moment an action becomes relevant, then get out of the way.

Highlights:

- Bevy 0.17 + Avian 2D physics with jumping, falling, landing damage, coyote time, jump buffering and camera follow.
- Four cave layers built around a guaranteed downward route, with optional side branches for extra risk/reward.
- Controlled falling is expected; dangerous long falls can injure or kill.
- Two-step mission: descend and recover cargo, then reach extraction.
- Cave crawlers, spike traps and Null Surge movement boosts add pressure without obscuring the core route.
- Body simulation with wounds, fractures, bleeding, blood volume, unconsciousness and death.
- Hunger and thirst reduce stamina recovery and movement performance when low; they do not silently drain health.
- A **modal first-run field manual** pauses gameplay at the exact onboarding moments for movement, jumping, pickups, hotbar selection/use and the first available craft.
- One larger objective panel lives at the top-left. Objective information is not duplicated elsewhere.
- The current layer is always shown in large white text at the top-centre and updates directly from world state, including after respawn.
- Pixel-style HUD: chunky framed panels, pixel icons, segmented coloured vitality bars with numeric percentages, and a bordered 1-9 hotbar.
- One animated right-side **FIELD LOG** slides in for meaningful events such as a new layer, pickups, danger and mission changes.
- 1-9 hotbar with unlimited same-item stacking. `F` uses the selected item.
- `C` opens a dedicated crafting screen.
- `Esc` opens pause/help.
- Different death causes use different generated sound effects.
- Personal best times persist locally, while a separate session leaderboard resets each time the game is launched.
- On macOS the UI uses installed **SF Mono** when available. Other systems automatically use Bevy's bundled fallback; no Apple font is distributed with the project.
- Original soundtrack and death effects are generated locally at launch; generated WAV files are not committed.
- The player's earlier walking feel is preserved; only the rendered character offset is raised so the boots sit visually on top of platform tiles instead of sinking into them.

See `docs/REBUILD_PLAN.md` for the gameplay-first roadmap.

## Fresh clone / random computer

Requirements: Git, Rust and Cargo.

```bash
git clone https://github.com/rin677/descent-null.git
cd descent-null
git switch feature/gameplay-clarity-pass
cargo test --release
cargo run --release
```

No account, downloaded font, external music pack or generated asset is required. Runtime-generated audio is recreated automatically. If SF Mono is not installed, the game falls back to Bevy's bundled font.

## Existing checkout

```bash
cd ~/descent-null
git switch feature/gameplay-clarity-pass
git pull
cargo test --release
cargo run --release
```

## Controls

- `A` / `D` or arrows - move
- `Space` - jump
- `1`-`9` - select a hotbar slot
- `F` - use selected item
- `C` - open/close crafting
- `1` / `2` / `3` while crafting - craft that recipe
- `E` - melee attack
- `Esc` - pause / help
- `R` - restart and generate a new cave
- `Q` while paused - quit

## Core loop

Follow the wide ledges downward, collect supplies, respond to the field manual when a new mechanic is first introduced, manage wounds and survival pressure, recover the guarded cargo at the bottom, and carry it to extraction. Faster successful runs are recorded in both the current-session and persistent personal leaderboards.
