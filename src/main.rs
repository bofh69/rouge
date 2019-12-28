rltk::add_wasm_support!();

mod components;
mod map;
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

pub struct State {
    ecs: World,
    pub paused: bool,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        if self.paused {
            if Some(rltk::VirtualKeyCode::P) == ctx.key {
                self.paused = false;
            } else if Some(rltk::VirtualKeyCode::Escape) == ctx.key {
                ctx.quit();
            }
        } else {
            ctx.cls();

            self.run_systems();

            player_input(self, ctx);

            map::draw_map(&self.ecs, ctx);

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();

            for (pos, render) in (&positions, &renderables).join() {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

struct WalkingSystem {}

impl<'a> System<'a> for WalkingSystem {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = 80 - 1;
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
        let mut lw = WalkingSystem {};
        lw.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
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
        paused: false,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    gs.ecs.insert(map);

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

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }

    rltk::main_loop(context, gs);
}
