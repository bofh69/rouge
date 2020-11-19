use crate::ecs::Ecs;
use crate::gamelog::OutputQueue;
use crate::messages::SufferDamageMessage;
use crate::queues::SufferDamageQueue;
use crate::{CombatStats, WantsToMelee};
use legion::{Entity, IntoQuery};

// TODO Make a proper system
pub(crate) fn melee_combat_system(ecs: &mut Ecs) {
    let combatees: Vec<_> = <(Entity, &WantsToMelee, &CombatStats)>::query()
        .iter(&ecs.world)
        .filter_map(|(entity, wants_to_melee, stats)| {
            if stats.hp > 0 {
                Some((*entity, wants_to_melee.target, stats.power))
            } else {
                None
            }
        })
        .collect();
    let suffer_damage_queue = resource_get!(ecs, SufferDamageQueue);

    for (attacker_entity, melee_target_entity, attacker_power) in combatees {
        let target = ecs.world.entry(melee_target_entity);
        let target = target.unwrap();
        let target_stats = target.get_component::<CombatStats>().unwrap();

        if target_stats.hp > 0 {
            let damage = i32::max(0, attacker_power - target_stats.defense);

            if damage == 0 {
                let mut output = resource_get_mut!(ecs, OutputQueue);
                output
                    .the(attacker_entity)
                    .is(attacker_entity)
                    .s("unable to hurt")
                    .the(melee_target_entity);
            } else {
                let mut output = resource_get_mut!(ecs, OutputQueue);
                output
                    .the(attacker_entity)
                    .v(attacker_entity, "hit")
                    .the(melee_target_entity)
                    .string(format!(", for {} hp", damage));
                suffer_damage_queue
                    .tx
                    .send(SufferDamageMessage {
                        target: melee_target_entity,
                        amount: damage,
                    })
                    .unwrap();
            }
        }
        ecs.world
            .entry(attacker_entity)
            .unwrap()
            .remove_component::<WantsToMelee>();
    }
}
