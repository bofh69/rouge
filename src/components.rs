use crate::MapPosition;
use bracket_lib::prelude::RGB;
use legion::Entity;

#[derive(Clone, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Debug)]
pub struct BlocksTile {}

#[derive(Clone, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Clone, Debug)]
pub struct Consumable {}

#[derive(Clone, Debug)]
pub struct HealthProvider {
    pub heal_amount: i32,
}

#[derive(Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Clone, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Clone, Debug)]
pub struct Item {}

#[derive(Debug, Clone)]
pub struct ItemIndex {
    pub index: u8,
}

#[derive(Clone, Debug)]
pub struct Monster {}

#[derive(Clone, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Player {}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Position(pub MapPosition);

impl From<MapPosition> for Position {
    fn from(pos: MapPosition) -> Self {
        Position(pos)
    }
}

#[derive(Clone, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Clone, Debug)]
pub struct ReceiveHealth {
    pub amount: i32,
}

#[derive(Clone)]
pub struct Renderable {
    pub glyph: u16,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Debug, Clone)]
pub struct SufferDamage {
    pub amount: i32,
}

#[derive(Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<MapPosition>,
    pub range: i32,
    pub dirty: bool,
}

// TODO should be changed to a message
// TODO how to handle "drop all"?
#[derive(Clone, Debug)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

// TODO should be changed to a message
// TODO how to handle "get all"?
#[derive(Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Clone, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<MapPosition>,
}
