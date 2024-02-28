use bevy::prelude::*;
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

#[derive(Component, Debug)]
pub(crate) struct SnakeEntity {
    pub(crate) length: usize,
    pub(crate) direction: IVec2,
    pub(crate) position: IVec2,

    pub(crate) tail: IVec2,

    pub level: usize,
}

#[derive(Component, Debug)]
pub(crate) struct SnakeMovement {
    pub(crate) interval: Duration,
    pub(crate) blocked: bool,
    pub(crate) move_timer: Timer,
}

#[derive(Component, Debug)]
pub(crate) struct DieTickTimer(pub(crate) Timer);

impl Deref for DieTickTimer {
    type Target = Timer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DieTickTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub(crate) struct PlayerInput;

#[derive(Component)]
pub(crate) struct AutomaticControl;
