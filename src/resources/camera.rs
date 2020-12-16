use crate::MapPosition;
use ::serde::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub(crate) struct Camera {
    pub w: i32,
    pub h: i32,

    pub offset: MapPosition,
    pub sub_tile_offset: (f32, f32),
    pub old_player_pos: MapPosition,
}
