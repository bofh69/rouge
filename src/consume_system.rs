use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::PlayerEntity;
use crate::ScreenPosition;
use crate::{camera::Camera, ecs::Ecs};
use legion::*;

pub(crate) fn consume_system(ecs: &mut Ecs) {
    let data: Vec<_> = <(Entity, &WantsToUseItem, &Name)>::query()
        .iter(&ecs.world)
        .map(|(entity, wants_to_use, drinker_name)| {
            (
                *entity,
                wants_to_use.item,
                wants_to_use.target,
                drinker_name.name.clone(),
            )
        })
        .collect();

    for (user_entity, wants_to_use_item, wants_to_use_target, drinker_name) in data {
        let item_entry = ecs.world.entry(wants_to_use_item);
        let player_entity = resource_get!(ecs, PlayerEntity).0;

        if item_entry.is_some() {
            let mut gamelog = ecs.resources.get_mut::<GameLog>().unwrap();

            gamelog.log(format!(
                "{} consume{} the {}",
                if user_entity == player_entity {
                    "You"
                } else {
                    &drinker_name
                },
                if user_entity == player_entity {
                    ""
                } else {
                    "s"
                },
                &item_entry.unwrap().get_component::<Name>().unwrap().name
            ));

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
                        .filter(|p| {
                            p.x >= 0 && p.x < camera.width() && p.y >= 0 && p.y < camera.height()
                        })
                        .map(|p| ScreenPosition { x: p.x, y: p.y })
                        {
                            let idx = map.map_pos_to_idx(camera.transform_screen_pos(tile_point));
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                    } else {
                        // Single target in tile
                        let idx = map.map_pos_to_idx(target);
                        for mob in map.tile_content[idx].iter() {
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
                    let mut target_entry = ecs.world.entry(target).unwrap();
                    if target == player_entity {
                        gamelog.log("You feel better.");
                    } else {
                        let name = &target_entry
                            .get_component::<Name>()
                            .expect("Target's name is missing")
                            .name;
                        gamelog.log(format!("The {} feel better.", name));
                    }
                    target_entry.add_component(ReceiveHealth {
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
                        let mut target_entry = ecs.world.entry(target).unwrap();
                        if target == player_entity {
                            gamelog.log(format!("You lose {} hp.", item_damage));
                        } else {
                            let name = &target_entry
                                .get_component::<Name>()
                                .expect("Target's name is missing")
                                .name;
                            gamelog.log(format!("The {} loses {} hp.", name, item_damage));
                        }
                        target_entry.add_component(SufferDamage {
                            amount: item_damage,
                        });
                    }
                }
            }
            ecs.world.remove(wants_to_use_item);
        } else if user_entity == player_entity {
            let mut gamelog = ecs.resources.get_mut::<GameLog>().unwrap();
            gamelog.log(format!(
                "You cannot use the {}",
                &item_entry.unwrap().get_component::<Name>().unwrap().name
            ));
        }
        ecs.world
            .entry(user_entity)
            .unwrap()
            .remove_component::<WantsToUseItem>();
    }
}
