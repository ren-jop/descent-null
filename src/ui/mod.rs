//! Minimal HUD for the physics proving ground. Not the final game UI.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::Body;
use crate::physics::{Grounded, LandingImpact};
use crate::player::Player;
use crate::survival::Survival;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLanding>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (capture_landings, update_hud));
    }
}

#[derive(Resource, Default)]
struct LastLanding {
    text: String,
}

#[derive(Component)]
struct HudText;

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        HudText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.90, 0.86, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

fn capture_landings(mut events: MessageReader<LandingImpact>, mut last: ResMut<LastLanding>) {
    for impact in events.read() {
        last.text = format!(
            "hard landing  {:.0} u/s  severity {:.2}",
            impact.downward_speed, impact.severity
        );
    }
}

fn update_hud(
    player: Query<(&LinearVelocity, Option<&Grounded>, &Body, &Survival), With<Player>>,
    last: Res<LastLanding>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok((velocity, grounded, body, survival)) = player.single() else {
        return;
    };
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let grounded = if grounded.is_some() { "grounded" } else { "airborne" };
    let body_line = if body.0.wound_count() == 0 {
        "no injuries".to_string()
    } else {
        format!(
            "{} wound(s)   pain {:.1}   bleeding {:.2}/s",
            body.0.wound_count(),
            body.0.total_pain(),
            body.0.total_bleed_rate()
        )
    };
    let status = if body.0.is_dead() {
        "   —  YOU DIED, press R"
    } else if body.0.is_ko() {
        "   —  UNCONSCIOUS, press R"
    } else {
        ""
    };
    let vitals = format!(
        "blood {:.0}%   hunger {:.0}%   thirst {:.0}%{status}",
        body.0.blood_volume() * 100.0,
        survival.0.hunger() * 100.0,
        survival.0.thirst() * 100.0,
    );
    **text = format!(
        "DESCENT: NULL  —  milestone 1 physics + early survival sim\n\
         WASD/arrows move   Space jump   R reset\n\
         {grounded}   vel ({:.0}, {:.0})\n\
         {body_line}\n\
         {vitals}\n\
         {}",
        velocity.x,
        velocity.y,
        last.text
    );
}
