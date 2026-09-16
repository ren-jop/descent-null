# Descent: Null

An original 2D survival-exploration game about descending into a hostile underground, managing injuries and supplies, recovering mission cargo, and delivering it to extraction.

## Current build

The current build is optimized around **first-time-player UX**. The game should explain itself through play instead of assuming the player already understands inventory, crafting, injuries, or objectives.

Highlights:

- Bevy 0.17 + Avian 2D physics with jumping, falling, landing damage, coyote time, jump buffering and camera follow.
- Four cave layers built around a guaranteed downward route, with optional side branches for extra risk/reward.
- Controlled falling is expected; dangerous long falls can injure or kill.
- Two-step mission: descend and recover cargo, then reach the green extraction marker.
- Cave crawlers, spike traps and Null Surge movement boosts add pressure without obscuring the core route.
- Body simulation with wounds, fractures, bleeding, blood volume, unconsciousness and death.
- Hunger and thirst reduce stamina recovery and movement performance when low; they do not silently drain health.
- Context-sensitive guide text prioritizes the player's actual next useful action: nearby pickup, treatment, crafting, descent or extraction.
- Pixel-style coloured vitality bars include numeric percentages.
- 1-9 hotbar with unlimited same-item stacking. `F` uses the selected item.
- `C` opens a dedicated crafting screen; a small prompt appears only when something can actually be crafted.
- Transient notifications use a dedicated right-side wireframe area rather than overlapping the vitals/hotbar.
- `Esc` opens pause/help.
- Different death causes use different generated sound effects.
- Personal best times persist locally, while a separate session leaderboard resets each time the game is launched.
- On macOS the UI will use the installed **SF Mono** font when available. Other systems automatically use Bevy's bundled fallback, so no Apple font is required or distributed with the project.
- Original soundtrack and death effects are generated locally at launch; generated WAV files are not committed.

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

Follow the wide ledges downward, collect supplies, react to contextual guidance, manage wounds and survival pressure, recover the guarded cargo at the bottom, and carry it to extraction. Faster successful runs are recorded in both the current-session and persistent personal leaderboards.
