use super::{CombatStats, Name, SufferDamage, WantsToMelee};
use crate::gamelog::GameLog;
use crate::ecs::*;
use legion::*;

pub(crate) fn melee_combat_system(ecs: &mut Ecs) {
    let combatees: Vec<_> = <(Entity, &WantsToMelee, &Name, &CombatStats)>::query()
        .iter(&ecs.world)
        .filter(|(_entity, _wants_to_melee, _name, stats)| stats.hp > 0)
        .map(|(entity, wants_to_melee, name, stats)| {
            (
                *entity,
                wants_to_melee.target,
                name.name.clone(),
                stats.power,
            )
        })
        .collect();

    for (attacker_entity, melee_target_entity, attacker_name, attacker_power) in combatees {
        let target = ecs.world.entry(melee_target_entity);
        let mut target = target.unwrap();
        let target_stats = target.get_component::<CombatStats>().unwrap();

        if target_stats.hp > 0 {
            let target_name = target.get_component::<Name>().unwrap().name.clone();

            let damage = i32::max(0, attacker_power - target_stats.defense);

            if damage == 0 {
                let mut gamelog = resource_get_mut!(ecs, GameLog);
                gamelog.log(&format!(
                    "{} is unable to hurt {}",
                    &attacker_name, &target_name
                ));
            } else {
                let mut gamelog = resource_get_mut!(ecs, GameLog);
                gamelog.log(&format!(
                    "{} hits {}, for {} hp.",
                    &attacker_name, &target_name, damage
                ));
                target.add_component(SufferDamage { amount: damage });
            }
        }
        ecs.world
            .entry(attacker_entity)
            .unwrap()
            .remove_component::<WantsToMelee>();
    }
}
