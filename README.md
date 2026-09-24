# Descent: Null

A 2D cave-survival game built in Rust with Bevy.

## Why

I wanted a second way to learn Rust that was very different from embedded work. This project let me learn ECS architecture, procedural generation, movement systems and player-facing iteration.

## Current systems

- movement, jumping, gravity and collision
- jump buffering and coyote time
- procedurally generated caves
- a guaranteed reachable route through generated levels
- injuries, bleeding and fractures
- hunger and thirst
- pickups, hotbar and crafting
- enemies and environmental hazards
- tutorial and HUD
- extraction objective
- session/run statistics

## Things I changed after testing

The first cave generator could create impossible jumps, so I changed generation to guarantee a reachable route and add variation around it.

The first inventory design also had a weight system. It added more confusion than useful decisions, so I removed it and kept the inventory/crafting flow simpler.

## Run

```bash
cargo test --release
cargo run --release
```

## Controls

- `A` / `D` or arrows — move
- `Space` — jump
- `E` — attack
- `F` — use a relevant supply
- `C` — craft
- `R` — restart
- `F3` — debug overlay

## Project page

https://ren-jop.github.io/descent-null/

## License

No open-source license has been selected yet.
