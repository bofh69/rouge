use crate::components::{
    Energy, InBackpack, ItemIndex, Position, WantsToDropItem, WantsToPickupItem,
};
use crate::ecs::Ecs;
use crate::resources::{OutputQueue, PlayerEntity, PlayerPosition};
use ::bracket_lib::prelude::YELLOW;
use ::legion::{Entity, IntoQuery};

// TODO: Make a proper system
pub(crate) fn drop_system(ecs: &mut Ecs) {
    let player_position = resource_get!(ecs, PlayerPosition).0;
    let player_entity = resource_get!(ecs, PlayerEntity).0;

    let things_to_drop: Vec<_> = <(Entity, &WantsToDropItem)>::query()
        .iter(&ecs.world)
        .map(|(entity, drop)| (*entity, drop.item))
        .collect();

    for (dropper_entity, item) in things_to_drop {
        let mut entry = ecs.world.entry(item).unwrap();
        entry.add_component(Position(player_position));
        entry.remove_component::<InBackpack>();
        if dropper_entity == player_entity {
            let mut output = resource_get_mut!(ecs, OutputQueue);
            output
                .the(dropper_entity)
                .v(dropper_entity, "drop")
                .the(item);
        }
        let mut who_entity = ecs.world.entry(dropper_entity).unwrap();
        who_entity.get_component_mut::<Energy>().unwrap().energy = -50;
        who_entity.remove_component::<WantsToDropItem>();
    }
}

// TODO: Make a proper system
pub(crate) fn pickup_system(ecs: &mut Ecs) {
    let player_entity = resource_get!(ecs, PlayerEntity).0;
    let mut output = resource_get_mut!(ecs, OutputQueue);

    let things_to_pickup: Vec<_> = <&WantsToPickupItem>::query()
        .iter(&ecs.world)
        .map(|pickup| (pickup.collected_by, pickup.item))
        .collect();

    for (who_entity, item_entity) in things_to_pickup {
        if who_entity == player_entity {
            let mut possible_indexes = std::collections::HashSet::new();
            for c in 0..52 {
                possible_indexes.insert(c);
            }
            for (item_idx, in_backpack) in <(&ItemIndex, &InBackpack)>::query().iter(&ecs.world) {
                if in_backpack.owner == player_entity {
                    possible_indexes.remove(&item_idx.index);
                }
            }
            let mut possible_indexes: Vec<_> = possible_indexes.drain().collect();
            possible_indexes.sort_unstable();

            let mut idx = 255_u8;
            if let Ok(ItemIndex { index }) = ecs
                .world
                .entry(item_entity)
                .unwrap()
                .get_component::<ItemIndex>()
            {
                if possible_indexes.contains(index) {
                    idx = *index;
                }
            }
            let mut item_entry = ecs.world.entry(item_entity).unwrap();
            if idx == 255_u8 {
                if possible_indexes.is_empty() {
                    output.s("Your backpack is full.");
                    continue;
                }
                idx = possible_indexes[0];
                item_entry.add_component(ItemIndex { index: idx });
            }
            output
                .the(who_entity)
                .v(who_entity, "pick")
                .s("up")
                .a(item_entity)
                .color(YELLOW)
                .string(format!(" ({})", crate::gui::index_to_letter(idx)));
        }

        let mut item_entry = ecs.world.entry(item_entity).unwrap();
        item_entry.remove_component::<Position>();
        item_entry.add_component(InBackpack { owner: who_entity });
        let mut who_entity = ecs.world.entry(who_entity).unwrap();
        who_entity.get_component_mut::<Energy>().unwrap().energy = -90;
        who_entity.remove_component::<WantsToPickupItem>();
    }
}
