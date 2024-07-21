// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

mod ui;

use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(UiPlugin)
        .init_state::<GlobalState>()
        .add_sub_state::<GameState>()
        .add_systems(Startup, setup)
        .run();
}

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum GlobalState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(SubStates, Debug, Default, Clone, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InGame)]
pub enum GameState {
    #[default]
    Running,
    Battle,
    Paused {
        in_settings: bool,
    },
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
