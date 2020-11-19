use crate::components::{BlocksTile, Monster, Position};
use crate::map::Map;
use legion::{Entity, system};
use std::option::Option;

#[system]
pub(crate) fn map_indexing_clear(#[resource] map: &mut Map) {
    map.populate_blocked();
    map.clear_content_index();
}

#[system(for_each)]
pub(crate) fn map_indexing(
    entity: &Entity,
    _block: &BlocksTile,
    pos: &Position,
    monster: Option<&Monster>,
    #[resource] map: &mut Map,
) {
    let idx = map.pos_to_idx(*pos);
    map.blocked[idx] = true;
    map.tile_content[idx].push(*entity);
    map.dangerous[idx] = monster.is_some();
}
