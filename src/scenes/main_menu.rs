use super::*;
use crate::ecs::Ecs;

#[derive(Debug)]
pub(crate) struct MainMenuScene {
    state: crate::gui::MainMenuState,
}

impl Scene<Ecs> for MainMenuScene {
    fn tick(&mut self, ecs: &mut Ecs, ctx: &mut BTerm) -> SceneResult<Ecs> {
        use crate::gui::MainMenuResult::*;
        use crate::gui::MainMenuState::*;
        ctx.cls();
        match crate::gui::show_main_menu(ctx, self.state) {
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
        }
    }

    #[allow(dead_code)]
    pub fn new_for_load() -> MainMenuScene {
        MainMenuScene {
            state: crate::gui::MainMenuState::Load,
        }
    }
}
