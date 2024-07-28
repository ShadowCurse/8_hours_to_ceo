use std::time::Duration;

use bevy::prelude::*;

use crate::{ui::UiStyle, GlobalState};

use super::GameState;

pub const DAMAGE_COLOR_DEFAULT: Color = Color::srgb(1.0, 0.0, 0.0);
pub const DAMAGE_COLOR_KEYBOARD: Color = Color::srgb(0.5, 0.5, 0.5);
pub const DAMAGE_COLOR_MARKER: Color = Color::srgb(52.0 / 255.0, 52.0 / 255.0, 209.0 / 255.0);
pub const DAMAGE_COLOR_FIRE_PUNCH: Color = Color::srgb(209.0 / 255.0, 115.0 / 255.0, 46.0 / 255.0);

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationFinishedEvent>().add_systems(
            Update,
            (run_sprite_animations, run_damage_text_animations).run_if(
                in_state(GameState::Running)
                    .or_else(in_state(GameState::Battle).or_else(in_state(GameState::Pickup))),
            ),
        );
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct AnimationFinishedEvent(pub AllAnimations);

#[derive(Component, Debug, Clone, Copy)]
pub struct DamageText {
    pub direction: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllAnimations {
    PlayerIdle,
    PlayerRun,
    PlayerAttack,
    PlayerDead,
    BossIdle,
    BossRun,
    BossAttack,
    BossDead,
}

#[derive(Component, Debug, Clone)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
    pub animation: AllAnimations,
    pub send_finish_event: bool,
    pub continues: bool,
}

impl AnimationConfig {
    pub fn new(
        first: usize,
        last: usize,
        fps: u8,
        animation: AllAnimations,
        send_finish_event: bool,
        continues: bool,
    ) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
            animation,
            send_finish_event,
            continues,
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

pub fn spawn_damage_text(
    commands: &mut Commands,
    ui_style: &UiStyle,
    damage: f32,
    transform: Transform,
    direction: Vec3,
    color: Color,
) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("{}", damage),
                TextStyle {
                    font: ui_style.text_style.font.clone(),
                    font_size: 30.0,
                    color,
                },
            ),
            transform,
            ..Default::default()
        },
        DamageText { direction },
        StateScoped(GlobalState::InGame),
    ));
}

fn run_sprite_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas)>,
    mut event_writer: EventWriter<AnimationFinishedEvent>,
) {
    for (mut config, mut atlas) in &mut query {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                if config.continues {
                    atlas.index = config.first_sprite_index;
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }
                if config.send_finish_event {
                    event_writer.send(AnimationFinishedEvent(config.animation));
                }
            } else {
                atlas.index += 1;
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}

fn run_damage_text_animations(
    time: Res<Time>,
    mut commands: Commands,
    mut damage_texts: Query<(Entity, &DamageText, &mut Transform)>,
) {
    for (e, dt, mut t) in damage_texts.iter_mut() {
        t.translation += dt.direction * time.delta_seconds() * 100.0;
        t.scale -= time.delta_seconds() * 4.0;

        if t.scale.x <= 0.1 {
            commands.get_entity(e).unwrap().despawn_recursive();
        }
    }
}
