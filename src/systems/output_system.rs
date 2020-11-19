use crate::components::{Item, Monster, Name, Position, Viewshed};
use crate::entity_adapter::EntityAdapterImpl;
use crate::resources::{GameLog, OutputQueue};
use crate::PlayerEntity;
use ::legion::system;
use legion::world::SubWorld;

#[system]
#[read_component(Name)]
#[read_component(Item)]
#[read_component(Position)]
#[read_component(Viewshed)]
#[read_component(Monster)]
pub(crate) fn output(
    world: &mut SubWorld,
    #[resource] output: &mut OutputQueue,
    #[resource] gamelog: &mut GameLog,
    #[resource] player: &PlayerEntity,
) {
    let mut entity_adapter = EntityAdapterImpl::new(world, gamelog, player.0);

    output.process_queue(&mut entity_adapter);
}
