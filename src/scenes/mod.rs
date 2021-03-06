mod game;
mod main_menu;
mod save_game;
mod show_text;

pub(crate) use main_menu::*;
// pub(crate) use save_game::*;

use bracket_lib::prelude::*;

pub(crate) enum SceneResult<T> {
    Continue,
    Pop,
    #[allow(dead_code)]
    Push(Box<dyn Scene<T>>),
    Replace(Box<dyn Scene<T>>),
}

pub(crate) trait Scene<T> {
    fn tick(&mut self, state: &mut T, ctx: &mut BTerm) -> SceneResult<T>;
}

pub(crate) struct SceneManager<T> {
    scenes: Vec<Box<dyn Scene<T>>>,
}

impl<T> SceneManager<T> {
    pub fn new() -> Self {
        Self { scenes: vec![] }
    }

    pub fn push(&mut self, scene: Box<dyn Scene<T>>) {
        self.scenes.push(scene)
    }

    pub fn tick(&mut self, state: &mut T, ctx: &mut BTerm) {
        if self.scenes.is_empty() {
            ctx.quit();
            return;
        }
        match self.scenes.last_mut().unwrap().tick(state, ctx) {
            SceneResult::Continue => (),
            SceneResult::Pop => {
                self.scenes.pop();
            }
            SceneResult::Push(new_scene) => {
                self.scenes.push(new_scene);
            }
            SceneResult::Replace(new_scene) => {
                self.scenes.pop();
                self.scenes.push(new_scene);
            }
        }
    }
}
