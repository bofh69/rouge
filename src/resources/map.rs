use crate::components::Position;
use crate::ecs::Ecs;
use crate::positions::{MapPosition, ScreenPosition};
use crate::resources::Camera;
use crate::Direction;
use ::bracket_lib::prelude::*;
use ::legion::*;
use ::serde::*;
use std::cmp::{max, min};

pub(crate) const MAP_WIDTH: i32 = 120;
pub(crate) const MAP_HEIGHT: i32 = 60;

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
pub(crate) enum WallType {
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

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
pub(crate) enum TileType {
    Stone,
    Wall(WallType),
    Floor,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    /// Does the tile contain anything that seems dangerous?
    pub dangerous: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
    only_revealed: bool,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        matches!(self.tiles[idx], TileType::Wall(_) | TileType::Stone)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits: SmallVec<[(usize, f32); 10]> = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;

        let pos = MapPosition { x, y };

        for dir in Direction::iter() {
            let pos = pos + *dir;
            if self.is_exit_valid(pos.x, pos.y) {
                let (dx, dy): (i32, i32) = (*dir).into();
                if dx * dy == 0 {
                    exits.push((self.map_pos_to_idx(pos), 1.0));
                } else {
                    exits.push((self.map_pos_to_idx(pos), 1.01));
                }
            }
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = Point::new(idx1 as i32 % self.width, idx1 as i32 / self.width);
        let p2 = Point::new(idx2 as i32 % self.width, idx2 as i32 / self.width);
        DistanceAlg::Chebyshev.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point {
            x: self.width,
            y: self.height,
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

    pub fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || x > self.width - 1 || y < 0 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_to_idx(x, y);
        if self.only_revealed && !self.revealed_tiles[idx] {
            return false;
        }
        !self.blocked[idx]
    }

    pub fn search_only_revealed(&mut self) {
        self.only_revealed = true
    }

    pub fn search_also_revealed(&mut self) {
        self.only_revealed = false
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
            self.blocked[i] = matches!(*tile, TileType::Wall(_) | TileType::Stone);
        }
    }

    pub fn is_solid(&self, pos: Point) -> bool {
        if !self.in_bounds(pos) {
            return true;
        }
        let idx = self.point2d_to_index(pos);
        matches!(self.tiles[idx], TileType::Wall(_) | TileType::Stone)
    }

    pub fn is_visible(&self, pos: MapPosition) -> bool {
        let idx = self.map_pos_to_idx(pos);
        self.visible_tiles[idx]
    }

    // Create points surrounding (x, y)
    fn points_around(x: i32, y: i32) -> Vec<Point> {
        vec![
            Point::new(x - 1, y - 1),
            Point::new(x, y - 1),
            Point::new(x + 1, y - 1),
            Point::new(x + 1, y),
            Point::new(x + 1, y + 1),
            Point::new(x, y + 1),
            Point::new(x - 1, y + 1),
            Point::new(x - 1, y),
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
        } else if !self.is_solid(Point::new(pos.x - 1, pos.y + dy))
            || !self.is_solid(Point::new(pos.x, pos.y + dy))
            || !self.is_solid(Point::new(pos.x + 1, pos.y + dy))
            || !self.is_solid(Point::new(pos.x - 1, pos.y))
            || !self.is_solid(Point::new(pos.x + 1, pos.y))
        {
            return true;
        }
        false
    }

    fn fix_walls(&mut self) {
        /* Remove single walls completely surrounded */
        /* Change single walls completely lonely */
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Point::new(x, y);
                if self.is_solid(pos) {
                    let idx = self.point2d_to_index(pos);
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
            dangerous: vec![false; size],
            tile_content: vec![vec![]; size],
            only_revealed: false,
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);
                if !map.rooms.is_empty() {
                    let new_pos = new_room.center();
                    let prev_pos = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_pos.x, new_pos.x, prev_pos.y);
                        map.apply_vertical_tunnel(prev_pos.y, new_pos.y, new_pos.x);
                    } else {
                        map.apply_vertical_tunnel(prev_pos.y, new_pos.y, prev_pos.x);
                        map.apply_horizontal_tunnel(prev_pos.x, new_pos.x, new_pos.y);
                    }
                }
                map.rooms.push(new_room);
            }
        }

        map.fix_walls();

        map.populate_blocked();

        map
    }

    pub(crate) fn find_closest_unknown(&self, pos: MapPosition) -> Option<MapPosition> {
        // TODO: Change to breadth first search, then return the position,
        // use autowalk to get there.

        let mut visited: Vec<bool> = Vec::with_capacity((self.height * self.width) as usize);
        for _y in 0..=self.height - 1 {
            for _x in 0..=self.width - 1 {
                visited.push(false);
            }
        }

        let mut expand = std::collections::VecDeque::new();
        expand.push_back(pos);

        while let Some(curr_pos) = expand.pop_front() {
            let idx = self.pos_to_idx(curr_pos.into());
            if !visited[idx] {
                visited[idx] = true;
                for dir in Direction::iter() {
                    let pos = curr_pos + *dir;
                    let idx = self.xy_to_idx(pos.x, pos.y);
                    if !self.is_solid(pos.into()) && !self.revealed_tiles[idx] {
                        return curr_pos.into();
                    } else if !self.is_solid(pos.into()) {
                        expand.push_back(pos);
                    }
                }
            }
        }

        None

        /*
        let mut starts = vec![];
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = self.xy_to_idx(x, y);
                let pos = Point::new(x, y);
                if !self.is_solid(pos) && !self.revealed_tiles[idx] {
                    for dir in Direction::iter() {
                        let pos = pos + *dir;
                        let idx = self.xy_to_idx(pos.x, pos.y);
                        if self.revealed_tiles[idx] && !self.is_solid(pos) {
                            starts.push(idx);
                            break;
                        }
                    }
                }
            }
        }

        let dijkstra_map =
            bracket_lib::prelude::DijkstraMap::new(self.width, self.height, &starts, self, 80.);

        let mut result = Direction::North;
        let mut min = f32::MAX;
        for dir in Direction::iter() {
            let pos = pos + *dir;
            if !self.is_solid(pos.into()) {
                let idx = self.xy_to_idx(pos.x, pos.y);
                let val = dijkstra_map.map[idx];

                if val < min {
                    result = *dir;
                    min = val;
                }
            }
        }
        result
        */
    }
}

