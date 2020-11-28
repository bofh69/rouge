use crate::positions::MapPosition;
use legion::Entity;

#[derive(Clone, Debug)]
pub(crate) struct ReceiveHealthMessage {
    pub target: Entity,
    pub amount: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct RemoveItemMessage {
    pub target: Entity,
}

#[derive(Debug, Clone)]
pub(crate) struct SufferDamageMessage {
    pub target: Entity,
    pub amount: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct WantsToMeleeMessage {
    pub attacker: Entity,
    pub target: Entity,
}

pub(crate) struct WantsToUseMessage {
    pub who: Entity,
    pub item: Entity,
    pub target: Option<MapPosition>,
}
