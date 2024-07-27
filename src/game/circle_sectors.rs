use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Wireframe2d},
};
use rand::Rng;
use std::{
    f32::consts::*,
    ops::{Index, IndexMut},
};

use crate::{
    ui::in_game::{BackpackSectorId, SelectedSectionButton},
    GlobalState,
};

use super::{
    chest::{spawn_chest, ChestIdx, ChestResources, Chests},
    cursor::CursorSector,
    enemy::{spawn_enemy, Enemies, EnemyIdx},
    hp_bar::HpBarResources,
    inventory::Inventory,
    GameState, Player, Z_CHEST, Z_CLOCK, Z_ENEMY, Z_SECTOR_BACKGROUND, Z_SECTOR_GROUND,
};

pub const CIRCLE_RADIUS: f32 = 200.0;
pub const CIRCLE_INNER_RADIUS: f32 = 180.0;

const SECTORS_NUM: u8 = 8;
const SECTOR_ANGLE: f32 = PI * 2.0 / SECTORS_NUM as f32;
pub const SECTOR_THINGS: usize = 4;
const SECTOR_THING_GAP: f32 = SECTOR_ANGLE / 8.0;

pub struct SectorsPlugin;

impl Plugin for SectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SectorPlacedEvent>()
            .add_systems(PreStartup, prepare_sector_resources)
            .add_systems(Startup, spawn_sectors)
            .add_systems(
                Update,
                (
                    update_minute_arrow,
                    update_hour_arrow,
                    sector_detect_player,
                    sector_spawn_things,
                )
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(
                Update,
                (sector_update_selected, sector_update_not_selected)
                    .run_if(state_exists::<GameState>),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SectorResources {
    material_default: Handle<ColorMaterial>,
    material_arrow_default: Handle<ColorMaterial>,
    material_knob_default: Handle<ColorMaterial>,
    sector_mesh_default: Handle<Mesh>,
    circle_mesh_default: Handle<Mesh>,
    arrow_mesh_default: Handle<Mesh>,
    knob_mesh_default: Handle<Mesh>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct FullCycles(pub u8);

#[derive(Event, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorPlacedEvent;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorIdx(pub usize);

#[derive(Debug)]
pub struct SectorInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub material: Handle<ColorMaterial>,
    pub background: Handle<Image>,
    pub card: Handle<Image>,
    pub drop_rate: f32,
    pub enemies: Vec<EnemyIdx>,
    pub chests: Vec<ChestIdx>,
}

#[derive(Resource, Debug)]
pub struct Sectors(Vec<SectorInfo>);

impl Index<SectorIdx> for Sectors {
    type Output = SectorInfo;
    fn index(&self, index: SectorIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<SectorIdx> for Sectors {
    fn index_mut(&mut self, index: SectorIdx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorPosition(pub u8);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SectorTimer(Timer);

impl Default for SectorTimer {
    fn default() -> Self {
        // 5..10 seconds
        let duration = 1.0; //1.0 + rand::random::<f32>() * 1.0;
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

            let Some(mut sector_slots) = world.get_mut::<SectorSlots>(sector_slot_entity.entity)
            else {
                return;
            };
            sector_slots.0[sector_slot_entity.slot_position] = None;
        });
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
struct MinuteArrow;

#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
struct HourArrow;

pub fn sector_id_to_start_angle(id: u8) -> f32 {
    id as f32 * SECTOR_ANGLE - SECTOR_ANGLE / 2.0
}

pub fn position_to_sector_position(position: Vec3) -> u8 {
    let angle = position.angle_between(Vec3::Y);
    let mut sector_id = ((angle / (SECTOR_ANGLE / 2.0)).floor() as u8).div_ceil(2);
    if position.x < 0.0 && sector_id != 0 {
        sector_id = SECTORS_NUM - sector_id;
    }
    sector_id
}

pub fn next_section_position(section_id: u8) -> u8 {
    if section_id == SECTORS_NUM - 1 {
        0
    } else {
        section_id + 1
    }
}

fn prepare_sector_resources(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let material_arrow_default = materials.add(Color::srgb(0.1, 0.1, 0.1));
    let material_knob_default = materials.add(Color::srgb(0.6, 0.1, 0.1));
    // CircularSector uses half_angle underneath
    let sector_mesh_default = meshes.add(CircularSector::new(CIRCLE_RADIUS, SECTOR_ANGLE / 2.0));
    let circle_mesh_default = meshes.add(Circle {
        radius: CIRCLE_INNER_RADIUS,
    });
    let arrow_mesh_default = meshes.add(Rectangle::new(5.0, 200.0));
    let knob_mesh_default = meshes.add(Circle { radius: 15.0 });

    commands.insert_resource(SectorResources {
        material_default: material_default.clone(),
        material_arrow_default,
        material_knob_default,
        sector_mesh_default,
        circle_mesh_default,
        arrow_mesh_default,
        knob_mesh_default,
    });

    let mut sectors = Sectors(vec![]);
    sectors.0.push(SectorInfo {
        name: "Default",
        description: "Default sector",
        material: materials.add(Color::srgb_u8(174, 174, 169)),
        background: asset_server.load("sectors/zone_default_bent.png"),
        card: asset_server.load("sectors_cards/zone_default_card.png"),
        drop_rate: 0.9,
        enemies: vec![EnemyIdx(0)],
        chests: vec![ChestIdx(0)],
    });
    sectors.0.push(SectorInfo {
        name: "Green",
        description: "Green sector",
        material: materials.add(Color::srgb_u8(180, 195, 190)),
        background: asset_server.load("sectors/zone_green_bent.png"),
        card: asset_server.load("sectors_cards/zone_green_card.png"),
        drop_rate: 0.9,
        enemies: vec![EnemyIdx(1)],
        chests: vec![ChestIdx(1)],
    });
    sectors.0.push(SectorInfo {
        name: "Yellow",
        description: "Yellow sector",
        material: materials.add(Color::srgb_u8(253, 252, 205)),
        background: asset_server.load("sectors/zone_yellow_bent.png"),
        card: asset_server.load("sectors_cards/zone_yellow_card.png"),
        drop_rate: 0.9,
        enemies: vec![EnemyIdx(2)],
        chests: vec![ChestIdx(2)],
    });
    sectors.0.push(SectorInfo {
        name: "Grey",
        description: "Grey sector",
        material: materials.add(Color::srgb_u8(125, 169, 157)),
        background: asset_server.load("sectors/zone_grey_bent.png"),
        card: asset_server.load("sectors_cards/zone_grey_card.png"),
        drop_rate: 0.9,
        enemies: vec![EnemyIdx(3)],
        chests: vec![ChestIdx(3)],
    });
    sectors.0.push(SectorInfo {
        name: "Brown",
        description: "Brown sector",
        material: materials.add(Color::srgb_u8(128, 93, 71)),
        background: asset_server.load("sectors/zone_brown_bent.png"),
        card: asset_server.load("sectors_cards/zone_brown_card.png"),
        drop_rate: 0.9,
        enemies: vec![EnemyIdx(3)],
        chests: vec![ChestIdx(3)],
    });
    commands.insert_resource(sectors);

    commands.insert_resource(FullCycles(0));
}

fn spawn_sectors(
    sectors: Res<Sectors>,
    sector_resources: Res<SectorResources>,
    mut commands: Commands,
) {
    // Sectors
    for i in 0..SECTORS_NUM {
        let mut transform = Transform::from_xyz(0.0, 0.0, Z_SECTOR_GROUND);
        let rotation = PI / (SECTORS_NUM / 2) as f32 * i as f32;
        // Rotation happens ccw, so make it cw.
        transform.rotate_local_z(-rotation);

        let sector_idx = SectorIdx(rand::thread_rng().gen_range(0..5));
        let sector_info = &sectors.0[sector_idx.0];
        let material = sector_info.material.clone();
        commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: sector_resources.sector_mesh_default.clone().into(),
                    material,
                    transform,
                    ..default()
                },
                SectorPosition(i),
                sector_idx,
                SectorTimer::default(),
                SectorSlots::default(),
            ))
            .with_children(|builder| {
                builder.spawn((
                    SpriteBundle {
                        sprite: Sprite::default(),
                        transform: Transform::from_xyz(
                            0.0,
                            CIRCLE_RADIUS + 15.0,
                            Z_SECTOR_BACKGROUND,
                        )
                        .with_scale(Vec3::ONE * 0.35),
                        texture: sector_info.background.clone(),
                        ..Default::default()
                    },
                    SectorPosition(i),
                ));
            });
    }

    // Center
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: sector_resources.circle_mesh_default.clone().into(),
            material: sector_resources.material_default.clone(),
            transform: Transform::from_xyz(0.0, 0.0, Z_CLOCK),
            ..default()
        },
        StateScoped(GlobalState::InGame),
    ));

    // Minute arrow
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: sector_resources.arrow_mesh_default.clone().into(),
            material: sector_resources.material_arrow_default.clone(),
            transform: Transform::from_xyz(0.0, 70.0, Z_CLOCK + 2.0),
            ..default()
        },
        MinuteArrow,
        StateScoped(GlobalState::InGame),
    ));

    // Hour arrow
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: sector_resources.arrow_mesh_default.clone().into(),
            material: sector_resources.material_arrow_default.clone(),
            transform: Transform::from_xyz(0.0, 50.0, Z_CLOCK + 2.0)
                .with_scale(Vec3::new(2.0, 0.8, 1.0)),
            ..default()
        },
        HourArrow,
        StateScoped(GlobalState::InGame),
    ));

    // Knob arrow
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: sector_resources.knob_mesh_default.clone().into(),
            material: sector_resources.material_knob_default.clone(),
            transform: Transform::from_xyz(0.0, 0.0, Z_CLOCK + 3.0),
            ..default()
        },
        StateScoped(GlobalState::InGame),
    ));

    // Numbers
    for i in 1..=12 {
        let top_position = Vec3::new(0.0, 150.0, Z_CLOCK + 1.0);
        let angle = PI / 6.0 * i as f32;
        let rotation = Quat::from_rotation_z(-angle);
        let rotated = rotation.mul_vec3(top_position);
        let transform = Transform::from_translation(rotated);
        commands.spawn((Text2dBundle {
            text: Text::from_section(
                format!("{}", i),
                TextStyle {
                    font_size: 40.0,
                    ..Default::default()
                },
            ),
            transform,
            ..default()
        },));
    }
}

