// Descent: Null -- terminal front end. All simulation logic lives in the
// library crate (src/lib.rs and friends); this file is only input,
// rendering, and the main loop.

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use descent_null::body::Region;
use descent_null::crafting::RecipeId;
use descent_null::game::{GameState, Status, LAYER_HEIGHT, LAYER_WIDTH, TICK_SECONDS};
use descent_null::inventory::ItemId;
use descent_null::world::{ResourceKind, Tile};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

const FRAME: Duration = Duration::from_millis(60);

/// What the "use item" menu is currently asking for.
enum Mode {
    Normal,
    ChooseTreatment, // pick which medical item to use
    ChooseRegion(ItemId), // then which body region to use it on
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let seed: u64 = rand::random();
    let mut game = GameState::new(seed);
    let mut last_tick = Instant::now();
    let mut mode = Mode::Normal;

    loop {
        let timeout = FRAME
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match &mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => game.restart(),
                        KeyCode::Up | KeyCode::Char('w') => game.try_move(0, -1),
                        KeyCode::Down | KeyCode::Char('s') => game.try_move(0, 1),
                        KeyCode::Left | KeyCode::Char('a') => game.try_move(-1, 0),
                        KeyCode::Right | KeyCode::Char('d') => game.try_move(1, 0),
                        KeyCode::Char('e') => {
                            game.eat_ration();
                        }
                        KeyCode::Char('f') => {
                            game.drink_water();
                        }
                        KeyCode::Char('c') => {
                            let _ = game.craft(RecipeId::BandageFromFiber);
                        }
                        KeyCode::Char('t') => mode = Mode::ChooseTreatment,
                        _ => {}
                    },
                    Mode::ChooseTreatment => {
                        let item = match key.code {
                            KeyCode::Char('1') => Some(ItemId::Bandage),
                            KeyCode::Char('2') => Some(ItemId::Splint),
                            KeyCode::Char('3') => Some(ItemId::Antiseptic),
                            KeyCode::Char('4') => Some(ItemId::ClottingAgent),
                            KeyCode::Char('5') => Some(ItemId::Painkiller),
                            KeyCode::Esc => {
                                mode = Mode::Normal;
                                None
                            }
                            _ => None,
                        };
                        if let Some(item) = item {
                            mode = Mode::ChooseRegion(item);
                        }
                    }
                    Mode::ChooseRegion(item) => {
                        let region = match key.code {
                            KeyCode::Char('1') => Some(Region::Head),
                            KeyCode::Char('2') => Some(Region::Torso),
                            KeyCode::Char('3') => Some(Region::LeftArm),
                            KeyCode::Char('4') => Some(Region::RightArm),
                            KeyCode::Char('5') => Some(Region::LeftLeg),
                            KeyCode::Char('6') => Some(Region::RightLeg),
                            KeyCode::Esc => None,
                            _ => None,
                        };
                        if let Some(region) = region {
                            game.use_medical_item(*item, region);
                        }
                        mode = Mode::Normal;
                    }
                }
            }
        }

        if last_tick.elapsed().as_secs_f64() >= TICK_SECONDS {
            game.tick();
            last_tick = Instant::now();
        }

        terminal.draw(|f| draw(f, &game, &mode))?;
    }
}

