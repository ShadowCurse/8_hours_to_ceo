use bevy::prelude::*;

use super::{items::ItemIdx, spells::SpellIdx};

pub struct InventoryPlugin;

const INVENTORY_ITEMS: usize = 4;
const INVENTORY_BACKPACK_ITEMS: usize = 8;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_inventory);
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Inventory {
    pub active_items: Stack<ItemIdx, INVENTORY_ITEMS>,
    pub backpack_items: Stack<ItemIdx, INVENTORY_BACKPACK_ITEMS>,
    pub active_spells: Stack<SpellIdx, INVENTORY_ITEMS>,
    pub backpack_spells: Stack<SpellIdx, INVENTORY_BACKPACK_ITEMS>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            active_items: Stack::new(),
            backpack_items: Stack::new(),
            active_spells: Stack::new(),
            backpack_spells: Stack::new(),
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

fn prepare_inventory(mut commands: Commands) {
    commands.insert_resource(Inventory::new());
}
