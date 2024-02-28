use std::time::Duration;

use bevy::prelude::*;

use crate::components::{AutomaticControl, DieTickTimer, PlayerInput, SnakeEntity, SnakeMovement};

#[derive(Bundle)]
pub(crate) struct SnakeBaseBundle {
    snake: SnakeEntity,
    movenent: SnakeMovement,
    die_tick: DieTickTimer,
}
impl From<SnakeEntity> for SnakeBaseBundle {
    fn from(value: SnakeEntity) -> Self {
        let move_interval = match value.level {
            0 => 0.15,
            1 => 0.2,
            2 => 0.17,
            3 => 0.12,
            4 => 0.2,
            _ => 0.2,
        };
        Self {
            snake: value,
            movenent: SnakeMovement {
                interval: Duration::from_secs_f32(move_interval),
                blocked: false,
                move_timer: Timer::from_seconds(move_interval, TimerMode::Repeating),
            },
            die_tick: DieTickTimer(Timer::from_seconds(0.15, TimerMode::Once)),
        }
    }
}

#[derive(Bundle)]
pub(crate) struct PlayerControlSnakeBundle {
    player_input: PlayerInput,
    snake: SnakeBaseBundle,
}
impl From<SnakeEntity> for PlayerControlSnakeBundle {
    fn from(snake: SnakeEntity) -> Self {
        Self {
            player_input: PlayerInput,
            snake: snake.into(),
        }
    }
}

#[derive(Bundle)]
pub(crate) struct AutomaticControlSnakeBundle {
    snake: SnakeBaseBundle,
    automatic_control: AutomaticControl,
}
impl From<SnakeEntity> for AutomaticControlSnakeBundle {
    fn from(snake: SnakeEntity) -> Self {
        Self {
            snake: snake.into(),
            automatic_control: AutomaticControl,
        }
    }
}
