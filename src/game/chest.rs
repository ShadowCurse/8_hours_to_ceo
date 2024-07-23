use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    sprite::MaterialMesh2dBundle,
};

use crate::GlobalState;

use super::circle_sectors::{SectorId, SectorType, SECTOR_THINGS};

pub struct ChestsPlugin;

impl Plugin for ChestsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_chest_resources);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ChestResources {
    pub material_default: Handle<ColorMaterial>,
    pub material_green: Handle<ColorMaterial>,
    pub material_red: Handle<ColorMaterial>,
    pub material_orange: Handle<ColorMaterial>,
    pub mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chest;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractedChest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChestDropInfo {
    pub items: Vec<usize>,
    pub spells: Vec<usize>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ChestsDropInfo {
    infos: [ChestDropInfo; SECTOR_THINGS],
}

impl ChestsDropInfo {
    pub fn get(&self, sector_type: SectorType) -> &ChestDropInfo {
        let idx = sector_type as usize;
        &self.infos[idx]
    }
}

fn prepare_chest_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let material_green = materials.add(Color::srgb(0.2, 0.8, 0.2));
    let material_red = materials.add(Color::srgb(0.8, 0.2, 0.2));
    let material_orange = materials.add(Color::srgb(0.8, 0.4, 0.2));
    let mesh_default = meshes.add(Rectangle::new(20.0, 10.0));

    commands.insert_resource(ChestResources {
        material_default,
        material_green,
        material_red,
        material_orange,
        mesh_default,
    });

    let chests_drop_info = ChestsDropInfo {
        infos: [
            // Default
            ChestDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
            // Green
            ChestDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
            // Red
            ChestDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
            // Orange
            ChestDropInfo {
                items: vec![0, 1],
                spells: vec![0, 1],
            },
        ],
    };

    commands.insert_resource(chests_drop_info);
}

pub fn spawn_chest<'a>(
    commands: &'a mut Commands,
    chest_resources: &ChestResources,
    sector_type: SectorType,
    sector_id: u8,
    transform: Transform,
    render_layer: RenderLayers,
) -> EntityCommands<'a> {
    let material = match sector_type {
        SectorType::Default => chest_resources.material_default.clone(),
        SectorType::Green => chest_resources.material_green.clone(),
        SectorType::Red => chest_resources.material_red.clone(),
        SectorType::Orange => chest_resources.material_orange.clone(),
    };
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: chest_resources.mesh_default.clone().into(),
            material,
            transform,
            ..default()
        },
        Chest,
        SectorId(sector_id),
        sector_type,
        render_layer,
        StateScoped(GlobalState::InGame),
    ))
}
