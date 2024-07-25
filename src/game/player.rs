use bevy::{ecs::system::EntityCommands, prelude::*, render::view::RenderLayers};

use super::{
    animation::AnimationConfig, AttackSpeed, Damage, Defense, GameCamera, GameState, Health,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_player_resources)
            .add_systems(
                Update,
                (player_run, camera_follow_player).run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Resource, Debug)]
pub struct PlayerResources {
    idle_texture: Handle<Image>,
    idle_atlas: TextureAtlas,
    idle_animation_config: AnimationConfig,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpeed(pub f32);

fn prepare_player_resources(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let idle_texture = asset_server.load("player/alex_idle_sheet.png");
    let idle_texture_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 1, None, None);
    let idle_atlas_handle = texture_atlas_layouts.add(idle_texture_layout);
    let idle_animation_config = AnimationConfig::new(1, 5, 10);

    let idle_atlas = TextureAtlas {
        layout: idle_atlas_handle,
        index: idle_animation_config.first_sprite_index,
    };

    commands.insert_resource(PlayerResources {
        idle_texture,
        idle_atlas,
        idle_animation_config,
    });
}

pub fn spawn_player<'a>(
    commands: &'a mut Commands,
    player_resources: &PlayerResources,
    transform: Transform,
    render_layer: RenderLayers,
) -> EntityCommands<'a> {
    commands.spawn((
        SpriteBundle {
            transform,
            texture: player_resources.idle_texture.clone(),
            ..default()
        },
        player_resources.idle_atlas.clone(),
        player_resources.idle_animation_config.clone(),
        Player,
        PlayerSpeed(0.1),
        Health(100.0),
        Damage(5.0),
        AttackSpeed::new(0.5),
        Defense(0.0),
        render_layer,
    ))
}

fn player_run(time: Res<Time>, mut player: Query<(&PlayerSpeed, &mut Transform)>) {
    let Ok((speed, mut transform)) = player.get_single_mut() else {
        return;
    };

    let to_center = transform.translation;
    let rotation = Quat::from_rotation_z(-speed.0 * time.delta_seconds());
    let rotated = rotation * to_center;

    transform.translation = rotated;
    transform.rotation *= rotation;
}

fn camera_follow_player(
    player: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera: Query<&mut Transform, (Without<Player>, With<GameCamera>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };

    let mut t = *player_transform;
    t.scale = Vec3::new(0.5, 0.5, 0.5);
    *camera_transform = t;
}
