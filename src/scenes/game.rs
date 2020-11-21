use crate::components::*;
use crate::ecs::Ecs;
use crate::player::player_input;
use crate::resources::{Camera, Map, PlayerEntity, PlayerPosition};
use crate::{gui, RunState};
use ::bracket_lib::prelude::*;
use ::legion::*;

use super::{Scene, SceneResult};

pub(crate) struct GameScene {
    schedule: Schedule,
    schedule2: Schedule,
}

impl Scene<Ecs> for GameScene {
    fn tick(&mut self, ecs: &mut Ecs, ctx: &mut BTerm) -> SceneResult<Ecs> {
        for i in 0..=crate::LAYERS {
            ctx.set_active_console(i);
            ctx.cls();
        }
        // ctx.print(35, 22, &format!("{} fps", ctx.fps as u32));

        {
            let mut schedule = Schedule::builder()
                .add_system(crate::systems::camera_update_system())
                .build();
            schedule.execute(&mut ecs.world, &mut ecs.resources);

            crate::resources::draw_map(ecs, ctx);

            let camera = resource_get!(ecs, Camera);
            let player_position = resource_get!(ecs, PlayerPosition);
            let screen_pos = camera.transform_map_pos(player_position.0);

            for i in 0..crate::LAYERS {
                ctx.set_active_console(i);
                ctx.set_scale(1.0 + i as f32 * 0.01, screen_pos.x, screen_pos.y);
            }
            ctx.set_active_console(crate::LAYERS);

            let mut data = <(&Position, &Renderable)>::query()
                .iter(&ecs.world)
                .filter(|(p, _)| camera.is_in_view(p.0))
                .collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

            let map = resource_get!(ecs, Map);
            for (pos, render) in data.iter() {
                if map.visible_tiles[map.pos_to_idx(**pos)] {
                    let point = camera.transform_map_pos(pos.0);
                    ctx.set(point.x, point.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        let oldrunstate = { *resource_get!(ecs, RunState) };
        let mut newrunstate = oldrunstate;

        match newrunstate {
            RunState::SaveGame => {
                // return SceneResult::Replace(Box::new(crate::scenes::SaveScene::new()));
                newrunstate = RunState::AwaitingInput;
            }
            RunState::PreRun => {
                self.run_systems(ecs);
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(ecs, ctx);
            }
            RunState::ReallyQuit => match gui::ask_bool(ctx, "Really quit?") {
                (gui::ItemMenuResult::Selected, false) | (gui::ItemMenuResult::Cancel, _) => {
                    newrunstate = RunState::AwaitingInput
                }
                (gui::ItemMenuResult::Selected, true) => ctx.quit(),
                _ => (),
            },
            RunState::EnergylessTick | RunState::Tick => {
                self.run_systems(ecs);
                let entity = resource_get!(ecs, PlayerEntity).0;
                newrunstate = ecs
                    .world
                    .entry(entity)
                    .map_or(RunState::ReallyQuit, |entry| {
                        if entry.get_component::<Energy>().unwrap().energy >= 0 {
                            RunState::AwaitingInput
                        } else {
                            RunState::Tick
                        }
                    });
            }
            RunState::ShowInventory(inv_type) => match gui::show_inventory(ecs, ctx, inv_type) {
                (gui::ItemMenuResult::Cancel, _) => newrunstate = RunState::AwaitingInput,
                (gui::ItemMenuResult::Selected, Some(item_entity)) => {
                    let player_entity = resource_get!(ecs, PlayerEntity).0;
                    match inv_type {
                        crate::InventoryType::Apply => {
                            let should_add_wants_to_use = {
                                let entry = ecs.world.entry(item_entity).unwrap();
                                if let Ok(range) = entry.get_component::<Ranged>() {
                                    let player_position = resource_get!(ecs, PlayerPosition).0;
                                    let camera = resource_get!(ecs, Camera);
                                    let start_pos = camera.transform_map_pos(player_position);
                                    let targeting_info =
                                        gui::TargetingInfo::new(range.range, start_pos, ctx);
                                    newrunstate =
                                        RunState::ShowTargeting(targeting_info, item_entity);
                                    false
                                } else {
                                    true
                                }
                            };
                            if should_add_wants_to_use {
                                let mut entry = ecs.world.entry(player_entity).unwrap();
                                entry.add_component(WantsToUseItem {
                                    item: item_entity,
                                    target: None,
                                });
                                newrunstate = RunState::Tick;
                            }
                        }
                        crate::InventoryType::Drop => {
                            ecs.world
                                .entry(player_entity)
                                .unwrap()
                                .add_component(WantsToDropItem { item: item_entity });
                            newrunstate = RunState::Tick;
                        }
                    }
                }
                _ => (),
            },
            RunState::ShowTargeting(ref mut targeting_info, item_entity) => {
                match targeting_info.show_targeting(ecs, ctx) {
                    (gui::ItemMenuResult::Cancel, _) => newrunstate = RunState::AwaitingInput,
                    (gui::ItemMenuResult::Selected, Some(target_position)) => {
                        let player_entity = resource_get!(ecs, PlayerEntity).0;

                        ecs.world
                            .entry(player_entity)
                            .unwrap()
                            .add_component(WantsToUseItem {
                                item: item_entity,
                                target: Some(target_position),
                            });
                        newrunstate = RunState::Tick;
                    }
                    _ => (),
                }
            }
        }

        /*
        if newrunstate != oldrunstate || oldrunstate != RunState::AwaitingInput {
            let time = resource_get!(ecs, Time);
            println!( "ORS: {:?}, NRS: {:?}, time={:?}", oldrunstate, newrunstate, time);
        }
        */

        gui::draw_ui(ecs, ctx);

        {
            ecs.resources.insert(newrunstate);
        }
        SceneResult::Continue
    }
}

impl GameScene {
    pub(crate) fn new(ecs: &mut Ecs) -> Self {
        ecs.resources.insert(RunState::PreRun);
        let mut builder = Schedule::builder();
        builder.add_system(crate::systems::regain_energy_system());
        crate::systems::add_viewshed_system(ecs, &mut builder);

        let mut builder2 = Schedule::builder();
        builder2
            // .flush()
            .add_system(crate::systems::damage_system())
            .add_system(crate::systems::health_system())
            .add_system(crate::systems::output_die_system())
            .flush()
            .add_system(crate::systems::output_system())
            .flush()
            .add_system(crate::systems::delete_the_dead_system())
            .add_system(crate::systems::delete_items_system())
            .flush()
            .add_system(crate::systems::map_indexing_clear_system())
            .add_system(crate::systems::map_indexing_system());
        Self {
            schedule: builder.build(),
            schedule2: builder2.build(),
        }
    }

    fn run_systems(&mut self, ecs: &mut Ecs) {
        self.schedule.execute(&mut ecs.world, &mut ecs.resources);

        crate::systems::monster_ai_system(ecs);
        crate::systems::melee_combat_system(ecs);
        crate::systems::drop_system(ecs);
        crate::systems::pickup_system(ecs);
        crate::systems::consume_system(ecs);

        self.schedule2.execute(&mut ecs.world, &mut ecs.resources);
    }
}
