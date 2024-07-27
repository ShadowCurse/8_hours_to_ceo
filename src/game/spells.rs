use std::ops::{Index, IndexMut};

use bevy::prelude::*;

use super::{
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
                    process_lightninig,
                    process_heal,
                    process_player_attack_up,
                    process_player_defence_up,
                    process_enemy_denfense_down,
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
pub struct Lightning {
    strikes: u32,
    delta_time: f32,
    damage: f32,
}

#[derive(Component, Debug, Clone)]
pub struct LightningSpell {
    timer: Timer,
    remaining_strikes: u32,
    damage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heal {
    heal: f32,
}

#[derive(Component, Debug, Clone)]
pub struct HealSpell {
    heal: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAttackUp {
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
pub struct PlayerDefenseUp {
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
pub struct EnemyDefenseDown {
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
    Lightning(Lightning),
    Heal(Heal),
    PlayerAttackUp(PlayerAttackUp),
    PlayerDefenseUp(PlayerDefenseUp),
    EnemyDefenseDown(EnemyDefenseDown),
}

#[derive(Debug)]
pub struct SpellInfo {
    pub name: &'static str,
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
        name: "Lightning",
        description: "Flashy lightning",
        image: asset_server.load("spells/tmp_spell.png"),
        drop_rate: 0.9,
        cooldown: Timer::from_seconds(2.0, TimerMode::Once),
        spell: Spell::Lightning(Lightning {
            strikes: 2,
            delta_time: 0.1,
            damage: 1.0,
        }),
    });
    spells.0.push(SpellInfo {
        name: "Heal",
        description: "Healing heal",
        image: asset_server.load("spells/tmp_spell.png"),
        drop_rate: 0.9,
        cooldown: Timer::from_seconds(5.0, TimerMode::Once),
        spell: Spell::Heal(Heal { heal: 20.0 }),
    });
    spells.0.push(SpellInfo {
        name: "Player attack up",
        description: "+10 damage for 1 second",
        image: asset_server.load("spells/tmp_spell.png"),
        drop_rate: 0.9,
        cooldown: Timer::from_seconds(5.0, TimerMode::Once),
        spell: Spell::PlayerAttackUp(PlayerAttackUp {
            duration: 1.0,
            attack: 10.0,
        }),
    });
    spells.0.push(SpellInfo {
        name: "Player defence up",
        description: "+10% defence for 1 second",
        image: asset_server.load("spells/tmp_spell.png"),
        drop_rate: 0.9,
        cooldown: Timer::from_seconds(5.0, TimerMode::Once),
        spell: Spell::PlayerDefenseUp(PlayerDefenseUp {
            duration: 1.0,
            defense: 0.1,
        }),
    });
    spells.0.push(SpellInfo {
        name: "EnemyDefenceDown",
        description: "-10% enemy defence for 1 second",
        image: asset_server.load("spells/tmp_spell.png"),
        drop_rate: 0.9,
        cooldown: Timer::from_seconds(5.0, TimerMode::Once),
        spell: Spell::EnemyDefenseDown(EnemyDefenseDown {
            duration: 1.0,
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
            Spell::Lightning(lightning) => {
                commands.spawn(LightningSpell {
                    timer: Timer::from_seconds(lightning.delta_time, TimerMode::Repeating),
                    remaining_strikes: lightning.strikes,
                    damage: lightning.damage,
                });
            }
            Spell::Heal(heal) => {
                commands.spawn(HealSpell { heal: heal.heal });
            }
            Spell::PlayerAttackUp(attack_up) => {
                commands.spawn(PlayerAttackUpSpell {
                    timer: Timer::from_seconds(attack_up.duration, TimerMode::Once),
                    active: false,
                    attack: attack_up.attack,
                });
            }
            Spell::PlayerDefenseUp(defense_up) => {
                commands.spawn(PlayerDefenseUpSpell {
                    timer: Timer::from_seconds(defense_up.duration, TimerMode::Once),
                    active: false,
                    defense: defense_up.defense,
                });
            }
            Spell::EnemyDefenseDown(defense_down) => {
                commands.spawn(EnemyDefenseDownSpell {
                    timer: Timer::from_seconds(defense_down.duration, TimerMode::Once),
                    active: false,
                    defense: defense_down.defense,
                });
            }
        }
    }
}

fn process_lightninig(
    time: Res<Time>,
    mut commands: Commands,
    mut lightnings: Query<(Entity, &mut LightningSpell)>,
    mut event_writer: EventWriter<DamageEnemyEvent>,
) {
    for (lightning_entity, mut lightning) in lightnings.iter_mut() {
        lightning.timer.tick(time.delta());
        if lightning.timer.finished() {
            event_writer.send(DamageEnemyEvent {
                damage: lightning.damage,
                color: Color::srgb(0.0, 0.0, 1.0),
            });
            lightning.remaining_strikes -= 1;

            if lightning.remaining_strikes == 0 {
                commands
                    .get_entity(lightning_entity)
                    .unwrap()
                    .despawn_recursive()
            }
        }
    }
}

fn process_heal(
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

fn process_player_attack_up(
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

fn process_player_defence_up(
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

fn process_enemy_denfense_down(
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
