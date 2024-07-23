use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    sprite::MaterialMesh2dBundle,
};

use crate::GlobalState;

use super::{circle_sectors::SectorId, AttackSpeed, Damage, Defense, Health};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_enemy_resources);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EnemyResources {
    pub mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnemyIdx(pub usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleEnemy;

#[derive(Debug, Clone, PartialEq)]
pub struct EnemyInfo {
    pub material: Handle<ColorMaterial>,
    pub spawn_rate: f32,
    pub items: Vec<usize>,
    pub spells: Vec<usize>,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Enemies(pub Vec<EnemyInfo>);

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

    commands.insert_resource(EnemyResources { mesh_default });

    let mut enemies = Enemies(vec![]);

    // Default
    enemies.0.push(EnemyInfo {
        material: material_default,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
    });
    // Green
    enemies.0.push(EnemyInfo {
        material: material_green,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
    });
    // Red
    enemies.0.push(EnemyInfo {
        material: material_red,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
    });
    // Orange
    enemies.0.push(EnemyInfo {
        material: material_orange,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
    });

    commands.insert_resource(enemies);
}

pub fn spawn_enemy<'a>(
    commands: &'a mut Commands,
    enemies: &Enemies,
    enemy_resources: &EnemyResources,
    enemy_idx: EnemyIdx,
    sector_id: SectorId,
    transform: Transform,
    render_layer: RenderLayers,
) -> EntityCommands<'a> {
    let material = enemies.0[enemy_idx.0].material.clone();
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: enemy_resources.mesh_default.clone().into(),
            material,
            transform,
            ..default()
        },
        Enemy,
        Health(30.0),
        Damage(1.0),
        AttackSpeed::new(1.0),
        Defense(0.0),
        sector_id,
        enemy_idx,
        render_layer,
        StateScoped(GlobalState::InGame),
    ))
}
