use bevy::prelude::*;

use super::GameState;

pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastSpell>()
            .add_systems(Startup, prepare_spells)
            .add_systems(Update, cast_spell.run_if(in_state(GameState::Battle)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpellIdx(pub usize);

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastSpell(pub SpellIdx);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spell {
    Lightning,
    Heal,
}

impl Spell {
    pub fn duration(&self) -> Option<f32> {
        match self {
            Self::Lightning => None,
            Self::Heal => None,
        }
    }

    pub fn cooldown(&self) -> f32 {
        match self {
            Self::Lightning => 2.0,
            Self::Heal => 5.0,
        }
    }
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
        spell: Spell::Lightning,
    });
    spells.0.push(SpellInfo {
        name: "Heal",
        drop_rate: 0.9,
        spell: Spell::Heal,
    });

    commands.insert_resource(spells);
}

fn cast_spell(spells: Res<Spells>, mut event_reader: EventReader<CastSpell>) {
    for e in event_reader.read() {
        let spell_info = &spells.0[e.0 .0];
        match spell_info.spell {
            Spell::Lightning => {
                println!("casting Lightning");
            }
            Spell::Heal => {
                println!("casting Heal");
            }
        }
    }
}
