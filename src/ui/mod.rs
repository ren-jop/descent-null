//! Beginner-first HUD: pixel-style vitals, contextual guidance, a centred
//! hotbar, and right-side wireframe notifications. The normal play screen
//! avoids long persistent prose and always prioritises the next useful action.

use std::fs;

use avian2d::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Virtual;

use crate::body::{Body, DamageCause, LastDamageCause};
use crate::items::{
    first_craftable, recipe_descriptions, CraftingMenu, LastEvent, Pickup,
    PlayerInventory, SelectedSlot,
};
use crate::player::Player;
use crate::survival::Survival;
use crate::world::{BestTimes, CurrentDepth, DepthAnnouncement, RunStats, SessionTimes};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseMenuState>()
            .init_resource::<ObjectiveTransition>()
            .init_resource::<UiFont>()
            .add_systems(PreStartup, load_ui_font)
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    apply_ui_font,
                    pause_controls,
                    update_objective,
                    update_objective_transition,
                    update_depth_and_backpack,
                    update_vitals,
                    update_hotbar,
                    update_event_toast,
                    update_depth_toast,
                    update_context_guide,
                    update_crafting_overlay,
                    update_pause_overlay,
                    update_death_overlay,
                    update_win_overlay,
                    enforce_gameplay_hud_visibility,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
struct PauseMenuState { open: bool }

#[derive(Resource, Default)]
struct UiFont(Option<Handle<Font>>);

#[derive(Resource)]
struct ObjectiveTransition {
    last_cargo: bool,
    remaining: f32,
    initialized: bool,
}
impl Default for ObjectiveTransition {
    fn default() -> Self {
        Self { last_cargo: false, remaining: 3.0, initialized: false }
    }
}

#[derive(Component)] struct GameplayHud;
#[derive(Component)] struct ObjectiveText;
#[derive(Component)] struct GuideText;
#[derive(Component)] struct ObjectiveToastFrame;
#[derive(Component)] struct ObjectiveToast;
#[derive(Component)] struct EventToastFrame;
#[derive(Component)] struct EventToast;
#[derive(Component)] struct DepthToastFrame;
#[derive(Component)] struct DepthToast;
#[derive(Component)] struct DepthText;
#[derive(Component)] struct BackpackText;
#[derive(Component)] struct CraftReadyText;
#[derive(Component)] struct CraftingOverlay;
#[derive(Component)] struct CraftingText;
#[derive(Component)] struct PauseOverlay;
#[derive(Component)] struct DeathOverlay;
#[derive(Component)] struct DeathText;
#[derive(Component)] struct WinOverlay;
#[derive(Component)] struct WinText;

#[derive(Component, Clone, Copy, PartialEq)]
enum VitalKind { Health, Hunger, Thirst, Stamina }
#[derive(Component)] struct VitalSegment { kind: VitalKind, index: usize }
#[derive(Component)] struct VitalValue(VitalKind);
#[derive(Component)] struct HotbarSlot(usize);
#[derive(Component)] struct HotbarIcon(usize);
#[derive(Component)] struct HotbarQty(usize);

fn panel_bg() -> Color { Color::srgba(0.025, 0.023, 0.023, 0.92) }
fn wire_color() -> Color { Color::srgba(0.62, 0.55, 0.38, 0.95) }

fn vital_color(kind: VitalKind) -> Color {
    match kind {
        VitalKind::Health => Color::srgb(0.78, 0.18, 0.18),
        VitalKind::Hunger => Color::srgb(0.78, 0.48, 0.15),
        VitalKind::Thirst => Color::srgb(0.20, 0.52, 0.88),
        VitalKind::Stamina => Color::srgb(0.68, 0.74, 0.22),
    }
}

fn load_ui_font(mut fonts: ResMut<Assets<Font>>, mut ui_font: ResMut<UiFont>) {
    #[cfg(target_os = "macos")]
    {
        // SF Mono ships with macOS Terminal on common installations. We read
        // it at runtime rather than committing Apple's font to the repo.
        let candidates = [
            "/System/Applications/Utilities/Terminal.app/Contents/Resources/Fonts/SFMono-Regular.otf",
            "/Applications/Utilities/Terminal.app/Contents/Resources/Fonts/SFMono-Regular.otf",
            "/System/Library/Fonts/SFNSMono.ttf",
        ];
        for path in candidates {
            let Ok(bytes) = fs::read(path) else { continue; };
            let Ok(font) = Font::try_from_bytes(bytes) else { continue; };
            ui_font.0 = Some(fonts.add(font));
            break;
        }
    }
}

