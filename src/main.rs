rltk::add_wasm_support!();

mod components;
mod damage_system;
mod gamelog;
mod gui;
mod map;
mod map_indexing_sysem;
mod melee_combat_system;
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
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut newrunstate = {
            let runstate = self.ecs.fetch::<RunState>();
            *runstate
        };

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
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

        gui::draw_ui(&self.ecs, ctx);
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

        let mut mcs = melee_combat_system::MeleeCombatSystem {};
        mcs.run_now(&self.ecs);

        let mut ds = damage_system::DamageSystem {};
        ds.run_now(&self.ecs);

        damage_system::delete_the_dead(&mut self.ecs);

        let mut mis = map_indexing_sysem::MapIndexingSystem {};
        mis.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() {
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut context = Rltk::init_simple8x8(80, 50, "Hello Rouge World", "resources");
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToMelee>();

    let mut rng = rltk::RandomNumberGenerator::new();

    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        let (glyph, name) = match rng.roll_dice(1, 2) {
            1 => ('g', "goblin".to_string()),
            _ => ('o', "orc".to_string()),
        };
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: rltk::to_cp437(glyph),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(BlocksTile {})
            .with(Monster {})
            .with(Name {
                name: format!("{} {}", name, i),
            })
            .with(CombatStats {
                hp: 16,
                max_hp: 16,
                power: 4,
                defense: 1,
            })
            .build();
    }

    let player_entity = gs
        .ecs
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
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            hp: 30,
            max_hp: 30,
            power: 5,
            defense: 2,
        })
        .build();

    gs.ecs.insert(map);
    gs.ecs.insert(rltk::Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rouge Rogue".to_string()],
    });

    rltk::main_loop(context, gs);
}
