use crate::resources::PlayerPosition;
use ::bracket_lib::prelude::Point;
use ::legion_typeuuid::*;
use ::serde::*;
use ::type_uuid::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, TypeUuid)]
#[uuid = "042e6d67-a9dc-47da-89e8-3151a8f96606"]
pub(crate) struct MapPosition {
    pub x: i32,
    pub y: i32,
}
register_serialize!(MapPosition);

impl From<MapPosition> for Point {
    fn from(pos: MapPosition) -> Self {
        Point::new(pos.x, pos.y)
    }
}

impl From<PlayerPosition> for MapPosition {
    fn from(pos: PlayerPosition) -> Self {
        pos.0
    }
}

impl std::ops::Add<(i32, i32)> for MapPosition {
    type Output = MapPosition;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        MapPosition {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl std::ops::Add<Point> for MapPosition {
    type Output = MapPosition;

    fn add(self, rhs: Point) -> Self::Output {
        MapPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<MapPosition> for MapPosition {
    type Output = Point;

    fn sub(self, rhs: MapPosition) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone, TypeUuid)]
#[uuid = "ec3131b4-ee4d-4536-8b07-a4384aa6a9bc"]
pub(crate) struct ScreenPosition {
    pub x: i32,
    pub y: i32,
}
register_serialize!(ScreenPosition);

impl From<ScreenPosition> for Point {
    fn from(pos: ScreenPosition) -> Point {
        Point::new(pos.x, pos.y)
    }
}

impl From<ScreenPosition> for (i32, i32) {
    fn from(pos: ScreenPosition) -> Self {
        (pos.x, pos.y)
    }
}

impl From<ScreenPosition> for (usize, usize) {
    fn from(pos: ScreenPosition) -> (usize, usize) {
        let x = if pos.x > 0 { pos.x as usize } else { 0 };
        let y = if pos.y > 0 { pos.y as usize } else { 0 };
        (x, y)
    }
}

impl From<Point> for ScreenPosition {
    fn from(pos: Point) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum Direction {
    West = 1,
    East = 2,
    South = 4,
    SouthWest = 5,
    SouthEast = 6,
    North = 8,
    NorthWest = 9,
    NorthEast = 10,
}

impl Direction {
    pub(crate) fn iter() -> impl Iterator<Item = &'static Direction> {
        use Direction::*;
        [
            North, NorthEast, East, SouthEast, South, SouthWest, West, NorthWest,
        ]
        .iter()
    }
}

fn dir_to_dx_dy(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::West => (-1, 0),
        Direction::East => (1, 0),
        Direction::South => (0, 1),
        Direction::SouthWest => (-1, 1),
        Direction::SouthEast => (1, 1),
        Direction::North => (0, -1),
        Direction::NorthWest => (-1, -1),
        Direction::NorthEast => (1, -1),
    }
}

impl From<(i32, i32)> for Direction {
    fn from(coord: (i32, i32)) -> Self {
        match coord {
            (-1, 0) => Direction::West,
            (1, 0) => Direction::East,
            (0, 1) => Direction::South,
            (-1, 1) => Direction::SouthWest,
            (1, 1) => Direction::SouthEast,
            (0, -1) => Direction::North,
            (-1, -1) => Direction::NorthWest,
            (1, -1) => Direction::NorthEast,
            _ => panic!("Incorrect direction"),
        }
    }
}

impl From<Direction> for (i32, i32) {
    fn from(dir: Direction) -> Self {
        dir_to_dx_dy(dir)
    }
}

impl From<Direction> for Point {
    fn from(dir: Direction) -> Self {
        dir_to_dx_dy(dir).into()
    }
}

impl std::ops::Add<Direction> for Point {
    type Output = Point;

    fn add(self, rhs: Direction) -> Self::Output {
        let rhs: Point = rhs.into();
        self + rhs
    }
}

impl std::ops::Add<Direction> for MapPosition {
    type Output = MapPosition;

    fn add(self, rhs: Direction) -> Self::Output {
        let rhs: Point = rhs.into();
        self + rhs
    }
}
