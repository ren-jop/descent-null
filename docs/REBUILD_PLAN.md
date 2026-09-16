# Descent: Null — gameplay-first roadmap

This document supersedes the original engine-rewrite plan. The engine swap is done: the project now runs on **Bevy 0.17 + Avian 2D**, has continuous physics, wound/cardio simulation, hunger/thirst/stamina, inventory/crafting, procedural cave runs, an enemy, HUD, death/reset flow, and a concrete cargo objective.

The next problem is no longer “build an engine foundation.” It is **turn the existing systems into an understandable, intentional game**.

## Product direction

Descent: Null should feel like a physical survival descent where the player can always answer three questions:

1. **What am I trying to do?**
2. **What is happening to me and why?**
3. **What can I do about it?**

Simulation depth is useful only when the player can read it and make decisions from it. New systems should therefore ship with their feedback, UI, audio, and player-facing purpose — not as hidden numbers first and polish later.

The core loop is:

`explore → notice risk → collect supplies → descend → get pressured/injured → diagnose → treat/craft → decide whether to push deeper → recover the cargo → complete the run`

## Current architecture

```text
physics ──────────────► body (impact, force, wounds)
body ─────────────────► physics (KO, fracture movement limits)
survival ─────────────► physics (stamina/exhaustion)
inventory ────────────► treatment / crafting
world/generation ─────► colliders, loot, enemies, objective
ai/combat ────────────► wounds ─► body
ui/audio ─────────────► reads simulation truth; never owns it
save ─────────────────► future
```

Keep this dependency direction. UI may explain state but must not become the source of state.

## Design rules going forward

- **No unexplained damage.** Blood/health loss must have a visible cause. Hunger and thirst should primarily create survival pressure through stamina/recovery, not silently masquerade as generic HP damage.
- **No invisible objectives.** The active objective stays visible until completed.
- **No mystery crafting.** Recipes must show ingredients, result, and why the result is useful.
- **Readable before realistic.** A larger character, item, icon, or warning is preferable to technically accurate but unreadable presentation.
- **One simulation truth.** Enemies and player damage should continue converging on wounds/body-state rather than parallel HP systems.
- **Every feature needs a player decision.** Do not add temperature, darkness, infection, equipment, etc. until there is something meaningful the player can do in response.
- **Death should teach.** Recaps should identify the cause and what the player could have done differently.

---

# Phase 2A — Gameplay clarity and repair

**Status: active. This is the highest priority before adding more content.**

Playable target: a new player can understand the goal, HUD, inventory, crafting, injuries, death, and restart without reading source code or the README.

### Implement now

- [x] Persistent cargo objective on the HUD.
- [x] Fixed 9-slot inventory presentation.
- [x] Recipe reference panel.
- [x] Death overlay and run statistics.
- [x] HUD reappears after death/restart rather than remaining hidden.
- [x] Larger player sprite presentation without changing collision dimensions.
- [x] Larger world pickups and cargo objective.
- [x] Larger HUD, inventory slots, typography, and event messages.
- [x] Label health/hunger/thirst/stamina instead of relying on icons alone.
- [x] Show percentages on vitals.
- [x] Explicitly show the cause of health loss (`BLEEDING`) and the treatment (`bandage`).
- [x] Stop hunger/thirst from silently draining blood volume.
- [x] Show crafting purpose alongside ingredients.
- [x] Show inventory carry weight in the inventory header.
- [x] Death recap explains critical blood loss rather than only saying “you died.”

### Still needed in this phase

- [ ] Replace the current placeholder/player sprite with a coherent final-resolution character sprite sheet.
- [ ] Rework cave tile art so player, terrain, loot, enemies, and UI share one deliberate pixel-density/art-direction standard.
- [ ] Add selected-slot highlighting and number-key selection so the fixed inventory becomes a real hotbar rather than only a status grid.
- [ ] Split `F uses best supply` into deliberate item use once hotbar selection exists.
- [ ] Replace `C crafts first affordable recipe` with recipe selection.
- [ ] Add a compact body-status panel showing wound location/type when injured.
- [ ] Add interaction prompts near pickups/objectives where ambiguity remains.
- [ ] Add UI scale setting (e.g. 100/125/150%).
- [ ] Test at common window sizes and ensure no overlap at 1280×720.

**Exit condition:** a first-time player can state the objective, identify why health is falling, identify how to stop it, understand what materials are for, and restart after death with a fully restored HUD.

---

# Phase 2B — Inventory, treatment, and crafting decisions

Playable target: inventory and medical care become choices rather than automatic helpers.

### Inventory / hotbar

- [ ] Persistent numbered hotbar (1–9).
- [ ] Selected item highlight.
- [ ] Item name + short purpose tooltip for selected slot.
- [ ] Drop item / swap slot controls.
- [ ] Clear stack and carry-weight feedback.
- [ ] Backpack/full inventory screen only when needed; keep the hotbar visible during play.

### Medical interaction

- [ ] Body inspection screen with named regions.
- [ ] Wound cards: region, wound type, bleeding, pain, treatment state.
- [ ] Choose treatment and target instead of always treating “the worst” automatically.
- [ ] Treatment duration with interruption from movement/damage.
- [ ] Treatment feedback animation/audio.

### Crafting

- [ ] Recipe selection UI.
- [ ] Craftable/unavailable visual state.
- [ ] Ingredient counts (`owned / required`).
- [ ] Result description and intended use.
- [ ] Keep transactional all-or-nothing ingredient consumption.

