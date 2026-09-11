// Descent: Null -- graphical front end (macroquad).
// All simulation logic lives in the library crate; this file is only
// input, rendering, and the main loop.

use descent_null::body::{Region, ALL_REGIONS};
use descent_null::crafting::RecipeId;
use descent_null::game::{GameState, Status, LAYER_HEIGHT, LAYER_WIDTH, TICK_SECONDS};
use descent_null::inventory::{item_def, ItemId};
use descent_null::world::{ResourceKind, Tile};
use macroquad::prelude::*;

const CELL: f32 = 22.0;
const MAP_PAD: f32 = 16.0;
const PANEL_W: f32 = 360.0;

const COL_BG: Color = Color::new(0.07, 0.06, 0.05, 1.0);
const COL_PANEL: Color = Color::new(0.11, 0.10, 0.09, 1.0);
const COL_WALL: Color = Color::new(0.28, 0.26, 0.24, 1.0);
const COL_EMPTY: Color = Color::new(0.14, 0.12, 0.10, 1.0);
const COL_WATER: Color = Color::new(0.18, 0.38, 0.62, 1.0);
const COL_HAZARD: Color = Color::new(0.72, 0.18, 0.14, 1.0);
const COL_FOOD: Color = Color::new(0.32, 0.68, 0.28, 1.0);
const COL_WATER_RES: Color = Color::new(0.30, 0.72, 0.78, 1.0);
const COL_MED: Color = Color::new(0.78, 0.32, 0.68, 1.0);
const COL_MAT: Color = Color::new(0.82, 0.68, 0.28, 1.0);
const COL_EXTRACT: Color = Color::new(0.95, 0.86, 0.42, 1.0);
const COL_PLAYER: Color = Color::new(0.98, 0.86, 0.28, 1.0);
const COL_ENEMY: Color = Color::new(0.86, 0.18, 0.16, 1.0);
const COL_TEXT: Color = Color::new(0.90, 0.86, 0.80, 1.0);
const COL_MUTED: Color = Color::new(0.62, 0.58, 0.52, 1.0);
const COL_ACCENT: Color = Color::new(0.86, 0.62, 0.28, 1.0);

enum Mode {
    Normal,
    ChooseTreatment,
    ChooseRegion(ItemId),
}

