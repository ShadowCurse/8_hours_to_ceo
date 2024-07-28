use std::ops::{Index, IndexMut};

use bevy::{audio::PlaybackMode, ecs::system::EntityCommands, prelude::*};
use rand::Rng;

use crate::GlobalState;

use super::{
    animation::{AllAnimations, AnimationConfig, AnimationFinishedEvent},
    circle_sectors::{SectorIdx, SectorPosition, Sectors},
    inventory::{Inventory, InventoryUpdateEvent},
    items::{ItemIdx, Items},
    sound::SoundResources,
    spells::{SpellIdx, Spells},
    GameState,
};

pub struct ChestsPlugin;

impl Plugin for ChestsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChestOppenedEvent>()
            .add_systems(Startup, prepare_chest_resources)
            .add_systems(
                Update,
                (chest_open_check, on_chest_open_finish).run_if(in_state(GameState::Pickup)),
            );
    }
}

#[derive(Event, Debug, Clone, PartialEq)]
pub struct ChestOppenedEvent;

#[derive(Resource, Debug, Clone)]
pub struct ChestResources {
    pub texture: Handle<Image>,
    pub animation_config: AnimationConfig,
    pub texture_atlas: TextureAtlas,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChestIdx(pub usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chest;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractedChest;

#[derive(Debug, Clone, PartialEq)]
pub struct ChestInfo {
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
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let chest_texture = asset_server.load("chest/chest.png");
    let chest_animation_config =
        AnimationConfig::new(0, 1, 5, AllAnimations::ChestOpen, true, false);
    let texture_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 2, 1, None, None);
    let atlas_handle = texture_atlas_layouts.add(texture_layout);
    let texture_atlas = TextureAtlas {
        layout: atlas_handle,
        index: 0,
    };
    commands.insert_resource(ChestResources {
        texture: chest_texture,
        animation_config: chest_animation_config,
        texture_atlas,
    });

    let mut chests = Chests(vec![]);
    // 0 Default
    chests.0.push(ChestInfo {
        spawn_rate: 0.25,
        // Paperclip, Stickynotes
        items: vec![ItemIdx(1), ItemIdx(5)],
        // Stand up
        spells: vec![SpellIdx(5)],
        // Green, Yellow
        sectors: vec![SectorIdx(1), SectorIdx(2)],
    });
    // 1 Green
    chests.0.push(ChestInfo {
        spawn_rate: 0.25,
        // Paperclip, Plant, Scissors,
        items: vec![ItemIdx(1), ItemIdx(2), ItemIdx(3)],
        // Excel
        spells: vec![SpellIdx(4)],
        sectors: vec![SectorIdx(2)],
    });
    // 2 Yellow
    chests.0.push(ChestInfo {
        spawn_rate: 0.25,
        // Coffee, Stickynotes
        items: vec![ItemIdx(0), ItemIdx(5)],
        // Lunchbox
        spells: vec![SpellIdx(3)],
        sectors: vec![SectorIdx(3), SectorIdx(4)],
    });
    // 3 Grey
    chests.0.push(ChestInfo {
        spawn_rate: 0.2,
        // Coffee, Paperclip, Scissors
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(3)],
        // Marker, Keyboard, Powerpoint
        spells: vec![SpellIdx(0), SpellIdx(1), SpellIdx(6)],
        sectors: vec![SectorIdx(3)],
    });
    // 4 Brown
    chests.0.push(ChestInfo {
        spawn_rate: 0.2,
        // Coffee, Scissors, Stapler
        items: vec![ItemIdx(0), ItemIdx(3), ItemIdx(4)],
        spells: vec![SpellIdx(2), SpellIdx(3), SpellIdx(4)],
        sectors: vec![SectorIdx(0), SectorIdx(3), SectorIdx(4)],
    });
    commands.insert_resource(chests);
}

pub fn spawn_chest<'a>(
    commands: &'a mut Commands,
    chest_resources: &ChestResources,
    chest_idx: ChestIdx,
    sector_id: SectorPosition,
    transform: Transform,
) -> EntityCommands<'a> {
    commands.spawn((
        SpriteBundle {
            transform,
            texture: chest_resources.texture.clone(),
            ..Default::default()
        },
        chest_resources.texture_atlas.clone(),
        Chest,
        sector_id,
        chest_idx,
        StateScoped(GlobalState::InGame),
    ))
}

fn chest_open_check(
    chest_resources: Res<ChestResources>,
    chest: Query<Entity, (With<InteractedChest>, Without<AnimationConfig>)>,
    mut commands: Commands,
) {
    let Ok(chest_entity) = chest.get_single() else {
        return;
    };

    let Some(mut e) = commands.get_entity(chest_entity) else {
        return;
    };

    // This start the chest animation.
    e.insert(chest_resources.animation_config.clone());
}

fn on_chest_open_finish(
    items: Res<Items>,
    chests: Res<Chests>,
    spells: Res<Spells>,
    sectors: Res<Sectors>,
    sounds: Res<SoundResources>,
    chest: Query<(Entity, &ChestIdx), With<InteractedChest>>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut inventory_update_event: EventWriter<InventoryUpdateEvent>,
    mut chest_openned_event: EventWriter<ChestOppenedEvent>,
    mut event_reader: EventReader<AnimationFinishedEvent>,
) {
    for e in event_reader.read() {
        if e.0 == AllAnimations::ChestOpen {
            // Chest open sound
            commands.spawn(AudioBundle {
                source: sounds.chest_open.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: sounds.volume,
                    ..Default::default()
                },
            });

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
                let random_item_idx =
                    chest_info.items[thread_rng.gen_range(0..chest_info.items.len())];
                let item = &items[random_item_idx];
                if thread_rng.gen_bool(item.drop_rate as f64) {
                    inventory.backpack_items.push(random_item_idx);
                }
            }

            if !chest_info.spells.is_empty() {
                let random_spell_idx =
                    chest_info.spells[thread_rng.gen_range(0..chest_info.spells.len())];
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

            info!("chest open event");
            inventory_update_event.send(InventoryUpdateEvent);
            chest_openned_event.send(ChestOppenedEvent);
        }
    }
}
