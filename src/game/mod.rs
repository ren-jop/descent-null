//! Main game plugin.

use avian2d::prelude::*;
use bevy::audio::Volume;
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

/// Layer the existing original ambience at different playback speeds. The
/// result has slow harmonic movement and a little tension instead of one
/// static drone, while staying subtle enough not to fight gameplay.
fn spawn_ambience(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ambience = asset_server.load("audio/ambience.wav");
    commands.spawn((
        AudioPlayer::new(ambience.clone()),
        PlaybackSettings::LOOP
            .with_volume(Volume::Linear(0.30))
            .with_speed(1.0),
    ));
    commands.spawn((
        AudioPlayer::new(ambience.clone()),
        PlaybackSettings::LOOP
            .with_volume(Volume::Linear(0.16))
            .with_speed(0.78),
    ));
    commands.spawn((
        AudioPlayer::new(ambience),
        PlaybackSettings::LOOP
            .with_volume(Volume::Linear(0.08))
            .with_speed(1.32),
    ));
}
