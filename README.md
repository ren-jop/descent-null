# Descent: Null

An original 2D survival-exploration game about descending into a hostile underground, managing injuries and supplies, recovering mission cargo, and delivering it to extraction.

## Current build

The current build is optimized around **first-time-player UX** while keeping the normal play screen clean.

Highlights:

- Bevy 0.17 + Avian 2D physics with jumping, falling, landing damage, coyote time, jump buffering and camera follow.
- Four cave layers built around a guaranteed downward route, with optional side branches for extra risk/reward.
- Controlled falling is expected; dangerous long falls can injure or kill.
- Two-step mission: descend and recover cargo, then reach extraction.
- Two enemy types: slower cave **Crawlers** and faster deep-layer **Skitters** that detect you earlier and pressure your legs.
- Spike traps and Null Surge movement boosts add pressure without obscuring the core route.
- Body simulation with wounds, fractures, bleeding, blood volume, unconsciousness and death.
- Survival is intentionally simple: **Health, Hunger and Thirst**. There is no separate stamina meter.
- Hunger has a direct readable consequence: below 65%, movement gradually slows; starvation becomes a major speed penalty.
- Thirst owns the vision consequence: below 50%, the vignette progressively narrows the visible cave. Critical dehydration is announced and also drains health/blood volume until the player drinks water.
- Untreated lacerations/fractures bleed fast enough that treatment matters without becoming instant death.
- New-player help is **non-blocking**. If the player immediately uses A/D or arrow keys, nothing interrupts them; if they mash unrelated keys first, the FIELD LOG gives a movement hint. Full controls remain available from `Esc` pause/help.
- One larger mission panel lives at the top-left. Objective information is not duplicated elsewhere.
- The current layer is always shown in large white SF Mono-style text at the true top-centre and updates directly from world state.
- Pixel-style HUD: chunky framed panels, pixel icons, segmented coloured Health/Hunger/Thirst bars with numeric percentages, and a bordered 1-9 hotbar.
- One compact animated right-side **FIELD LOG** slides in only for meaningful events such as entering a layer, discovering supplies, traps, crafting readiness, hunger/thirst warnings and other important state changes.
- A compact pixel **SESSION RECORDS** leaderboard lives in the bottom-right. Press `L` or click it to expand the full top-five board.
- The leaderboard is **session-only**, which is intended for expo/shared-computer play. It resets when the app closes.
- When a completed run qualifies for the top five, a retro name-entry screen appears. Type up to 12 letters/numbers/spaces, use Backspace to edit, then press Enter to save the result to the current session leaderboard.
- 1-9 hotbar with unlimited same-item stacking. `F` uses the selected item.
- `C` opens a dedicated crafting screen.
- `Esc` opens pause/help.
- Different death causes use different generated sound effects, including dehydration.
- On macOS the UI uses installed **SF Mono** when available. Other systems automatically use Bevy's bundled fallback; no Apple font is distributed with the project.
- Original soundtrack and death effects are generated locally at launch; generated WAV files are not committed.
- The player's preferred walking feel is preserved; only the rendered character offset is raised so the boots sit visually on top of platform tiles instead of sinking into them.

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
- `L` - expand/collapse the session leaderboard
- mouse click on the mini/full leaderboard - expand/collapse it
- during top-five name entry: letters/numbers/space type, Backspace edits, Enter saves
- `Esc` - pause / help
- `R` - restart and generate a new cave
- `Q` while paused - quit

## Core loop

Follow the wide ledges downward, collect supplies, react to concise FIELD LOG warnings when something actually matters, manage wounds/hunger/thirst, survive Crawlers and Skitters, recover the guarded cargo at the bottom, and carry it to extraction. Fast successful runs compete for the shared top-five **current-session** leaderboard.
