use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{MaterialMesh2dBundle, Wireframe2d},
    window::{PrimaryWindow, WindowResized},
};
use rand::Rng;

use crate::{
    ui::in_game::{UI_RIGHT_SIZE, UI_TOP_SIZE},
    GlobalState,
};

pub mod chest;
pub mod circle_sectors;
pub mod enemy;
pub mod inventory;
pub mod items;
pub mod spells;

use chest::{Chest, ChestIdx, Chests, ChestsPlugin, InteractedChest};
use circle_sectors::{position_to_sector_position, SectorPosition, SectorsPlugin};
use enemy::{BattleEnemy, Enemies, Enemy, EnemyIdx, EnemyPlugin};
use inventory::{Inventory, InventoryPlugin};
use items::{Items, ItemsPlugin};
use spells::{Spells, SpellsPlugin};

const INTERACTION_DISTANCE: f32 = 30.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChestsPlugin,
            SectorsPlugin,
            EnemyPlugin,
            InventoryPlugin,
            ItemsPlugin,
            SpellsPlugin,
        ))
        .add_event::<InventoryUpdate>()
        .add_sub_state::<GameState>()
        .add_systems(Startup, setup_game)
        .add_systems(OnEnter(GameState::Preparing), spawn_base_game)
        .add_systems(
            Update,
            (
                on_window_resize,
                player_run,
                camera_follow_player,
                initiate_battle,
                initiate_pickup,
            )
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update,
            (battle_auto_attack, battle_end_check).run_if(in_state(GameState::Battle)),
        )
        .add_systems(Update, (pickup_end).run_if(in_state(GameState::Pickup)))
        .add_systems(
            OnTransition {
                exited: GameState::Running,
                entered: GameState::Paused,
            },
            move_camera_default,
        )
        .add_systems(Update, game_pause.run_if(state_exists::<GameState>));
    }
}

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InGame)]
pub enum GameState {
    #[default]
    Preparing,
    Running,
    Pickup,
    Battle,
    Paused,
}

#[derive(Event, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InventoryUpdate;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameImage {
    pub image: Handle<Image>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GameRenderLayer {
    layer: RenderLayers,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameCamera;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpeed(f32);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health(pub f32);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Damage(pub f32);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Defense(pub f32);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct AttackSpeed(pub Timer);

impl AttackSpeed {
    pub fn new(seconds: f32) -> Self {
        Self(Timer::from_seconds(seconds, TimerMode::Repeating))
    }
}

fn setup_game(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(primary_window) = windows.get_single() else {
        return;
    };

    let size = Extent3d {
        width: (primary_window.resolution.width() * (100.0 - UI_RIGHT_SIZE) / 100.0) as u32,
        height: (primary_window.resolution.height() * (100.0 - UI_TOP_SIZE) / 100.0) as u32,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    let first_pass_layer = RenderLayers::layer(1);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: image_handle.clone().into(),
                clear_color: Color::BLACK.into(),
                ..default()
            },
            ..default()
        },
        GameCamera,
        first_pass_layer.clone(),
    ));

    commands.insert_resource(GameRenderLayer {
        layer: first_pass_layer,
    });
    commands.insert_resource(GameImage {
        image: image_handle,
    });
}

fn on_window_resize(
    game_image: Res<GameImage>,
    mut images: ResMut<Assets<Image>>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for e in resize_reader.read() {
        let image = images.get_mut(&game_image.image).unwrap();
        let size = Extent3d {
            width: (e.width * (100.0 - UI_RIGHT_SIZE) / 100.0) as u32,
            height: (e.height * (100.0 - UI_TOP_SIZE) / 100.0) as u32,
            ..default()
        };
        image.resize(size);
    }
}

fn spawn_base_game(
    game_render_layer: Res<GameRenderLayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle { radius: 20.0 });
    let material = materials.add(Color::srgb(0.1, 0.9, 0.2));

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform::from_xyz(0.0, 220.0, 1.0),
            ..default()
        },
        Player,
        PlayerSpeed(0.1),
        Health(100.0),
        Damage(5.0),
        AttackSpeed::new(0.5),
        Defense(0.0),
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));

    game_state.set(GameState::Running);
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

