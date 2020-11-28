use crate::components::{Energy, InBackpack, ItemIndex, Position};
use crate::messages::{WantsToDropMessage, WantsToPickupMessage};
use crate::queues::{WantsToDropQueue, WantsToPickupQueue};
use crate::resources::{OutputQueue, PlayerEntity, PlayerPosition};
use ::bracket_lib::prelude::YELLOW;
use ::legion::systems::CommandBuffer;
use ::legion::world::SubWorld;
use ::legion::*;

#[system]
#[write_component(Energy)]
pub(crate) fn drop(
    world: &mut SubWorld,
    cb: &mut CommandBuffer,
    #[resource] player_position: &PlayerPosition,
    #[resource] player_entity: &PlayerEntity,
    #[resource] wants_to_drop_queue: &mut WantsToDropQueue,
    #[resource] output: &OutputQueue,
) {
    let player_position = player_position.0;
    let player_entity = player_entity.0;

    for WantsToDropMessage {
        who: dropper_entity,
        item,
    } in wants_to_drop_queue.try_iter()
    {
        // TODO: Use Dropper's position.
        cb.add_component(item, Position(player_position));
        cb.remove_component::<InBackpack>(item);
        if dropper_entity == player_entity {
            output
                .the(dropper_entity)
                .v(dropper_entity, "drop")
                .the(item);
        }
        let mut who_entity = world.entry_mut(dropper_entity).unwrap();
        who_entity.get_component_mut::<Energy>().unwrap().energy = -50;
    }
}

#[system]
#[write_component(Energy)]
#[write_component(InBackpack)]
#[write_component(ItemIndex)]
pub(crate) fn pickup(
    world: &mut SubWorld,
    cb: &mut CommandBuffer,
    #[resource] player_entity: &PlayerEntity,
    #[resource] wants_to_pickup_queue: &mut WantsToPickupQueue,
    #[resource] output: &OutputQueue,
) {
    let player_entity = player_entity.0;

    for WantsToPickupMessage {
        who: who_entity,
        item: item_entity,
    } in wants_to_pickup_queue.try_iter()
    {
        if who_entity == player_entity {
            let mut possible_indexes = std::collections::HashSet::new();
            for c in 0..52 {
                possible_indexes.insert(c);
            }
            for (item_idx, in_backpack) in <(&ItemIndex, &InBackpack)>::query().iter(world) {
                if in_backpack.owner == player_entity {
                    possible_indexes.remove(&item_idx.index);
                }
            }
            let mut possible_indexes: Vec<_> = possible_indexes.drain().collect();
            possible_indexes.sort_unstable();

            let mut idx = 255_u8;
            let item_entry = world.entry_mut(item_entity).unwrap();
            if let Ok(ItemIndex { index }) = item_entry.get_component::<ItemIndex>() {
                if possible_indexes.contains(index) {
                    idx = *index;
                }
            }
            if idx == 255_u8 {
                if possible_indexes.is_empty() {
                    output.s("Your backpack is full.");
                    continue;
                }
                idx = possible_indexes[0];
                cb.add_component(item_entity, ItemIndex { index: idx });
            }
            output
                .the(who_entity)
                .v(who_entity, "pick")
                .s("up")
                .a(item_entity)
                .color(YELLOW)
                .string(format!(" ({})", crate::gui::index_to_letter(idx)));
        } else {
            output
                .the(who_entity)
                .v(who_entity, "pick")
                .s("up")
                .a(item_entity);
        }

        cb.remove_component::<Position>(item_entity);
        cb.add_component(item_entity, InBackpack { owner: who_entity });

        let mut who_entity = world.entry_mut(who_entity).unwrap();
        who_entity.get_component_mut::<Energy>().unwrap().energy = -90;
    }
}
