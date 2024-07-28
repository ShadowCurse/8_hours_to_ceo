use std::ops::{Index, IndexMut};

use bevy::prelude::*;
use rand::Rng;

use super::{
    animation::{DAMAGE_COLOR_FIRE_PUNCH, DAMAGE_COLOR_KEYBOARD, DAMAGE_COLOR_MARKER},
    enemy::{BattleEnemy, DamageEnemyEvent},
    Damage, Defense, GameState, Health, Player,
};

pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpellEvent>()
            .add_systems(Startup, prepare_spells)
            .add_systems(Update, cooldown_spells.run_if(state_exists::<GameState>))
            .add_systems(
                Update,
                (
                    cast_spell,
                    process_damage_spell,
                    process_heal_spell,
                    process_player_attack_up_spell,
                    process_player_defence_up_spell,
                    process_enemy_denfense_down_spell,
                )
                    .run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpellIdx(pub usize);

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastSpellEvent(pub SpellIdx);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageSpellInfo {
    strikes: u32,
    delta_time: f32,
    damage: f32,
    color: Color,
    chance: f32,
}

#[derive(Component, Debug, Clone)]
pub struct DamageSpell {
    timer: Timer,
    remaining_strikes: u32,
    damage: f32,
    color: Color,
    chance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealSpellInfo {
    heal: f32,
}

#[derive(Component, Debug, Clone)]
pub struct HealSpell {
    heal: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAttackUpSpellInfo {
    duration: f32,
    attack: f32,
}

#[derive(Component, Debug, Clone)]
pub struct PlayerAttackUpSpell {
    timer: Timer,
    active: bool,
    attack: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerDefenseUpSpellInfo {
    duration: f32,
    defense: f32,
}

#[derive(Component, Debug, Clone)]
pub struct PlayerDefenseUpSpell {
    timer: Timer,
    active: bool,
    defense: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnemyDefenseDownSpellInfo {
    duration: f32,
    defense: f32,
}

#[derive(Component, Debug, Clone)]
pub struct EnemyDefenseDownSpell {
    timer: Timer,
    active: bool,
    defense: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spell {
    Damage(DamageSpellInfo),
    Heal(HealSpellInfo),
    PlayerAttackUp(PlayerAttackUpSpellInfo),
    PlayerDefenseUp(PlayerDefenseUpSpellInfo),
    EnemyDefenseDown(EnemyDefenseDownSpellInfo),
}

#[derive(Debug)]
pub struct SpellInfo {
    pub description: &'static str,
    pub image: Handle<Image>,
    pub drop_rate: f32,
    pub cooldown: Timer,
    pub spell: Spell,
}

#[derive(Resource, Debug)]
pub struct Spells(Vec<SpellInfo>);

impl Index<SpellIdx> for Spells {
    type Output = SpellInfo;
    fn index(&self, index: SpellIdx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<SpellIdx> for Spells {
    fn index_mut(&mut self, index: SpellIdx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

fn prepare_spells(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut spells = Spells(vec![]);

    spells.0.push(SpellInfo {
        description: "Throw 2 markers at the coworker. Each deals 5 damage.",
        image: asset_server.load("spells/spell_marker.png"),
        drop_rate: 0.8,
        cooldown: Timer::from_seconds(2.0, TimerMode::Once),
        spell: Spell::Damage(DamageSpellInfo {
            strikes: 2,
            delta_time: 0.1,
            damage: 5.0,
            color: DAMAGE_COLOR_MARKER,
            chance: 1.0,
        }),
    });
    spells.0.push(SpellInfo {
        description: "Slams keyboard into coworker face. Deals 10 damage.",
        image: asset_server.load("spells/spell_keyboard.png"),
        drop_rate: 0.7,
        cooldown: Timer::from_seconds(5.0, TimerMode::Once),
        spell: Spell::Damage(DamageSpellInfo {
            strikes: 1,
            delta_time: 0.0,
            damage: 10.0,
            color: DAMAGE_COLOR_KEYBOARD,
            chance: 1.0,
        }),
    });
    spells.0.push(SpellInfo {
        description: "50% chance to layoff coworker and deal 100 damage",
        image: asset_server.load("spells/spell_punch.png"),
        drop_rate: 0.1,
        cooldown: Timer::from_seconds(15.0, TimerMode::Once),
        spell: Spell::Damage(DamageSpellInfo {
            strikes: 1,
            delta_time: 0.0,
            damage: 100.0,
            color: DAMAGE_COLOR_FIRE_PUNCH,
            chance: 0.5,
        }),
    });
    spells.0.push(SpellInfo {
        description: "Delicious lunch. Restores 10 hp.",
        image: asset_server.load("spells/spell_lunchbox.png"),
        drop_rate: 0.2,
        cooldown: Timer::from_seconds(10.0, TimerMode::Once),
        spell: Spell::Heal(HealSpellInfo { heal: 10.0 }),
    });
    spells.0.push(SpellInfo {
        description: "Excels player damage by 10 for 2 seconds.",
        image: asset_server.load("spells/spell_excel.png"),
        drop_rate: 0.3,
        cooldown: Timer::from_seconds(8.0, TimerMode::Once),
        spell: Spell::PlayerAttackUp(PlayerAttackUpSpellInfo {
            duration: 2.0,
            attack: 10.0,
        }),
    });
    spells.0.push(SpellInfo {
        description: "Attending standup raises defence by 10% for 2 seconds.",
        image: asset_server.load("spells/spell_standup.png"),
        drop_rate: 0.3,
        cooldown: Timer::from_seconds(8.0, TimerMode::Once),
        spell: Spell::PlayerDefenseUp(PlayerDefenseUpSpellInfo {
            duration: 2.0,
            defense: 0.1,
        }),
    });
    spells.0.push(SpellInfo {
        description:
            "Present future plans to coworker. Lowers coworker defence by 10% for 2 seconds.",
        image: asset_server.load("spells/spell_powerpoint.png"),
        drop_rate: 0.3,
        cooldown: Timer::from_seconds(5.0, TimerMode::Once),
        spell: Spell::EnemyDefenseDown(EnemyDefenseDownSpellInfo {
            duration: 2.0,
            defense: 0.1,
        }),
    });

    commands.insert_resource(spells);
}

fn cooldown_spells(time: Res<Time>, mut spells: ResMut<Spells>) {
    for spell_info in spells.0.iter_mut() {
        spell_info.cooldown.tick(time.delta());
    }
}

fn cast_spell(
    mut commands: Commands,
    mut spells: ResMut<Spells>,
    mut event_reader: EventReader<CastSpellEvent>,
) {
    for e in event_reader.read() {
        let spell_info = &mut spells.0[e.0 .0];
        if !spell_info.cooldown.finished() {
            continue;
        } else {
            spell_info.cooldown.reset();
        }
        match spell_info.spell {
            Spell::Damage(damage_spell_info) => {
                commands.spawn(DamageSpell {
                    timer: Timer::from_seconds(damage_spell_info.delta_time, TimerMode::Repeating),
                    remaining_strikes: damage_spell_info.strikes,
                    damage: damage_spell_info.damage,
                    color: damage_spell_info.color,
                    chance: damage_spell_info.chance,
                });
            }
            Spell::Heal(heal_spell_info) => {
                commands.spawn(HealSpell {
                    heal: heal_spell_info.heal,
                });
            }
            Spell::PlayerAttackUp(attack_up_spell_info) => {
                commands.spawn(PlayerAttackUpSpell {
                    timer: Timer::from_seconds(attack_up_spell_info.duration, TimerMode::Once),
                    active: false,
                    attack: attack_up_spell_info.attack,
                });
            }
            Spell::PlayerDefenseUp(defense_up_spell_info) => {
                commands.spawn(PlayerDefenseUpSpell {
                    timer: Timer::from_seconds(defense_up_spell_info.duration, TimerMode::Once),
                    active: false,
                    defense: defense_up_spell_info.defense,
                });
            }
            Spell::EnemyDefenseDown(defense_down_spell_info) => {
                commands.spawn(EnemyDefenseDownSpell {
                    timer: Timer::from_seconds(defense_down_spell_info.duration, TimerMode::Once),
                    active: false,
                    defense: defense_down_spell_info.defense,
                });
            }
        }
    }
}

fn process_damage_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut damage_spelll: Query<(Entity, &mut DamageSpell)>,
    mut event_writer: EventWriter<DamageEnemyEvent>,
) {
    for (entity, mut damage_spell) in damage_spelll.iter_mut() {
        damage_spell.timer.tick(time.delta());
        if damage_spell.timer.finished() {
            if rand::thread_rng().gen_bool(damage_spell.chance as f64) {
                event_writer.send(DamageEnemyEvent {
                    damage: damage_spell.damage,
                    color: damage_spell.color,
                });
            }
            damage_spell.remaining_strikes -= 1;

            if damage_spell.remaining_strikes == 0 {
                commands.get_entity(entity).unwrap().despawn_recursive()
            }
        }
    }
}

fn process_heal_spell(
    heals: Query<(Entity, &HealSpell)>,
    mut commands: Commands,
    mut player: Query<&mut Health, With<Player>>,
) {
    let Ok(mut player_health) = player.get_single_mut() else {
        return;
    };

    for (heal_entity, heal) in heals.iter() {
        player_health.heal(heal.heal);
        commands
            .get_entity(heal_entity)
            .unwrap()
            .despawn_recursive()
    }
}

fn process_player_attack_up_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<&mut Damage, With<Player>>,
    mut player_attack_up: Query<(Entity, &mut PlayerAttackUpSpell)>,
) {
    let Ok(mut attack) = player.get_single_mut() else {
        return;
    };

    for (entity, mut attack_up) in player_attack_up.iter_mut() {
        if attack_up.active {
            attack_up.timer.tick(time.delta());
            if attack_up.timer.finished() {
                attack.0 -= attack_up.attack;
                commands.get_entity(entity).unwrap().despawn_recursive()
            }
        } else {
            attack_up.active = true;
            attack.0 += attack_up.attack;
        }
    }
}

fn process_player_defence_up_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<&mut Defense, With<Player>>,
    mut player_defense_up: Query<(Entity, &mut PlayerDefenseUpSpell)>,
) {
    let Ok(mut defense) = player.get_single_mut() else {
        return;
    };

    for (entity, mut defense_up) in player_defense_up.iter_mut() {
        if defense_up.active {
            defense_up.timer.tick(time.delta());
            if defense_up.timer.finished() {
                defense.0 -= defense_up.defense;
                commands.get_entity(entity).unwrap().despawn_recursive()
            }
        } else {
            defense_up.active = true;
            defense.0 += defense_up.defense;
        }
    }
}

fn process_enemy_denfense_down_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy: Query<&mut Defense, With<BattleEnemy>>,
    mut enemy_defense_down: Query<(Entity, &mut EnemyDefenseDownSpell)>,
) {
    let Ok(mut defense) = enemy.get_single_mut() else {
        for (entity, _) in enemy_defense_down.iter() {
            commands.get_entity(entity).unwrap().despawn_recursive()
        }
        return;
    };

    for (entity, mut defense_down) in enemy_defense_down.iter_mut() {
        if defense_down.active {
            defense_down.timer.tick(time.delta());
            if defense_down.timer.finished() {
                defense.0 += defense_down.defense;
                commands.get_entity(entity).unwrap().despawn_recursive()
            }
        } else {
            defense_down.active = true;
            defense.0 -= defense_down.defense;
        }
    }
}
