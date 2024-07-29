use std::ops::{Index, IndexMut};

use bevy::prelude::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_items);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemIdx(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Coffecup,
    Paperclip,
    Plant,
    Scissiors,
    Stapler,
    Stickynotes,
}

impl Item {
    pub fn add_damage(&self) -> f32 {
        match self {
            Self::Coffecup => 0.0,
            Self::Paperclip => 0.0,
            Self::Plant => 0.0,
            Self::Scissiors => 11.0,
            Self::Stapler => 22.0,
            Self::Stickynotes => 0.0,
        }
    }

    pub fn add_defense(&self) -> f32 {
        match self {
            Self::Coffecup => 0.0,
            Self::Paperclip => 0.2,
            Self::Plant => 0.0,
            Self::Scissiors => 0.0,
            Self::Stapler => 0.0,
            Self::Stickynotes => 0.1,
        }
    }

    pub fn heal(&self) -> f32 {
        match self {
            Self::Coffecup => 20.0,
            Self::Paperclip => 0.0,
            Self::Plant => 10.0,
            Self::Scissiors => 0.0,
            Self::Stapler => 0.0,
            Self::Stickynotes => 0.0,
        }
    }
}

#[derive(Debug)]
pub struct ItemInfo {
    pub description: &'static str,
    pub image: Handle<Image>,
    pub drop_rate: f32,
    pub item: Item,
}

#[derive(Resource, Debug)]
pub struct Items(Vec<ItemInfo>);

impl Index<ItemIdx> for Items {
    type Output = ItemInfo;
    fn index(&self, index: ItemIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<ItemIdx> for Items {
    fn index_mut(&mut self, index: ItemIdx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

fn prepare_items(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut items = Items(vec![]);

    // 0 coffee
    items.0.push(ItemInfo {
        description: "Smoking hot coffe for burnout nerves. Heals 20 hp after each battle.",
        image: asset_server.load("items/item_coffecup.png"),
        drop_rate: 0.2,
        item: Item::Coffecup,
    });
    // 1 paperclip
    items.0.push(ItemInfo {
        description: "As paperclip holds papers, you hold your ground. Adds 20% defence.",
        image: asset_server.load("items/item_paperclip.png"),
        drop_rate: 0.9,
        item: Item::Paperclip,
    });
    // 2 plant
    items.0.push(ItemInfo {
        description: "Decorative plant. Eat a leaf after each battle to restore 10 hp.",
        image: asset_server.load("items/item_pot.png"),
        drop_rate: 0.9,
        item: Item::Plant,
    });
    // 3 scissors
    items.0.push(ItemInfo {
        description: "Scissors for cutting pay. Adds 11 damage.",
        image: asset_server.load("items/item_scissors.png"),
        drop_rate: 0.9,
        item: Item::Scissiors,
    });
    // 4 stapler
    items.0.push(ItemInfo {
        description: "Stapler for closing your oppenent's arguments. Adds 22 damage.",
        image: asset_server.load("items/item_stapler.png"),
        drop_rate: 0.9,
        item: Item::Stapler,
    });
    // 5 stickynotes
    items.0.push(ItemInfo {
        description: "With stickynotes you never forget about deadlines. Adds +10% defence.",
        image: asset_server.load("items/item_stickynotes.png"),
        drop_rate: 0.9,
        item: Item::Stickynotes,
    });

    commands.insert_resource(items);
}
