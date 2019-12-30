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
        ReadStorage<'a, Potion>,
        WriteStorage<'a, WantsToDrinkPotion>,
        WriteStorage<'a, ReceiveHealth>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut gamelog,
            names,
            potions,
            mut wants_to_drinks,
            mut receive_healths,
        ) = data;

        for (drinker_entity, wants_to_drink, name) in
            (&entities, &mut wants_to_drinks, &names).join()
        {
            gamelog.log(format!(
                "{} drink the {}",
                if drinker_entity == player_entity.0 {
                    "You"
                } else {
                    &name.name
                },
                names.get(wants_to_drink.potion).unwrap().name
            ));
            let potion = potions.get(wants_to_drink.potion).unwrap();
            receive_healths
                .insert(
                    drinker_entity,
                    ReceiveHealth {
                        amount: potion.heal_amount,
                    },
                )
                .expect("Failed to insert");
            entities
                .delete(wants_to_drink.potion)
                .expect("Delete failed");
        }

        wants_to_drinks.clear();
    }
}