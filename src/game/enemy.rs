use std::ops::{Index, IndexMut};

use bevy::{audio::PlaybackMode, ecs::system::EntityCommands, prelude::*};
use rand::Rng;

use crate::{ui::UiStyle, GlobalState};

use super::{
    animation::{spawn_damage_text, AllAnimations, AnimationConfig, AnimationFinishedEvent},
    circle_sectors::{PlayerProgress, SectorIdx, SectorPosition, Sectors},
    hp_bar::{hp_bar_bundle, HpBarResources},
    inventory::{Inventory, InventoryUpdateEvent},
    items::{ItemIdx, Items},
    player::DamagePlayerEvent,
    sound::SoundResources,
    spells::{SpellIdx, Spells},
    AttackSpeed, Damage, Defense, GameState, Health,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEnemyEvent>()
            .add_event::<EnemyDeadEvent>()
            .add_systems(Startup, prepare_enemy_resources)
            .add_systems(
                Update,
                (
                    enemy_attack,
                    on_attack_finish,
                    enemy_take_damage,
                    on_dead_finish,
                    enemy_check_dead,
                )
                    .run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Event, Debug, Clone, PartialEq)]
pub struct DamageEnemyEvent {
    pub damage: f32,
    pub color: Color,
}

#[derive(Event, Debug, Clone, PartialEq)]
pub struct EnemyDeadEvent;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnemyIdx(pub usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Enemy {
    pub is_boss: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleEnemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleEnemyDead;

#[derive(Debug, Clone)]
pub struct EnemyInfo {
    pub idle_texture: Handle<Image>,
    pub idle_animation_config: AnimationConfig,

    pub attack_texture: Handle<Image>,
    pub attack_animation_config: AnimationConfig,

    pub dead_texture: Handle<Image>,
    pub dead_animation_config: AnimationConfig,

    pub texture_atlas: TextureAtlas,

    pub spawn_rate: f32,
    pub items: Vec<ItemIdx>,
    pub spells: Vec<SpellIdx>,
    pub sectors: Vec<SectorIdx>,

    pub hp: f32,
    pub damage: f32,
}

#[derive(Resource, Debug, Clone)]
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
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut enemies = Enemies(vec![]);

    // Big boss
    let idle_texture = asset_server.load("enemy/boss_idle_sheet.png");
    let idle_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossIdle, false, true);

    let attack_texture = asset_server.load("enemy/boss_attack_sheet.png");
    let attack_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossAttack, true, false);

    let dead_texture = asset_server.load("enemy/boss_dead_sheet.png");
    let dead_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossDead, true, false);

    let texture_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 1, None, None);
    let atlas_handle = texture_atlas_layouts.add(texture_layout);
    let texture_atlas = TextureAtlas {
        layout: atlas_handle,
        index: 1,
    };
    enemies.0.push(EnemyInfo {
        idle_texture,
        idle_animation_config,

        attack_texture,
        attack_animation_config,

        dead_texture,
        dead_animation_config,

        texture_atlas,

        spawn_rate: 0.3,
        items: vec![ItemIdx(0), ItemIdx(1), ItemIdx(2)],
        spells: vec![
            SpellIdx(0),
            SpellIdx(1),
            SpellIdx(2),
            SpellIdx(3),
            SpellIdx(4),
        ],
        sectors: vec![SectorIdx(0)],

        hp: 700.0,
        damage: 20.0,
    });

    // Green
    let idle_texture = asset_server.load("enemy/greenmob_idle_sheet.png");
    let idle_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossIdle, false, true);

    let attack_texture = asset_server.load("enemy/greenmob_attack_sheet.png");
    let attack_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossAttack, true, false);

    let dead_texture = asset_server.load("enemy/greenmob_dead_sheet.png");
    let dead_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossDead, true, false);

    let texture_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 1, None, None);
    let atlas_handle = texture_atlas_layouts.add(texture_layout);
    let texture_atlas = TextureAtlas {
        layout: atlas_handle,
        index: 1,
    };
    // 1
    enemies.0.push(EnemyInfo {
        idle_texture: idle_texture.clone(),
        idle_animation_config: idle_animation_config.clone(),

        attack_texture: attack_texture.clone(),
        attack_animation_config: attack_animation_config.clone(),

        dead_texture: dead_texture.clone(),
        dead_animation_config: dead_animation_config.clone(),

        texture_atlas: texture_atlas.clone(),

        spawn_rate: 0.3,

        // Plant, Stickynotes
        items: vec![ItemIdx(2), ItemIdx(5)],
        // Marker, Lunchbox
        spells: vec![SpellIdx(0), SpellIdx(3)],
        sectors: vec![SectorIdx(1), SectorIdx(2)],

        hp: 50.0,
        damage: 3.0,
    });

    // 2
    enemies.0.push(EnemyInfo {
        idle_texture,
        idle_animation_config,

        attack_texture,
        attack_animation_config,

        dead_texture,
        dead_animation_config,

        texture_atlas,

        spawn_rate: 0.3,

        // Coffee, Plant, Stickynotes
        items: vec![ItemIdx(0), ItemIdx(2), ItemIdx(5)],
        // Marker, Keyboard
        spells: vec![SpellIdx(0), SpellIdx(1)],
        sectors: vec![SectorIdx(3), SectorIdx(4)],

        hp: 75.0,
        damage: 5.0,
    });

    // Orange
    let idle_texture = asset_server.load("enemy/orangemob_idle_sheet.png");
    let idle_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossIdle, false, true);

    let attack_texture = asset_server.load("enemy/orangemob_attack_sheet.png");
    let attack_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossAttack, true, false);

    let dead_texture = asset_server.load("enemy/orangemob_dead_sheet.png");
    let dead_animation_config =
        AnimationConfig::new(0, 5, 10, AllAnimations::BossDead, true, false);

    let texture_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 1, None, None);
    let atlas_handle = texture_atlas_layouts.add(texture_layout);
    let texture_atlas = TextureAtlas {
        layout: atlas_handle,
        index: 1,
    };
    // 3
    enemies.0.push(EnemyInfo {
        idle_texture: idle_texture.clone(),
        idle_animation_config: idle_animation_config.clone(),

        attack_texture: attack_texture.clone(),
        attack_animation_config: attack_animation_config.clone(),

        dead_texture: dead_texture.clone(),
        dead_animation_config: dead_animation_config.clone(),

        texture_atlas: texture_atlas.clone(),

        spawn_rate: 0.3,
        // Paperclip, Scissors, Stickynotes
        items: vec![ItemIdx(1), ItemIdx(3), ItemIdx(5)],
        spells: vec![],
        sectors: vec![SectorIdx(2), SectorIdx(3)],

        hp: 120.0,
        damage: 8.0,
    });

    // 4
    enemies.0.push(EnemyInfo {
        idle_texture,
        idle_animation_config,

        attack_texture,
        attack_animation_config,

        dead_texture,
        dead_animation_config,

        texture_atlas,

        spawn_rate: 0.3,
        // Plant, Stapler
        items: vec![ItemIdx(2), ItemIdx(4)],
        spells: vec![SpellIdx(2)],
        sectors: vec![SectorIdx(4)],

        hp: 140.0,
        damage: 10.0,
    });

    commands.insert_resource(enemies);
}