struct Hit {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Hit {
    fn contains(&self, mx: f32, my: f32) -> bool {
        mx >= self.x && mx <= self.x + self.w && my >= self.y && my <= self.y + self.h
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Descent: Null".to_owned(),
        window_width: 1280,
        window_height: 800,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // `macroquad::prelude` exports a different `rand`; use the crates.io crate.
    let seed: u64 = ::rand::random();
    let mut game = GameState::new(seed);
    let mut mode = Mode::Normal;
    let mut tick_accum = 0.0;
    let mut move_cooldown = 0.0;

    loop {
        let dt = get_frame_time();
        tick_accum += dt as f64;
        move_cooldown = (move_cooldown - dt).max(0.0);

        handle_input(&mut game, &mut mode, &mut move_cooldown);

        if tick_accum >= TICK_SECONDS {
            game.tick();
            tick_accum = 0.0;
        }

        draw_frame(&game, &mode);
        next_frame().await;
    }
}

fn panel_origin() -> (f32, f32, f32, f32) {
    let map_w = LAYER_WIDTH as f32 * CELL;
    let x = MAP_PAD + map_w + 24.0;
    let y = 40.0;
    (x, y, PANEL_W, screen_height() - 56.0)
}

fn action_button_hits() -> [Hit; 4] {
    let (px, py, pw, ph) = panel_origin();
    let btn_y = py + ph - 48.0;
    let btn_w = (pw - 40.0) / 4.0;
    let x = px + 12.0;
    [
        Hit { x, y: btn_y, w: btn_w - 6.0, h: 32.0 },
        Hit { x: x + btn_w, y: btn_y, w: btn_w - 6.0, h: 32.0 },
        Hit { x: x + btn_w * 2.0, y: btn_y, w: btn_w - 6.0, h: 32.0 },
        Hit { x: x + btn_w * 3.0, y: btn_y, w: btn_w - 6.0, h: 32.0 },
    ]
}

fn handle_input(game: &mut GameState, mode: &mut Mode, move_cooldown: &mut f32) {
    if is_key_pressed(KeyCode::Q) {
        std::process::exit(0);
    }
    if is_key_pressed(KeyCode::Escape) {
        match mode {
            Mode::Normal => std::process::exit(0),
            _ => *mode = Mode::Normal,
        }
    }

    if is_key_pressed(KeyCode::R) {
        game.restart();
        *mode = Mode::Normal;
        return;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let btns = action_button_hits();
        if matches!(mode, Mode::Normal) {
            if btns[0].contains(mx, my) {
                game.eat_ration();
            } else if btns[1].contains(mx, my) {
                game.drink_water();
            } else if btns[2].contains(mx, my) {
                let _ = game.craft(RecipeId::BandageFromFiber);
            } else if btns[3].contains(mx, my) {
                *mode = Mode::ChooseTreatment;
            }
        }
        handle_choice_clicks(game, mode, mx, my);
    }

    match mode {
        Mode::Normal => {
            if is_key_pressed(KeyCode::E) {
                game.eat_ration();
            }
            if is_key_pressed(KeyCode::F) {
                game.drink_water();
            }
            if is_key_pressed(KeyCode::C) {
                let _ = game.craft(RecipeId::BandageFromFiber);
            }
            if is_key_pressed(KeyCode::T) {
                *mode = Mode::ChooseTreatment;
            }

            if *move_cooldown <= 0.0 {
                let mut dx = 0;
                let mut dy = 0;
                if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                    dy = -1;
                } else if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                    dy = 1;
                } else if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                    dx = -1;
                } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                    dx = 1;
                }
                if dx != 0 || dy != 0 {
                    game.try_move(dx, dy);
                    *move_cooldown = 0.12;
                }
            }
        }
        Mode::ChooseTreatment => {
            let item = if is_key_pressed(KeyCode::Key1) {
                Some(ItemId::Bandage)
            } else if is_key_pressed(KeyCode::Key2) {
                Some(ItemId::Splint)
            } else if is_key_pressed(KeyCode::Key3) {
                Some(ItemId::Antiseptic)
            } else if is_key_pressed(KeyCode::Key4) {
                Some(ItemId::ClottingAgent)
            } else if is_key_pressed(KeyCode::Key5) {
                Some(ItemId::Painkiller)
            } else {
                None
            };
            if let Some(item) = item {
                *mode = Mode::ChooseRegion(item);
            }
        }
        Mode::ChooseRegion(item) => {
            let region = if is_key_pressed(KeyCode::Key1) {
                Some(Region::Head)
            } else if is_key_pressed(KeyCode::Key2) {
                Some(Region::Torso)
            } else if is_key_pressed(KeyCode::Key3) {
                Some(Region::LeftArm)
            } else if is_key_pressed(KeyCode::Key4) {
                Some(Region::RightArm)
            } else if is_key_pressed(KeyCode::Key5) {
                Some(Region::LeftLeg)
            } else if is_key_pressed(KeyCode::Key6) {
                Some(Region::RightLeg)
            } else {
                None
            };
            if let Some(region) = region {
                game.use_medical_item(*item, region);
                *mode = Mode::Normal;
            }
        }
    }
}

fn handle_choice_clicks(game: &mut GameState, mode: &mut Mode, mx: f32, my: f32) {
    let (px, py, _pw, ph) = panel_origin();
    let left = px + 16.0;
    let cy = py + ph - 118.0;
    match mode {
        Mode::ChooseTreatment => {
            let items = [
                ItemId::Bandage,
                ItemId::Splint,
                ItemId::Antiseptic,
                ItemId::ClottingAgent,
                ItemId::Painkiller,
            ];
            for (i, item) in items.iter().enumerate() {
                let row = i / 3;
                let col = i % 3;
                let hit = Hit {
                    x: left + col as f32 * 110.0,
                    y: cy + 20.0 + row as f32 * 24.0,
                    w: 104.0,
                    h: 20.0,
                };
                if hit.contains(mx, my) {
                    *mode = Mode::ChooseRegion(*item);
                    return;
                }
            }
        }
        Mode::ChooseRegion(item) => {
            for (i, region) in ALL_REGIONS.iter().enumerate() {
                let row = i / 3;
                let col = i % 3;
                let hit = Hit {
                    x: left + col as f32 * 110.0,
                    y: cy + 20.0 + row as f32 * 24.0,
                    w: 104.0,
                    h: 20.0,
                };
                if hit.contains(mx, my) {
                    game.use_medical_item(*item, *region);
                    *mode = Mode::Normal;
                    return;
                }
            }
        }
        Mode::Normal => {}
    }
}

