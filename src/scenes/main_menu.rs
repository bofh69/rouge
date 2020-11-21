use super::*;
use crate::ecs::Ecs;
use crate::gui::MainMenuResult::*;
use crate::gui::MainMenuState::*;

#[derive(Debug)]
pub(crate) struct MainMenuScene {
    state: crate::gui::MainMenuState,
    comet_line: i32,
}

impl Scene<Ecs> for MainMenuScene {
    fn tick(&mut self, ecs: &mut Ecs, ctx: &mut BTerm) -> SceneResult<Ecs> {
        let time = {
            let time = resource_get!(ecs, crate::resources::Time);
            (time.real_time_ms % 10000) as f32 / 1000.
        };
        ctx.cls();
        match crate::gui::show_main_menu(ctx, time, &mut self.comet_line, self.state) {
            Selected(New) => SceneResult::Replace(Box::new(super::game::GameScene::new(ecs))),
            Selected(Quit) => SceneResult::Pop,
            Selected(Load) => {
                // TODO Implent call to load
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
    pub fn new() -> MainMenuScene {
        MainMenuScene {
            state: crate::gui::MainMenuState::New,
            comet_line: 0,
        }
    }

    #[allow(dead_code)]
    pub fn new_for_load() -> MainMenuScene {
        MainMenuScene {
            state: crate::gui::MainMenuState::Load,
            comet_line: 0,
        }
    }
}
