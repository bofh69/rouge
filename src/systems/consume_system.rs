use crate::components::*;
use crate::messages::{ReceiveHealthMessage, RemoveItemMessage, SufferDamageMessage};
use crate::queues::{ReceiveHealthQueue, RemoveItemQueue, SufferDamageQueue, WantsToUseQueue};
use crate::resources::{Camera, Map, OutputQueue};
use crate::PlayerEntity;
use crate::ScreenPosition;
use legion::*;

#[allow(clippy::too_many_arguments)]
#[system]
#[read_component(HealthProvider)]
#[read_component(InflictsDamage)]
#[read_component(Item)]
#[read_component(AreaOfEffect)]
pub(crate) fn consume(
    world: &legion::world::SubWorld,
    #[resource] receive_health_queue: &ReceiveHealthQueue,
    #[resource] remove_item_queue: &RemoveItemQueue,
    #[resource] suffer_damage_queue: &SufferDamageQueue,
    #[resource] wants_to_use_queue: &mut WantsToUseQueue,
    #[resource] player_entity: &PlayerEntity,
    #[resource] output: &OutputQueue,
    #[resource] camera: &Camera,
    #[resource] map: &Map,
) {
    for (user_entity, wants_to_use_item, wants_to_use_target) in wants_to_use_queue
        .try_iter()
        .map(|msg| (msg.who, msg.item, msg.target))
    {
        let item_entry = world.entry_ref(wants_to_use_item);

        if let Ok(item_entry) = item_entry {
            output
                .the(user_entity)
                .v(user_entity, "consume")
                .the(wants_to_use_item);

            let mut targets: Vec<Entity> = Vec::new();
            match wants_to_use_target {
                None => {
                    targets.push(player_entity.0);
                }
                Some(target) => {
                    let area_effect = item_entry.get_component::<AreaOfEffect>();
                    if let Ok(area_effect) = area_effect {
                        // AoE
                        let screen_point = camera.transform_map_pos(target).into();
                        for tile_point in bracket_lib::prelude::field_of_view(
                            screen_point,
                            area_effect.radius,
                            map,
                        )
                        .iter()
                        .filter_map(|p| {
                            if p.x >= 0 && p.x < camera.width() && p.y >= 0 && p.y < camera.height()
                            {
                                Some(ScreenPosition { x: p.x, y: p.y })
                            } else {
                                None
                            }
                        }) {
                            let idx = map.map_pos_to_idx(camera.transform_screen_pos(tile_point));
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
                if let Ok(health) = item_entry.get_component::<HealthProvider>() {
                    Some(health.heal_amount)
                } else {
                    None
                }
            };
            if let Some(heal_amount) = heal_amount {
                for target in targets {
                    if target == player_entity.0 {
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
                    if let Ok(damage) = item_entry.get_component::<InflictsDamage>() {
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
            remove_item_queue.send(RemoveItemMessage {
                target: wants_to_use_item,
            });
        } else if user_entity == player_entity.0 {
            output.s("You cannot use").the(wants_to_use_item);
        }
    }
}
