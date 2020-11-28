use crate::components::{Energy, InBackpack, ItemIndex, Position, WantsToPickupItem};
use crate::ecs::Ecs;
use crate::messages::WantsToDropMessage;
use crate::queues::WantsToDropQueue;
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

// TODO: Make a proper system
pub(crate) fn pickup_system(ecs: &mut Ecs) {
    let player_entity = resource_get!(ecs, PlayerEntity).0;
    let output = resource_get!(ecs, OutputQueue);

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
        } else {
            output
                .the(who_entity)
                .v(who_entity, "pick")
                .s("up")
                .a(item_entity);
        }

        let mut item_entry = ecs.world.entry(item_entity).unwrap();
        item_entry.remove_component::<Position>();
        item_entry.add_component(InBackpack { owner: who_entity });
        let mut who_entity = ecs.world.entry(who_entity).unwrap();
        who_entity.get_component_mut::<Energy>().unwrap().energy = -90;
        who_entity.remove_component::<WantsToPickupItem>();
    }
}
