use super::*;
use crate::ecs::Ecs;
use crate::gui::MainMenuResult::*;
use crate::gui::MainMenuState::*;

#[derive(Debug)]
pub(crate) struct MainMenuScene {
    state: crate::gui::MainMenuState,
    time: f32,
}

impl Scene<Ecs> for MainMenuScene {
    fn tick(&mut self, ecs: &mut Ecs, ctx: &mut BTerm) -> SceneResult<Ecs> {
        self.time += ctx.frame_time_ms / 1000.;
        while self.time > 10. {
            self.time -= 10.;
        }
        ctx.cls();
        ctx.print(30, 1, format!("{}", self.time));
        match crate::gui::show_main_menu(ctx, self.time, self.state) {
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
            time: 0.,
        }
    }

    #[allow(dead_code)]
    pub fn new_for_load() -> MainMenuScene {
        MainMenuScene {
            state: crate::gui::MainMenuState::Load,
            time: 0.,
        }
    }
}
