use crate::components::*;
use crate::messages::{ReceiveHealthMessage, SufferDamageMessage};
use crate::queues::{ReceiveHealthQueue, SufferDamageQueue};
use crate::resources::{Map, OutputQueue, PlayerEntity};
use legion::world::SubWorld;
use legion::{system, systems::CommandBuffer, Entity, EntityStore};

#[system]
#[write_component(CombatStats)]
pub(crate) fn damage(world: &mut SubWorld, #[resource] queue: &SufferDamageQueue) {
    for SufferDamageMessage { target, amount } in queue.rx.try_iter() {
        if let Ok(ref mut entry) = world.entry_mut(target) {
            if let Ok(stats) = entry.get_component_mut::<CombatStats>() {
                stats.hp -= amount;
            }
        }
    }
}

#[system]
#[write_component(CombatStats)]
pub(crate) fn health(world: &mut SubWorld, #[resource] receive_health_queue: &ReceiveHealthQueue) {
    for ReceiveHealthMessage { target, amount } in receive_health_queue.rx.try_iter() {
        if let Ok(ref mut entry) = world.entry_mut(target) {
            if let Ok(stats) = entry.get_component_mut::<CombatStats>() {
                if stats.max_hp == stats.hp {
                    stats.max_hp += 1 + amount / 8;
                    stats.hp = stats.max_hp;
                } else {
                    stats.hp = i32::min(stats.max_hp, stats.hp + amount);
                }
            }
        }
    }
}

#[system(for_each)]
pub(crate) fn output_die(
    entity: &Entity,
    stats: &mut CombatStats,
    #[resource] output: &mut OutputQueue,
    #[resource] player_entity: &PlayerEntity,
) {
    if stats.hp < 1 {
        if player_entity.0 == *entity {
            output.s("You are dead");
        } else {
            output.the(*entity).v(*entity, "die");
        }
    }
}

#[system(for_each)]
pub(crate) fn delete_the_dead(
    entity: &Entity,
    stats: &mut CombatStats,
    pos: &Position,
    cb: &mut CommandBuffer,
    #[resource] player_entity: &PlayerEntity,
    #[resource] map: &mut Map,
) {
    if stats.hp < 1 && player_entity.0 != *entity {
        let idx = map.pos_to_idx(pos.0.into());
        // TODO: Handle via Events instead
        map.blocked[idx] = false;
        map.dangerous[idx] = false;
        cb.remove(*entity);
    }
}

#[system(for_each)]
pub(crate) fn delete_items(entity: &Entity, _remove: &RemoveItem, cb: &mut CommandBuffer) {
    cb.remove(*entity);
}
