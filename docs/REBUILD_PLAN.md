# Descent: Null - gameplay-first roadmap

The engine foundation is in place. The priority now is a readable survival game with a strong first run, clear consequences, and a reason to explore instead of a prototype that exposes every system at once.

## Product rule

At any moment the player should be able to answer:

1. What am I trying to do?
2. What is happening to me and why?
3. What can I do about it?

The current loop is:

`learn controls -> explore -> collect -> descend -> avoid/fight hazards -> diagnose injuries -> use/craft supplies -> recover cargo -> climb back to EXIT -> extract`

## Direct-shift pass - implemented

- [x] Replace the screen-filling HUD with a compact mission/depth/vitals/hotbar layout.
- [x] Remove permanent crafting text from normal gameplay.
- [x] Remove unsupported Unicode UI symbols that rendered as square boxes.
- [x] Add progressive first-run onboarding for movement, jump, pickups and hotbar use.
- [x] Add a 1-9 selectable hotbar.
- [x] Make `F` use the selected item instead of an automatic "best supply" action.
- [x] Label the 12-unit inventory limit as **pack weight**, not slot count.
- [x] Make `C` open a dedicated crafting menu.
- [x] Make recipes explicitly selectable with 1/2/3 and explain their purpose.
- [x] Add an `Esc` pause/help menu with resume, restart, quit and controls.
- [x] Stop hunger/thirst from silently draining health.
- [x] Show direct bleeding/fracture treatment warnings only when relevant.
- [x] Make catastrophic multi-layer falls fatal so dropping to the bottom is not a shortcut.
- [x] Change the mission from instant cargo victory to cargo recovery plus surface extraction.
- [x] Increase jump capability so the procedural route can be climbed during extraction.
- [x] Guard the cargo chamber with enemies.
- [x] Increase enemy pressure on deeper layers.
- [x] Add spike hazards that produce real wounds.
- [x] Add temporary Null Surge movement powerups.
- [x] Remove the floating player nameplate.
- [x] Replace the awkward placeholder player render with a clearer explorer silhouette while keeping the same collider.
- [x] Enable nearest-neighbour image sampling for crisp pixel art.
- [x] Layer the existing original ambience at different speeds/volumes for more movement.
- [x] Add GitHub CI for release tests/builds.

## Next priority - art and game feel

The current coded explorer is intentionally a bridge, not final character art.

- [ ] Author a real original player sprite sheet: idle, walk, jump/fall, hurt, attack, treatment.
- [ ] Standardize player, terrain, pickups, hazards and enemies to one pixel-density target.
- [ ] Replace stretched platform textures with modular cave tile pieces and edge/corner variants.
- [ ] Give each cave layer a distinct palette/material identity without reducing readability.
- [ ] Add proper pickup, landing, injury, crafting, enemy and objective sound effects.
- [ ] Replace/extend the ambience with several authored musical layers or tracks for safe, deep, danger and escape states.
- [ ] Add animation and stronger enemy attack telegraphs.

## Next priority - richer cave decisions

- [ ] Generate recognizable landmark rooms rather than only randomized platforms.
- [ ] Add safe pockets / temporary rest rooms.
- [ ] Add optional risk-reward side routes with better loot or powerups.
- [ ] Add explicit reachability validation for both descent and extraction routes.
- [ ] Add a dedicated cargo chamber layout instead of only placing guards on the bottom floor.
- [ ] Add an escape-state world change after cargo recovery: stronger enemy pressure, lighting shift, alarms or new hazards.

## Inventory / medical depth

- [ ] Selected-item tooltip with item purpose.
- [ ] Drop / swap controls.
- [ ] Full backpack screen when more management is needed; keep the hotbar compact in play.
- [ ] Body inspection screen with wound region/type/bleeding/pain/treatment state.
- [ ] Let the player choose treatment target rather than always treating the worst wound.
- [ ] Add treatment duration/interruption so medical care is a tactical decision.
- [ ] Crafting menu should show owned/required material counts and disabled unavailable recipes.

## Enemy and combat depth

Do not add many enemies that all behave the same.

- [ ] Improve crawler telegraphing, hit feedback, disengage behavior and animation.
- [ ] Add a fast small enemy that pressures movement rather than raw health.
- [ ] Add a ranged/spitter archetype with a clearly telegraphed projectile or toxic zone.
- [ ] Add a heavy cargo-guardian archetype with slow, dangerous attacks.
- [ ] Add knockback/stagger and clearer melee timing.
- [ ] Make avoidance and route choice as valid as fighting.

## Powerups and environmental survival

Powerups should create temporary playstyle changes rather than generic stat inflation.

- [x] Null Surge movement boost prototype.
- [ ] Temporary impact shield.
- [ ] Flare / portable light source.
- [ ] Short dash or emergency jump charge.
- [ ] Temporary enemy-detection or hazard-sense effect.
- [ ] Darkness + flashlight/battery gameplay.
- [ ] Telegraphed rockfall / unstable terrain.
- [ ] Toxic pockets with a visible warning and a countermeasure.

## Beginner experience and accessibility

- [x] Safe opening layer before normal enemy pressure.
- [x] Progressive first-run hints that disappear after learning the basics.
- [x] Pause screen doubles as a permanent control reference.
- [ ] Guarantee useful tutorial loot near the opening rather than relying entirely on random generation.
- [ ] Add UI scale options.
- [ ] Add music/SFX volume sliders.
- [ ] Add reduced screenshake option.
- [ ] Add remappable controls.
- [ ] Never use colour alone to communicate danger.

## Run structure

- [ ] Better death recap including the final wound/hazard/enemy chain.
- [ ] End-of-run stats: enemies, treatments, crafted items, cargo time, extraction time.
- [ ] Optional objectives and rare discoveries.
- [ ] Seed display/replay for debugging and challenge runs.
- [ ] Save settings and basic progression state.
- [ ] Prefer knowledge/discovery unlocks over permanent health/damage inflation.

## Immediate implementation order after this branch is stable

1. Compile/test and playtest the new HUD/hotbar/crafting/pause/objective loop.
2. Fix any route-generation cases that cannot be climbed back out.
3. Real character sprite sheet + coherent cave tile set.
4. Crawler animation/telegraph + gameplay SFX.
5. Landmark rooms and risk/reward branches.
6. Light/flashlight/battery gameplay.
7. Second enemy archetype.
8. Richer escape phase after cargo recovery.

## Regression checklist

- Restart restores body, survival, inventory, selected slot, world and HUD.
- Health does not decrease from hunger/thirst alone.
- Bleeding lowers blood and the HUD explains the cause.
- Bandages stop bleeding only when bleeding exists.
- Inventory never exceeds weight capacity.
- Hotbar slot selection maps to the item actually used.
- Crafting only consumes the chosen recipe's ingredients.
- Cargo recovery does not immediately complete the mission.
- Catastrophic falls cannot be used to bypass the cave safely.
- Every generated run has a practical route to the objective and back to extraction.
- Pause and crafting freeze gameplay and restore the HUD correctly when closed.

The guiding rule remains: **a systemic game is only as deep as the decisions the player can understand.**
