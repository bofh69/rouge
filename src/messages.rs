use legion::Entity;

#[derive(Clone, Debug)]
pub(crate) struct ReceiveHealthMessage {
    pub target: Entity,
    pub amount: i32,
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
