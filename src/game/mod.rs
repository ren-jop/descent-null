//! Main game plugin.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::BodyPlugin;
use crate::enemy::EnemyPlugin;
use crate::items::ItemsPlugin;
use crate::physics::PhysicsGameplayPlugin;
use crate::player::PlayerPlugin;
use crate::survival::SurvivalPlugin;
use crate::ui::HudPlugin;
use crate::world::CavePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Descent: Null".into(),
                        resolution: (1280, 800).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            PhysicsPlugins::default().with_length_unit(20.0),
            PhysicsGameplayPlugin,
            BodyPlugin,
            SurvivalPlugin,
            ItemsPlugin,
            EnemyPlugin,
            PlayerPlugin,
            CavePlugin,
            HudPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.045, 0.04, 0.045)))
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 1000.0))
        .add_systems(Startup, spawn_ambience);
    }
}

fn spawn_ambience(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/ambience.wav")),
        PlaybackSettings::LOOP,
    ));
}
