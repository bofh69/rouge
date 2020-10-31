use crate::components::{BlocksTile, Position};
use crate::map::Map;
use legion::*;

#[system]
pub(crate) fn map_indexing_prepare(#[resource] map: &mut Map) {
    map.populate_blocked();
    map.clear_content_index();
}

#[system(for_each)]
pub(crate) fn map_indexing(
    entity: &Entity,
    _block: &BlocksTile,
    pos: &Position,
    #[resource] map: &mut Map,
) {
    let idx = map.pos_to_idx(*pos);
    map.blocked[idx] = true;
    map.tile_content[idx].push(*entity);
}