fn update_minute_arrow(
    player: Query<&Transform, (With<Player>, Without<MinuteArrow>)>,
    mut minute_arrow: Query<&mut Transform, (With<MinuteArrow>, Without<Player>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let Ok(mut minute_transform) = minute_arrow.get_single_mut() else {
        return;
    };
    let mut t = Transform::from_xyz(0.0, 70.0, Z_CLOCK + 2.0);
    t.rotate_around(Vec3::ZERO, player_transform.rotation);
    *minute_transform = t;
}

fn update_hour_arrow(
    full_cycles: Res<FullCycles>,
    player: Query<&Transform, (With<Player>, Without<MinuteArrow>, Without<HourArrow>)>,
    minute_arrow: Query<&mut Transform, (With<MinuteArrow>, Without<Player>, Without<HourArrow>)>,
    mut hour_arrow: Query<&mut Transform, (With<HourArrow>, Without<Player>, Without<MinuteArrow>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let Ok(minute_transform) = minute_arrow.get_single() else {
        return;
    };

    let Ok(mut hour_transform) = hour_arrow.get_single_mut() else {
        return;
    };

    let mut minute_angle = minute_transform.translation.angle_between(Vec3::Y);
    if player_transform.translation.x < 0.0 {
        minute_angle = 2.0 * PI - minute_angle;
    }

    let hour_arrow_angle = PI / 6.0 * full_cycles.0 as f32 + PI / 6.0 * minute_angle / (2.0 * PI);

    let mut t = Transform::from_xyz(0.0, 50.0, Z_CLOCK + 2.0).with_scale(Vec3::new(2.0, 0.8, 1.0));
    t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-hour_arrow_angle));
    *hour_transform = t;
}

