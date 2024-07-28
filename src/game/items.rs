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
            Self::Scissiors => 10.0,
            Self::Stapler => 5.0,
            Self::Stickynotes => 0.0,
        }
    }

    pub fn add_defense(&self) -> f32 {
        match self {
            Self::Coffecup => 0.0,
            Self::Paperclip => 0.05,
            Self::Plant => 0.0,
            Self::Scissiors => 0.0,
            Self::Stapler => 0.0,
            Self::Stickynotes => 0.01,
        }
    }

    pub fn heal(&self) -> f32 {
        match self {
            Self::Coffecup => 5.0,
            Self::Paperclip => 0.0,
            Self::Plant => 1.0,
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

    items.0.push(ItemInfo {
        description: "Smoking hot coffe for burnout nerves. Heals 5 hp.",
        image: asset_server.load("items/item_coffecup.png"),
        drop_rate: 0.2,
        item: Item::Coffecup,
    });
    items.0.push(ItemInfo {
        description: "As paperclip holds papers, you hold your ground. Adds 5% defence.",
        image: asset_server.load("items/item_paperclip.png"),
        drop_rate: 0.9,
        item: Item::Paperclip,
    });
    items.0.push(ItemInfo {
        description: "Decorative plant. Eat a leaf after each battle to restore 1 hp.",
        image: asset_server.load("items/item_pot.png"),
        drop_rate: 0.9,
        item: Item::Plant,
    });
    items.0.push(ItemInfo {
        description: "Scissors to cut through your opponent's arguments. Add 10 damage.",
        image: asset_server.load("items/item_scissors.png"),
        drop_rate: 0.9,
        item: Item::Scissiors,
    });
    items.0.push(ItemInfo {
        description: "Stapler for closing your oppenent arguments. Adds 5 damage.",
        image: asset_server.load("items/item_stapler.png"),
        drop_rate: 0.9,
        item: Item::Stapler,
    });
    items.0.push(ItemInfo {
        description: "With stickynotes you never forget about deadlines. +2% defence.",
        image: asset_server.load("items/item_stickynotes.png"),
        drop_rate: 0.9,
        item: Item::Stickynotes,
    });

    commands.insert_resource(items);
}
