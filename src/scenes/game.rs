use super::*;
use crate::camera::{Camera, CameraSystem};
use crate::components::*;
use crate::map::Map;
use crate::player::player_input;
use crate::visibility_system::VisibilitySystem;
use crate::PlayerPosition;
use crate::{gui, PlayerEntity, RunState};
use rltk::*;
use specs::prelude::*;

#[derive(Debug)]
pub struct GameScene {}

impl Scene<World> for GameScene {
    fn tick(&mut self, ecs: &mut World, ctx: &mut Rltk) -> SceneResult<World> {
        ctx.cls();
        {
            let mut camera_sys = CameraSystem {};
            camera_sys.run_now(ecs);

            crate::map::draw_map(ecs, ctx);

            let map = ecs.fetch::<Map>();

            let camera = ecs.fetch::<Camera>();
            let positions = ecs.read_storage::<Position>();
            let renderables = ecs.read_storage::<Renderable>();

            let mut data = (&positions, &renderables)
                .join()
                .filter(|(p, _)| camera.is_in_view(p.0))
                .collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

            for (pos, render) in data.iter() {
                if map.visible_tiles[map.pos_to_idx(**pos)] {
                    let point = camera.transform_map_pos(pos.0);
                    ctx.set(point.x, point.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        let mut newrunstate = {
            let runstate = ecs.fetch::<RunState>();
            *runstate
        };

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
                    let player_entity = ecs.fetch::<PlayerEntity>();
                    match inv_type {
                        crate::InventoryType::Apply => {
                            if let Some(range) = ecs.read_storage::<Ranged>().get(item_entity) {
                                let player_position = ecs.fetch::<PlayerPosition>().0;
                                let camera = ecs.fetch::<Camera>();
                                let start_pos = camera.transform_map_pos(player_position);
                                let targeting_info =
                                    gui::TargetingInfo::new(range.range, start_pos, ctx);
                                newrunstate = RunState::ShowTargeting(targeting_info, item_entity);
                            } else {
                                let mut wants_to_drink = ecs.write_storage::<WantsToUseItem>();
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
                        crate::InventoryType::Drop => {
                            let mut wants_to_drop = ecs.write_storage::<WantsToDropItem>();
                            wants_to_drop
                                .insert(player_entity.0, WantsToDropItem { item: item_entity })
                                .expect("Could not insert");
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
                        let player_entity = ecs.fetch::<PlayerEntity>();

                        let mut wants_to_use = ecs.write_storage::<WantsToUseItem>();
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

        gui::draw_ui(ecs, ctx);

        {
            let mut runwriter = ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        SceneResult::Continue
    }
}

impl GameScene {
    pub fn new(ecs: &mut World) -> Self {
        ecs.insert(RunState::PreRun);
        Self {}
    }

    fn run_systems(&mut self, ecs: &mut World) {
        let mut vis = VisibilitySystem {};
        vis.run_now(ecs);

        let mut monai = crate::monster_ai_systems::MonsterAiSystem {};
        monai.run_now(ecs);

        let mut mcs = crate::melee_combat_system::MeleeCombatSystem {};
        mcs.run_now(ecs);

        let mut drop = crate::inventory_system::ItemDroppingSystem {};
        drop.run_now(ecs);

        let mut pickup = crate::inventory_system::ItemCollectionSystem {};
        pickup.run_now(ecs);

        let mut drink = crate::consume_system::UseItemSystem {};
        drink.run_now(ecs);

        let mut ds = crate::damage_system::DamageSystem {};
        ds.run_now(ecs);

        crate::damage_system::delete_the_dead(ecs);

        let mut mis = crate::map_indexing_sysem::MapIndexingSystem {};
        mis.run_now(ecs);

        ecs.maintain();
    }
}
