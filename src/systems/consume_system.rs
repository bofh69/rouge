use crate::components::*;
use crate::ecs::Ecs;
use crate::messages::{ReceiveHealthMessage, RemoveItemMessage, SufferDamageMessage};
use crate::queues::{ReceiveHealthQueue, RemoveItemQueue, SufferDamageQueue};
use crate::resources::{Camera, Map, OutputQueue};
use crate::PlayerEntity;
use crate::ScreenPosition;
use legion::*;

// TODO, make a proper system
pub(crate) fn consume_system(ecs: &mut Ecs) {
    let data: Vec<_> = <(Entity, &WantsToUseItem)>::query()
        .iter(&ecs.world)
        .map(|(entity, wants_to_use)| (*entity, wants_to_use.item, wants_to_use.target))
        .collect();

    let mut cb = legion::systems::CommandBuffer::new(&ecs.world);

    {
        let receive_health_queue = resource_get!(ecs, ReceiveHealthQueue);
        let suffer_damage_queue = resource_get!(ecs, SufferDamageQueue);
        for (user_entity, wants_to_use_item, wants_to_use_target) in data {
            let item_entry = ecs.world.entry(wants_to_use_item);
            let player_entity = resource_get!(ecs, PlayerEntity).0;

            if item_entry.is_some() {
                let output = ecs.resources.get::<OutputQueue>().unwrap();
                output
                    .the(user_entity)
                    .v(user_entity, "consume")
                    .the(wants_to_use_item);

                let mut targets: Vec<Entity> = Vec::new();
                match wants_to_use_target {
                    None => {
                        targets.push(player_entity);
                    }
                    Some(target) => {
                        let entry = ecs.world.entry(wants_to_use_item).unwrap();
                        let area_effect = entry.get_component::<AreaOfEffect>();
                        let map = resource_get!(ecs, Map);
                        if let Ok(area_effect) = area_effect {
                            // AoE
                            let camera = resource_get!(ecs, Camera);
                            let screen_point = camera.transform_map_pos(target).into();
                            for tile_point in bracket_lib::prelude::field_of_view(
                                screen_point,
                                area_effect.radius,
                                &*map,
                            )
                            .iter()
                            .filter_map(|p| {
                                if p.x >= 0
                                    && p.x < camera.width()
                                    && p.y >= 0
                                    && p.y < camera.height()
                                {
                                    Some(ScreenPosition { x: p.x, y: p.y })
                                } else {
                                    None
                                }
                            }) {
                                let idx =
                                    map.map_pos_to_idx(camera.transform_screen_pos(tile_point));
                                for mob in &map.tile_content[idx] {
                                    targets.push(*mob);
                                }
                            }
                        } else {
                            // Single target in tile
                            let idx = map.map_pos_to_idx(target);
                            for mob in &map.tile_content[idx] {
                                targets.push(*mob);
                            }
                        }
                    }
                }
                let heal_amount: Option<_> = {
                    if let Ok(health) = ecs
                        .world
                        .entry(wants_to_use_item)
                        .unwrap()
                        .get_component::<HealthProvider>()
                    {
                        Some(health.heal_amount)
                    } else {
                        None
                    }
                };
                if let Some(heal_amount) = heal_amount {
                    for target in targets {
                        if target == player_entity {
                            output.s("You feel better.");
                        } else {
                            output.the(target).v(target, "feel").s("better");
                        }

                        receive_health_queue.send(ReceiveHealthMessage {
                            target,
                            amount: heal_amount,
                        });
                    }
                } else {
                    let item_damage: Option<_> = {
                        if let Ok(damage) = ecs
                            .world
                            .entry(wants_to_use_item)
                            .unwrap()
                            .get_component::<InflictsDamage>()
                        {
                            Some(damage.damage)
                        } else {
                            None
                        }
                    };
                    if let Some(item_damage) = item_damage {
                        for target in targets {
                            output
                                .the(target)
                                .v(target, "lose")
                                .string(format!("{} hp", item_damage));
                            suffer_damage_queue.send(SufferDamageMessage {
                                target,
                                amount: item_damage,
                            });
                        }
                    }
                }
                let queue = resource_get!(ecs, RemoveItemQueue);
                queue.send(RemoveItemMessage {
                    target: wants_to_use_item,
                });
            } else if user_entity == player_entity {
                let output = ecs.resources.get::<OutputQueue>().unwrap();
                output.s("You cannot use").the(wants_to_use_item);
            }
            cb.remove_component::<WantsToUseItem>(user_entity);
        }
    }
    cb.flush(&mut ecs.world);
}
