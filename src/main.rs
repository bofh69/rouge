bracket_lib::prelude::add_wasm_support!();

#[macro_use]
mod ecs;

mod camera;
mod components;
mod entity_adapter;
mod gamelog;
mod gui;
mod map;
mod player;
mod scenes;
mod spawner;
mod systems;

use bracket_lib::prelude::*;
use camera::Camera;
use components::*;
use legion::*;
use map::Map;
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(PartialEq, Copy, Clone)]
pub(crate) enum InventoryType {
    Apply,
    Drop,
}

#[derive(PartialEq, Copy, Clone)]
pub(crate) enum RunState {
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

pub(crate) struct PlayerEntity(Entity);

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
pub(crate) enum PlayerTarget {
    None,
    Position(MapPosition),
    Dir(Direction),
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct PlayerPosition(pub MapPosition);

impl Into<Position> for PlayerPosition {
    fn into(self) -> Position {
        Position(self.0)
    }
}

pub(crate) struct State {
    ecs: ecs::Ecs,
    scene_manager: scenes::SceneManager<ecs::Ecs>,
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

// embedded_resource!(TILE_FONT, "../resources/cheepicus8x8.png");

const LAYERS: usize = 7;

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    const SCREEN_WIDTH: i32 = 80;
    const SCREEN_HEIGHT: i32 = 50;
    let map = Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();

    // link_resource!(TILE_FONT, "resources/cheepicus8x8.png");

    let mut builder = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)?
        .with_title("Rouge World")
        .with_font("terminal8x8.png", 8, 8)
        .with_advanced_input(true)
        .with_resource_path("resources")
        .with_vsync(true);
    // Add layers for walls.
    for _i in 0..LAYERS - 1 {
        builder = builder.with_sparse_console_no_bg(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png");
    }
    // Layer for GUI:
    builder = builder.with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png");
    let mut context = builder.build()?;
    context.set_active_console(LAYERS);

    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: ecs::Ecs::new(),
        scene_manager: scenes::SceneManager::new(),
        old_shift: false,
    };

    gs.ecs.resources.insert(RandomNumberGenerator::new());
    gs.ecs.resources.insert(gamelog::GameLog::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    let player_entity = spawner::player(&mut gs.ecs, player_pos.x, player_pos.y);

    let mut output_queue = gamelog::OutputQueue::new(Mutex::new(VecDeque::new()), player_entity);
    output_queue.s("Welcome to ").color(RED).s("Rouge");
    gs.ecs.resources.insert(output_queue);

    let player_pos = PlayerPosition(MapPosition {
        x: player_pos.x,
        y: player_pos.y,
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
