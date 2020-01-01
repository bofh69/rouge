rltk::add_wasm_support!();

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
mod spawner;
mod visibility_system;

#[macro_use]
extern crate specs_derive;

use components::*;
use map::Map;
use player::*;
use rltk::{Console, GameState, Rltk};
use specs::prelude::*;
use visibility_system::VisibilitySystem;

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
    ShowTargeting(i32, Entity),
}

pub struct PlayerEntity(Entity);

#[derive(Debug, Copy, Clone)]
pub struct PlayerPosition(i32, i32);

impl Into<rltk::Point> for PlayerPosition {
    fn into(self) -> rltk::Point {
        /* Should go via Camera */
        rltk::Point::new(self.0, self.1)
    }
}

impl Into<Position> for PlayerPosition {
    fn into(self) -> Position {
        Position {
            x: self.0,
            y: self.1,
        }
    }
}

pub struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        {
            map::draw_map(&self.ecs, ctx);

            let map = self.ecs.fetch::<Map>();

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();

            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

            for (pos, render) in data.iter() {
                if map.visible_tiles[map.xy_idx(pos.x, pos.y)] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

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
            RunState::ReallyQuit => match gui::ask_bool(ctx, "Really quit?") {
                (gui::ItemMenuResult::Selected, false) | (gui::ItemMenuResult::Cancel, _) => {
                    newrunstate = RunState::AwaitingInput
                }
                (gui::ItemMenuResult::Selected, true) => ctx.quit(),
                _ => (),
            },
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory(inv_type) => match gui::show_inventory(self, ctx, inv_type) {
                (gui::ItemMenuResult::Cancel, _) => newrunstate = RunState::AwaitingInput,
                (gui::ItemMenuResult::Selected, Some(item_entity)) => {
                    let player_entity = self.ecs.fetch::<PlayerEntity>();
                    match inv_type {
                        InventoryType::Apply => {
                            if let Some(range) = self.ecs.read_storage::<Ranged>().get(item_entity)
                            {
                                newrunstate = RunState::ShowTargeting(range.range, item_entity);
                            } else {
                                let mut wants_to_drink = self.ecs.write_storage::<WantsToUseItem>();
                                wants_to_drink
                                    .insert(
                                        player_entity.0,
                                        WantsToUseItem {
                                            item: item_entity,
                                            target: None,
                                        },
                                    )
                                    .expect("Could not insert");
                                newrunstate = RunState::PlayerTurn;
                            }
                        }
                        InventoryType::Drop => {
                            let mut wants_to_drop = self.ecs.write_storage::<WantsToDropItem>();
                            wants_to_drop
                                .insert(player_entity.0, WantsToDropItem { item: item_entity })
                                .expect("Could not insert");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
                _ => (),
            },
            RunState::ShowTargeting(range, item_entity) => {
                match gui::show_targeting(self, ctx, range) {
                    (gui::ItemMenuResult::Cancel, _) => newrunstate = RunState::AwaitingInput,
                    (gui::ItemMenuResult::Selected, Some(target_position)) => {
                        let player_entity = self.ecs.fetch::<PlayerEntity>();

                        let mut wants_to_use = self.ecs.write_storage::<WantsToUseItem>();
                        wants_to_use
                            .insert(
                                player_entity.0,
                                WantsToUseItem {
                                    item: item_entity,
                                    target: Some(target_position),
                                },
                            )
                            .expect("Could not insert");
                        newrunstate = RunState::PlayerTurn;
                    }
                    _ => (),
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
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

        let mut drop = inventory_system::ItemDroppingSystem {};
        drop.run_now(&self.ecs);

        let mut pickup = inventory_system::ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

        let mut drink = consume_system::UseItemSystem {};
        drink.run_now(&self.ecs);

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

    let mut context = Rltk::init_simple8x8(80, 50, "Rouge World", "resources");
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<HealthProvider>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ReceiveHealth>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rouge".to_string()],
    });

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(map);
    gs.ecs.insert(PlayerPosition(player_x, player_y));
    gs.ecs.insert(PlayerEntity(player_entity));

    rltk::main_loop(context, gs);
}