fn draw_frame(game: &GameState, mode: &Mode) {
    clear_background(COL_BG);

    let map_w = LAYER_WIDTH as f32 * CELL;
    let map_h = LAYER_HEIGHT as f32 * CELL;
    let map_x = MAP_PAD;
    let map_y = 48.0;

    draw_text(
        &format!("DESCENT: NULL  ·  depth {}  ·  seed {}", game.depth, game.seed),
        MAP_PAD,
        28.0,
        26.0,
        COL_ACCENT,
    );

    draw_rectangle(map_x - 4.0, map_y - 4.0, map_w + 8.0, map_h + 8.0, COL_PANEL);
    draw_map(game, map_x, map_y);

    let log_y = map_y + map_h + 14.0;
    draw_text(game.last_log(), MAP_PAD, log_y + 16.0, 18.0, COL_TEXT);

    let (panel_x, panel_y, panel_w, panel_h) = panel_origin();
    draw_panel(game, mode, panel_x, panel_y, panel_w, panel_h);

    match game.status {
        Status::Extracted => {
            draw_banner("EXTRACTED  —  press R for a new run", Color::new(0.25, 0.55, 0.28, 0.92))
        }
        Status::Dead => {
            draw_banner("YOU DIDN'T MAKE IT  —  press R to try again", Color::new(0.55, 0.12, 0.10, 0.92))
        }
        Status::Playing => {}
    }
}

fn draw_banner(msg: &str, color: Color) {
    let w = screen_width();
    let y = screen_height() * 0.42;
    draw_rectangle(0.0, y, w, 64.0, color);
    let tw = measure_text(msg, None, 28, 1.0);
    draw_text(msg, (w - tw.width) * 0.5, y + 42.0, 28.0, WHITE);
}

fn draw_map(game: &GameState, origin_x: f32, origin_y: f32) {
    for y in 0..LAYER_HEIGHT {
        for x in 0..LAYER_WIDTH {
            let px = origin_x + x as f32 * CELL;
            let py = origin_y + y as f32 * CELL;
            let color = tile_color(game.layer.tiles[y][x]);
            draw_rectangle(px, py, CELL - 1.0, CELL - 1.0, color);

            let dx = x as i32 - game.player.x;
            let dy = y as i32 - game.player.y;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > 8.0 {
                let shade = ((dist - 8.0) * 0.04).min(0.45);
                draw_rectangle(px, py, CELL - 1.0, CELL - 1.0, Color::new(0.0, 0.0, 0.0, shade));
            }
        }
    }

    for enemy in game.enemies.iter().filter(|e| !e.is_dead()) {
        let px = origin_x + enemy.x as f32 * CELL + CELL * 0.5;
        let py = origin_y + enemy.y as f32 * CELL + CELL * 0.5;
        draw_circle(px, py, CELL * 0.32, COL_ENEMY);
    }

    let px = origin_x + game.player.x as f32 * CELL + CELL * 0.5;
    let py = origin_y + game.player.y as f32 * CELL + CELL * 0.5;
    draw_circle(px, py, CELL * 0.38, COL_PLAYER);
}

fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Empty => COL_EMPTY,
        Tile::Wall => COL_WALL,
        Tile::Water => COL_WATER,
        Tile::Hazard => COL_HAZARD,
        Tile::Resource(ResourceKind::Food) => COL_FOOD,
        Tile::Resource(ResourceKind::Water) => COL_WATER_RES,
        Tile::Resource(ResourceKind::Medical) => COL_MED,
        Tile::Resource(ResourceKind::Material) => COL_MAT,
        Tile::Extraction => COL_EXTRACT,
    }
}

