use bevy::{prelude::*, render::view::RenderLayers, sprite::MaterialMesh2dBundle};

use crate::GlobalState;

use super::{
    circle_sectors::{SectorId, SectorType},
    AttackSpeed, Damage, Health,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_enemy_resources);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EnemyResources {
    pub material_default: Handle<ColorMaterial>,
    pub material_green: Handle<ColorMaterial>,
    pub material_red: Handle<ColorMaterial>,
    pub material_orange: Handle<ColorMaterial>,
    pub mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleEnemy;

fn prepare_enemy_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let material_green = materials.add(Color::srgb(0.2, 0.8, 0.2));
    let material_red = materials.add(Color::srgb(0.8, 0.2, 0.2));
    let material_orange = materials.add(Color::srgb(0.8, 0.4, 0.2));
    let mesh_default = meshes.add(Circle { radius: 10.0 });

    commands.insert_resource(EnemyResources {
        material_default,
        material_green,
        material_red,
        material_orange,
        mesh_default,
    });
}

pub fn spawn_enemy(
    commands: &mut Commands,
    enemy_resources: &EnemyResources,
    sector_type: SectorType,
    sector_id: u8,
    transform: Transform,
    render_layer: RenderLayers,
) {
    let material = match sector_type {
        SectorType::Default => enemy_resources.material_default.clone(),
        SectorType::Green => enemy_resources.material_green.clone(),
        SectorType::Red => enemy_resources.material_red.clone(),
        SectorType::Orange => enemy_resources.material_orange.clone(),
    };
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: enemy_resources.mesh_default.clone().into(),
            material,
            transform,
            ..default()
        },
        Enemy,
        Health(10.0),
        Damage(1.0),
        AttackSpeed::new(1.0),
        SectorId(sector_id),
        render_layer,
        StateScoped(GlobalState::InGame),
    ));
}