fn sector_detect_player(
    player: Query<&Transform, With<Player>>,
    mut full_cycles: ResMut<FullCycles>,
    mut local: Local<u8>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let sector_id = position_to_sector_position(player_transform.translation);

    if sector_id != *local {
        if sector_id == 0 {
            full_cycles.0 += 1;
        }
        println!("player is in the sector: {sector_id}");
        *local = sector_id;
    }
}

fn sector_update_selected(
    sectors: Res<Sectors>,
    inventory: Res<Inventory>,
    cursor_sector: Res<CursorSector>,
    selected_section_button: Res<SelectedSectionButton>,
    buttons: Query<&BackpackSectorId>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut sectors_bottom: Query<(&SectorPosition, &mut SectorIdx, &mut Handle<ColorMaterial>)>,
    mut sectors_background: Query<(&SectorPosition, &mut Handle<Image>)>,
    mut event_writer: EventWriter<SectorPlacedEvent>,
) {
    let Some(cursor_sector_position) = cursor_sector.0 else {
        return;
    };

    let Some(selected_button_entity) = selected_section_button.0 else {
        return;
    };

    let Ok(sector_id) = buttons.get(selected_button_entity) else {
        return;
    };

    let Some(sector_idx) = inventory.backpack_sectors[sector_id.0 as usize] else {
        return;
    };

    let to_be_placed_sector_info = &sectors[sector_idx];

    for (sector_position, mut current_sector_idx, mut material) in sectors_bottom.iter_mut() {
        if *sector_position == cursor_sector_position {
            *material = to_be_placed_sector_info.material.clone();
            if mouse_input.just_pressed(MouseButton::Left) {
                *current_sector_idx = sector_idx;
                event_writer.send(SectorPlacedEvent);
            }
            break;
        }
    }

    for (sector_position, mut background) in sectors_background.iter_mut() {
        if *sector_position == cursor_sector_position {
            *background = to_be_placed_sector_info.background.clone();
            break;
        }
    }
}

