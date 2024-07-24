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
    Scissors,
    Bucket,
    Plant,
}

impl Item {
    pub fn add_damage(&self) -> f32 {
        match self {
            Self::Scissors => 10.0,
            Self::Bucket => 0.0,
            Self::Plant => 0.0,
        }
    }

    pub fn add_defense(&self) -> f32 {
        match self {
            Self::Scissors => 0.0,
            Self::Bucket => 0.1,
            Self::Plant => 0.0,
        }
    }

    pub fn heal(&self) -> f32 {
        match self {
            Self::Scissors => 0.0,
            Self::Bucket => 0.0,
            Self::Plant => 5.0,
        }
    }
}

#[derive(Debug)]
pub struct ItemInfo {
    pub name: &'static str,
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
    items.0.push(ItemInfo {
        name: "Plant",
        drop_rate: 0.9,
        item: Item::Plant,
    });

    commands.insert_resource(items);
}
