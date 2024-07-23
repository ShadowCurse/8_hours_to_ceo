use bevy::prelude::*;

use super::{enemy::BattleEnemy, Defense, GameState, Health, Player};

pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpell>()
            .add_systems(Startup, prepare_spells)
            .add_systems(
                Update,
                (cast_spell, process_lightninig, process_heal).run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpellIdx(pub usize);

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastSpell(pub SpellIdx);

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
pub enum Spell {
    Lightning(Lightning),
    Heal(Heal),
}

#[derive(Debug)]
pub struct SpellInfo {
    pub name: &'static str,
    pub drop_rate: f32,
    pub spell: Spell,
}

#[derive(Resource, Debug)]
pub struct Spells(pub Vec<SpellInfo>);

fn prepare_spells(mut commands: Commands) {
    let mut spells = Spells(vec![]);

    spells.0.push(SpellInfo {
        name: "Lightning",
        drop_rate: 0.9,
        spell: Spell::Lightning(Lightning {
            strikes: 2,
            delta_time: 0.1,
            damage: 1.0,
        }),
    });
    spells.0.push(SpellInfo {
        name: "Heal",
        drop_rate: 0.9,
        spell: Spell::Heal(Heal { heal: 20.0 }),
    });

    commands.insert_resource(spells);
}

fn cast_spell(
    spells: Res<Spells>,

    mut commands: Commands,
    mut event_reader: EventReader<CastSpell>,
) {
    for e in event_reader.read() {
        let spell_info = &spells.0[e.0 .0];
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
        }
    }
}

fn process_lightninig(
    time: Res<Time>,
    mut commands: Commands,
    mut lightnings: Query<(Entity, &mut LightningSpell)>,
    mut enemy: Query<(&Defense, &mut Health), With<BattleEnemy>>,
) {
    let Ok((enemy_defense, mut enemy_health)) = enemy.get_single_mut() else {
        return;
    };

    for (lightning_entity, mut lightning) in lightnings.iter_mut() {
        lightning.timer.tick(time.delta());
        if lightning.timer.finished() {
            let damage = lightning.damage * (1.0 - enemy_defense.0);
            enemy_health.0 -= damage;
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
        player_health.0 += heal.heal;
        commands
            .get_entity(heal_entity)
            .unwrap()
            .despawn_recursive()
    }
}