fn sector_update_not_selected(
    sectors: Res<Sectors>,
    cursor_sector: Res<CursorSector>,
    mut sectors_bottom: Query<(&SectorPosition, &SectorIdx, &mut Handle<ColorMaterial>)>,
    mut sectors_background: Query<(&SectorPosition, &mut Handle<Image>)>,
    mut local: Local<Option<SectorPosition>>,
) {
    if *local == cursor_sector.0 {
        return;
    }
    *local = cursor_sector.0;

    for (sector_position, sector_idx, mut material) in sectors_bottom.iter_mut() {
        let sector_info = &sectors[*sector_idx];
        if let Some(cs) = cursor_sector.0 {
            if *sector_position == cs {
                continue;
            }
        }
        *material = sector_info.material.clone();

        for (bg_sector_position, mut background) in sectors_background.iter_mut() {
            if *sector_position == *bg_sector_position {
                *background = sector_info.background.clone();
                break;
            }
        }
    }
}

fn sector_spawn_things(
    time: Res<Time>,
    chests: Res<Chests>,
    enemies: Res<Enemies>,
    sectors: Res<Sectors>,
    chest_resources: Res<ChestResources>,
    hp_bar_resources: Res<HpBarResources>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut s: Query<(
        Entity,
        &SectorPosition,
        &SectorIdx,
        &mut SectorTimer,
        &mut SectorSlots,
    )>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_sector_id = position_to_sector_position(player_transform.translation);
    let player_next_sector_id = next_section_position(player_sector_id);

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

                let enemy_spawned = if !sector_info.enemies.is_empty() {
                    let random_enemy_idx =
                        sector_info.enemies[thread_rng.gen_range(0..sector_info.enemies.len())];
                    let enemy_info = &enemies[random_enemy_idx];
                    if thread_rng.gen_bool(enemy_info.spawn_rate as f64) {
                        slots.0[empty_slot_position] = Some(SlotType::Enemy);

                        let mut t = Transform::from_xyz(0.0, CIRCLE_RADIUS + 30.0, Z_ENEMY)
                            .with_scale(Vec3::new(2.0, 2.0, 2.0));
                        t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-angle));

                        spawn_enemy(
                            &mut commands,
                            enemies.as_ref(),
                            random_enemy_idx,
                            *id,
                            hp_bar_resources.as_ref(),
                            t,
                        )
                        .insert(SectorSlotEntity {
                            entity,
                            slot_position: empty_slot_position,
                        });
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !enemy_spawned && !sector_info.chests.is_empty() {
                    let random_chest_idx =
                        sector_info.chests[thread_rng.gen_range(0..sector_info.chests.len())];
                    let chest_info = &chests[random_chest_idx];

                    if thread_rng.gen_bool(chest_info.spawn_rate as f64) {
                        slots.0[empty_slot_position] = Some(SlotType::Item);

                        let mut t = Transform::from_xyz(0.0, CIRCLE_RADIUS + 5.0, Z_CHEST);
                        t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-angle));

                        spawn_chest(
                            &mut commands,
                            chests.as_ref(),
                            chest_resources.as_ref(),
                            random_chest_idx,
                            *id,
                            t,
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
