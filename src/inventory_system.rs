use crate::components::{
    InBackpack, ItemIndex, Name, Position, WantsToDropItem, WantsToPickupItem,
};
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
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, ItemIndex>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToPickupItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            names,
            mut backpack,
            mut item_index,
            mut positions,
            mut wants_pickup,
        ) = data;

        for pickup in wants_pickup.join() {
            if pickup.collected_by == player_entity.0 {
                let mut possible_indexes = std::collections::HashSet::new();
                for c in 0..52 {
                    possible_indexes.insert(c);
                }
                for (item_idx, in_backpack) in (&item_index, &backpack).join() {
                    if in_backpack.owner == player_entity.0 {
                        possible_indexes.remove(&item_idx.index);
                    }
                }
                let mut possible_indexes: Vec<_> = possible_indexes.drain().collect();
                possible_indexes.sort();

                let mut idx = 255u8;
                if let Some(ItemIndex { index }) = item_index.get(pickup.item) {
                    if possible_indexes.contains(index) {
                        idx = *index;
                    }
                }
                if idx == 255u8 {
                    if possible_indexes.is_empty() {
                        gamelog.log("Your backpack is full.");
                        continue;
                    }
                    idx = possible_indexes[0];
                    item_index
                        .insert(pickup.item, ItemIndex { index: idx })
                        .expect("Unable to insert ItemIndex entry");
                }
                gamelog.log(format!(
                    "You pick up the {} ({}).",
                    names.get(pickup.item).unwrap().name,
                    crate::gui::index_to_letter(idx)
                ));
            }
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");
        }
        wants_pickup.clear();
    }
}
