use crate::resources::PlayerPosition;
use ::bracket_lib::prelude::Point;
use ::serde::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub(crate) struct MapPosition {
    pub x: i32,
    pub y: i32,
}

impl Into<Point> for MapPosition {
    fn into(self) -> Point {
        Point::new(self.x, self.y)
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

#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) struct ScreenPosition {
    pub x: i32,
    pub y: i32,
}

impl Into<Point> for ScreenPosition {
    fn into(self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl Into<(i32, i32)> for ScreenPosition {
    fn into(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Into<(usize, usize)> for ScreenPosition {
    fn into(self) -> (usize, usize) {
        let x = if self.x > 0 { self.x as usize } else { 0 };
        let y = if self.y > 0 { self.y as usize } else { 0 };
        (x, y)
    }
}

impl Into<ScreenPosition> for Point {
    fn into(self) -> ScreenPosition {
        ScreenPosition {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
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

impl Into<Direction> for (i32, i32) {
    fn into(self) -> Direction {
        match self {
            (-1, 0) => Direction::West,
            (1, 0) => Direction::East,
            (0, 1) => Direction::South,
            (-1, 1) => Direction::SouthWest,
            (1, 1) => Direction::SouthEast,
            (0, -1) => Direction::North,
            (-1, -1) => Direction::NorthWest,
            (1, -1) => Direction::NorthEast,
            _ => panic!("Incorrect direcion"),
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
