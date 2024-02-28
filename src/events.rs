use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct SnakeGrowthEvent {
    pub snake: Entity,
}

#[derive(Event, Debug)]
pub(crate) struct SnakeDiedEvent {
    pub snake: Entity,
}

#[derive(Event, Debug)]
pub(crate) struct SpawnSnakeEvent {
    pub position: IVec2,
    pub level: usize,
}
