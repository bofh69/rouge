use crate::camera::Camera;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::PlayerEntity;
use crate::ScreenPosition;
use specs::prelude::*;

pub struct UseItemSystem {}

impl<'a> System<'a> for UseItemSystem {
    #![allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, AreaOfEffect>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, HealthProvider>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, WantsToUseItem>,
        WriteStorage<'a, ReceiveHealth>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            camera,
            map,
            player_entity,
            mut gamelog,
            aoe,
            names,
            consumables,
            healthproviders,
            inflict_damage,
            mut wants_to_uses,
            mut receive_healths,
            mut suffer_damage,
        ) = data;

        for (drinker_entity, wants_to_use, name) in (&entities, &mut wants_to_uses, &names).join() {
            let item_name = &names.get(wants_to_use.item).unwrap().name;
            if let Some(_consumable) = consumables.get(wants_to_use.item) {
                gamelog.log(format!(
                    "{} consume{} the {}",
                    if drinker_entity == player_entity.0 {
                        "You"
                    } else {
                        &name.name
                    },
                    if drinker_entity == player_entity.0 {
                        ""
                    } else {
                        "s"
                    },
                    item_name
                ));
                let mut targets: Vec<Entity> = Vec::new();
                match wants_to_use.target {
                    None => {
                        targets.push(player_entity.0);
                    }
                    Some(target) => {
                        let area_effect = aoe.get(wants_to_use.item);
                        match area_effect {
                            None => {
                                // Single target in tile
                                let idx = map.map_pos_to_idx(target);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                            Some(area_effect) => {
                                // AoE
                                let screen_point = camera.transform_map_pos(target).into();
                                for tile_point in
                                    rltk::field_of_view(screen_point, area_effect.radius, &*map)
                                        .iter()
                                        .filter(|p| {
                                            p.x >= 0
                                                && p.x < camera.width()
                                                && p.y >= 0
                                                && p.y < camera.height()
                                        })
                                        .map(|p| ScreenPosition { x: p.x, y: p.y })
                                {
                                    let idx =
                                        map.map_pos_to_idx(camera.transform_screen_pos(tile_point));
                                    for mob in map.tile_content[idx].iter() {
                                        targets.push(*mob);
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(health) = healthproviders.get(wants_to_use.item) {
                    for target in targets {
                        if target == player_entity.0 {
                            gamelog.log("You feel better.");
                        } else {
                            let name = names.get(target).expect("Target's name is missing");
                            gamelog.log(format!("The {} feel better.", name.name));
                        }
                        receive_healths
                            .insert(
                                target,
                                ReceiveHealth {
                                    amount: health.heal_amount,
                                },
                            )
                            .expect("Failed to insert");
                    }
                } else if let Some(item_damages) = inflict_damage.get(wants_to_use.item) {
                    for target in targets {
                        if target == player_entity.0 {
                            gamelog.log(format!("You lose {} hp.", item_damages.damage));
                        } else {
                            let name = names.get(target).expect("Target's name is missing");
                            gamelog.log(format!(
                                "The {} loses {} hp.",
                                name.name, item_damages.damage
                            ));
                        }
                        suffer_damage
                            .insert(
                                target,
                                SufferDamage {
                                    amount: item_damages.damage,
                                },
                            )
                            .expect("Failed to insert");
                    }
                }
                entities.delete(wants_to_use.item).expect("Delete failed");
            } else if drinker_entity == player_entity.0 {
                gamelog.log(format!("You cannot use the {}", item_name));
            }
        }

        wants_to_uses.clear();
    }
}
