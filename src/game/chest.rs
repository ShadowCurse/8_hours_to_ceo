use std::ops::{Index, IndexMut};

use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    sprite::MaterialMesh2dBundle,
};
use rand::Rng;

use crate::GlobalState;

use super::{
    circle_sectors::{SectorIdx, SectorPosition, Sectors},
    inventory::{Inventory, InventoryUpdate},
    items::{ItemIdx, Items},
    spells::{SpellIdx, Spells},
    GameState,
};

pub struct ChestsPlugin;

impl Plugin for ChestsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChestOppened>()
            .add_systems(Startup, prepare_chest_resources)
            .add_systems(Update, chest_open_check.run_if(in_state(GameState::Pickup)));
    }
}

#[derive(Event, Debug, Clone, PartialEq)]
pub struct ChestOppened;

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
    pub items: Vec<ItemIdx>,
    pub spells: Vec<SpellIdx>,
    pub sectors: Vec<SectorIdx>,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Chests(Vec<ChestInfo>);

impl Index<ChestIdx> for Chests {
    type Output = ChestInfo;
    fn index(&self, index: ChestIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<ChestIdx> for Chests {
    fn index_mut(&mut self, index: ChestIdx) -> &mut Self::Output {
        &mut self.0[index.0]
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

    commands.insert_resource(ChestResources { mesh_default });

    let mut chests = Chests(vec![]);
    // Default
    chests.0.push(ChestInfo {
        material: material_default,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(0)],
    });
    // Green
    chests.0.push(ChestInfo {
        material: material_green,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(1)],
    });
    // Red
    chests.0.push(ChestInfo {
        material: material_red,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(2)],
    });
    // Orange
    chests.0.push(ChestInfo {
        material: material_orange,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(3)],
    });
    commands.insert_resource(chests);
}

pub fn spawn_chest<'a>(
    commands: &'a mut Commands,
    chests: &Chests,
    chest_resources: &ChestResources,
    chest_idx: ChestIdx,
    sector_id: SectorPosition,
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

fn chest_open_check(
    items: Res<Items>,
    chests: Res<Chests>,
    spells: Res<Spells>,
    sectors: Res<Sectors>,
    chest: Query<(Entity, &ChestIdx), With<InteractedChest>>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut inventory_update_event: EventWriter<InventoryUpdate>,
    mut chest_openned_event: EventWriter<ChestOppened>,
) {
    let Ok((chest_entity, chest_idx)) = chest.get_single() else {
        return;
    };

    commands
        .get_entity(chest_entity)
        .unwrap()
        .despawn_recursive();

    let chest_info = &chests[*chest_idx];

    let mut thread_rng = rand::thread_rng();

    if !chest_info.items.is_empty() {
        let random_item_idx = chest_info.items[thread_rng.gen_range(0..chest_info.items.len())];
        let item = &items[random_item_idx];
        if thread_rng.gen_bool(item.drop_rate as f64) {
            inventory.backpack_items.push(random_item_idx);
        }
    }

    if !chest_info.spells.is_empty() {
        let random_spell_idx = chest_info.spells[thread_rng.gen_range(0..chest_info.spells.len())];
        let spell = &spells[random_spell_idx];
        if thread_rng.gen_bool(spell.drop_rate as f64) {
            inventory.backpack_spells.push(random_spell_idx);
        }
    }

    if !chest_info.sectors.is_empty() {
        let random_sector_idx =
            chest_info.sectors[thread_rng.gen_range(0..chest_info.sectors.len())];
        let sector = &sectors[random_sector_idx];
        if thread_rng.gen_bool(sector.drop_rate as f64) {
            inventory.backpack_sectors.push(random_sector_idx);
        }
    }

    inventory_update_event.send(InventoryUpdate);
    chest_openned_event.send(ChestOppened);
}
