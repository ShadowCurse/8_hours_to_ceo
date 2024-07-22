use bevy::prelude::*;

pub struct InventoryPlugin;

const INVENTORY_ITEMS: usize = 4;
const INVENTORY_BACKPACK_ITEMS: usize = 4;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_inventory);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inventory {
    pub active_items: Stack<Item, INVENTORY_ITEMS>,
    pub backpack_item: Stack<Item, INVENTORY_BACKPACK_ITEMS>,
    pub active_spells: Stack<Spell, INVENTORY_ITEMS>,
    pub backpack_spells: Stack<Spell, INVENTORY_BACKPACK_ITEMS>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            active_items: Stack::new(),
            backpack_item: Stack::new(),
            active_spells: Stack::new(),
            backpack_spells: Stack::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stack<T, const N: usize> {
    inner: [Option<T>; N],
}

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        Self {
            inner: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
        }
    }

    pub fn push(&mut self, item: T) {
        for i in 1..self.inner.len() {
            self.inner.swap(i, i - 1);
        }

        self.inner[self.inner.len() - 1] = Some(item);
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> {
        self.inner.iter().map(|a| a.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Scissors,
    Bucket,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Spell {
    Lightning,
    Heal,
}

fn prepare_inventory(mut commands: Commands) {
    commands.insert_resource(Inventory::new());
}
