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
    player::DamagePlayer,
    spells::{SpellIdx, Spells},
    AttackSpeed, Damage, Defense, GameState, Health,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEnemy>()
            .add_event::<BattleEnemyDead>()
            .add_systems(Startup, prepare_enemy_resources)
            .add_systems(
                Update,
                (enemy_attack, enemy_take_damage, enemy_check_dead)
                    .run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Event, Debug, Clone, PartialEq)]
pub struct DamageEnemy(pub f32);

#[derive(Event, Debug, Clone, PartialEq)]
pub struct BattleEnemyDead;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EnemyResources {
    pub mesh_default: Handle<Mesh>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnemyIdx(pub usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleEnemy;

#[derive(Debug, Clone, PartialEq)]
pub struct EnemyInfo {
    pub material: Handle<ColorMaterial>,
    pub spawn_rate: f32,
    pub items: Vec<ItemIdx>,
    pub spells: Vec<SpellIdx>,
    pub sectors: Vec<SectorIdx>,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Enemies(Vec<EnemyInfo>);

impl Index<EnemyIdx> for Enemies {
    type Output = EnemyInfo;
    fn index(&self, index: EnemyIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<EnemyIdx> for Enemies {
    fn index_mut(&mut self, index: EnemyIdx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

fn prepare_enemy_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let material_green = materials.add(Color::srgb(0.2, 0.8, 0.2));
    let material_red = materials.add(Color::srgb(0.8, 0.2, 0.2));
    let material_orange = materials.add(Color::srgb(0.8, 0.4, 0.2));
    let mesh_default = meshes.add(Circle { radius: 10.0 });

    commands.insert_resource(EnemyResources { mesh_default });

    let mut enemies = Enemies(vec![]);

    // Default
    enemies.0.push(EnemyInfo {
        material: material_default,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(0)],
    });
    // Green
    enemies.0.push(EnemyInfo {
        material: material_green,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(1)],
    });
    // Red
    enemies.0.push(EnemyInfo {
        material: material_red,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(2)],
    });
    // Orange
    enemies.0.push(EnemyInfo {
        material: material_orange,
        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(3)],
    });

    commands.insert_resource(enemies);
}

pub fn spawn_enemy<'a>(
    commands: &'a mut Commands,
    enemies: &Enemies,
    enemy_resources: &EnemyResources,
    enemy_idx: EnemyIdx,
    sector_id: SectorPosition,
    transform: Transform,
    render_layer: RenderLayers,
) -> EntityCommands<'a> {
    let material = enemies.0[enemy_idx.0].material.clone();
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: enemy_resources.mesh_default.clone().into(),
            material,
            transform,
            ..default()
        },
        Enemy,
        Health(30.0),
        Damage(1.0),
        AttackSpeed::new(1.0),
        Defense(0.0),
        sector_id,
        enemy_idx,
        render_layer,
        StateScoped(GlobalState::InGame),
    ))
}

fn enemy_attack(
    time: Res<Time>,
    mut enemy: Query<(&Damage, &mut AttackSpeed), With<BattleEnemy>>,
    mut event_writer: EventWriter<DamagePlayer>,
) {
    let Ok((damage, mut attack_speed)) = enemy.get_single_mut() else {
        return;
    };

    attack_speed.0.tick(time.delta());

    if attack_speed.0.finished() {
        event_writer.send(DamagePlayer(damage.0));
    }
}

fn enemy_take_damage(
    mut enemy: Query<(&Defense, &mut Health), With<BattleEnemy>>,
    mut event_reader: EventReader<DamageEnemy>,
) {
    let Ok((enemy_defense, mut enemy_health)) = enemy.get_single_mut() else {
        return;
    };

    for e in event_reader.read() {
        let damage = e.0 * (1.0 - enemy_defense.0);
        println!("enemy takes: {damage} damage");
        enemy_health.0 -= damage;
    }
}

fn enemy_check_dead(
    items: Res<Items>,
    spells: Res<Spells>,
    enemies: Res<Enemies>,
    sectors: Res<Sectors>,
    enemy: Query<(Entity, &Health, &EnemyIdx), With<BattleEnemy>>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut inventory_update_event: EventWriter<InventoryUpdate>,
    mut battle_enemy_dead_event: EventWriter<BattleEnemyDead>,
) {
    let Ok((enemy_entity, enemy_health, enemy_idx)) = enemy.get_single() else {
        return;
    };

    if enemy_health.0 <= 0.0 {
        commands
            .get_entity(enemy_entity)
            .unwrap()
            .despawn_recursive();

        let enemy_info = &enemies[*enemy_idx];

        let mut thread_rng = rand::thread_rng();

        if !enemy_info.items.is_empty() {
            let random_item_idx = enemy_info.items[thread_rng.gen_range(0..enemy_info.items.len())];
            let item = &items[random_item_idx];
            if thread_rng.gen_bool(item.drop_rate as f64) {
                inventory.backpack_items.push(random_item_idx);
            }
        }

        if !enemy_info.spells.is_empty() {
            let random_spell_idx =
                enemy_info.spells[thread_rng.gen_range(0..enemy_info.spells.len())];
            let spell = &spells[random_spell_idx];
            if thread_rng.gen_bool(spell.drop_rate as f64) {
                inventory.backpack_spells.push(random_spell_idx);
            }
        }

        if !enemy_info.sectors.is_empty() {
            let random_sector_idx =
                enemy_info.sectors[thread_rng.gen_range(0..enemy_info.sectors.len())];
            let sector = &sectors[random_sector_idx];
            if thread_rng.gen_bool(sector.drop_rate as f64) {
                inventory.backpack_sectors.push(random_sector_idx);
            }
        }

        inventory_update_event.send(InventoryUpdate);
        battle_enemy_dead_event.send(BattleEnemyDead);
    }
}
