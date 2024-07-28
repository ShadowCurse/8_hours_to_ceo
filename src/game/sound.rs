use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_sounds);
    }
}

#[derive(Resource, Debug)]
pub struct SoundResources {
    pub player_attack: Handle<AudioSource>,
    pub enemy_attack: Handle<AudioSource>,
    pub boss_attack: Handle<AudioSource>,
    pub chest_open: Handle<AudioSource>,
    pub volume: Volume,
}

#[derive(Component)]
pub struct BackgroundMusic;

fn prepare_sounds(asset_server: Res<AssetServer>, mut commands: Commands) {
    let player_attack = asset_server.load("sounds/alex_attack.ogg");
    let enemy_attack = asset_server.load("sounds/enemy_attack.ogg");
    let boss_attack = asset_server.load("sounds/boss_attack.ogg");
    let chest_open = asset_server.load("sounds/chest_open.ogg");
    let background = asset_server.load("sounds/background.ogg");

    commands.insert_resource(SoundResources {
        player_attack,
        enemy_attack,
        boss_attack,
        chest_open,
        volume: Volume::new(1.0),
    });

    // Background music
    commands.spawn((
        AudioBundle {
            source: background,
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.5),
                ..Default::default()
            },
        },
        BackgroundMusic,
    ));
}
