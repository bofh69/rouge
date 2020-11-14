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

#[macro_export]
macro_rules! resource_get_mut {
    ($ecs:expr, $T:ty) => {
        $ecs.resources
            .get_mut::<$T>()
            .expect("Resource is expected")
    };
}

#[macro_export]
macro_rules! resource_get {
    ($ecs:ident, $T:ty) => {
        $ecs.resources.get::<$T>().expect("Resource is expected")
    };
}