fn game_pause(
    key_input: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut game_state_next: ResMut<NextState<GameState>>,
    mut local: Local<GameState>,
) {
    if key_input.just_pressed(KeyCode::Space) {
        if game_state.get() == &GameState::Paused {
            game_state_next.set(*local);
        } else {
            *local = *game_state.get();
            game_state_next.set(GameState::Paused);
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

fn move_camera_default(mut camera: Query<&mut Transform, With<GameCamera>>) {
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };
    *camera_transform = Transform::default();
}

fn initiate_battle(
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemies: Query<(Entity, &Transform, &SectorPosition), (With<Enemy>, Without<Player>)>,
    mut commands: Commands,
    mut game_sate: ResMut<NextState<GameState>>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_sector_id = position_to_sector_position(player_transform.translation);

    for (enemy_entity, enemy_transform, sector_id) in enemies.iter() {
        if sector_id.0 != player_sector_id {
            continue;
        }
        if (enemy_transform.translation - player_transform.translation).length()
            < INTERACTION_DISTANCE
        {
            // Marke enemy as the one we fight
            commands
                .get_entity(enemy_entity)
                .unwrap()
                .insert(BattleEnemy);

            game_sate.set(GameState::Battle);
        }
    }
}

fn battle_auto_attack(
    time: Res<Time>,
    items: Res<Items>,
    inventory: Res<Inventory>,
    mut player: Query<
        (&Damage, &Defense, &mut Health, &mut AttackSpeed),
        (With<Player>, Without<BattleEnemy>),
    >,
    mut enemy: Query<
        (&Damage, &Defense, &mut Health, &mut AttackSpeed),
        (With<BattleEnemy>, Without<Player>),
    >,
) {
    let Ok((player_damage, player_defense, mut player_health, mut player_attack_speed)) =
        player.get_single_mut()
    else {
        return;
    };

    let Ok((enemy_damage, enemy_defense, mut enemy_health, mut enemy_attack_speed)) =
        enemy.get_single_mut()
    else {
        return;
    };

    player_attack_speed.0.tick(time.delta());
    enemy_attack_speed.0.tick(time.delta());

    if player_attack_speed.0.finished() {
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

        let damage = damage * (1.0 - enemy_defense.0);
        enemy_health.0 -= damage;
    }

    if enemy_attack_speed.0.finished() {
        let player_defense = player_defense.0
            + inventory
                .active_items
                .iter()
                .map(|item_idx| {
                    if let Some(i) = item_idx {
                        items[*i].item.add_defense()
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();

        let damage = enemy_damage.0 * (1.0 - player_defense);
        player_health.0 -= damage;
    }
}

fn battle_end_check(
    items: Res<Items>,
    spells: Res<Spells>,
    enemies: Res<Enemies>,
    enemy: Query<(Entity, &Health, &EnemyIdx), (With<BattleEnemy>, Without<Player>)>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut event_writer: EventWriter<InventoryUpdate>,
    mut game_state: ResMut<NextState<GameState>>,
    mut player: Query<(&mut Health, &mut AttackSpeed), (With<Player>, Without<BattleEnemy>)>,
) {
    let Ok((mut player_health, mut player_attack_speed)) = player.get_single_mut() else {
        return;
    };

    let Ok((enemy_entity, enemy_health, enemy_idx)) = enemy.get_single() else {
        return;
    };

    if enemy_health.0 <= 0.0 {
        player_attack_speed.0.reset();
        commands
            .get_entity(enemy_entity)
            .unwrap()
            .despawn_recursive();

        let enemy_info = &enemies[*enemy_idx];

        let mut thread_rng = rand::thread_rng();

        let random_item_idx = enemy_info.items[thread_rng.gen_range(0..enemy_info.items.len())];
        let item = &items[random_item_idx];
        if thread_rng.gen_bool(item.drop_rate as f64) {
            inventory.backpack_items.push(random_item_idx);
        }

        let random_spell_idx = enemy_info.spells[thread_rng.gen_range(0..enemy_info.spells.len())];
        let spell = &spells[random_spell_idx];
        if thread_rng.gen_bool(spell.drop_rate as f64) {
            inventory.backpack_spells.push(random_spell_idx);
        }

        let heal = inventory
            .active_items
            .iter()
            .map(|item_idx| {
                if let Some(i) = item_idx {
                    items[*i].item.heal()
                } else {
                    0.0
                }
            })
            .sum::<f32>();
        player_health.0 += heal;

        event_writer.send(InventoryUpdate);
        game_state.set(GameState::Running);
    }

    if player_health.0 == 0.0 {
        println!("Player died...");
    }
}

fn initiate_pickup(
    player: Query<&Transform, (With<Player>, Without<Chest>)>,
    chests: Query<(Entity, &Transform, &SectorPosition), (With<Chest>, Without<Player>)>,
    mut commands: Commands,
    mut game_sate: ResMut<NextState<GameState>>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_sector_id = position_to_sector_position(player_transform.translation);

    for (chest_entity, chest_transform, sector_id) in chests.iter() {
        if sector_id.0 != player_sector_id {
            continue;
        }
        if (chest_transform.translation - player_transform.translation).length()
            < INTERACTION_DISTANCE
        {
            // Mark chest as the one we interact with
            commands
                .get_entity(chest_entity)
                .unwrap()
                .insert(InteractedChest);

            game_sate.set(GameState::Pickup);
        }
    }
}

fn pickup_end(
    items: Res<Items>,
    chests: Res<Chests>,
    spells: Res<Spells>,
    chest: Query<(Entity, &ChestIdx), (With<InteractedChest>, Without<Player>)>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut event_writer: EventWriter<InventoryUpdate>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok((chest_entity, chest_idx)) = chest.get_single() else {
        return;
    };

    commands
        .get_entity(chest_entity)
        .unwrap()
        .despawn_recursive();

    let chest_info = &chests[*chest_idx];

    let mut thread_rng = rand::thread_rng();

    let random_item_idx = chest_info.items[thread_rng.gen_range(0..chest_info.items.len())];
    let item = &items[random_item_idx];
    if thread_rng.gen_bool(item.drop_rate as f64) {
        inventory.backpack_items.push(random_item_idx);
    }

    let random_spell_idx = chest_info.spells[thread_rng.gen_range(0..chest_info.spells.len())];
    let spell = &spells[random_spell_idx];
    if thread_rng.gen_bool(spell.drop_rate as f64) {
        inventory.backpack_spells.push(random_spell_idx);
    }

    event_writer.send(InventoryUpdate);
    game_state.set(GameState::Running);
}
