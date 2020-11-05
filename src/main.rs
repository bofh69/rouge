bracket_lib::prelude::add_wasm_support!();

mod camera;
mod components;
mod consume_system;
mod damage_system;
mod gamelog;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_systems;
mod player;
mod rect;
mod scenes;
mod spawner;
mod visibility_system;

use bracket_lib::prelude::*;
use camera::Camera;
use components::*;
use legion::*;
use map::Map;

#[derive(PartialEq, Copy, Clone)]
pub enum InventoryType {
    Apply,
    Drop,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    ReallyQuit,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory(InventoryType),
    ShowTargeting(gui::TargetingInfo, Entity),
    SaveGame,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct MapPosition {
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

impl std::ops::Sub<MapPosition> for MapPosition {
    type Output = Point;

    fn sub(self, rhs: MapPosition) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct ScreenPosition {
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

pub struct PlayerEntity(Entity);

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Direction {
    West = 1,
    East = 2,
    South = 4,
    SouthWest = 5,
    SouthEast = 6,
    North = 8,
    NorthWest = 9,
    NorthEast = 10,
}

impl From<Direction> for (i32, i32) {
    fn from(dir: Direction) -> Self {
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
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PlayerTarget {
    None,
    Position(MapPosition),
    Dir(Direction),
}

#[derive(Debug, Copy, Clone)]
pub struct PlayerPosition(pub MapPosition);

impl Into<Position> for PlayerPosition {
    fn into(self) -> Position {
        Position(self.0)
    }
}

pub struct Ecs {
    ecs: World,
    resources: Resources,
}

pub struct State {
    ecs: Ecs,
    scene_manager: scenes::SceneManager<Ecs>,
    old_shift: bool,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        {
            let mut input = INPUT.lock();

            ctx.shift = input.key_pressed_set().contains(&VirtualKeyCode::LShift)
                || input.key_pressed_set().contains(&VirtualKeyCode::RShift);
            if !ctx.shift && self.old_shift != ctx.shift && ctx.key.is_none() {
                ctx.key = Some(VirtualKeyCode::LShift);
            }
            self.old_shift = ctx.shift;

            #[allow(clippy::single_match)]
            input.for_each_message(|event| match event {
                BEvent::CloseRequested => ctx.quitting = true,
                _ => (),
            });
        }

        self.scene_manager.tick(&mut self.ecs, ctx);
    }
}

impl State {}

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    const SCREEN_WIDTH: i32 = 80;
    const SCREEN_HEIGHT: i32 = 50;
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut context = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)?
        .with_title("Rouge World")
        .with_advanced_input(true)
        .with_resource_path("resources")
        .build()?;

    // let mut context = BTerm::init_simple8x8(SCREEN_WIDTH, SCREEN_HEIGHT, "Rouge World", "resources");

    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: Ecs {
            ecs: World::default(),
            resources: Resources::default(),
        },
        scene_manager: scenes::SceneManager::new(),
        old_shift: false,
    };

    gs.ecs.resources.insert(RandomNumberGenerator::new());
    gs.ecs.resources.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rouge".to_string()],
    });

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    let player_pos = PlayerPosition(MapPosition {
        x: player_x,
        y: player_y,
    });
    gs.ecs.resources.insert(Camera::new(
        player_pos,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32 - 7,
    ));
    gs.ecs.resources.insert(map);
    gs.ecs.resources.insert(player_pos);
    gs.ecs.resources.insert(PlayerEntity(player_entity));
    gs.ecs.resources.insert(PlayerTarget::None);

    gs.scene_manager
        .push(Box::new(scenes::MainMenuScene::new()));

    main_loop(context, gs)?;
    Ok(())
}