fn draw_panel(game: &GameState, mode: &Mode, x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, COL_PANEL);
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(0.22, 0.20, 0.16, 1.0));

    let p = &game.player;
    let mut cy = y + 28.0;
    let left = x + 16.0;
    let bar_w = w - 32.0;

    draw_text("VITALS", left, cy, 20.0, COL_ACCENT);
    cy += 18.0;
    cy = draw_bar(left, cy, bar_w, "Hunger", p.survival.hunger, Color::from_rgba(196, 140, 64, 255));
    cy = draw_bar(left, cy, bar_w, "Thirst", p.survival.thirst, Color::from_rgba(72, 156, 210, 255));
    cy = draw_bar(left, cy, bar_w, "Stamina", p.survival.stamina, Color::from_rgba(88, 176, 96, 255));
    cy = draw_bar(left, cy, bar_w, "Blood", p.body.cardio.blood_volume, Color::from_rgba(176, 48, 48, 255));
    cy = draw_bar(left, cy, bar_w, "Conscious", p.body.cardio.consciousness, Color::from_rgba(160, 120, 200, 255));
    cy = draw_bar(left, cy, bar_w, "Shock", p.body.cardio.shock, Color::from_rgba(210, 90, 40, 255));

    cy += 8.0;
    draw_text(
        &format!(
            "Temp {:>3.0}   Fatigue {:>3.0}   Mood {:>3.0}",
            p.survival.temperature, p.survival.fatigue, p.survival.mood
        ),
        left,
        cy,
        16.0,
        COL_MUTED,
    );

    cy += 28.0;
    draw_text("BODY", left, cy, 20.0, COL_ACCENT);
    cy += 8.0;
    for r in ALL_REGIONS {
        let rs = p.body.region(r);
        cy += 22.0;
        let flags = format!(
            "{}{}{}",
            if rs.fracture { "F" } else { "-" },
            if rs.bleeding > 0.0 { "B" } else { "-" },
            if rs.infection > 5.0 { "I" } else { "-" },
        );
        let cond_col = Color::new(1.0 - rs.condition / 100.0, rs.condition / 100.0, 0.15, 1.0);
        draw_text(
            &format!("{:<9} {:>3.0} [{}]", region_name(r), rs.condition, flags),
            left,
            cy,
            16.0,
            cond_col,
        );
    }

    cy += 28.0;
    draw_text(
        &format!(
            "INVENTORY  ({:.1}/{:.1} kg)",
            p.inventory.total_weight(),
            p.inventory.max_weight
        ),
        left,
        cy,
        20.0,
        COL_ACCENT,
    );
    if p.inventory.slots.is_empty() {
        cy += 22.0;
        draw_text("(empty)", left, cy, 16.0, COL_MUTED);
    }
    for stack in &p.inventory.slots {
        cy += 20.0;
        if cy > y + h - 140.0 {
            break;
        }
        draw_text(
            &format!("{}× {}", stack.quantity, item_def(stack.id).name),
            left,
            cy,
            16.0,
            COL_TEXT,
        );
    }

    let hint_y = y + h - 118.0;
    match mode {
        Mode::Normal => {
            draw_text("WASD / arrows  move", left, hint_y, 15.0, COL_MUTED);
            draw_text("E eat   F drink   C craft bandage", left, hint_y + 18.0, 15.0, COL_MUTED);
            draw_text("T treat wound   R new run   Q quit", left, hint_y + 36.0, 15.0, COL_MUTED);
        }
        Mode::ChooseTreatment => {
            draw_text("Treat with (click or number):", left, hint_y, 16.0, COL_ACCENT);
            draw_choice_chip(left, hint_y + 20.0, "1 bandage");
            draw_choice_chip(left + 110.0, hint_y + 20.0, "2 splint");
            draw_choice_chip(left + 220.0, hint_y + 20.0, "3 antiseptic");
            draw_choice_chip(left, hint_y + 44.0, "4 clotting");
            draw_choice_chip(left + 110.0, hint_y + 44.0, "5 painkiller");
        }
        Mode::ChooseRegion(_) => {
            draw_text("Apply to (click or number):", left, hint_y, 16.0, COL_ACCENT);
            draw_choice_chip(left, hint_y + 20.0, "1 head");
            draw_choice_chip(left + 110.0, hint_y + 20.0, "2 torso");
            draw_choice_chip(left + 220.0, hint_y + 20.0, "3 L arm");
            draw_choice_chip(left, hint_y + 44.0, "4 R arm");
            draw_choice_chip(left + 110.0, hint_y + 44.0, "5 L leg");
            draw_choice_chip(left + 220.0, hint_y + 44.0, "6 R leg");
        }
    }

    let labels = ["Eat", "Drink", "Craft", "Treat"];
    for (i, hit) in action_button_hits().iter().enumerate() {
        draw_rectangle(hit.x, hit.y, hit.w, hit.h, Color::new(0.18, 0.16, 0.13, 1.0));
        draw_rectangle_lines(hit.x, hit.y, hit.w, hit.h, 1.0, COL_ACCENT);
        let tw = measure_text(labels[i], None, 16, 1.0);
        draw_text(
            labels[i],
            hit.x + (hit.w - tw.width) * 0.5,
            hit.y + 22.0,
            16.0,
            COL_TEXT,
        );
    }
}

fn draw_choice_chip(x: f32, y: f32, label: &str) {
    draw_rectangle(x, y, 104.0, 20.0, Color::new(0.16, 0.14, 0.12, 1.0));
    draw_text(label, x + 6.0, y + 15.0, 14.0, COL_TEXT);
}

fn draw_bar(x: f32, y: f32, w: f32, label: &str, value: f32, color: Color) -> f32 {
    let y = y + 20.0;
    draw_text(&format!("{label} {:>3.0}", value), x, y, 14.0, COL_MUTED);
    let bar_y = y + 6.0;
    draw_rectangle(x, bar_y, w, 8.0, Color::new(0.08, 0.07, 0.06, 1.0));
    let fill = (value / 100.0).clamp(0.0, 1.0) * w;
    draw_rectangle(x, bar_y, fill, 8.0, color);
    y + 14.0
}

fn region_name(r: Region) -> &'static str {
    match r {
        Region::Head => "Head",
        Region::Torso => "Torso",
        Region::LeftArm => "L.Arm",
        Region::RightArm => "R.Arm",
        Region::LeftLeg => "L.Leg",
        Region::RightLeg => "R.Leg",
    }
}
