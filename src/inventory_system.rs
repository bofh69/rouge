use crate::gamelog::GameLog;
use crate::{
    components::{InBackpack, ItemIndex, Name, Position, WantsToDropItem, WantsToPickupItem},
    Ecs,
};
use crate::{PlayerEntity, PlayerPosition};
use legion::*;

pub fn drop_system(ecs: &mut Ecs) {
    let player_position = ecs.resources.get::<PlayerPosition>().unwrap().0;
    let player_entity = ecs.resources.get::<PlayerEntity>().unwrap().0;

    let things_to_drop: Vec<_> = <(Entity, &WantsToDropItem)>::query()
        .iter(&ecs.ecs)
        .map(|(entity, drop)| (*entity, drop.item))
        .collect();

    for (dropper_entity, item) in things_to_drop {
        let mut entry = ecs.ecs.entry(item).unwrap();
        entry.add_component(Position(player_position));
        entry.remove_component::<InBackpack>();
        if dropper_entity == player_entity {
            let mut gamelog = ecs.resources.get_mut::<GameLog>().unwrap();
            gamelog.log(format!(
                "You drop the {}.",
                entry
                    .get_component::<Name>()
                    .map_or("unknown item", |n| &n.name)
            ));
        }
        ecs.ecs
            .entry(dropper_entity)
            .unwrap()
            .remove_component::<WantsToDropItem>();
    }
}

pub fn pickup_system(ecs: &mut Ecs) {
    let player_entity = ecs.resources.get::<PlayerEntity>().unwrap().0;
    let mut gamelog = ecs.resources.get_mut::<GameLog>().unwrap();

    let things_to_pickup: Vec<_> = <&WantsToPickupItem>::query()
        .iter(&ecs.ecs)
        .map(|pickup| (pickup.collected_by, pickup.item))
        .collect();

    for (who_entity, item_entity) in things_to_pickup {
        if who_entity == player_entity {
            let mut possible_indexes = std::collections::HashSet::new();
            for c in 0..52 {
                possible_indexes.insert(c);
            }
            for (item_idx, in_backpack) in <(&ItemIndex, &InBackpack)>::query().iter(&ecs.ecs) {
                if in_backpack.owner == player_entity {
                    possible_indexes.remove(&item_idx.index);
                }
            }
            let mut possible_indexes: Vec<_> = possible_indexes.drain().collect();
            possible_indexes.sort_unstable();

            let mut idx = 255u8;
            if let Ok(ItemIndex { index }) = ecs
                .ecs
                .entry(item_entity)
                .unwrap()
                .get_component::<ItemIndex>()
            {
                if possible_indexes.contains(index) {
                    idx = *index;
                }
            }
            let mut item_entry = ecs.ecs.entry(item_entity).unwrap();
            if idx == 255u8 {
                if possible_indexes.is_empty() {
                    gamelog.log("Your backpack is full.");
                    continue;
                }
                idx = possible_indexes[0];
                item_entry.add_component(ItemIndex { index: idx });
            }
            gamelog.log(format!(
                "You pick up the {} ({}).",
                item_entry.get_component::<Name>().unwrap().name,
                crate::gui::index_to_letter(idx)
            ));
        }

        let mut item_entry = ecs.ecs.entry(item_entity).unwrap();
        item_entry.remove_component::<Position>();
        item_entry.add_component(InBackpack { owner: who_entity });
        ecs.ecs
            .entry(who_entity)
            .unwrap()
            .remove_component::<WantsToPickupItem>();
    }
}
