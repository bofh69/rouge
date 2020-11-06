use legion::*;

pub(crate) struct Ecs {
    pub world: World,
    pub resources: Resources,
}

impl Ecs {

    pub fn new() -> Self {
        Self {
            world: World::default(),
            resources: Resources::default(),
        }
    }
}