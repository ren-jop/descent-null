//! Minimal HUD for the physics proving ground. Not the final game UI.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::{Grounded, LandingImpact};
use crate::player::Player;

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
            "hard landing  {:.0} u/s  severity {:.2}  (body sim comes in milestone 2)",
            impact.downward_speed, impact.severity
        );
    }
}

fn update_hud(
    player: Query<(&LinearVelocity, Option<&Grounded>), With<Player>>,
    last: Res<LastLanding>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok((velocity, grounded)) = player.single() else {
        return;
    };
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let grounded = if grounded.is_some() { "grounded" } else { "airborne" };
    **text = format!(
        "DESCENT: NULL  —  milestone 1 physics proving ground\n\
         WASD/arrows move   Space jump   R reset\n\
         {grounded}   vel ({:.0}, {:.0})\n\
         {}",
        velocity.x,
        velocity.y,
        last.text
    );
}
