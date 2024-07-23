use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    sprite::MaterialMesh2dBundle,
};

use crate::GlobalState;

use super::circle_sectors::SectorId;

pub struct ChestsPlugin;

impl Plugin for ChestsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_chest_resources);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ChestResources {
    pub mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChestIdx(pub usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chest;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractedChest;

#[derive(Debug, Clone, PartialEq)]
pub struct ChestInfo {
    pub material: Handle<ColorMaterial>,
    pub spawn_rate: f32,
    pub items: Vec<usize>,
    pub spells: Vec<usize>,
    pub sectors: Vec<usize>,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Chests(pub Vec<ChestInfo>);

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

    commands.insert_resource(ChestResources { mesh_default });

    let mut chests = Chests(vec![]);
    // Default
    chests.0.push(ChestInfo {
        material: material_default,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
        sectors: vec![],
    });
    // Green
    chests.0.push(ChestInfo {
        material: material_green,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
        sectors: vec![],
    });
    // Red
    chests.0.push(ChestInfo {
        material: material_red,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
        sectors: vec![],
    });
    // Orange
    chests.0.push(ChestInfo {
        material: material_orange,
        spawn_rate: 0.3,
        items: vec![0, 1, 2],
        spells: vec![0, 1],
        sectors: vec![],
    });
    commands.insert_resource(chests);
}

pub fn spawn_chest<'a>(
    commands: &'a mut Commands,
    chests: &Chests,
    chest_resources: &ChestResources,
    chest_idx: ChestIdx,
    sector_id: SectorId,
    transform: Transform,
    render_layer: RenderLayers,
) -> EntityCommands<'a> {
    let material = chests.0[chest_idx.0].material.clone();
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: chest_resources.mesh_default.clone().into(),
            material,
            transform,
            ..default()
        },
        Chest,
        sector_id,
        chest_idx,
        render_layer,
        StateScoped(GlobalState::InGame),
    ))
}