pub fn spawn_enemy<'a>(
    commands: &'a mut Commands,
    hardness: f32,
    enemies: &Enemies,
    enemy_idx: EnemyIdx,
    sector_id: SectorPosition,
    hp_bar_resources: &HpBarResources,
    transform: Transform,
    is_boss: bool,
) -> EntityCommands<'a> {
    let enemy_info = &enemies[enemy_idx];
    let mut c = commands.spawn((
        SpriteBundle {
            transform,
            texture: enemy_info.idle_texture.clone(),
            ..Default::default()
        },
        enemy_info.texture_atlas.clone(),
        enemy_info.idle_animation_config.clone(),
        Enemy { is_boss },
        Health::new(enemy_info.hp * hardness),
        Damage(enemy_info.damage * hardness),
        AttackSpeed::new(1.0),
        Defense(0.0),
        sector_id,
        enemy_idx,
        StateScoped(GlobalState::InGame),
    ));
    let parent_entity = c.id();
    c.with_children(|builder| {
        builder.spawn(hp_bar_bundle(hp_bar_resources, parent_entity));
    });
    c
}

fn enemy_attack(
    time: Res<Time>,
    enemies: Res<Enemies>,
    mut enemy: Query<
        (
            &EnemyIdx,
            &mut AttackSpeed,
            &mut AnimationConfig,
            &mut Handle<Image>,
            &mut TextureAtlas,
        ),
        With<BattleEnemy>,
    >,
) {
    let Ok((enemy_idx, mut attack_speed, mut config, mut texture, mut atlas)) =
        enemy.get_single_mut()
    else {
        return;
    };

    attack_speed.0.tick(time.delta());
    if attack_speed.0.finished() {
        let enemy_info = &enemies[*enemy_idx];

        *texture = enemy_info.attack_texture.clone();
        atlas.index = enemy_info.attack_animation_config.first_sprite_index;
        *config = enemy_info.attack_animation_config.clone();
    }
}

