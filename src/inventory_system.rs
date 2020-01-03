use crate::components::{InBackpack, Name, Position, WantsToDropItem, WantsToPickupItem};
use crate::gamelog::GameLog;
use crate::{PlayerEntity, PlayerPosition};
use specs::prelude::*;

pub struct ItemDroppingSystem {}

impl<'a> System<'a> for ItemDroppingSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, PlayerEntity>,
        ReadExpect<'a, PlayerPosition>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToDropItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            player_position,
            mut gamelog,
            mut wants_to_drop,
            mut positions,
            names,
            mut backpack,
        ) = data;

        for (dropper_entity, drop) in (&entities, &wants_to_drop).join() {
            positions
                .insert(drop.item, player_position.0.into())
                .expect("Insert failed");
            backpack
                .remove(drop.item)
                .expect("Unable to insert backpack entry");

            if dropper_entity == player_entity.0 {
                gamelog.log(format!(
                    "You drop the {}.",
                    names.get(drop.item).map_or("Unnamed", |x| &x.name)
                ));
            }
        }
        wants_to_drop.clear();
    }
}

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, PlayerEntity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == player_entity.0 {
                gamelog.log(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }
        wants_pickup.clear();
    }
}