fn apply_ui_font(ui_font: Res<UiFont>, mut fonts: Query<&mut TextFont, Added<TextFont>>) {
    let Some(handle) = ui_font.0.as_ref() else { return; };
    for mut font in &mut fonts {
        font.font = handle.clone();
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_objective(&mut commands);
    spawn_status_corner(&mut commands);
    spawn_vitals(&mut commands);
    spawn_hotbar(&mut commands, &asset_server);
    spawn_toasts(&mut commands);
    spawn_crafting_overlay(&mut commands);
    spawn_pause_overlay(&mut commands);
    spawn_death_overlay(&mut commands);
    spawn_win_overlay(&mut commands);
}

fn spawn_objective(commands: &mut Commands) {
    commands.spawn((
        GameplayHud,
        ObjectiveText,
        Text::new("OBJECTIVE 1/2  RECOVER CARGO\nDescend toward the bottom chamber."),
        TextFont { font_size: 17.0, ..default() },
        TextColor(Color::srgb(0.97, 0.86, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0), left: Val::Px(16.0), width: Val::Px(440.0),
            padding: UiRect::all(Val::Px(10.0)), ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    commands.spawn((
        GameplayHud,
        GuideText,
        Text::new("GUIDE  Move with A / D. Drop onto the wide ledge below."),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.78, 0.94, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(84.0), left: Val::Px(16.0), width: Val::Px(440.0),
            padding: UiRect::all(Val::Px(10.0)), ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.04, 0.03, 0.90)),
    ));
}

fn spawn_status_corner(commands: &mut Commands) {
    commands.spawn((
        GameplayHud, DepthText, Text::new("LAYER 0 / 4"),
        TextFont { font_size: 15.0, ..default() },
        TextColor(Color::srgb(0.84, 0.81, 0.72)),
        Node { position_type: PositionType::Absolute, top: Val::Px(16.0), right: Val::Px(18.0), ..default() },
    ));
    commands.spawn((
        GameplayHud, BackpackText, Text::new("BACKPACK  0"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.70, 0.68, 0.62)),
        Node { position_type: PositionType::Absolute, top: Val::Px(39.0), right: Val::Px(18.0), ..default() },
    ));
    commands.spawn((
        GameplayHud, CraftReadyText, Text::new(""),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.58, 0.90, 0.56)),
        Node { position_type: PositionType::Absolute, top: Val::Px(60.0), right: Val::Px(18.0), ..default() },
    ));
}

fn spawn_vitals(commands: &mut Commands) {
    const LEFT: f32 = 14.0;
    const BOTTOM: f32 = 14.0;
    const PANEL_W: f32 = 286.0;
    const PANEL_H: f32 = 132.0;
    const BAR_X: f32 = 92.0;
    const SEG_W: f32 = 13.0;
    const SEG_GAP: f32 = 2.0;
    const SEG_H: f32 = 15.0;
    const ROW_H: f32 = 28.0;

    commands.spawn((
        GameplayHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(LEFT), bottom: Val::Px(BOTTOM),
            width: Val::Px(PANEL_W), height: Val::Px(PANEL_H), ..default()
        },
        BackgroundColor(panel_bg()),
    ));

    let rows = [
        (VitalKind::Health, "HEALTH"),
        (VitalKind::Hunger, "HUNGER"),
        (VitalKind::Thirst, "THIRST"),
        (VitalKind::Stamina, "STAMINA"),
    ];

    for (row, (kind, label)) in rows.into_iter().enumerate() {
        let y = BOTTOM + PANEL_H - 27.0 - row as f32 * ROW_H;
        commands.spawn((
            GameplayHud,
            Text::new(label),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.82, 0.80, 0.74)),
            Node { position_type: PositionType::Absolute, left: Val::Px(LEFT + 10.0), bottom: Val::Px(y), ..default() },
        ));

        for index in 0..10 {
            commands.spawn((
                GameplayHud,
                VitalSegment { kind, index },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(LEFT + BAR_X + index as f32 * (SEG_W + SEG_GAP)),
                    bottom: Val::Px(y + 1.0), width: Val::Px(SEG_W), height: Val::Px(SEG_H), ..default()
                },
                BackgroundColor(vital_color(kind)),
            ));
        }

        commands.spawn((
            GameplayHud,
            VitalValue(kind),
            Text::new("100%"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::WHITE),
            Node { position_type: PositionType::Absolute, left: Val::Px(LEFT + 246.0), bottom: Val::Px(y + 1.0), ..default() },
        ));
    }
}