fn on_attack_finish(
    enemy: Query<(&Enemy, &Damage), With<BattleEnemy>>,
    sounds: Res<SoundResources>,
    mut commands: Commands,
    mut event_reader: EventReader<AnimationFinishedEvent>,
    mut event_writer: EventWriter<DamagePlayerEvent>,
) {
    let Ok((enemy, damage)) = enemy.get_single() else {
        return;
    };

    for e in event_reader.read() {
        if e.0 == AllAnimations::BossAttack {
            // Attack sound
            commands.spawn(AudioBundle {
                source: if enemy.is_boss {
                    sounds.boss_attack.clone()
                } else {
                    sounds.enemy_attack.clone()
                },
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: sounds.volume,
                    ..Default::default()
                },
            });

            event_writer.send(DamagePlayerEvent(damage.0));
        }
    }
}

fn enemy_take_damage(
    ui_style: Res<UiStyle>,
    mut commands: Commands,
    mut enemy: Query<(&Transform, &Defense, &mut Health), With<BattleEnemy>>,
    mut event_reader: EventReader<DamageEnemyEvent>,
) {
    let Ok((enemy_transform, enemy_defense, mut enemy_health)) = enemy.get_single_mut() else {
        return;
    };

    for e in event_reader.read() {
        let damage = e.damage * (1.0 - enemy_defense.0);
        enemy_health.take_damage(damage);

        spawn_damage_text(
            &mut commands,
            ui_style.as_ref(),
            damage,
            *enemy_transform,
            enemy_transform.translation.normalize(),
            e.color,
        );
    }
}

fn enemy_check_dead(
    enemies: Res<Enemies>,
    mut commands: Commands,
    mut enemy: Query<
        (
            Entity,
            &Health,
            &EnemyIdx,
            &mut AnimationConfig,
            &mut Handle<Image>,
            &mut TextureAtlas,
        ),
        With<BattleEnemy>,
    >,
) {
    let Ok((enemy_entity, enemy_health, enemy_idx, mut config, mut texture, mut atlas)) =
        enemy.get_single_mut()
    else {
        return;
    };

    if enemy_health.current() == 0.0 {
        commands
            .get_entity(enemy_entity)
            .unwrap()
            .remove::<BattleEnemy>()
            .insert(BattleEnemyDead);

        let enemy_info = &enemies[*enemy_idx];

        // Start dead animation
        *texture = enemy_info.dead_texture.clone();
        atlas.index = enemy_info.dead_animation_config.first_sprite_index;
        *config = enemy_info.dead_animation_config.clone();
    }
}

fn on_dead_finish(
    items: Res<Items>,
    spells: Res<Spells>,
    enemies: Res<Enemies>,
    sectors: Res<Sectors>,
    enemy: Query<(Entity, &EnemyIdx), With<BattleEnemyDead>>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut event_reader: EventReader<AnimationFinishedEvent>,
    mut inventory_update_event: EventWriter<InventoryUpdateEvent>,
    mut enemy_dead_event: EventWriter<EnemyDeadEvent>,
) {
    let Ok((entity, enemy_idx)) = enemy.get_single() else {
        return;
    };

    for e in event_reader.read() {
        if e.0 == AllAnimations::BossDead {
            commands.get_entity(entity).unwrap().despawn_recursive();

            let enemy_info = &enemies[*enemy_idx];

            let mut thread_rng = rand::thread_rng();

            if !enemy_info.items.is_empty() {
                let random_item_idx =
                    enemy_info.items[thread_rng.gen_range(0..enemy_info.items.len())];
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

            info!("enemy dead event");
            inventory_update_event.send(InventoryUpdateEvent);
            enemy_dead_event.send(EnemyDeadEvent);
        }
    }
}
