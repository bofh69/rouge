use crate::components::{Player, Position, Viewshed};
use crate::positions::MapPosition;
use crate::resources::{Map, PlayerEntity, PlayerTarget};
use ::bracket_lib::prelude::field_of_view;
use ::legion::world::SubWorld;
use ::legion::*;

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
                                .filter_map(|p| {
                                    if p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height {
                                        Some(MapPosition { x: p.x, y: p.y })
                                    } else {
                                        None
                                    }
                                })
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

    schedule_builder.add_system(update_map_for_player_system());
}

#[system]
#[read_component(Player)]
#[write_component(Viewshed)]
fn update_map_for_player(
    world: &mut SubWorld,
    #[resource] player_update: &ViewshedPlayerUpdate,
    #[resource] map: &mut Map,
    #[resource] player_target: &mut PlayerTarget,
) {
    if player_update.0 {
        for vt in &mut map.visible_tiles {
            *vt = false;
        }

        for (viewshed, _player) in <(&mut Viewshed, &Player)>::query().iter_mut(world) {
            for vis in &viewshed.visible_tiles {
                let idx = map.map_pos_to_idx(*vis);
                map.revealed_tiles[idx] = true;
                map.visible_tiles[idx] = true;
                if map.dangerous[idx] {
                    *player_target = PlayerTarget::None;
                }
            }
        }
    }
}
