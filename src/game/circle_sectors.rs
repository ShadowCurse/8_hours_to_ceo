use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Wireframe2d},
};

use crate::GlobalState;

use super::{GameRenderLayer, GameState, Player};

pub struct SectorsPlugin;

impl Plugin for SectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Preparing), spawn_sectors)
            .add_systems(
                Update,
                (sector_detect_player, sector_spawn_things).run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SectorMaterials {
    default: Handle<ColorMaterial>,
    with_player: Handle<ColorMaterial>,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectorType {
    #[default]
    Default,
    Green,
    Red,
    Orange,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorId(u8);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct SectorTimer(Timer);

impl Default for SectorTimer {
    fn default() -> Self {
        // 5..10 seconds
        let duration = 5.0 + rand::random::<f32>() * 5.0;
        Self(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotType {
    Enemy,
    Item,
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub struct SectorSlots([Option<SlotType>; 4]);

fn spawn_sectors(
    game_render_layer: Res<GameRenderLayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let sector_default_material = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let sector_with_player_material = materials.add(Color::srgb(0.2, 0.8, 0.2));

    let mesh = meshes.add(CircularSector::new(200.0, std::f32::consts::FRAC_PI_8));

    // Sectors
    for i in 0..8 {
        let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
        // Rotate PI/8 more to start sector at 0/12
        transform
            .rotate_local_z(std::f32::consts::FRAC_PI_8 + std::f32::consts::FRAC_PI_4 * i as f32);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: sector_default_material.clone(),
                transform,
                ..default()
            },
            SectorId(i),
            SectorType::default(),
            SectorTimer::default(),
            SectorSlots::default(),
            game_render_layer.layer.clone(),
            StateScoped(GlobalState::InGame),
        ));
    }

    // Center
    let mesh = meshes.add(Circle { radius: 180.0 });
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material: sector_default_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));

    commands.insert_resource(SectorMaterials {
        default: sector_default_material,
        with_player: sector_with_player_material,
    });
}

fn sector_detect_player(
    sector_materials: Res<SectorMaterials>,
    player: Query<&Transform, With<Player>>,
    mut sectors: Query<(&SectorId, &mut Handle<ColorMaterial>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let to_center = player_transform.translation.normalize();
    let angle = to_center.angle_between(Vec3::Y);

    let mut sector_id = (angle / std::f32::consts::FRAC_PI_4).floor() as u8;

    if 0.0 < player_transform.translation.x {
        sector_id = 7 - sector_id;
    }

    let previous_sector_id = if sector_id == 7 { 0 } else { sector_id + 1 };

    for (sector, mut material_handle) in sectors.iter_mut() {
        if sector.0 == sector_id {
            *material_handle = sector_materials.with_player.clone();
        }
        if sector.0 == previous_sector_id {
            *material_handle = sector_materials.default.clone();
        }
    }
}

fn sector_spawn_things(
    time: Res<Time>,
    mut sectors: Query<(&SectorId, &mut SectorTimer, &mut SectorSlots)>,

    mut commands: Commands,
    game_render_layer: Res<GameRenderLayer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (id, mut timer, mut slots) in sectors.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            if let Some(empty_slot_position) = slots.0.iter().position(|slot| slot.is_none()) {
                slots.0[empty_slot_position] = Some(SlotType::Enemy);

                let material = materials.add(Color::srgb(0.9, 0.2, 0.2));
                let mesh = meshes.add(Circle { radius: 10.0 });

                // Take end of the sector, subtract enought for the slot and take
                // middle of the slot.
                let angle = (7 - id.0) as f32 * std::f32::consts::FRAC_PI_4
                    - std::f32::consts::PI / 16.0 * empty_slot_position as f32
                    - std::f32::consts::PI / 32.0;

                let mut t = Transform::from_xyz(0.0, 210.0, 0.0);
                t.rotate_around(Vec3::ZERO, Quat::from_rotation_z(angle));

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: mesh.into(),
                        material,
                        transform: t,
                        ..default()
                    },
                    Wireframe2d,
                    game_render_layer.layer.clone(),
                    StateScoped(GlobalState::InGame),
                ));
            }
        }
    }
}
