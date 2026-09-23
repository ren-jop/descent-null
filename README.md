# Descent: Null

**Author:** Ren Jopson  
**Project page:** https://rin677.github.io/descent-null/

An original 2D survival-exploration simulation. You are a test subject
in a hostile underground. The design is systemic: physics, body, hunger,
inventory, enemies, and environment should all create consequences.

Inspired by the *gameplay philosophy* of games like Casualties: Unknown —
not a copy. Original name, world, items, creatures, code, and art. No
assets from any other game are used anywhere in this repo. (Kenney's CC0
UI packs were reviewed for HUD design conventions per project direction,
but couldn't actually be downloaded into the authoring sandbox — network
egress there is allowlisted and kenney.nl/opengameart.org aren't on it.
The HUD below is original art in that same spirit, not their files.)

## Honest status

**Phase 1 done. Phase 2 substantially done: treatment, crafting, a real
enemy, a proper HUD, cave regeneration on death, and ambient audio all
exist. Hazards beyond falling/enemies and light-as-a-mechanic don't yet.**

What is real right now:

- Bevy 0.17 + Avian 2D physics: gravity, collision, slopes, jumping,
  falling, coyote time, jump buffering, camera follow.
- A procedurally generated, four-layer descending cave, **regenerated
  from scratch on every restart** (not just resetting the player into
  the same layout — `R` despawns the whole cave and builds a new one),
  with side ledges, scattered loot, and enemies from layer 1 down.
- A cave crawler enemy: idle until you're within range, then it chases
  and bites. Bites create a real wound (through the same pipeline as
  fall damage). Fight back with `E`.
- A real body sim: wounds (bruise/laceration/fracture), pain, bleeding
  draining blood volume every frame. **Bleed rates were badly
  miscalibrated until this round** — a severe landing could drain your
  entire blood volume in a fraction of a second with no way to react.
  Fixed: a single worst-case wound now takes tens of seconds to become
  fatal if ignored, not a fraction of one. Under 40% blood volume
  you're unconscious; under 15% dead.
- Hunger and thirst now drain a bit faster (3 min / 2.5 min — fast
  enough to actually notice in a normal session) and have a graduated
  effect on stamina regen starting well before they hit zero, instead
  of an invisible cliff at exactly empty that a short session never
  reached.
- A weight-limited inventory (12 units), auto-picked-up on approach.
- Treatment that works: `F` uses the single most relevant supply.
- Basic crafting: `C` crafts the first affordable recipe.
- A real HUD: bottom-left icon+bar cluster — **health is now bigger,
  brighter green, and framed with a visible border** so it doesn't read
  as an empty black box even when nearly depleted — with smaller
  hunger/thirst above it and stamina only while moving. Bottom-right
  inventory grid, a depth label, and fading toasts for events.
- A developer debug overlay (`F3`, off by default, force-hidden during
  death/win regardless of its toggle state).
- Full-screen death and win overlays with real run stats; `R` fully
  resets everything, including regenerating the cave.
- **Ambient background audio** — an original, seamlessly-looping low
  drone (`assets/audio/ambience.wav`, generated for this project),
  quiet and atmospheric rather than a soundtrack.
- **Camera shake on hard landings** — a bit of physical feedback for
  the exact moment that also causes real fall damage. Decays quickly;
  doesn't drift or compound over repeated hits.
- Original pixel-art sprites for the player, rock tiles, items, the
  enemy, and HUD icons (`assets/sprites/`).
- Unit tests across physics, body/cardio, survival (including the new
  graduated hardship behavior), inventory, crafting, and the enemy.

What is **not** in this build yet: environmental hazards beyond falling
and the enemy, more than one enemy type, light/darkness as an actual
visibility-gating mechanic (cosmetic vignette only), random events,
NPCs, a return trip / real extraction, and any sound effects beyond the
ambient loop (footsteps, damage, pickups — no SFX system yet, just
music).

See `docs/REBUILD_PLAN.md` for the original milestone plan this extends,
and `CONTEXT.md` for architecture notes and known-risk areas if you're
picking this project up.

## Run it

```
cargo test --release
cargo run --release
```

## Controls

- `A` `D` or arrows — move
- `Space` — jump
- `E` — melee attack (hits the nearest enemy in range)
- `F` — use the most relevant supply (eat/drink/bandage/splint/medkit)
- `C` — craft, if you have materials for anything
- `R` — restart the run (full reset)
- `F3` — toggle the developer debug overlay (off by default)

Descend, grab what you can carry, and watch your vitals. Cave crawlers
show up from layer 1 down — avoid them or fight them, but every bite
draws real blood. Get hurt and `F` treats it if you're carrying the
right item. Reach the cargo at the bottom to win; bleed out, starve, or
dehydrate first and it's a death screen instead.
