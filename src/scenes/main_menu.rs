use super::*;
use crate::gui::MainMenuResult::*;
use crate::gui::MainMenuState::*;
use crate::State;
use legion::Schedule;

pub(crate) struct MainMenuScene {
    state: crate::gui::MainMenuState,
    schedule: Schedule,
}

impl Scene<State> for MainMenuScene {
    fn tick(&mut self, gs: &mut State, ctx: &mut BTerm) -> SceneResult<State> {
        ctx.cls();
        self.schedule
            .execute(&mut gs.ecs.world, &mut gs.ecs.resources);
        match crate::gui::show_main_menu(ctx, &mut gs.ecs, self.state) {
            Selected(New) => {
                crate::new(&mut gs.ecs).unwrap();
                {
                    let mut fil = std::fs::File::create("save.dat").unwrap();
                    crate::save(&gs, &mut fil).unwrap();
                }
                SceneResult::Replace(Box::new(super::game::GameScene::new(gs)))
            }
            Selected(Quit) => SceneResult::Pop,
            Selected(Load) => {
                if let Ok(mut fil) = std::fs::File::open("save.dat") {
                    crate::load(&mut gs.ecs, &mut fil).unwrap();
                    std::mem::drop(fil);
                    let _ = std::fs::remove_file("save.dat");
                    SceneResult::Replace(Box::new(super::game::GameScene::new(gs)))
                } else {
                    SceneResult::Continue
                }
            }
            NoSelection(state) => {
                self.state = state;
                SceneResult::Continue
            }
        }
    }
}

impl MainMenuScene {
    fn build_schedule() -> Schedule {
        let mut builder = Schedule::builder();
        builder
            .add_system(crate::systems::delete_after_time_system())
            .add_system(crate::systems::delete_after_tick_system())
            .flush()
            .build()
    }

    pub fn new() -> MainMenuScene {
        MainMenuScene {
            state: crate::gui::MainMenuState::New,
            schedule: Self::build_schedule(),
        }
    }
}
