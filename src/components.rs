use crate::MapPosition;
use rltk::RGB;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::{
    error::NoError,
    saveload::{ConvertSaveload, Marker},
    Entity,
};

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct HealthProvider {
    pub heal_amount: i32,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Item {}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct ItemIndex {
    pub index: u8,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Player {}

#[derive(PartialEq, ConvertSaveload, Component, Copy, Clone, Debug)]
pub struct Position(pub MapPosition);

impl From<MapPosition> for Position {
    fn from(pos: MapPosition) -> Self {
        Position(pos)
    }
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct ReceiveHealth {
    pub amount: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

pub struct SerializeMe {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper(pub crate::map::Map);

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<MapPosition>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<MapPosition>,
}
