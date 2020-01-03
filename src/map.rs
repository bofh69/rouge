use crate::camera::Camera;
use crate::components::Position;
use crate::rect::Rect;
use crate::MapPosition;
use crate::ScreenPosition;
use rltk::{Algorithm2D, BaseMap, Console, Point, Rltk, RGB};
use specs::prelude::*;
use std::cmp::{max, min};

pub const MAP_WIDTH: i32 = 120;
pub const MAP_HEIGHT: i32 = 60;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum WallType {
    Vertical,          /* - */
    Horizontal,        /* | */
    TopLeftCorner,     /* ┌ */
    TopRightCorner,    /* ┐ */
    BottomLeftCorner,  /* └ */
    BottomRightCorner, /* ┘ */
    TeeDown,           /* T */
    TeeUp,             /* ┴ */
    TeeLeft,           /* ├ */
    TeeRight,          /* ┤ */
    Cross,             /* + */
    Pilar,             /* ● */
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Stone,
    Wall(WallType),
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: i32) -> bool {
        match self.tiles[idx as usize] {
            TileType::Wall(_) => true,
            TileType::Stone => true,
            _ => false,
        }
    }

    fn get_available_exits(&self, idx: i32) -> Vec<(i32, f32)> {
        let mut exits: Vec<(i32, f32)> = Vec::new();
        let x = idx % self.width;
        let y = idx / self.width;

        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        }
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        }
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - self.width, 1.0))
        }
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + self.width, 1.0))
        }
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - self.width) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - self.width) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + self.width) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + self.width) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: i32, idx2: i32) -> f32 {
        let p1 = Point::new(idx1 % self.width, idx1 / self.width);
        let p2 = Point::new(idx2 % self.width, idx2 / self.width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn in_bounds(&self, pos: Point) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    fn point2d_to_index(&self, pt: Point) -> i32 {
        (pt.y * self.width) + pt.x
    }

    fn index_to_point2d(&self, idx: i32) -> Point {
        Point {
            x: idx % self.width,
            y: idx / self.width,
        }
    }
}

impl Map {
    pub fn xy_to_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn pos_to_idx(&self, pos: Position) -> usize {
        (pos.0.y * self.width + pos.0.x) as usize
    }

    pub fn map_pos_to_idx(&self, pos: MapPosition) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_to_idx(x, y);
        !self.blocked[idx]
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_to_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_to_idx(x, y);
            if idx > 0 && idx < (self.width * self.height) as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_to_idx(x, y);
            if idx > 0 && idx < (self.width * self.height) as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = match *tile {
                TileType::Wall(_) => true,
                TileType::Stone => true,
                _ => false,
            }
        }
    }

    pub fn is_solid(&self, pos: Point) -> bool {
        if !self.in_bounds(pos) {
            return true;
        }
        let idx = self.point2d_to_index(pos);
        match self.tiles[idx as usize] {
            TileType::Wall(_) => true,
            TileType::Stone => true,
            _ => false,
        }
    }

    // Create points surrounding (x, y)
    fn points_around(x: i32, y: i32) -> Vec<Point> {
        vec![
            Point::new(x - 1, y - 1),
            Point::new(x, y - 1),
            Point::new(x + 1, y - 1),
            Point::new(x - 1, y),
            Point::new(x + 1, y),
            Point::new(x - 1, y + 1),
            Point::new(x, y + 1),
            Point::new(x + 1, y + 1),
        ]
    }

    fn wall_continues(&self, pos: Point, dx: i32, dy: i32) -> bool {
        let new_pos = Point::new(pos.x + dx, pos.y + dy);
        if !self.is_solid(new_pos) {
            return false;
        }
        if dx != 0 {
            if !self.is_solid(Point::new(pos.x + dx, pos.y - 1))
                || !self.is_solid(Point::new(pos.x + dx, pos.y))
                || !self.is_solid(Point::new(pos.x + dx, pos.y + 1))
                || !self.is_solid(Point::new(pos.x, pos.y - 1))
                || !self.is_solid(Point::new(pos.x, pos.y + 1))
            {
                return true;
            }
        } else {
            if !self.is_solid(Point::new(pos.x - 1, pos.y + dy))
                || !self.is_solid(Point::new(pos.x, pos.y + dy))
                || !self.is_solid(Point::new(pos.x + 1, pos.y + dy))
                || !self.is_solid(Point::new(pos.x - 1, pos.y))
                || !self.is_solid(Point::new(pos.x + 1, pos.y))
            {
                return true;
            }
        }
        return false;
    }

    fn fix_walls(&mut self) {
        /* Remove single walls completely surrounded */
        /* Change single walls completely lonely */
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Point::new(x, y);
                if self.is_solid(pos) {
                    let idx = self.point2d_to_index(pos) as usize;
                    let count_walls = Self::points_around(x, y)
                        .iter()
                        .filter(|p| !self.is_solid(**p))
                        .count();
                    if count_walls == 0 {
                        self.tiles[idx] = TileType::Stone;
                    } else if count_walls == 8 {
                        self.tiles[idx] = TileType::Wall(WallType::Pilar);
                    }
                }
            }
        }
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Point::new(x, y);
                let idx = self.xy_to_idx(x, y);
                if let TileType::Wall(_) = self.tiles[idx] {
                    let mut walls = 0;
                    if self.wall_continues(pos, -1, 0) {
                        walls += 1;
                    }
                    if self.wall_continues(pos, 1, 0) {
                        walls += 2;
                    }
                    if self.wall_continues(pos, 0, -1) {
                        walls += 4;
                    }
                    if self.wall_continues(pos, 0, 1) {
                        walls += 8;
                    }
                    let walltype = match walls {
                        0 => WallType::Pilar,
                        1 => WallType::Horizontal,
                        2 => WallType::Horizontal,
                        3 => WallType::Horizontal,
                        4 => WallType::Vertical,
                        5 => WallType::BottomRightCorner,
                        6 => WallType::BottomLeftCorner,
                        7 => WallType::TeeUp,
                        8 => WallType::Vertical,
                        9 => WallType::TopRightCorner,
                        10 => WallType::TopLeftCorner,
                        11 => WallType::TeeDown,
                        12 => WallType::Vertical,
                        13 => WallType::TeeLeft,
                        14 => WallType::TeeRight,
                        15 => WallType::Cross,
                        _ => unreachable!(),
                    };
                    self.tiles[idx] = TileType::Wall(walltype);
                }
            }
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn new(width: i32, height: i32) -> Map {
        let size = (width * height) as usize;
        let tiles = vec![TileType::Wall(WallType::Cross); size];
        let rooms: Vec<Rect> = Vec::new();

        Map {
            tiles,
            rooms,
            width,
            height,
            revealed_tiles: vec![false; size],
            visible_tiles: vec![false; size],
            blocked: vec![false; size],
            tile_content: vec![vec![]; size],
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);

        let mut rng = rltk::RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room);
            }
        }

        map.fix_walls();

        map.populate_blocked();

        map
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let camera = ecs.fetch::<Camera>();

    for y in 0..camera.height() {
        for x in 0..camera.width() {
            let point = ScreenPosition { x, y };
            let pos = camera.transform_screen_pos(point);
            if pos.x < 0 || pos.x >= map.width || pos.y < 0 || pos.y >= map.height {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0., 0., 0.),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('¿'),
                );
            } else {
                let idx = map.map_pos_to_idx(pos);
                let tile = map.tiles[idx];
                // Render a tile depending upon the tile type
                if map.revealed_tiles[idx] {
                    let glyph;
                    let mut fg;
                    match tile {
                        TileType::Floor => {
                            fg = RGB::from_f32(0.5, 0.5, 0.5);
                            glyph = rltk::to_cp437('.');
                        }
                        TileType::Wall(WallType::Vertical) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('│');
                        }
                        TileType::Wall(WallType::Horizontal) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('─');
                        }
                        TileType::Wall(WallType::TopLeftCorner) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('┌');
                        }
                        TileType::Wall(WallType::TopRightCorner) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('┐');
                        }
                        TileType::Wall(WallType::BottomLeftCorner) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('└');
                        }
                        TileType::Wall(WallType::BottomRightCorner) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('┘');
                        }
                        TileType::Wall(WallType::TeeDown) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            if y < map.height - 1 && map.revealed_tiles[idx + map.width as usize] {
                                glyph = rltk::to_cp437('┬');
                            } else {
                                glyph = rltk::to_cp437('─');
                            }
                        }
                        TileType::Wall(WallType::TeeUp) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            if y > 0 && map.revealed_tiles[idx - map.width as usize] {
                                glyph = rltk::to_cp437('┴');
                            } else {
                                glyph = rltk::to_cp437('─');
                            }
                        }
                        TileType::Wall(WallType::TeeRight) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            if x < map.width - 1 && map.revealed_tiles[idx + 1] {
                                glyph = rltk::to_cp437('├');
                            } else {
                                glyph = rltk::to_cp437('│');
                            }
                        }
                        TileType::Wall(WallType::TeeLeft) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            if x > 0 && map.revealed_tiles[idx - 1] {
                                glyph = rltk::to_cp437('┤');
                            } else {
                                glyph = rltk::to_cp437('│');
                            }
                        }
                        TileType::Wall(WallType::Cross) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            // TODO: What walls have been seen?
                            glyph = rltk::to_cp437('┼');
                        }
                        TileType::Wall(WallType::Pilar) => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = 9;
                        }
                        TileType::Stone => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = rltk::to_cp437('#');
                        }
                    }
                    if !map.visible_tiles[idx] {
                        fg = fg.to_greyscale();
                    }
                    ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn fix_walls() {
        use super::*;
        let mut map = Map::new(5, 5);
        map.fix_walls();
        assert_eq!(map.tiles, vec![TileType::Stone; 5 * 5]);
        for tile in map.tiles.iter_mut() {
            *tile = TileType::Floor;
        }
        /* .....
         * ...--
         * .---
         * .|...
         * .....
         */
        map.tiles[2 * 5 + 3] = TileType::Wall(WallType::Cross);
        map.tiles[2 * 5 + 4] = TileType::Wall(WallType::Cross);
        map.tiles[3 * 5 + 1] = TileType::Wall(WallType::Cross);
        map.tiles[3 * 5 + 2] = TileType::Wall(WallType::Cross);
        map.tiles[3 * 5 + 3] = TileType::Wall(WallType::Cross);
        map.tiles[3 * 5 + 4] = TileType::Wall(WallType::Cross);
        map.tiles[4 * 5 + 1] = TileType::Wall(WallType::Cross);
        map.fix_walls();
        assert_eq!(
            map.tiles[2 * 5 + 3],
            TileType::Wall(WallType::TopLeftCorner)
        );
        assert_eq!(map.tiles[2 * 5 + 4], TileType::Wall(WallType::Horizontal));
        assert_eq!(
            map.tiles[3 * 5 + 1],
            TileType::Wall(WallType::TopLeftCorner)
        );
        assert_eq!(map.tiles[3 * 5 + 2], TileType::Wall(WallType::Horizontal));
        assert_eq!(map.tiles[3 * 5 + 3], TileType::Wall(WallType::TeeUp));
        assert_eq!(map.tiles[4 * 5 + 1], TileType::Wall(WallType::Vertical));
    }
}
