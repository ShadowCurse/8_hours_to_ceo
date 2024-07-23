use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Wireframe2d},
};
use rand::Rng;
use std::f32::consts::*;

use crate::GlobalState;

use super::{
    chest::{spawn_chest, ChestIdx, ChestResources, Chests},
    enemy::{spawn_enemy, Enemies, EnemyIdx, EnemyResources},
    GameRenderLayer, GameState, Player,
};

const SECTORS_NUM: u8 = 8;
const SECTOR_GAP: f32 = PI * 2.0 / 256.0;
const SECTOR_ANGLE: f32 = PI * 2.0 / SECTORS_NUM as f32;
const SECTOR_ANGLE_WITH_GAP: f32 = SECTOR_ANGLE - SECTOR_GAP * 2.0;
pub const SECTOR_THINGS: usize = 4;
const SECTOR_THING_GAP: f32 = SECTOR_ANGLE / 8.0;

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
    mesh_default: Handle<Mesh>,
    circle_mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorIdx(pub usize);

#[derive(Debug)]
pub struct SectorInfo {
    pub name: &'static str,
    pub material: Handle<ColorMaterial>,
    pub drop_rate: f32,
    pub enemies: Vec<usize>,
    pub chests: Vec<usize>,
}

#[derive(Resource, Debug)]
pub struct Sectors(pub Vec<SectorInfo>);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorId(pub u8);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SectorTimer(Timer);

impl Default for SectorTimer {
    fn default() -> Self {
        // 5..10 seconds
        let duration = 1.0 + rand::random::<f32>() * 1.0;
        Self(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotType {
    Enemy,
    Item,
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub struct SectorSlots([Option<SlotType>; SECTOR_THINGS]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectorSlotEntity {
    entity: Entity,
    slot_position: usize,
}

impl Component for SectorSlotEntity {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, slot_entity, _component_id| {
            let sector_slot_entity = *world.get::<SectorSlotEntity>(slot_entity).unwrap();

            let mut sector_slots = world
                .get_mut::<SectorSlots>(sector_slot_entity.entity)
                .unwrap();
            sector_slots.0[sector_slot_entity.slot_position] = None;
        });
    }
}

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

pub fn next_section_id(section_id: u8) -> u8 {
    if section_id == SECTORS_NUM - 1 {
        0
    } else {
        section_id + 1
    }
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
        material_default: material_default.clone(),
        mesh_default,
        circle_mesh_default,
    });

    let mut sectors = Sectors(vec![]);
    sectors.0.push(SectorInfo {
        name: "Default",
        material: material_default,
        drop_rate: 0.9,
        enemies: vec![0],
        chests: vec![0],
    });
    sectors.0.push(SectorInfo {
        name: "Green",
        material: material_green,
        drop_rate: 0.9,
        enemies: vec![1],
        chests: vec![1],
    });
    sectors.0.push(SectorInfo {
        name: "Red",
        material: material_red,
        drop_rate: 0.9,
        enemies: vec![2],
        chests: vec![2],
    });
    sectors.0.push(SectorInfo {
        name: "Orange",
        material: material_orange,
        drop_rate: 0.9,
        enemies: vec![3],
        chests: vec![3],
    });
    commands.insert_resource(sectors);
}

fn spawn_sectors(
    sectors: Res<Sectors>,
    sector_resources: Res<SectorResources>,
    game_render_layer: Res<GameRenderLayer>,
    mut commands: Commands,
) {
    // Sectors
    for i in 0..SECTORS_NUM {
        let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
        let rotation = PI / (SECTORS_NUM / 2) as f32 * i as f32;
        // Rotation happens ccw, so make it cw.
        transform.rotate_local_z(-rotation);

        let sector_idx = SectorIdx(rand::thread_rng().gen_range(0..4));
        let material = sectors.0[sector_idx.0].material.clone();
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: sector_resources.mesh_default.clone().into(),
                material,
                transform,
                ..default()
            },
            SectorId(i),
            sector_idx,
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
    chests: Res<Chests>,
    enemies: Res<Enemies>,
    sectors: Res<Sectors>,
    enemy_resources: Res<EnemyResources>,
    chest_resources: Res<ChestResources>,
    game_render_layer: Res<GameRenderLayer>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut s: Query<(
        Entity,
        &SectorId,
        &SectorIdx,
        &mut SectorTimer,
        &mut SectorSlots,
    )>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_sector_id = position_to_sector_id(player_transform.translation);
    let player_next_sector_id = next_section_id(player_sector_id);

    for (entity, id, sector_idx, mut timer, mut slots) in s.iter_mut() {
        timer.0.tick(time.delta());

        // Don't spawn anything in the current and next zone
        if id.0 == player_sector_id || id.0 == player_next_sector_id {
            continue;
        }

        if timer.0.finished() {
            if let Some(empty_slot_position) = slots.0.iter().position(|slot| slot.is_none()) {
                let angle = sector_id_to_start_angle(id.0) + SECTOR_ANGLE / 2.0
                    - SECTOR_THING_GAP / 2.0 * (SECTOR_THINGS - 1) as f32
                    + SECTOR_THING_GAP * empty_slot_position as f32;

                let sector_info = &sectors.0[sector_idx.0];
                let mut thread_rng = rand::thread_rng();

                let random_enemy = thread_rng.gen_range(0..sector_info.enemies.len());
                let random_enemy_idx = sector_info.enemies[random_enemy];

                let enemy_info = &enemies.0[random_enemy_idx];
                if thread_rng.gen_bool(enemy_info.spawn_rate as f64) {
                    slots.0[empty_slot_position] = Some(SlotType::Enemy);

                    let mut t = Transform::from_xyz(0.0, 210.0, 0.0);
                    t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-angle));

                    spawn_enemy(
                        &mut commands,
                        enemies.as_ref(),
                        enemy_resources.as_ref(),
                        EnemyIdx(random_enemy_idx),
                        *id,
                        t,
                        game_render_layer.layer.clone(),
                    )
                    .insert(SectorSlotEntity {
                        entity,
                        slot_position: empty_slot_position,
                    });
                } else {
                    let random_chest = thread_rng.gen_range(0..sector_info.chests.len());
                    let random_chest_idx = sector_info.chests[random_chest];

                    let chest_info = &chests.0[random_chest_idx];

                    if thread_rng.gen_bool(chest_info.spawn_rate as f64) {
                        slots.0[empty_slot_position] = Some(SlotType::Item);

                        let mut t = Transform::from_xyz(0.0, 205.0, 0.0);
                        t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-angle));

                        spawn_chest(
                            &mut commands,
                            chests.as_ref(),
                            chest_resources.as_ref(),
                            ChestIdx(random_chest_idx),
                            *id,
                            t,
                            game_render_layer.layer.clone(),
                        )
                        .insert(SectorSlotEntity {
                            entity,
                            slot_position: empty_slot_position,
                        });
                    }
                }
            }
        }
    }
}
