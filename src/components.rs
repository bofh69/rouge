use crate::MapPosition;
use bracket_lib::prelude::RGB;
use legion::Entity;

#[derive(Clone, Debug)]
pub(crate) struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct BlocksTile {}

#[derive(Clone, Debug)]
pub(crate) struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct Consumable {}

#[derive(Clone, Debug)]
pub(crate) struct HealthProvider {
    pub heal_amount: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct InBackpack {
    pub owner: Entity,
}

#[derive(Clone, Debug)]
pub(crate) struct InflictsDamage {
    pub damage: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct Item {}

#[derive(Debug, Clone)]
pub(crate) struct ItemIndex {
    pub index: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct Monster {}

#[derive(Clone, Debug)]
pub(crate) struct Name {
    pub name: String,
    pub proper_name: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Player {}

#[derive(PartialEq, Copy, Clone, Debug)]
pub(crate) struct Position(pub MapPosition);

impl From<MapPosition> for Position {
    fn from(pos: MapPosition) -> Self {
        Position(pos)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Ranged {
    pub range: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct ReceiveHealthMessage {
    pub target: Entity,
    pub amount: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct RemoveItem {}

#[derive(Clone)]
pub(crate) struct Renderable {
    pub glyph: u16,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct SufferDamage {
    pub amount: i32,
}

#[derive(Clone)]
pub(crate) struct Viewshed {
    pub visible_tiles: Vec<MapPosition>,
    pub range: i32,
    pub dirty: bool,
}

// TODO should be changed to a message
// TODO how to handle "drop all"?
#[derive(Clone, Debug)]
pub(crate) struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Debug, Clone)]
pub(crate) struct WantsToMelee {
    pub target: Entity,
}

// TODO should be changed to a message
// TODO how to handle "get all"?
#[derive(Debug, Clone)]
pub(crate) struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Clone, Debug)]
pub(crate) struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<MapPosition>,
}
