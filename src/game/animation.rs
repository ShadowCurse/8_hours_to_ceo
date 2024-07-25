use std::time::Duration;

use bevy::prelude::*;

use super::GameState;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationFinished>().add_systems(
            Update,
            execute_animations
                .run_if(in_state(GameState::Running).or_else(in_state(GameState::Battle))),
        );
    }
}

#[derive(Event, Debug, Clone)]
pub struct AnimationFinished(pub AllAnimations);

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
}

impl AnimationConfig {
    pub fn new(
        first: usize,
        last: usize,
        fps: u8,
        animation: AllAnimations,
        send_finish_event: bool,
    ) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
            animation,
            send_finish_event,
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas)>,
    mut event_writer: EventWriter<AnimationFinished>,
) {
    for (mut config, mut atlas) in &mut query {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                atlas.index = config.first_sprite_index;
                if config.send_finish_event {
                    event_writer.send(AnimationFinished(config.animation));
                }
            } else {
                atlas.index += 1;
            }
            config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
        }
    }
}
