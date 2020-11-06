use super::map::Map;
use crate::components::Player;
use crate::PlayerEntity;
use crate::{MapPosition, PlayerTarget, Position, Viewshed};
use bracket_lib::prelude::field_of_view;
use legion::*;

struct ViewshedPlayerUpdate(bool);

pub(crate) fn add_viewshed_system(
    ecs: &mut crate::ecs::Ecs,
    schedule_builder: &mut systems::Builder,
) {
    ecs.resources.insert(ViewshedPlayerUpdate(false));

    let system = SystemBuilder::new("Viewshed")
        .read_resource::<PlayerEntity>()
        .write_resource::<Map>()
        .write_resource::<ViewshedPlayerUpdate>()
        .with_query(<(Entity, Write<Viewshed>, Read<Position>)>::query())
        .build(
            move |_commands, world, (player_entity, map, viewshed_player_update), query| {
                viewshed_player_update.0 = false;
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
                            viewshed_player_update.0 = true;
                        }
                        viewshed.dirty = false;
                    }
                }
            },
        );
    schedule_builder.add_system(system);

    let system = SystemBuilder::new("Update Map")
        .read_resource::<ViewshedPlayerUpdate>()
        .write_resource::<Map>()
        .write_resource::<PlayerTarget>()
        .with_query(<(Write<Viewshed>, Read<Player>)>::query())
        .build(
            move |_commands, world, (viewshed_player_update, map, player_target), query| {
                if viewshed_player_update.0 {
                    for vt in map.visible_tiles.iter_mut() {
                        *vt = false;
                    }

                    for (viewshed, _player) in query.iter_mut(world) {
                        for vis in viewshed.visible_tiles.iter() {
                            let idx = map.map_pos_to_idx(*vis);
                            map.revealed_tiles[idx] = true;
                            map.visible_tiles[idx] = true;
                            if map.dangerous[idx] {
                                **player_target = PlayerTarget::None;
                            }
                        }
                    }
                }
            },
        );
    schedule_builder.add_system(system);
}
