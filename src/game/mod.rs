use bevy::prelude::*;

use crate::GlobalState;

pub mod animation;
pub mod chest;
pub mod circle_sectors;
pub mod cursor;
pub mod enemy;
pub mod hp_bar;
pub mod inventory;
pub mod items;
pub mod player;
pub mod spells;

use animation::AnimationPlugin;
use chest::{Chest, ChestOppenedEvent, ChestsPlugin, InteractedChest};
use circle_sectors::{position_to_sector_position, SectorPosition, SectorsPlugin};
use cursor::CursorPlugin;
use enemy::{BattleEnemy, Enemy, EnemyDeadEvent, EnemyPlugin};
use hp_bar::{HpBarPlugin, HpBarResources};
use inventory::InventoryPlugin;
use items::ItemsPlugin;
use player::{spawn_player, Player, PlayerPlugin, PlayerResources, PlayerState};
use spells::SpellsPlugin;

const INTERACTION_DISTANCE: f32 = 30.0;

pub const Z_WALL: f32 = 0.0;
pub const Z_SECTORS: f32 = 1.0;
pub const Z_CLOCK_CENTER: f32 = 2.0;
pub const Z_CLOCK_NUMBERS: f32 = 3.0;
pub const Z_CLOCK_ARROWS: f32 = 4.0;
pub const Z_CLOCK_KNOB: f32 = 5.0;
pub const Z_SECTOR_BACKGROUND: f32 = 2.0;
pub const Z_ENEMY: f32 = 4.0;
pub const Z_CHEST: f32 = 4.0;
pub const Z_PLAYER: f32 = 5.0;

const CAMERA_FOLLOW_SPEED: f32 = 8.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AnimationPlugin,
            ChestsPlugin,
            SectorsPlugin,
            CursorPlugin,
            EnemyPlugin,
            HpBarPlugin,
            InventoryPlugin,
            ItemsPlugin,
            PlayerPlugin,
            SpellsPlugin,
        ))
        .add_sub_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Preparing), spawn_base_game)
        .add_systems(OnEnter(GlobalState::MainMenu), camera_target_main_menu)
        .add_systems(OnEnter(GameState::Running), camera_target_player)
        .add_systems(OnEnter(GameState::Battle), camera_target_player)
        .add_systems(OnEnter(GameState::Paused), camera_target_pause)
        .add_systems(OnEnter(GameState::Win), || println!("win"))
        .add_systems(
            Update,
            (initiate_battle, initiate_pickup).run_if(in_state(GameState::Running)),
        )
        .add_systems(Update, battle_end_check.run_if(in_state(GameState::Battle)))
        .add_systems(Update, pickup_end_check.run_if(in_state(GameState::Pickup)))
        .add_systems(Update, game_pause.run_if(state_exists::<GameState>))
        .add_systems(Update, camera_follow_target);
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
    Win,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameCamera {
    target: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GameCameraPossibleTarget {
    scale: Vec3,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct MainMenuCameraTarget;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PauseCameraTarget;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    current: f32,
    max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn percent(&self) -> f32 {
        self.current / self.max
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }

    pub fn heal(&mut self, heal: f32) {
        self.current = (self.current + heal).min(self.max);
    }
}

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

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        GameCameraPossibleTarget { scale: Vec3::ONE },
        PauseCameraTarget,
    ));
    let mm_target = commands
        .spawn((
            Transform::default(),
            GameCameraPossibleTarget {
                scale: Vec3::new(2.0, 2.0, 2.0),
            },
            MainMenuCameraTarget,
        ))
        .id();
    commands.spawn((
        Camera2dBundle {
            transform: Transform::default().with_scale(Vec3::new(2.0, 2.0, 2.0)),
            ..Default::default()
        },
        GameCamera { target: mm_target },
    ));
}

fn camera_follow_target(
    time: Res<Time>,
    ui_scale: Res<UiScale>,
    camera_possible_targets: Query<(&Transform, &GameCameraPossibleTarget), Without<GameCamera>>,
    mut camera: Query<(&GameCamera, &mut Transform), Without<GameCameraPossibleTarget>>,
) {
    let Ok((camera, mut camera_transform)) = camera.get_single_mut() else {
        return;
    };

    let Ok((target_transform, target)) = camera_possible_targets.get(camera.target) else {
        return;
    };

    let exp_decay = |a: Vec3, b: Vec3, dt: f32| b + (a - b) * (-CAMERA_FOLLOW_SPEED * dt).exp();
    camera_transform.translation = exp_decay(
        camera_transform.translation,
        target_transform.translation,
        time.delta_seconds(),
    );
    camera_transform.scale = exp_decay(
        camera_transform.scale,
        target.scale / ui_scale.0,
        time.delta_seconds(),
    );

    let exp_decay = |a: Vec4, b: Vec4, dt: f32| b + (a - b) * (-CAMERA_FOLLOW_SPEED * dt).exp();
    camera_transform.rotation = Quat::from_vec4(exp_decay(
        camera_transform.rotation.to_array().into(),
        target_transform.rotation.to_array().into(),
        time.delta_seconds(),
    ));
}

fn camera_target_main_menu(
    main_menu_target: Query<Entity, With<MainMenuCameraTarget>>,
    mut camera: Query<&mut GameCamera>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(mm_target) = main_menu_target.get_single() else {
        return;
    };

    camera.target = mm_target;
}

fn camera_target_player(player: Query<Entity, With<Player>>, mut camera: Query<&mut GameCamera>) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let Ok(player) = player.get_single() else {
        return;
    };

    camera.target = player;
}

fn camera_target_pause(
    pause_target: Query<Entity, With<PauseCameraTarget>>,
    mut camera: Query<&mut GameCamera>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let Ok(pause_target) = pause_target.get_single() else {
        return;
    };

    camera.target = pause_target;
}

fn spawn_base_game(
    hp_bar_resources: Res<HpBarResources>,
    player_resources: Res<PlayerResources>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut player_state: ResMut<NextState<PlayerState>>,
) {
    spawn_player(
        &mut commands,
        player_resources.as_ref(),
        hp_bar_resources.as_ref(),
        Transform::from_xyz(0.0, 230.0, Z_PLAYER).with_scale(Vec3::new(2.0, 2.0, 2.0)),
    );

    game_state.set(GameState::Running);
    player_state.set(PlayerState::Run);
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

fn initiate_battle(
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemies: Query<(Entity, &Transform, &SectorPosition), (With<Enemy>, Without<Player>)>,
    mut commands: Commands,
    mut game_sate: ResMut<NextState<GameState>>,
    mut player_state: ResMut<NextState<PlayerState>>,
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
            player_state.set(PlayerState::Idle);
        }
    }
}

fn battle_end_check(
    mut game_state: ResMut<NextState<GameState>>,
    mut event_reader: EventReader<EnemyDeadEvent>,
) {
    for _ in event_reader.read() {
        game_state.set(GameState::Running);
    }
}

fn initiate_pickup(
    player: Query<&Transform, (With<Player>, Without<Chest>)>,
    chests: Query<(Entity, &Transform, &SectorPosition), (With<Chest>, Without<Player>)>,
    mut commands: Commands,
    mut game_sate: ResMut<NextState<GameState>>,
    mut player_state: ResMut<NextState<PlayerState>>,
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
            player_state.set(PlayerState::Idle);
        }
    }
}

fn pickup_end_check(
    mut event_reader: EventReader<ChestOppenedEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _ in event_reader.read() {
        game_state.set(GameState::Running);
    }
}
