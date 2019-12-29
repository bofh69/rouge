rltk::add_wasm_support!();

mod components;
mod map;
mod monster_ai_systems;
mod player;
mod rect;
mod visibility_system;

use rltk::{Console, GameState, Rltk, RGB};
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
use components::*;
use map::Map;
use player::*;
use visibility_system::VisibilitySystem;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    ecs: World,
    pub runstate: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        map::draw_map(&self.ecs, ctx);

        let map = self.ecs.fetch::<Map>();

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            if map.visible_tiles[map.xy_idx(pos.x, pos.y)] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
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

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut monai = monster_ai_systems::MonsterAiSystem {};
        monai.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() {
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let context = Rltk::init_simple8x8(
        map.width as u32,
        map.height as u32,
        "Hello Rouge World",
        "resources",
    );
    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();

    let mut rng = rltk::RandomNumberGenerator::new();

    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();
        let glyph = rltk::to_cp437(match rng.roll_dice(1, 2) {
            1 => 'g',
            _ => 'o',
        });
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .build();
    }

    gs.ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

    gs.ecs.insert(map);
    gs.ecs.insert(rltk::Point::new(player_x, player_y));

    rltk::main_loop(context, gs);
}
