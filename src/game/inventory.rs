use bevy::prelude::*;

pub struct InventoryPlugin;

const INVENTORY_ITEMS: usize = 4;
const INVENTORY_BACKPACK_ITEMS: usize = 8;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (prepare_inventory, prepare_items, prepare_spells));
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stack<T, const N: usize> {
    inner: [Option<T>; N],
}

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        let mut inner: [Option<T>; N] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        for i in inner.iter_mut() {
            *i = None;
        }
        Self { inner }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemIdx(pub usize);

#[derive(Debug)]
pub struct ItemInfo {
    pub name: &'static str,
    pub drop_rate: f32,
    pub item: Item,
}

#[derive(Resource, Debug)]
pub struct Items(pub Vec<ItemInfo>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpellIdx(pub usize);

#[derive(Debug)]
pub struct SpellInfo {
    pub name: &'static str,
    pub drop_rate: f32,
    pub spell: Spell,
}

#[derive(Resource, Debug)]
pub struct Spells(pub Vec<SpellInfo>);

fn prepare_inventory(mut commands: Commands) {
    commands.insert_resource(Inventory::new());
}

fn prepare_items(mut commands: Commands) {
    let mut items = Items(vec![]);

    items.0.push(ItemInfo {
        name: "Scissors",
        drop_rate: 0.9,
        item: Item::Scissors,
    });
    items.0.push(ItemInfo {
        name: "Bucket",
        drop_rate: 0.9,
        item: Item::Bucket,
    });

    commands.insert_resource(items);
}

fn prepare_spells(mut commands: Commands) {
    let mut spells = Spells(vec![]);

    spells.0.push(SpellInfo {
        name: "Lightning",
        drop_rate: 0.9,
        spell: Spell::Lightning,
    });
    spells.0.push(SpellInfo {
        name: "Heal",
        drop_rate: 0.9,
        spell: Spell::Heal,
    });

    commands.insert_resource(spells);
}
