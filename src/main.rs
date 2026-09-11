use bevy::prelude::*;
use descent_null::game::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
