use std::ops::Index;

use bevy::prelude::*;

use crate::ui::in_game::{BackpackSectorId, SelectedSectionButton};

use super::{
    circle_sectors::{SectorIdx, SectorPlacedEvent},
    items::ItemIdx,
    spells::SpellIdx,
    GameState,
};

pub struct InventoryPlugin;

const INVENTORY_ITEMS: usize = 4;
const INVENTORY_BACKPACK_ITEMS: usize = 4;
const INVENTORY_BACKPACK_SECTORS: usize = 4;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InventoryUpdateEvent>()
            .add_systems(Startup, prepare_inventory)
            .add_systems(Update, on_sector_placed.run_if(state_exists::<GameState>));
    }
}

#[derive(Event, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InventoryUpdateEvent;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Inventory {
    pub active_items: Stack<ItemIdx, INVENTORY_ITEMS>,
    pub backpack_items: Stack<ItemIdx, INVENTORY_BACKPACK_ITEMS>,
    pub active_spells: Stack<SpellIdx, INVENTORY_ITEMS>,
    pub backpack_spells: Stack<SpellIdx, INVENTORY_BACKPACK_ITEMS>,
    pub backpack_sectors: Stack<SectorIdx, INVENTORY_BACKPACK_SECTORS>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            active_items: Stack::new(),
            backpack_items: Stack::new(),
            active_spells: Stack::new(),
            backpack_spells: Stack::new(),
            backpack_sectors: Stack::new(),
        }
    }

    pub fn equip_item(&mut self, id: usize) {
        if let Some(item_idx) = self.backpack_items.inner[id] {
            self.backpack_items.remove(id);
            self.active_items.push(item_idx);
        }
    }

    pub fn equip_spell(&mut self, id: usize) {
        if let Some(spell_idx) = self.backpack_spells.inner[id] {
            self.backpack_spells.remove(id);
            self.active_spells.push(spell_idx);
        }
    }

    pub fn get_spell_idx(&self, id: usize) -> Option<SpellIdx> {
        self.active_spells.inner[id]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stack<T: Copy, const N: usize> {
    inner: [Option<T>; N],
}

impl<T: Copy, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        let mut inner: [Option<T>; N] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        for i in inner.iter_mut() {
            *i = None;
        }
        Self { inner }
    }

    pub fn push(&mut self, item: T) {
        self.inner.copy_within(0..N - 1, 1);
        self.inner[0] = Some(item);
    }

    pub fn remove(&mut self, position: usize) {
        self.inner.copy_within(position + 1..N, position);
        self.inner[self.inner.len() - 1] = None;
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> {
        self.inner.iter().map(|a| a.as_ref())
    }
}

impl<T: Copy, const N: usize> Index<usize> for Stack<T, N> {
    type Output = Option<T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

fn prepare_inventory(mut commands: Commands) {
    commands.insert_resource(Inventory::new());
}

fn on_sector_placed(
    selected_section_button: Res<SelectedSectionButton>,
    section_buttons: Query<&BackpackSectorId, With<UiImage>>,
    mut inventory: ResMut<Inventory>,
    mut event_reader: EventReader<SectorPlacedEvent>,
    mut event_writer: EventWriter<InventoryUpdateEvent>,
) {
    for _ in event_reader.read() {
        let Some(button_entity) = selected_section_button.0 else {
            return;
        };
        let Ok(sector_id) = section_buttons.get(button_entity) else {
            return;
        };
        inventory.backpack_sectors.remove(sector_id.0 as usize);
        event_writer.send(InventoryUpdateEvent);
    }
}
