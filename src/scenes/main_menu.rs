use super::*;
use crate::ecs::Ecs;
use crate::gui::MainMenuResult::*;
use crate::gui::MainMenuState::*;
use legion::Schedule;

pub(crate) struct MainMenuScene {
    state: crate::gui::MainMenuState,
    schedule: Schedule,
}

impl Scene<Ecs> for MainMenuScene {
    fn tick(&mut self, ecs: &mut Ecs, ctx: &mut BTerm) -> SceneResult<Ecs> {
        ctx.cls();
        self.schedule.execute(&mut ecs.world, &mut ecs.resources);
        match crate::gui::show_main_menu(ctx, ecs, self.state) {
            Selected(New) => {
                crate::new(ecs).unwrap();
                SceneResult::Replace(Box::new(super::game::GameScene::new(ecs)))
            }
            Selected(Quit) => SceneResult::Pop,
            Selected(Load) => {
                // TODO Implement call to load
                SceneResult::Pop
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

    #[allow(dead_code)]
    pub fn new_for_load() -> MainMenuScene {
        MainMenuScene {
            state: crate::gui::MainMenuState::Load,
            schedule: Self::build_schedule(),
        }
    }
}
