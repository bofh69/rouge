use super::map::Map;
use crate::components::{Monster, Player};
use crate::PlayerEntity;
use crate::{MapPosition, PlayerTarget, Position, Viewshed};
use bracket_lib::prelude::field_of_view;
use legion::*;

struct ViewshedPlayerUpdate(bool);

pub(crate) fn add_viewshed_system(ecs: &mut crate::Ecs, schedule_builder: &mut systems::Builder) {
    ecs.resources.insert(ViewshedPlayerUpdate(false));
    let system = SystemBuilder::new("Viewshed")
        .read_resource::<PlayerEntity>()
        .write_resource::<Map>()
        .write_resource::<ViewshedPlayerUpdate>()
        .with_query(<(Entity, Write<Viewshed>, Read<Position>)>::query())
        .build(
            move |_commands, world, (player_entity, map, viewshed_player_update), query| {
                **viewshed_player_update = ViewshedPlayerUpdate(false);
                for (ent, viewshed, pos) in query.iter_mut(world) {
                    if viewshed.dirty {
                        viewshed.visible_tiles.clear();

                        /* The points here are in map space */
                        viewshed.visible_tiles =
                            field_of_view(pos.0.into(), viewshed.range, &**map)
                                .iter()
                                .filter(|p| {
                                    p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height
                                })
                                .map(|p| MapPosition { x: p.x, y: p.y })
                                .collect();

                        // If this is the player, reveal what they can see
                        if *ent == player_entity.0 {
                            **viewshed_player_update = ViewshedPlayerUpdate(true);
                        }
                        viewshed.dirty = false;
                    }
                }
            },
        );
    schedule_builder.add_system(system);

    let system = SystemBuilder::new("Viewshed part 2")
        .read_resource::<ViewshedPlayerUpdate>()
        .read_resource::<PlayerEntity>()
        .write_resource::<Map>()
        .write_resource::<PlayerTarget>()
        .with_query(<(Write<Viewshed>, Read<Player>)>::query())
        .build(
            move |_commands,
                  world,
                  (viewshed_player_update, player_entity, map, player_target),
                  query| {
                if viewshed_player_update.0 {
                    // let mut clear_target = false;
                    {
                        for vt in map.visible_tiles.iter_mut() {
                            *vt = false;
                        }

                        for (viewshed, _player) in query.iter_mut(world) {
                            let mut visible_objects = Vec::new();
                            for vis in viewshed.visible_tiles.iter() {
                                let idx = map.map_pos_to_idx(*vis);
                                map.revealed_tiles[idx] = true;
                                map.visible_tiles[idx] = true;
                                if PlayerTarget::None != **player_target {
                                    for ent in map.tile_content[idx].iter() {
                                        visible_objects.push(*ent);
                                    }
                                }
                            }
                        }

                        // TODO Fix clear_target somehow.
                        // Add stop_target value to map?

                        /*
                        for ent in visible_objects.iter() {
                            if let Some(tile_content_entry) = world.entry(*ent) {
                                if tile_content_entry.get_component::<Monster>().is_ok() {
                                    clear_target = true;
                                    break;
                                }
                            }
                        }
                        */
                    }
                    /*
                    if clear_target {
                        **player_target = PlayerTarget::None;
                    }
                    */
                }
            },
        );
    schedule_builder.add_system(system);
}

/*
#[system]
#[read_component(Position)]
#[write_component(Viewshed)]
pub fn viewshed(world: &mut legion::world::SubWorld,
    #[resource] player_entity: &PlayerEntity,
    #[resource] map: &mut Map,
    #[resource] player_target: &mut PlayerTarget
) {

    let player_entity = player_entity.0;
    let mut update_player = false;

    // TODO: Clean up!
    {
        let mut query = <(Entity, &mut Viewshed, &Position)>::query();
        for (ent, viewshed, pos) in query.iter_mut(&mut world) {
            if viewshed.dirty {
                viewshed.visible_tiles.clear();

                /* The points here are in map space */
                viewshed.visible_tiles = field_of_view(pos.0.into(), viewshed.range, &*map)
                    .iter()
                    .filter(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height)
                    .map(|p| MapPosition { x: p.x, y: p.y })
                    .collect();

                // If this is the player, reveal what they can see
                if *ent == player_entity {
                    update_player = true;
                }
                viewshed.dirty = false;
            }
        }
    }

    if update_player {
        let mut clear_target = false;
        {
            for vt in map.visible_tiles.iter_mut() {
                *vt = false;
            }

            let viewshed = ecs
                .ecs
                .entry(player_entity)
                .unwrap()
                .into_component::<Viewshed>()
                .unwrap();

            let mut visible_objects = Vec::new();
            for vis in viewshed.visible_tiles.iter() {
                let idx = map.map_pos_to_idx(*vis);
                map.revealed_tiles[idx] = true;
                map.visible_tiles[idx] = true;
                if PlayerTarget::None != player_target {
                    for ent in map.tile_content[idx].iter() {
                        visible_objects.push(*ent);
                    }
                }
            }

            for ent in visible_objects.iter() {
                if let Some(tile_content_entry) = world.entry(*ent) {
                    if tile_content_entry.get_component::<Monster>().is_ok() {
                        clear_target = true;
                        break;
                    }
                }
            }
        }
        if clear_target {
            *player_target = PlayerTarget::None;
        }
    }
}

*/
