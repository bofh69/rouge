use crate::components::*;
use crate::gamelog::GameLog;
use crate::PlayerEntity;
use specs::prelude::*;

pub struct DrinkPotionSystem {}

impl<'a> System<'a> for DrinkPotionSystem {
    #![allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, HealthProvider>,
        WriteStorage<'a, WantsToDrinkPotion>,
        WriteStorage<'a, ReceiveHealth>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut gamelog,
            names,
            consumables,
            healthproviders,
            mut wants_to_drinks,
            mut receive_healths,
        ) = data;

        for (drinker_entity, wants_to_drink, name) in
            (&entities, &mut wants_to_drinks, &names).join()
        {
            let item_name = &names.get(wants_to_drink.potion).unwrap().name;
            if let Some(_consumable) = consumables.get(wants_to_drink.potion) {
                gamelog.log(format!(
                    "{} drink{} the {}",
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
                if let Some(health) = healthproviders.get(wants_to_drink.potion) {
                    receive_healths
                        .insert(
                            drinker_entity,
                            ReceiveHealth {
                                amount: health.heal_amount,
                            },
                        )
                        .expect("Failed to insert");
                }
                entities
                    .delete(wants_to_drink.potion)
                    .expect("Delete failed");
            } else if drinker_entity == player_entity.0 {
                gamelog.log(format!("You cannot drink the {}", item_name));
            }
        }

        wants_to_drinks.clear();
    }
}
