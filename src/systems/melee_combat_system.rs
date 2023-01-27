use crate::components::{CombatStats, Energy};
use crate::messages::{SufferDamageMessage, WantsToMeleeMessage};
use crate::queues::{SufferDamageQueue, WantsToMeleeQueue};
use crate::resources::OutputQueue;
use legion::world::SubWorld;
use legion::*;

#[system]
#[read_component(CombatStats)]
#[write_component(Energy)]
pub(crate) fn melee_combat(
    world: &mut SubWorld,
    #[resource] suffer_damage_queue: &SufferDamageQueue,
    #[resource] wants_to_melee_queue: &mut WantsToMeleeQueue,
    #[resource] output: &OutputQueue,
) {
    for WantsToMeleeMessage {
        attacker: attacker_entity,
        target: melee_target_entity,
    } in wants_to_melee_queue.try_iter()
    {
        if let Ok(attacker_entry) = world.entry_ref(attacker_entity) {
            let attacker_power = attacker_entry.get_component::<CombatStats>().unwrap().power;
            let target = world.entry_ref(melee_target_entity);
            if let Ok(target) = target {
                let target_stats = target.get_component::<CombatStats>().unwrap();

                if target_stats.hp > 0 {
                    let damage = i32::max(0, attacker_power - target_stats.defense);

                    if damage == 0 {
                        output
                            .the(attacker_entity)
                            .is(attacker_entity)
                            .s("unable to hurt")
                            .the(melee_target_entity);
                    } else {
                        output
                            .the(attacker_entity)
                            .v(attacker_entity, "hit")
                            .the(melee_target_entity)
                            .string(format!(", for {damage} hp"));
                        suffer_damage_queue.send(SufferDamageMessage {
                            target: melee_target_entity,
                            amount: damage,
                        });
                    }
                }
            } else {
                output
                    .the(attacker_entity)
                    .v(attacker_entity, "want")
                    .s("to attak a ghost?");
            }
            let mut entry = world.entry_mut(attacker_entity).unwrap();
            entry.get_component_mut::<Energy>().unwrap().energy = -120;
        }
    }
}