fn spawn_hotbar(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        GameplayHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0), bottom: Val::Px(12.0), width: Val::Percent(100.0),
            justify_content: JustifyContent::Center, column_gap: Val::Px(4.0), ..default()
        },
    )).with_children(|bar| {
        for index in 0..9 {
            bar.spawn((
                HotbarSlot(index),
                Node { width: Val::Px(40.0), height: Val::Px(40.0), position_type: PositionType::Relative, ..default() },
                BackgroundColor(if index == 0 { Color::srgba(0.72, 0.58, 0.24, 0.98) } else { Color::srgba(0.12, 0.11, 0.10, 0.92) }),
            )).with_children(|slot| {
                slot.spawn((
                    HotbarIcon(index), ImageNode::new(asset_server.load("sprites/item_scrap.png")),
                    Node { display: Display::None, position_type: PositionType::Absolute, left: Val::Px(5.0), top: Val::Px(5.0), width: Val::Px(30.0), height: Val::Px(30.0), ..default() },
                ));
                slot.spawn((
                    HotbarQty(index), Text::new(""), TextFont { font_size: 10.0, ..default() }, TextColor(Color::WHITE),
                    Node { display: Display::None, position_type: PositionType::Absolute, right: Val::Px(2.0), bottom: Val::Px(1.0), ..default() },
                ));
                slot.spawn((
                    Text::new((index + 1).to_string()), TextFont { font_size: 10.0, ..default() }, TextColor(Color::srgb(0.90, 0.86, 0.72)),
                    Node { position_type: PositionType::Absolute, left: Val::Px(3.0), top: Val::Px(1.0), ..default() },
                ));
            });
        }
    });
}

fn spawn_wire_toast<TFrame: Component, TText: Component>(
    commands: &mut Commands,
    frame: TFrame,
    text_tag: TText,
    top: f32,
    font_size: f32,
    text_color: Color,
) {
    commands.spawn((
        GameplayHud,
        frame,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(top), right: Val::Px(18.0), width: Val::Px(330.0),
            padding: UiRect::all(Val::Px(2.0)), ..default()
        },
        BackgroundColor(wire_color()),
    )).with_children(|outer| {
        outer.spawn((
            text_tag,
            Text::new(""),
            TextFont { font_size, ..default() },
            TextColor(text_color),
            Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), ..default() },
            BackgroundColor(panel_bg()),
        ));
    });
}

fn spawn_toasts(commands: &mut Commands) {
    spawn_wire_toast(commands, ObjectiveToastFrame, ObjectiveToast, 96.0, 18.0, Color::srgb(0.98, 0.86, 0.48));
    spawn_wire_toast(commands, EventToastFrame, EventToast, 180.0, 14.0, Color::srgb(0.96, 0.92, 0.78));
    spawn_wire_toast(commands, DepthToastFrame, DepthToast, 264.0, 14.0, Color::srgb(0.86, 0.81, 0.68));
}

fn spawn_crafting_overlay(commands: &mut Commands) {
    commands.spawn((
        CraftingOverlay,
        Node {
            display: Display::None, position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0),
            width: Val::Percent(100.0), height: Val::Percent(100.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.76)),
    )).with_children(|overlay| {
        overlay.spawn((
            CraftingText, Text::new("CRAFTING"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(0.94, 0.88, 0.70)),
            Node { width: Val::Px(540.0), padding: UiRect::all(Val::Px(22.0)), ..default() },
            BackgroundColor(Color::srgba(0.06, 0.055, 0.05, 0.98)),
        ));
    });
}

fn spawn_pause_overlay(commands: &mut Commands) {
    commands.spawn((
        PauseOverlay,
        Node {
            display: Display::None, position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0),
            width: Val::Percent(100.0), height: Val::Percent(100.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.01, 0.01, 0.86)),
    )).with_children(|overlay| {
        overlay.spawn((
            Text::new("PAUSED\n\nEsc  Resume\nR  Restart run\nQ  Quit\n\nA/D  Move     Space  Jump\n1-9  Select   F  Use\nC  Craft      E  Attack\n\nMISSION\nDescend, recover the cargo, reach EXTRACTION."),
            TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(0.91, 0.89, 0.82)),
            Node { width: Val::Px(480.0), padding: UiRect::all(Val::Px(24.0)), ..default() },
            BackgroundColor(Color::srgba(0.07, 0.065, 0.06, 0.98)),
        ));
    });
}