**Exit condition:** the player intentionally chooses an item, treatment, or recipe and understands its consequence before committing.

---

# Phase 3 — World purpose and navigation

Playable target: the cave becomes a place to understand and make route decisions in, not only a stack of randomized platforms.

- [ ] Distinct visual identity for each layer.
- [ ] Landmark generation so runs have memorable locations.
- [ ] Main route plus optional risk/reward branches.
- [ ] Rest/safe pockets with lower enemy pressure.
- [ ] Better cargo chamber presentation at the bottom.
- [ ] Environmental storytelling props that do not require dialogue.
- [ ] Improve procedural reachability checks beyond the current loose platform randomization.
- [ ] Seed display/replay support for debugging and challenge runs.
- [ ] Decide the longer-term run ending: cargo-at-bottom completion vs cargo recovery plus return-to-surface extraction.

**Exit condition:** players can describe where they are, why they might detour, and what deeper layers change besides loot rarity.

---

# Phase 4 — Environmental survival mechanics

Playable target: the environment creates readable hazards with preparation and counterplay.

Add one hazard at a time. Each hazard needs **warning → consequence → countermeasure**.

Recommended order:

1. **Darkness / light**
   - flashlight or lamp
   - battery consumption
   - darkness genuinely limits useful visibility
   - visible battery state and low-power warning

2. **Unstable terrain / rockfall**
   - telegraphed warning
   - impact wounds through the existing body system
   - route choice / avoidance

3. **Toxic pockets**
   - visual/audio cue before exposure
   - short-term stamina/consciousness pressure
   - mask/filter or route avoidance

4. **Temperature** only after equipment/inventory can support clothing or heat decisions.

Do not add survival meters whose only behavior is “bar slowly falls.”

---

# Phase 5 — Enemy and combat depth

Playable target: enemies create tactical choices beyond walking toward the player and dealing contact damage.

- [ ] Perception states: idle / investigate / chase / disengage.
- [ ] Sight and sound awareness.
- [ ] Attack anticipation frames / telegraphs.
- [ ] Distinct wound profiles by attack type.
- [ ] Player weapons/tools with reach, recovery, and commitment.
- [ ] Knockback and stagger through physics.
- [ ] Avoidance/stealth as a valid alternative to combat.
- [ ] Second enemy archetype only after the crawler is fully readable and fun.
- [ ] Remove any remaining enemy-only HP abstractions when practical; converge on body/damage events.

**Exit condition:** the player can predict enemy intent and choose fight, evade, or bypass.

---

# Phase 6 — Run structure and progression

Playable target: repeated runs differ meaningfully and teach the player something.

- [ ] Better death recap: direct wound chain / killer / final cause.
- [ ] Run summary: deepest layer, survival time, cargo, enemies, treatments, crafted items.
- [ ] Procedural events with clear risk/reward.
- [ ] Rare discoveries / optional objectives.
- [ ] Knowledge-based unlocks or discoveries rather than large permanent stat bonuses.
- [ ] Difficulty knobs based on world pressure, not inflated enemy health.
- [ ] Save/settings support.

Avoid meta-progression that makes the simulation irrelevant by simply giving permanent health/damage upgrades.

---

# Phase 7 — Feel, audio, and accessibility

This is not “final polish”; pieces of it should accompany every earlier phase. This phase is the comprehensive pass.

- [ ] Footsteps by surface.
- [ ] Landing sounds by severity.
- [ ] Injury, bleeding, treatment, crafting, pickup, enemy, and cargo SFX.
- [ ] Strong but restrained impact particles/screenshake.
- [ ] Player animation set: idle, walk, airborne, hurt, treat, attack.
- [ ] Enemy animation set.
- [ ] Pixel-perfect texture filtering and consistent sprite scale.
- [ ] Remappable controls.
- [ ] UI scale.
- [ ] Color-accessible warnings (never encode danger by color alone).
- [ ] Reduced screenshake toggle.
- [ ] Audio sliders.
- [ ] Pause/settings menu.

---

# Phase 8 — Content expansion

Only expand the catalogue after the loop above is readable and fun.

- [ ] More cave modules / landmarks.
- [ ] More item families.
- [ ] More recipes with distinct strategic roles.
- [ ] More enemy archetypes.
- [ ] More environmental interactions.
- [ ] Random events.
- [ ] Optional narrative discoveries.

The game should not become broader faster than it becomes clearer.

---

## Near-term implementation order

Use this order for the next work sessions:

1. **Hotbar selection + selected item usage**
2. **Selectable crafting menu**
3. **Body/wound inspection panel**
4. **Character sprite + tile/art-direction pass**
5. **Light/battery gameplay**
6. **Crawler telegraphs/perception polish**
7. **Layer landmarks + stronger procedural structure**
8. **SFX + animation pass**

Do not jump to additional enemies, temperature, infection, or large content packs before steps 1–4 are solid.

## Testing priorities

Keep pure simulation tests, but also add integration/regression tests where practical for player-facing bugs:

- R reset restores body, survival, inventory, stats, cave, and HUD visibility.
- Health does not decrease from hunger/thirst alone.
- Active bleeding decreases blood volume and HUD reports bleeding.
- Bandage stops ongoing bleeding.
- Inventory never exceeds capacity.
- Crafting consumes exactly the required ingredients.
- Cargo completion cannot trigger from ordinary loot.
- New cave generation places the player in a valid starting state.

The guiding rule is simple: **a systemic game is only as deep as the decisions the player can actually understand.**
