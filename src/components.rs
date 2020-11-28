use crate::MapPosition;
use ::bracket_lib::prelude::RGB;
use ::legion::Entity;
use ::legion_typeuuid::*;
use ::serde::*;
use ::type_uuid::*;

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "0ba9a288-a1a7-45b5-8964-44cbc0a8b953"]
pub(crate) struct AreaOfEffect {
    pub radius: i32,
}
register_serialize!(AreaOfEffect);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "b7e17796-b15d-4498-b3b0-4eeb79af3878"]
pub(crate) struct BlocksTile {}
register_serialize!(BlocksTile);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "7b5aa67d-1cff-49f6-bdab-6f446f9d22a1"]
pub(crate) struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}
register_serialize!(CombatStats);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "f149f629-12cd-4a04-a158-ad4fbfd221d7"]
pub(crate) struct Consumable {}
register_serialize!(Consumable);

/// The object is removed at the given tick.
#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "3993ee01-ce19-4a06-aa23-b0b500d61d18"]
pub(crate) struct EndTick {
    pub end_tick: i64,
}
register_serialize!(EndTick);

/// The object is removed at the given time.
#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "9fe4c623-7dcd-45f1-bd63-16254360c5ec"]
pub(crate) struct EndTime {
    pub end_time_ms: i64,
}
register_serialize!(EndTime);

/// Animated objects need energy to perform actions.
/// The more an action cost, the more energy it drains.
/// Energy >= 0 means the object can act, the new energy becomes -action_cost.
/// If the player has energy >= 0, it makes a turn.
/// Then monster with the highest energy go first. If it can't do anything, it will have more energy
/// next time.
#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "9225dc28-62ff-4e46-be43-76436da77561"]
pub(crate) struct Energy {
    pub energy: i32,
}
register_serialize!(Energy);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "9ea2eda5-8e86-48ca-a831-8044fe7f4064"]
pub(crate) struct HealthProvider {
    pub heal_amount: i32,
}
register_serialize!(HealthProvider);

#[derive(Serialize, Deserialize, Debug, Clone, TypeUuid)]
#[uuid = "6f3fedb4-3dd9-4a2d-a2a6-51149b614254"]
pub(crate) struct InBackpack {
    pub owner: Entity,
}
register_serialize!(InBackpack);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "0d38045c-4cb0-46f6-aec3-92c478e4a6db"]
pub(crate) struct InflictsDamage {
    pub damage: i32,
}
register_serialize!(InflictsDamage);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "3fe6f537-42ab-4ea7-868b-06dd465ec123"]
pub(crate) struct Item {}
register_serialize!(Item);

#[derive(Serialize, Deserialize, Debug, Clone, TypeUuid)]
#[uuid = "e34d9ba1-6289-4c1c-95fb-0075ee34fa09"]
pub(crate) struct ItemIndex {
    pub index: u8,
}
register_serialize!(ItemIndex);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "974bf33c-2dd4-4317-9747-680e4ecefb54"]
pub(crate) struct Monster {}
register_serialize!(Monster);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "d866e77f-91de-4917-a65d-c16d8b858543"]
pub(crate) struct Name {
    pub name: String,
    pub proper_name: bool,
}
register_serialize!(Name);

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "c186ed8d-325b-4adc-a5de-2ae2a6f0ce25"]
pub(crate) struct Player {}
register_serialize!(Player);

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug, TypeUuid)]
#[uuid = "7a11cab0-db87-48b0-acfc-74056cd9a625"]
pub(crate) struct Position(pub MapPosition);
register_serialize!(Position);

impl From<MapPosition> for Position {
    fn from(pos: MapPosition) -> Self {
        Position(pos)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeUuid)]
#[uuid = "6d29666a-c126-44d9-a90d-864890f804ee"]
pub(crate) struct Ranged {
    pub range: i32,
}
register_serialize!(Ranged);

#[derive(Serialize, Deserialize, Clone, TypeUuid)]
#[uuid = "6315dfee-74b9-42f4-91dc-145b17723c2e"]
pub(crate) struct Renderable {
    pub glyph: u16,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}
register_serialize!(Renderable);

#[derive(Serialize, Deserialize, Clone, TypeUuid)]
#[uuid = "92d745b2-217a-426f-b6e4-2b0c2811fcd9"]
pub(crate) struct Viewshed {
    pub visible_tiles: Vec<MapPosition>,
    pub range: i32,
    pub dirty: bool,
}
register_serialize!(Viewshed);
