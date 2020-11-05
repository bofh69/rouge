use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::{MapPosition, PlayerPosition, ScreenPosition};
use legion::*;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Camera {
    w: i32,
    h: i32,

    offset: MapPosition,
    sub_tile_offset: (f32, f32),
    old_player_pos: MapPosition,
}

fn diff_to_interval(v: i32, min: i32, max: i32) -> i32 {
    if v < min {
        min - v
    } else if v > max {
        max - v
    } else {
        0
    }
}

#[system]
pub(crate) fn camera_update(
    #[resource] camera: &mut Camera,
    #[resource] player_position: &PlayerPosition,
) {
    let pos: MapPosition = player_position.0;

    if !camera.is_in_view(pos) {
        camera.center(*player_position);
    } else if camera.old_player_pos != pos {
        let screen_pos = pos - camera.offset;
        let (dx, dy);
        dx = diff_to_interval(screen_pos.x, camera.w / 3, 2 * camera.w / 3);
        dy = diff_to_interval(screen_pos.y, camera.h / 3, 2 * camera.h / 3);

        camera.move_view(-dx, -dy);

        camera.old_player_pos = pos;
    }
}

impl Camera {
    pub fn new(pos: PlayerPosition, width: i32, height: i32) -> Self {
        let mut camera = Self {
            w: width,
            h: height,
            offset: MapPosition { x: -1, y: -1 },
            sub_tile_offset: (0.0, 0.0),
            old_player_pos: MapPosition { x: -1, y: -1 },
        };
        camera.center(pos);
        camera
    }

    // Hard jump to new position
    pub fn center(&mut self, pos: PlayerPosition) {
        self.old_player_pos = pos.0;
        self.sub_tile_offset = (0.0, 0.0);

        let (x, y) = (
            i32::min(MAP_WIDTH - self.w, i32::max(0, (pos.0).x - self.w / 2)),
            i32::min(MAP_HEIGHT - self.h, i32::max(0, (pos.0).y - self.h / 2)),
        );

        self.offset = MapPosition { x, y };
    }

    pub fn move_view(&mut self, dx: i32, dy: i32) {
        let (x, y) = (
            i32::min(MAP_WIDTH - self.w, i32::max(0, self.offset.x + dx)),
            i32::min(MAP_HEIGHT - self.h, i32::max(0, self.offset.y + dy)),
        );

        self.offset = MapPosition { x, y };
    }

    pub fn transform_screen_pos(&self, p: ScreenPosition) -> MapPosition {
        MapPosition {
            x: p.x + self.offset.x,
            y: p.y + self.offset.y,
        }
    }

    pub fn transform_map_pos(&self, p: MapPosition) -> ScreenPosition {
        ScreenPosition {
            x: p.x - self.offset.x,
            y: p.y - self.offset.y,
        }
    }

    pub fn is_in_view(&self, p: MapPosition) -> bool {
        !(p.x < self.offset.x
            || p.x >= (self.offset.x + self.w)
            || p.y < self.offset.y
            || p.y >= (self.offset.y + self.h))
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }
}
