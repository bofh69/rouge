use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::PlayerEntity;
use specs::prelude::*;

pub struct DrinkPotionSystem {}

impl<'a> System<'a> for DrinkPotionSystem {
    #![allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
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
            map,
            player_entity,
            mut gamelog,
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
                if let Some(health) = healthproviders.get(wants_to_use.item) {
                    receive_healths
                        .insert(
                            drinker_entity,
                            ReceiveHealth {
                                amount: health.heal_amount,
                            },
                        )
                        .expect("Failed to insert");
                }
                let item_damages = inflict_damage.get(wants_to_use.item);
                match item_damages {
                    None => {}
                    Some(damage) => {
                        rltk::console::log("Looking for target mob");
                        let target_point = wants_to_use.target.unwrap();
                        let idx = map.xy_idx(target_point.x, target_point.y);
                        for mob in map.tile_content[idx].iter() {
                            rltk::console::log("Found target mob");
                            suffer_damage
                                .insert(
                                    *mob,
                                    SufferDamage {
                                        amount: damage.damage,
                                    },
                                )
                                .expect("Unable to insert");
                            if drinker_entity == player_entity.0 {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(wants_to_use.item).unwrap();
                                gamelog.log(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name.name, mob_name.name, damage.damage
                                ));
                            }
                            break;
                        }
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