fn spawn_death_overlay(commands: &mut Commands) {
    commands.spawn((
        DeathOverlay,
        Node {
            display: Display::None, position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0),
            width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(12.0), ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.01, 0.01, 0.96)),
    )).with_children(|overlay| {
        overlay.spawn((Text::new("YOU DIED"), TextFont { font_size: 48.0, ..default() }, TextColor(Color::srgb(0.76, 0.20, 0.20))));
        overlay.spawn((DeathText, Text::new(""), TextFont { font_size: 17.0, ..default() }, TextColor(Color::srgb(0.82, 0.79, 0.73))));
    });
}

fn spawn_win_overlay(commands: &mut Commands) {
    commands.spawn((
        WinOverlay,
        Node {
            display: Display::None, position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0),
            width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(12.0), ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.03, 0.015, 0.96)),
    )).with_children(|overlay| {
        overlay.spawn((Text::new("MISSION COMPLETE"), TextFont { font_size: 44.0, ..default() }, TextColor(Color::srgb(0.48, 0.82, 0.44))));
        overlay.spawn((WinText, Text::new(""), TextFont { font_size: 17.0, ..default() }, TextColor(Color::srgb(0.82, 0.79, 0.73))));
    });
}

fn pause_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseMenuState>,
    mut crafting: ResMut<CraftingMenu>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut player_velocity: Query<&mut LinearVelocity, With<Player>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if crafting.open {
            crafting.open = false;
            if !pause.open { virtual_time.unpause(); }
            return;
        }
        pause.open = !pause.open;
    }
    if pause.open {
        if let Ok(mut velocity) = player_velocity.single_mut() { *velocity = LinearVelocity::ZERO; }
        if keyboard.just_pressed(KeyCode::KeyR) { pause.open = false; }
        if keyboard.just_pressed(KeyCode::KeyQ) { exit.write(AppExit::Success); }
    }
    if pause.open || crafting.open { virtual_time.pause(); } else { virtual_time.unpause(); }
}

fn update_objective(stats: Res<RunStats>, mut text: Query<&mut Text, With<ObjectiveText>>) {
    let Ok(mut text) = text.single_mut() else { return; };
    **text = if stats.cargo_recovered {
        "OBJECTIVE 2/2  REACH EXTRACTION\nCarry the cargo to the green marker.".to_string()
    } else {
        "OBJECTIVE 1/2  RECOVER CARGO\nDescend toward the bottom chamber.".to_string()
    };
}

fn update_objective_transition(
    time: Res<Time>, stats: Res<RunStats>, mut state: ResMut<ObjectiveTransition>,
    mut frame: Query<&mut Node, With<ObjectiveToastFrame>>,
    mut toast: Query<&mut Text, With<ObjectiveToast>>,
) {
    let (Ok(mut frame), Ok(mut text)) = (frame.single_mut(), toast.single_mut()) else { return; };
    if !state.initialized {
        state.initialized = true;
        state.last_cargo = stats.cargo_recovered;
        state.remaining = 3.0;
        **text = "NEW OBJECTIVE\nRECOVER THE CARGO".to_string();
    } else if stats.cargo_recovered != state.last_cargo {
        state.last_cargo = stats.cargo_recovered;
        state.remaining = 3.0;
        **text = if stats.cargo_recovered { "OBJECTIVE UPDATED\nREACH EXTRACTION".to_string() } else { "NEW RUN\nRECOVER THE CARGO".to_string() };
    }
    if state.remaining > 0.0 {
        state.remaining = (state.remaining - time.delta_secs()).max(0.0);
        frame.display = Display::Flex;
    } else { frame.display = Display::None; }
}

fn update_depth_and_backpack(
    depth: Res<CurrentDepth>, inventory: Query<&PlayerInventory, With<Player>>,
    mut depth_text: Query<&mut Text, (With<DepthText>, Without<BackpackText>)>,
    mut backpack_text: Query<&mut Text, (With<BackpackText>, Without<DepthText>)>,
) {
    if let Ok(mut text) = depth_text.single_mut() { **text = format!("LAYER {} / 4", depth.0); }
    if let (Ok(inventory), Ok(mut text)) = (inventory.single(), backpack_text.single_mut()) {
        **text = format!("BACKPACK  {}", inventory.0.used_weight());
    }
}

