//! App plugin for the current milestone. Physics and render are separate
//! from future survival ticks.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::body::BodyPlugin;
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
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Descent: Null".into(),
                    resolution: (1280, 800).into(),
                    ..default()
                }),
                ..default()
            }),
            // 1 meter ≈ 20 pixels so Avian can tune solver stability.
            PhysicsPlugins::default().with_length_unit(20.0),
            PhysicsGameplayPlugin,
            BodyPlugin,
            SurvivalPlugin,
            ItemsPlugin,
            PlayerPlugin,
            CavePlugin,
            HudPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.06, 0.05, 0.05)))
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 1000.0));
    }
}