fn get_glyph_for_wall(map: &Map, idx: usize, x: i32, y: i32, walltype: WallType) -> u16 {
    let mut walls = 0;

    if x > 0 && map.revealed_tiles[idx - 1_usize] {
        walls += 1 // Left
    }
    if x < map.width - 2 && map.revealed_tiles[idx + 1_usize] {
        walls += 2 // Right
    }
    if y > 0 && map.revealed_tiles[idx - map.width as usize] {
        walls += 4 // up
    }
    if y < map.height - 2 && map.revealed_tiles[idx + map.width as usize] {
        walls += 8 // Down
    }

    match walltype {
        WallType::Vertical => to_cp437('│'),
        WallType::Horizontal => to_cp437('─'),
        WallType::TopLeftCorner => to_cp437('┌'),
        WallType::TopRightCorner => to_cp437('┐'),
        WallType::BottomLeftCorner => to_cp437('└'),
        WallType::BottomRightCorner => to_cp437('┘'),
        WallType::TeeDown => match walls & (1 + 2 + 8) {
            9 => to_cp437('┐'),
            10 => to_cp437('┌'),
            11 => to_cp437('┬'),
            _ => to_cp437('─'),
        },
        WallType::TeeUp => match walls & (1 + 2 + 4) {
            5 => to_cp437('┘'),
            6 => to_cp437('└'),
            7 => to_cp437('┴'),
            _ => to_cp437('─'),
        },
        WallType::TeeRight => match walls & (2 + 4 + 8) {
            6 => to_cp437('└'),
            10 => to_cp437('┌'),
            14 => to_cp437('├'),
            _ => to_cp437('│'),
        },
        WallType::TeeLeft => match walls & (1 + 4 + 8) {
            5 => to_cp437('┘'),
            9 => to_cp437('┐'),
            13 => to_cp437('┤'),
            _ => to_cp437('│'),
        },
        WallType::Cross => match walls & (1 + 2 + 4 + 8) {
            4 => to_cp437('┴'),
            5 => to_cp437('┘'),
            6 => to_cp437('└'),
            7 => to_cp437('┴'),
            8 => to_cp437('┬'),
            9 => to_cp437('┐'),
            10 => to_cp437('┌'),
            11 => to_cp437('┬'),
            12 => to_cp437('│'),
            13 => to_cp437('┤'),
            14 => to_cp437('├'),
            15 => to_cp437('┼'),
            _ => to_cp437('─'),
        },
        WallType::Pilar => 9, /* o */
    }
}

pub(crate) fn draw_map(ecs: &Ecs, ctx: &mut BTerm) {
    let map = resource_get!(ecs, Map);
    let camera = resource_get!(ecs, Camera);

    for y in 0..camera.height() {
        for x in 0..camera.width() {
            let point = ScreenPosition { x, y };
            let pos = camera.transform_screen_pos(point);
            if pos.x < 0 || pos.x >= map.width || pos.y < 0 || pos.y >= map.height {
                ctx.set(
                    x,
                    y,
                    RGBA::from_f32(0., 0., 0., 1.),
                    RGBA::from_f32(0., 0., 0., 1.),
                    to_cp437('¿'),
                );
            } else {
                let idx = map.map_pos_to_idx(pos);
                let tile = map.tiles[idx];
                // Render a tile depending upon the tile type
                if map.revealed_tiles[idx] {
                    let glyph;

                    let bg = RGB::from_f32(0., 0., 0.);
                    let mut fg;

                    match tile {
                        TileType::Floor => {
                            fg = RGB::from_f32(0.5, 0.5, 0.5);
                            glyph = to_cp437('·');
                        }
                        TileType::Wall(walltype) => {
                            fg = RGB::from_f32(0.7, 0.9, 0.7);
                            glyph = get_glyph_for_wall(&map, idx, pos.x, pos.y, walltype)
                        }
                        TileType::Stone => {
                            fg = RGB::from_f32(0.0, 1.0, 0.0);
                            glyph = to_cp437('#');
                        }
                    }
                    if !map.visible_tiles[idx] {
                        fg = fg.to_greyscale();
                    }
                    for i in 0..crate::LAYERS {
                        if i == 0 || tile != TileType::Floor {
                            ctx.set_active_console(i);
                            let mix = (crate::LAYERS - 1 - i) as f32 / (crate::LAYERS - 1) as f32;
                            let mix = mix / 4.0;
                            ctx.set(x, y, fg.lerp(RGB::from_f32(0., 0., 0.), mix), bg, glyph);
                        }
                    }
                    ctx.set_active_console(0);
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