fn draw(f: &mut ratatui::Frame, game: &GameState, mode: &Mode) {
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(LAYER_WIDTH as u16 + 2), Constraint::Min(28)])
        .split(f.size());

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(4)])
        .split(outer[0]);

    let title = format!("DESCENT: NULL -- depth {} -- seed {}", game.depth, game.seed);
    f.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        left[0],
    );

    let mut rows: Vec<Line> = Vec::with_capacity(LAYER_HEIGHT);
    for y in 0..LAYER_HEIGHT {
        let mut spans = Vec::with_capacity(LAYER_WIDTH);
        for x in 0..LAYER_WIDTH {
            if x as i32 == game.player.x && y as i32 == game.player.y {
                spans.push(Span::styled("@", Style::default().fg(Color::Yellow)));
                continue;
            }
            if let Some(_e) = game
                .enemies
                .iter()
                .find(|e| !e.is_dead() && e.x == x as i32 && e.y == y as i32)
            {
                spans.push(Span::styled("x", Style::default().fg(Color::Red)));
                continue;
            }
            let (ch, color) = match game.layer.tiles[y][x] {
                Tile::Empty => (' ', Color::DarkGray),
                Tile::Wall => ('#', Color::Gray),
                Tile::Water => ('~', Color::Blue),
                Tile::Hazard => ('^', Color::Red),
                Tile::Resource(ResourceKind::Food) => ('*', Color::Green),
                Tile::Resource(ResourceKind::Water) => ('o', Color::Cyan),
                Tile::Resource(ResourceKind::Medical) => ('+', Color::Magenta),
                Tile::Resource(ResourceKind::Material) => ('%', Color::Yellow),
                Tile::Extraction => ('E', Color::LightYellow),
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        rows.push(Line::from(spans));
    }
    let block = Block::default().borders(Borders::ALL).title(" the descent ");
    f.render_widget(Paragraph::new(rows).block(block), left[1]);

    f.render_widget(
        Paragraph::new(game.last_log().to_string()).alignment(Alignment::Center),
        left[2],
    );

    // --- right-hand status panel ---
    let p = &game.player;
    let mut lines = vec![
        Line::from(format!(
            "HUNGER {:>3}  THIRST {:>3}  STAM {:>3}",
            p.survival.hunger as i32, p.survival.thirst as i32, p.survival.stamina as i32
        )),
        Line::from(format!(
            "TEMP {:>3}  FATIGUE {:>3}  MOOD {:>3}",
            p.survival.temperature as i32, p.survival.fatigue as i32, p.survival.mood as i32
        )),
        Line::from(format!(
            "BLOOD {:>3}  CONSCIOUS {:>3}  SHOCK {:>3}",
            p.body.cardio.blood_volume as i32,
            p.body.cardio.consciousness as i32,
            p.body.cardio.shock as i32
        )),
        Line::from(""),
        Line::from("BODY"),
    ];
    for r in descent_null::body::ALL_REGIONS {
        let rs = p.body.region(r);
        let flags = format!(
            "{}{}",
            if rs.fracture { "F" } else { "-" },
            if rs.bleeding > 0.0 { "B" } else { "-" },
        );
        lines.push(Line::from(format!(
            "{:<9} {:>3} [{}]",
            format!("{:?}", r),
            rs.condition as i32,
            flags
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("INVENTORY"));
    if p.inventory.slots.is_empty() {
        lines.push(Line::from("  (empty)"));
    }
    for stack in &p.inventory.slots {
        lines.push(Line::from(format!(
            "  {}x {}",
            stack.quantity,
            descent_null::inventory::item_def(stack.id).name
        )));
    }

    lines.push(Line::from(""));
    match mode {
        Mode::Normal => {
            lines.push(Line::from("move: wasd/arrows  e: eat  f: drink"));
            lines.push(Line::from("c: craft bandage  t: treat wound  r: new run  q: quit"));
        }
        Mode::ChooseTreatment => {
            lines.push(Line::from("treat with:"));
            lines.push(Line::from("1 bandage 2 splint 3 antiseptic"));
            lines.push(Line::from("4 clotting agent 5 painkiller  esc cancel"));
        }
        Mode::ChooseRegion(_) => {
            lines.push(Line::from("apply to region:"));
            lines.push(Line::from("1 head 2 torso 3 l.arm 4 r.arm 5 l.leg 6 r.leg"));
        }
    }

    match game.status {
        Status::Extracted => lines.push(Line::from("EXTRACTED -- press r for a new run")),
        Status::Dead => lines.push(Line::from("YOU DIDN'T MAKE IT -- press r to try again")),
        Status::Playing => {}
    }

    let block = Block::default().borders(Borders::ALL).title(" status ");
    f.render_widget(Paragraph::new(lines).block(block), outer[1]);
}
