use super::*;
use specs::prelude::World;

pub struct MainMenuScene {
    pub state: crate::gui::MainMenuState,
}

impl Scene<World> for MainMenuScene {
    fn tick(&mut self, _ecs: &mut World, ctx: &mut Rltk) -> SceneResult<World> {
        use crate::gui::MainMenuResult::*;
        use crate::gui::MainMenuState::*;
        ctx.cls();
        match crate::gui::show_main_menu(ctx, self.state) {
            Selected(New) => SceneResult::Replace(Box::new(super::game::GameScene {})),
            Selected(Quit) => SceneResult::Pop,
            Selected(Load) => {
                // TODO
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
}