fn update_vitals(
    player: Query<(&Body, &Survival), With<Player>>,
    mut segments: Query<(&VitalSegment, &mut BackgroundColor)>,
    mut values: Query<(&VitalValue, &mut Text)>,
) {
    let Ok((body, survival)) = player.single() else { return; };
    let value_for = |kind: VitalKind| -> f32 {
        match kind {
            VitalKind::Health => body.0.blood_volume(),
            VitalKind::Hunger => survival.0.hunger(),
            VitalKind::Thirst => survival.0.thirst(),
            VitalKind::Stamina => survival.0.stamina(),
        }.clamp(0.0, 1.0)
    };
    for (segment, mut bg) in &mut segments {
        let lit = (value_for(segment.kind) * 10.0).ceil() as usize;
        bg.0 = if segment.index < lit { vital_color(segment.kind) } else { Color::srgb(0.10, 0.09, 0.09) };
    }
    for (kind, mut text) in &mut values {
        **text = format!("{}%", (value_for(kind.0) * 100.0).round() as i32);
    }
}

fn update_hotbar(
    inventory: Query<&PlayerInventory, With<Player>>, selected: Res<SelectedSlot>, asset_server: Res<AssetServer>,
    mut slots: Query<(&HotbarSlot, &mut BackgroundColor)>,
    mut icons: Query<(&HotbarIcon, &mut Node, &mut ImageNode), Without<HotbarQty>>,
    mut quantities: Query<(&HotbarQty, &mut Node, &mut Text), (Without<HotbarIcon>, Without<CraftReadyText>)>,
    mut craft_ready: Query<&mut Text, (With<CraftReadyText>, Without<HotbarQty>)>,
) {
    let Ok(inventory) = inventory.single() else { return; };
    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.0 == selected.0 { Color::srgba(0.72, 0.58, 0.24, 0.98) } else { Color::srgba(0.12, 0.11, 0.10, 0.92) };
    }
    for (slot, mut node, mut image) in &mut icons {
        if let Some(stack) = inventory.0.stacks().get(slot.0) {
            node.display = Display::Flex;
            image.image = asset_server.load(stack.kind.sprite_path());
        } else { node.display = Display::None; }
    }
    for (slot, mut node, mut text) in &mut quantities {
        if let Some(stack) = inventory.0.stacks().get(slot.0) {
            node.display = Display::Flex;
            **text = stack.quantity.to_string();
        } else { node.display = Display::None; }
    }
    if let Ok(mut text) = craft_ready.single_mut() {
        **text = first_craftable(&inventory.0)
            .map(|kind| format!("CRAFT READY  {}  [C]", kind.label().to_uppercase()))
            .unwrap_or_default();
    }
}

fn update_event_toast(
    last: Res<LastEvent>,
    mut frame: Query<&mut Node, With<EventToastFrame>>,
    mut toast: Query<&mut Text, With<EventToast>>,
) {
    let (Ok(mut frame), Ok(mut text)) = (frame.single_mut(), toast.single_mut()) else { return; };
    if last.remaining > 0.0 {
        frame.display = Display::Flex;
        **text = last.text.clone();
    } else { frame.display = Display::None; }
}

fn update_depth_toast(
    announcement: Res<DepthAnnouncement>,
    mut frame: Query<&mut Node, With<DepthToastFrame>>,
    mut toast: Query<&mut Text, With<DepthToast>>,
) {
    let (Ok(mut frame), Ok(mut text)) = (frame.single_mut(), toast.single_mut()) else { return; };
    if announcement.remaining > 0.0 {
        frame.display = Display::Flex;
        **text = announcement.text.clone();
    } else { frame.display = Display::None; }
}

