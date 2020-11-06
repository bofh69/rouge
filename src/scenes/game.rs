use crate::ecs::Ecs;
use crate::{camera::Camera, components::*, map::Map};
use crate::{gui, PlayerEntity, RunState};
use crate::{player::player_input, PlayerPosition};
use bracket_lib::prelude::*;
use legion::*;

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
        ctx.print(35, 22, &format!("{} fps", ctx.fps as u32));

        {
            let mut schedule = Schedule::builder()
                .add_system(crate::camera::camera_update_system())
                .build();
            schedule.execute(&mut ecs.world, &mut ecs.resources);

            crate::map::draw_map(ecs, ctx);

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

        let mut newrunstate = { *resource_get!(ecs, RunState) };

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
            RunState::PlayerTurn => {
                self.run_systems(ecs);
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems(ecs);
                newrunstate = RunState::AwaitingInput;
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
                                newrunstate = RunState::PlayerTurn;
                            }
                        }
                        crate::InventoryType::Drop => {
                            ecs.world
                                .entry(player_entity)
                                .unwrap()
                                .add_component(WantsToDropItem { item: item_entity });
                            newrunstate = RunState::PlayerTurn;
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
                        newrunstate = RunState::PlayerTurn;
                    }
                    _ => (),
                }
            }
        }

        gui::draw_ui(ecs, ctx);

        {
            ecs.resources.insert(newrunstate);
        }
        SceneResult::Continue
    }
}

impl GameScene {
    pub fn new(ecs: &mut Ecs) -> Self {
        ecs.resources.insert(RunState::PreRun);
        let mut builder = Schedule::builder();
        builder.add_system(crate::camera::camera_update_system());
        crate::visibility_system::add_viewshed_system(ecs, &mut builder);

        let mut builder2 = Schedule::builder();
        builder2
            .add_system(crate::damage_system::damage_system())
            .add_system(crate::damage_system::health_system())
            .add_system(crate::damage_system::delete_the_dead_system())
            .flush()
            .add_system(crate::map_indexing_system::map_indexing_clear_system())
            .add_system(crate::map_indexing_system::map_indexing_system());
        Self {
            schedule: builder.build(),
            schedule2: builder2.build(),
        }
    }

    fn run_systems(&mut self, ecs: &mut Ecs) {
        self.schedule.execute(&mut ecs.world, &mut ecs.resources);

        crate::monster_ai_systems::system(ecs);
        crate::melee_combat_system::melee_combat_system(ecs);
        crate::inventory_system::drop_system(ecs);
        crate::inventory_system::pickup_system(ecs);
        crate::consume_system::consume_system(ecs);

        self.schedule2.execute(&mut ecs.world, &mut ecs.resources);
    }
}
