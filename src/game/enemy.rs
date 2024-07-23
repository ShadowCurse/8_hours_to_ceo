use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    sprite::MaterialMesh2dBundle,
};

use crate::GlobalState;

use super::{
    circle_sectors::{SectorId, SectorType, SECTOR_THINGS},
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyDropInfo {
    pub items: Vec<usize>,
    pub spells: Vec<usize>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EnemiesDropInfo {
    infos: [EnemyDropInfo; SECTOR_THINGS],
}

impl EnemiesDropInfo {
    pub fn get(&self, sector_type: SectorType) -> &EnemyDropInfo {
        let idx = sector_type as usize;
        &self.infos[idx]
    }
}

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

    let enemies_drop_info = EnemiesDropInfo {
        infos: [
            // Default
            EnemyDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
            // Green
            EnemyDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
            // Red
            EnemyDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
            // Orange
            EnemyDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
        ],
    };

    commands.insert_resource(enemies_drop_info);
}

pub fn spawn_enemy<'a>(
    commands: &'a mut Commands,
    enemy_resources: &EnemyResources,
    sector_type: SectorType,
    sector_id: u8,
    transform: Transform,
    render_layer: RenderLayers,
) -> EntityCommands<'a> {
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
        sector_type,
        render_layer,
        StateScoped(GlobalState::InGame),
    ))
}
