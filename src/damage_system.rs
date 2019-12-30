extern crate specs;
use crate::components::*;
use crate::gamelog::GameLog;
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, ReceiveHealth>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, mut health) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount;
        }

        // Lets be kind...
        for (mut stats, health) in (&mut stats, &health).join() {
            if stats.max_hp == stats.hp {
                stats.max_hp += 1 + health.amount / 8;
                stats.hp = stats.max_hp;
            } else {
                stats.hp = i32::min(stats.max_hp, stats.hp + health.amount);
            }
        }

        damage.clear();
        health.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let mut gamelog = ecs.write_resource::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => dead.push(entity),
                    Some(_) => gamelog.log("You are dead"),
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
