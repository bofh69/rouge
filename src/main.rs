rltk::add_wasm_support!();

mod camera;
mod components;
mod consume_system;
mod damage_system;
mod gamelog;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_sysem;
mod melee_combat_system;
mod monster_ai_systems;
mod player;
mod rect;
mod scenes;
mod spawner;
mod visibility_system;

#[macro_use]
extern crate specs_derive;

use camera::Camera;
use components::*;
use map::Map;
use rltk::{GameState, Point, Rltk};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

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

#[derive(PartialEq, Serialize, Deserialize, Debug, Copy, Clone)]
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

pub struct State {
    ecs: World,
    scene_manager: scenes::SceneManager<World>,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        self.scene_manager.tick(&mut self.ecs, ctx);
    }
}

// struct PlayerInputSystem {}

// impl<'a> System<'a> for PlayerInputSystem {
//     type SystemData = (ReadStorage<'a Player>,
//                        WriteStorage<'a Position>,
//                        World);

//     fn run(&mut self, (player, mut pos, ecs) : Self.SystemData) {
//         player
//     }
// }

impl State {}

fn main() {
    const SCREEN_WIDTH: u32 = 80;
    const SCREEN_HEIGHT: u32 = 50;
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut context = Rltk::init_simple8x8(SCREEN_WIDTH, SCREEN_HEIGHT, "Rouge World", "resources");
    /*
    let cw = context.backend.platform.context_wrapper.as_ref();
    let mh = cw
        .expect("Unref context wrapper")
        .el
        .available_monitors()
        .nth(0)
        .expect("Getting first monitor");
    cw.unwrap()
        .wc
        .window()
        .set_fullscreen(Some(glutin::window::Fullscreen::Borderless(mh)));
    */

    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new(),
        scene_manager: scenes::SceneManager::new(),
    };

    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<HealthProvider>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ItemIndex>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ReceiveHealth>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rouge".to_string()],
    });

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // let data = serde_json::to_string(&map).unwrap();
    // println!("{}", data);

    let player_pos = PlayerPosition(MapPosition {
        x: player_x,
        y: player_y,
    });
    gs.ecs.insert(Camera::new(
        player_pos,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32 - 7,
    ));
    gs.ecs.insert(map);
    gs.ecs.insert(player_pos);
    gs.ecs.insert(PlayerEntity(player_entity));
    gs.ecs.insert(PlayerTarget::None);

    gs.scene_manager
        .push(Box::new(scenes::MainMenuScene::new()));

    rltk::main_loop(context, gs);
}
