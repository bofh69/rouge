use rltk::RGB;
use specs::prelude::*;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct LeftMover {}

#[derive(Component, Debug)]
pub struct Player {}