fn update_context_guide(
    stats: Res<RunStats>,
    depth: Res<CurrentDepth>,
    player: Query<(&Transform, &Body, &Survival, &PlayerInventory), With<Player>>,
    pickups: Query<(&Transform, &Pickup), Without<Player>>,
    mut guide: Query<&mut Text, With<GuideText>>,
) {
    let (Ok((transform, body, survival, inventory)), Ok(mut text)) = (player.single(), guide.single_mut()) else { return; };
    let pos = transform.translation.truncate();

    let nearby = pickups
        .iter()
        .filter_map(|(pickup_transform, pickup)| {
            let distance = pos.distance(pickup_transform.translation.truncate());
            (distance < 110.0).then_some((distance, pickup.0.kind))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));

    **text = if body.0.total_bleed_rate() > 0.0005 {
        "GUIDE  BLEEDING — select BANDAGE, then press F".to_string()
    } else if body.0.has_untreated_leg_fracture() {
        "GUIDE  FRACTURED LEG — select SPLINT, then press F".to_string()
    } else if survival.0.thirst() < 0.30 {
        "GUIDE  LOW THIRST is slowing movement/stamina — use WATER".to_string()
    } else if survival.0.hunger() < 0.30 {
        "GUIDE  LOW HUNGER is slowing movement/stamina — use FOOD".to_string()
    } else if stats.cargo_recovered {
        "GUIDE  CARGO SECURED — cross to the green EXTRACTION marker".to_string()
    } else if let Some((_, kind)) = nearby {
        format!("GUIDE  {} nearby — walk into it to collect", kind.label().to_uppercase())
    } else if let Some(kind) = first_craftable(&inventory.0) {
        format!("GUIDE  {} can be crafted now — press C", kind.label().to_uppercase())
    } else if depth.0 == 0 {
        "GUIDE  GO DOWN — controlled drops onto wide ledges are intended".to_string()
    } else if depth.0 < 4 {
        format!("GUIDE  DESCEND — current layer {} of 4", depth.0)
    } else {
        "GUIDE  BOTTOM CHAMBER — locate and secure the mission cargo".to_string()
    };
}

fn update_crafting_overlay(
    crafting: Res<CraftingMenu>, mut overlay: Query<&mut Node, With<CraftingOverlay>>, mut text: Query<&mut Text, With<CraftingText>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else { return; };
    overlay.display = if crafting.open { Display::Flex } else { Display::None };
    if crafting.open {
        if let Ok(mut text) = text.single_mut() {
            **text = format!("CRAFTING\n\n{}\n\n1 / 2 / 3 craft   C or Esc close", recipe_descriptions().join("\n\n"));
        }
    }
}

fn update_pause_overlay(pause: Res<PauseMenuState>, mut overlay: Query<&mut Node, With<PauseOverlay>>) {
    if let Ok(mut overlay) = overlay.single_mut() { overlay.display = if pause.open { Display::Flex } else { Display::None }; }
}

fn update_death_overlay(
    player: Query<&Body, With<Player>>, stats: Res<RunStats>, cause: Res<LastDamageCause>,
    mut overlay: Query<&mut Node, With<DeathOverlay>>, mut text: Query<&mut Text, With<DeathText>>,
) {
    let Ok(body) = player.single() else { return; };
    let Ok(mut overlay) = overlay.single_mut() else { return; };
    if body.0.is_dead() && !stats.extracted {
        overlay.display = Display::Flex;
        if let Ok(mut text) = text.single_mut() {
            let cause_text = match cause.0 {
                DamageCause::Fall => "Fatal fall / impact",
                DamageCause::Trap => "Trap injury",
                DamageCause::Enemy => "Crawler attack / blood loss",
                DamageCause::Unknown if body.0.total_bleed_rate() > 0.0005 => "Untreated bleeding",
                DamageCause::Unknown => "Severe trauma",
            };
            **text = format!("{cause_text}\nLayer {}   {:.0}s survived\n\nR  Restart", stats.deepest_layer, stats.elapsed_secs);
        }
    } else { overlay.display = Display::None; }
}

fn update_win_overlay(
    stats: Res<RunStats>, best: Res<BestTimes>, session: Res<SessionTimes>,
    mut overlay: Query<&mut Node, With<WinOverlay>>, mut text: Query<&mut Text, With<WinText>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else { return; };
    if stats.extracted {
        overlay.display = Display::Flex;
        if let Ok(mut text) = text.single_mut() {
            **text = format!(
                "RUN TIME  {:.1}s\n\nTHIS SESSION\n{}\n\nPERSONAL BESTS\n{}\n\nR  Run again",
                stats.elapsed_secs,
                session.formatted(),
                best.formatted(),
            );
        }
    } else { overlay.display = Display::None; }
}

fn enforce_gameplay_hud_visibility(
    player: Query<&Body, With<Player>>, stats: Res<RunStats>, pause: Res<PauseMenuState>, crafting: Res<CraftingMenu>,
    mut hud: Query<&mut Node, With<GameplayHud>>,
) {
    let dead = player.single().map(|body| body.0.is_dead()).unwrap_or(false);
    let show = !(dead || stats.extracted || pause.open || crafting.open);
    for mut node in &mut hud { node.display = if show { Display::Flex } else { Display::None }; }
}
