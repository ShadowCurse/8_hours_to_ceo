use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Wireframe2d},
};
use rand::Rng;
use std::f32::consts::*;

use crate::GlobalState;

use super::{enemy::EnemyResources, GameRenderLayer, GameState, Player};

const SECTORS_NUM: u8 = 8;
const SECTOR_GAP: f32 = PI * 2.0 / 256.0;
const SECTOR_ANGLE: f32 = PI * 2.0 / SECTORS_NUM as f32;
const SECTOR_ANGLE_WITH_GAP: f32 = SECTOR_ANGLE - SECTOR_GAP * 2.0;

pub struct SectorsPlugin;

impl Plugin for SectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_sector_resources)
            .add_systems(OnEnter(GameState::Preparing), spawn_sectors)
            .add_systems(
                Update,
                (sector_detect_player, sector_spawn_things).run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SectorResources {
    material_default: Handle<ColorMaterial>,
    material_green: Handle<ColorMaterial>,
    material_red: Handle<ColorMaterial>,
    material_orange: Handle<ColorMaterial>,
    mesh_default: Handle<Mesh>,
    circle_mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SectorType {
    #[default]
    Default,
    Green,
    Red,
    Orange,
}

impl SectorType {
    fn random() -> Self {
        let idx: u8 = rand::thread_rng().gen_range(0..4);
        unsafe { std::mem::transmute(idx) }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorId(u8);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SectorTimer(Timer);

impl Default for SectorTimer {
    fn default() -> Self {
        // 5..10 seconds
        // let duration = 5.0 + rand::random::<f32>() * 5.0;
        Self(Timer::from_seconds(0.0, TimerMode::Repeating))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotType {
    Enemy,
    Item,
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub struct SectorSlots([Option<SlotType>; 4]);

pub fn sector_id_to_start_angle(id: u8) -> f32 {
    id as f32 * SECTOR_ANGLE - SECTOR_ANGLE / 2.0
}

pub fn position_to_sector_id(position: Vec3) -> u8 {
    let angle = position.angle_between(Vec3::Y);
    let mut sector_id = ((angle / (SECTOR_ANGLE / 2.0)).floor() as u8).div_ceil(2);
    if position.x < 0.0 && sector_id != 0 {
        sector_id = SECTORS_NUM - sector_id;
    }
    sector_id
}

fn prepare_sector_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let material_green = materials.add(Color::srgb(0.2, 0.8, 0.2));
    let material_red = materials.add(Color::srgb(0.8, 0.2, 0.2));
    let material_orange = materials.add(Color::srgb(0.8, 0.4, 0.2));
    // CircularSector uses half_angle underneath
    let mesh_default = meshes.add(CircularSector::new(200.0, SECTOR_ANGLE_WITH_GAP / 2.0));

    let circle_mesh_default = meshes.add(Circle { radius: 180.0 });

    commands.insert_resource(SectorResources {
        material_default,
        material_green,
        material_red,
        material_orange,
        mesh_default,
        circle_mesh_default,
    });
}

fn spawn_sectors(
    game_render_layer: Res<GameRenderLayer>,
    sector_resources: Res<SectorResources>,
    mut commands: Commands,
) {
    // Sectors
    for i in 0..SECTORS_NUM {
        let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
        let rotation = PI / (SECTORS_NUM / 2) as f32 * i as f32;
        // Rotation happens ccw, so make it cw.
        transform.rotate_local_z(-rotation);
        let st = SectorType::random();
        let material = match st {
            SectorType::Default => sector_resources.material_default.clone(),
            SectorType::Green => sector_resources.material_green.clone(),
            SectorType::Red => sector_resources.material_red.clone(),
            SectorType::Orange => sector_resources.material_orange.clone(),
        };
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: sector_resources.mesh_default.clone().into(),
                material,
                transform,
                ..default()
            },
            SectorId(i),
            st,
            SectorTimer::default(),
            SectorSlots::default(),
            game_render_layer.layer.clone(),
            StateScoped(GlobalState::InGame),
        ));
    }

    // Center
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: sector_resources.circle_mesh_default.clone().into(),
            material: sector_resources.material_default.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));
}

fn sector_detect_player(player: Query<&Transform, With<Player>>, mut local: Local<u8>) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let sector_id = position_to_sector_id(player_transform.translation);

    if sector_id != *local {
        println!("player is in the sector: {sector_id}");
        *local = sector_id;
    }
}

fn sector_spawn_things(
    time: Res<Time>,
    game_render_layer: Res<GameRenderLayer>,
    enemy_resources: Res<EnemyResources>,
    mut commands: Commands,
    mut sectors: Query<(&SectorId, &SectorType, &mut SectorTimer, &mut SectorSlots)>,
) {
    for (id, st, mut timer, mut slots) in sectors.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            if let Some(empty_slot_position) = slots.0.iter().position(|slot| slot.is_none()) {
                slots.0[empty_slot_position] = Some(SlotType::Enemy);

                let angle = sector_id_to_start_angle(id.0)
                    + SECTOR_ANGLE / 4.0 * empty_slot_position as f32
                    + SECTOR_ANGLE / 8.0;

                let mut t = Transform::from_xyz(0.0, 210.0, 0.0);
                t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-angle));

                let material = match st {
                    SectorType::Default => enemy_resources.material_default.clone(),
                    SectorType::Green => enemy_resources.material_green.clone(),
                    SectorType::Red => enemy_resources.material_red.clone(),
                    SectorType::Orange => enemy_resources.material_orange.clone(),
                };
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: enemy_resources.mesh_default.clone().into(),
                        material,
                        transform: t,
                        ..default()
                    },
                    game_render_layer.layer.clone(),
                    StateScoped(GlobalState::InGame),
                ));
            }
        }
    }
}