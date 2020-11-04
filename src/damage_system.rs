use crate::gamelog::GameLog;
use crate::Map;
use crate::{components::*, PlayerEntity};
use legion::{systems::CommandBuffer, *};

#[system(for_each)]
pub(crate) fn damage(
    entity: &Entity,
    stats: &mut CombatStats,
    damage: &SufferDamage,
    cb: &mut CommandBuffer,
) {
    stats.hp -= damage.amount;
    cb.remove_component::<SufferDamage>(*entity);
}

#[system(for_each)]
pub(crate) fn health(
    entity: &Entity,
    stats: &mut CombatStats,
    health: &ReceiveHealth,
    cb: &mut CommandBuffer,
) {
    if stats.max_hp == stats.hp {
        stats.max_hp += 1 + health.amount / 8;
        stats.hp = stats.max_hp;
    } else {
        stats.hp = i32::min(stats.max_hp, stats.hp + health.amount);
    }
    cb.remove_component::<ReceiveHealth>(*entity);
}

#[system(for_each)]
pub(crate) fn delete_the_dead(
    entity: &Entity,
    stats: &mut CombatStats,
    name: &Name,
    pos: &Position,
    cb: &mut CommandBuffer,
    #[resource] gamelog: &mut GameLog,
    #[resource] player_entity: &PlayerEntity,
    #[resource] map: &mut Map,
) {
    if stats.hp < 1 {
        if player_entity.0 == *entity {
            gamelog.log("You are dead");
        } else {
            gamelog.log(format!("{} dies.", &name.name));
            let idx = map.pos_to_idx(pos.0.into());
            // TODO: Handle via Events instead
            dbg!(&name);
            dbg!(&idx);
            dbg!(&pos);
            map.blocked[idx] = false;
            map.dangerous[idx] = false;
            cb.remove(*entity);
        }
    }
}
