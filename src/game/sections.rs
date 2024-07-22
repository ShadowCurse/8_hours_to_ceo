use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Wireframe2d},
};

use crate::GlobalState;

use super::{GameRenderLayer, GameState, Player};

pub struct SectionsPlugin;

impl Plugin for SectionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Preparing), spawn_sections)
            .add_systems(
                Update,
                (section_detect_player).run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SectorMaterials {
    default: Handle<ColorMaterial>,
    with_player: Handle<ColorMaterial>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircleSector(u8);

fn spawn_sections(
    game_render_layer: Res<GameRenderLayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let section_default_material = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let section_with_player_material = materials.add(Color::srgb(0.2, 0.8, 0.2));

    let mesh = meshes.add(CircularSector::new(200.0, std::f32::consts::FRAC_PI_8));

    // Sectors
    for i in 0..8 {
        let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
        // Rotate PI/8 more to start section at 0/12
        transform
            .rotate_local_z(std::f32::consts::FRAC_PI_8 + std::f32::consts::FRAC_PI_4 * i as f32);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: section_default_material.clone(),
                transform,
                ..default()
            },
            CircleSector(i),
            game_render_layer.layer.clone(),
            StateScoped(GlobalState::InGame),
        ));
    }

    // Center
    let mesh = meshes.add(Circle { radius: 180.0 });
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material: section_default_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));

    commands.insert_resource(SectorMaterials {
        default: section_default_material,
        with_player: section_with_player_material,
    });
}

fn section_detect_player(
    section_materials: Res<SectorMaterials>,
    player: Query<&Transform, With<Player>>,
    mut sections: Query<(&CircleSector, &mut Handle<ColorMaterial>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let to_center = player_transform.translation.normalize();
    let angle = to_center.angle_between(Vec3::Y);

    let mut section_id = (angle / std::f32::consts::FRAC_PI_4).floor() as u8;

    if 0.0 < player_transform.translation.x {
        section_id = 7 - section_id;
    }

    let previous_section_id = if section_id == 7 { 0 } else { section_id + 1 };

    for (section, mut material_handle) in sections.iter_mut() {
        if section.0 == section_id {
            *material_handle = section_materials.with_player.clone();
        }
        if section.0 == previous_section_id {
            *material_handle = section_materials.default.clone();
        }
    }
}
