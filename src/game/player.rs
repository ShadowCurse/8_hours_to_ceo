use bevy::{ecs::system::EntityCommands, prelude::*, render::view::RenderLayers};

use crate::GlobalState;

use super::{
    animation::{AllAnimations, AnimationConfig, AnimationFinished},
    enemy::DamageEnemy,
    inventory::Inventory,
    items::Items,
    AttackSpeed, Damage, Defense, GameCamera, GameState, Health,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<PlayerState>()
            .add_systems(Startup, prepare_player_resources)
            .add_systems(OnEnter(PlayerState::Idle), player_start_idle)
            .add_systems(OnEnter(PlayerState::Run), player_start_run)
            .add_systems(OnEnter(PlayerState::Attack), player_start_attack)
            .add_systems(OnEnter(PlayerState::Dead), player_start_dead)
            .add_systems(
                Update,
                (player_run, camera_follow_player).run_if(in_state(GameState::Running)),
            )
            .add_systems(
                Update,
                (player_attack, on_attack_finish).run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Resource, Debug)]
pub struct PlayerResources {
    idle_texture: Handle<Image>,
    idle_animation_config: AnimationConfig,

    run_texture: Handle<Image>,
    run_animation_config: AnimationConfig,

    attack_texture: Handle<Image>,
    attack_animation_config: AnimationConfig,

    dead_texture: Handle<Image>,
    dead_animation_config: AnimationConfig,

    texture_atlas: TextureAtlas,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpeed(pub f32);

// Run -> Idle -> Run
//         |   -> Attack
//         |      |
//          <---- Idle
//             -> Dead
#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InGame)]
pub enum PlayerState {
    #[default]
    Idle,
    Run,
    Attack,
    Dead,
}

fn prepare_player_resources(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let idle_texture = asset_server.load("player/alex_idle_sheet.png");
    let idle_animation_config = AnimationConfig::new(1, 5, 10, AllAnimations::PlayerIdle, false);

    let run_texture = asset_server.load("player/alex_run_sheet.png");
    let run_animation_config = AnimationConfig::new(1, 5, 10, AllAnimations::PlayerRun, false);

    let attack_texture = asset_server.load("player/alex_attack_sheet.png");
    let attack_animation_config = AnimationConfig::new(1, 5, 10, AllAnimations::PlayerAttack, true);

    let dead_texture = asset_server.load("player/alex_dead_sheet.png");
    let dead_animation_config = AnimationConfig::new(1, 5, 10, AllAnimations::PlayerDead, true);

    let texture_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 1, None, None);
    let atlas_handle = texture_atlas_layouts.add(texture_layout);
    let texture_atlas = TextureAtlas {
        layout: atlas_handle,
        index: 1,
    };

    commands.insert_resource(PlayerResources {
        idle_texture,
        idle_animation_config,

        run_texture,
        run_animation_config,

        attack_texture,
        attack_animation_config,

        dead_texture,
        dead_animation_config,

        texture_atlas,
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
        player_resources.texture_atlas.clone(),
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

fn player_start_idle(
    player_resources: Res<PlayerResources>,
    mut player: Query<(&mut AnimationConfig, &mut Handle<Image>, &mut TextureAtlas)>,
) {
    let Ok((mut config, mut texture, mut atlas)) = player.get_single_mut() else {
        return;
    };

    *texture = player_resources.idle_texture.clone();
    atlas.index = config.first_sprite_index;
    *config = player_resources.idle_animation_config.clone();
}

fn player_start_run(
    player_resources: Res<PlayerResources>,
    mut player: Query<(&mut AnimationConfig, &mut Handle<Image>, &mut TextureAtlas)>,
) {
    let Ok((mut config, mut texture, mut atlas)) = player.get_single_mut() else {
        return;
    };

    *texture = player_resources.run_texture.clone();
    atlas.index = config.first_sprite_index;
    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
    *config = player_resources.run_animation_config.clone();
}

fn player_start_attack(
    player_resources: Res<PlayerResources>,
    mut player: Query<(&mut AnimationConfig, &mut Handle<Image>, &mut TextureAtlas)>,
) {
    let Ok((mut config, mut texture, mut atlas)) = player.get_single_mut() else {
        return;
    };

    *texture = player_resources.attack_texture.clone();
    atlas.index = config.first_sprite_index;
    *config = player_resources.attack_animation_config.clone();
}

fn player_start_dead(
    player_resources: Res<PlayerResources>,
    mut player: Query<(&mut AnimationConfig, &mut Handle<Image>, &mut TextureAtlas)>,
) {
    let Ok((mut config, mut texture, mut atlas)) = player.get_single_mut() else {
        return;
    };

    *texture = player_resources.dead_texture.clone();
    atlas.index = config.first_sprite_index;
    *config = player_resources.dead_animation_config.clone();
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

fn player_attack(
    time: Res<Time>,
    mut player: Query<&mut AttackSpeed, With<Player>>,
    mut player_state: ResMut<NextState<PlayerState>>,
) {
    let Ok(mut player_attack_speed) = player.get_single_mut() else {
        return;
    };

    player_attack_speed.0.tick(time.delta());

    if player_attack_speed.0.finished() {
        player_state.set(PlayerState::Attack);
    }
}

fn on_attack_finish(
    items: Res<Items>,
    inventory: Res<Inventory>,
    player: Query<&Damage, With<Player>>,
    mut event_reader: EventReader<AnimationFinished>,
    mut event_writer: EventWriter<DamageEnemy>,
    mut player_state: ResMut<NextState<PlayerState>>,
) {
    let Ok(player_damage) = player.get_single() else {
        return;
    };

    for e in event_reader.read() {
        if e.0 == AllAnimations::PlayerAttack {
            let damage = player_damage.0
                + inventory
                    .active_items
                    .iter()
                    .map(|item_idx| {
                        if let Some(i) = item_idx {
                            items[*i].item.add_damage()
                        } else {
                            0.0
                        }
                    })
                    .sum::<f32>();

            event_writer.send(DamageEnemy(damage));
            player_state.set(PlayerState::Idle);
        }
    }
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
