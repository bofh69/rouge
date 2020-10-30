use crate::MapPosition;
use bracket_lib::prelude::RGB;
use specs::prelude::*;
use specs::{
    Entity,
};

#[derive(Component, Clone, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Clone, Debug)]
pub struct BlocksTile {}

#[derive(Component, Clone, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Clone, Debug)]
pub struct Consumable {}

#[derive(Component, Clone, Debug)]
pub struct HealthProvider {
    pub heal_amount: i32,
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Clone, Debug)]
pub struct Item {}

#[derive(Component, Debug, Clone)]
pub struct ItemIndex {
    pub index: u8,
}

#[derive(Component, Clone, Debug)]
pub struct Monster {}

#[derive(Component, Clone, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Clone, Debug)]
pub struct Player {}

#[derive(PartialEq, Component, Copy, Clone, Debug)]
pub struct Position(pub MapPosition);

impl From<MapPosition> for Position {
    fn from(pos: MapPosition) -> Self {
        Position(pos)
    }
}

#[derive(Component, Clone, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Clone, Debug)]
pub struct ReceiveHealth {
    pub amount: i32,
}

#[derive(Component, Clone)]
pub struct Renderable {
    pub glyph: u16,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Debug, Clone)]
pub struct SufferDamage {
    pub amount: i32,
}

#[derive(Component, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<MapPosition>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Clone, Debug)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<MapPosition>,
}
