mod camera;
mod gamelog;
mod map;

pub(crate) use camera::*;
pub(crate) use gamelog::*;
pub(crate) use map::*;

use crate::components::Position;
use crate::positions::{Direction, MapPosition};

use ::legion::Entity;

pub(crate) struct PlayerEntity(pub Entity);

#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) enum PlayerTarget {
    None,
    Position(MapPosition),
    Dir(Direction),
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct PlayerPosition(pub MapPosition);

impl Into<Position> for PlayerPosition {
    fn into(self) -> Position {
        Position(self.0)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct Time {
    pub real_time_ms: i64,
    pub last_real_time_ms: i64,
    pub tick: i64,
}

// TODO: pub(crate) fn new(gs: &State) -> Result<()> {
// TODO: pub(crate) fn save(gs: &State, writer: &mut dyn Write) -> Result<()> {
// pub(crate) fn load(gs: &mut State, reader: &mut dyn Read) -> Result<()> {
