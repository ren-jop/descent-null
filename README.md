# Descent: Null

An original 2D survival-exploration simulation. You are a test subject
in a hostile underground. The design is systemic: physics, body, hunger,
inventory, and environment should all create consequences.

Inspired by the *gameplay philosophy* of games like Casualties: Unknown —
not a copy. Original name, world, items, creatures, code, and art. No
assets from any other game are used anywhere in this repo.

## Honest status

**Phase 1 is done. Phase 2 is partway done: treatment items, basic
crafting, and original pixel-art sprites exist; hazards beyond falling,
enemies, and a real light/darkness gameplay mechanic don't yet.**

What is real right now:

- Bevy 0.17 + Avian 2D physics: gravity, collision, slopes, jumping,
  falling, coyote time, jump buffering, camera follow.
- A procedurally generated, four-layer descending cave, different every
  run, with side ledges and loot that skews rarer/better with depth.
- Original pixel-art sprites (`assets/sprites/`, generated for this
  project) for the player, per-depth-layer rock tiles, and every item —
  no more flat-color rectangles. A soft vignette rides on the camera for
  atmosphere (visual only so far — it doesn't gate what you can see or
  pick up; that's the real light/darkness system, still phase 2).
- A real body sim: hard landings wound both legs (bruise / laceration /
  fracture by impact speed). Wounds add pain and — above laceration
  severity — bleeding, which drains blood volume every frame (slow
  passive regen when nothing's actively bleeding). Under 40% blood
  volume you're unconscious; under 15% you're dead.
- Hunger, thirst, and stamina, all draining over real time. Empty hunger
  or thirst also drains blood volume, feeding the same KO/death path as
  bleeding. Low stamina caps movement speed; an unsplinted leg fracture
  caps it harder. (Stamina only drains from sustained movement now, not
  every step — it shouldn't make ordinary walking feel starved anymore.)
- A weight-limited inventory (12 units) with items scattered through the
  cave and picked up automatically on approach.
- Treatment that does something: `F` uses the single most relevant
  supply — eat if hungry, drink if thirsty, bandage active bleeding,
  splint a leg fracture, or medkit if hurt.
- Basic crafting: `C` crafts the first recipe you can afford (scrap +
  cloth → bandage, metal + scrap → splint, cloth + metal + battery →
  medkit).
- The HUD shows depth, injuries, and inventory as text, but blood,
  hunger, thirst, and stamina are now real bar gauges, not just numbers.
- **Death is a real stop, not just a status line**: going below 15%
  blood volume shows a full-screen banner (deepest layer reached, time
  survived) and freezes the run there. `R` is the only way past it —
  full restart: position, wounds, vitals, inventory, and run stats all
  reset. (The cave layout is fixed per process launch, not regenerated
  on `R` yet.)
- Unit tests across physics, body/cardio, survival, inventory, and
  crafting.

What is **not** in this build yet: environmental hazards beyond falling
(spikes, traps, toxic areas), enemies/combat, light/darkness as an
actual visibility-gating mechanic, random events, an objective/
extraction goal, NPCs, and regenerating the cave on restart.

See `docs/REBUILD_PLAN.md` for the original milestone plan this extends.

## Run it

```
cargo test --release
cargo run --release
```

## Controls

- `A` `D` or arrows — move (acceleration, not grid steps)
- `Space` — jump
- `F` — use the most relevant supply (eat/drink/bandage/splint/medkit)
- `C` — craft, if you have materials for anything
- `R` — restart the run (full reset: position, body, vitals, inventory,
  stats) — also what dying forces you into

Descend, grab what you can carry, and watch your vitals. Get hurt and
`F` will bandage or splint it if you're carrying the right item. Craft
more supplies from scrap/cloth/metal/battery with `C`. Bleed out,
starve, or dehydrate and it's a full-screen stop, not a quiet fade —
there's still no way to get hurt other than falling, though; enemies
and hazards are next.
