use crate::components::Energy;
use crate::resources::Time;
use crate::RunState;
use ::legion::*;
use legion::world::SubWorld;

#[system]
#[write_component(Energy)]
pub(crate) fn regain_energy(
    world: &mut SubWorld,
    #[resource] rs: &RunState,
    #[resource] time: &mut Time,
) {
    if *rs == RunState::EnergylessTick {
        return;
    }

    let mut max = i32::MIN;

    // Find highest energy below zero.
    for (energy) in <&Energy>::query().iter(world) {
        if energy.energy < 0 && energy.energy > max {
            max = energy.energy;
        }
    }

    // TODO: Put a cap on max to make turn based animations smoother
    // or only when a particle has been spawned?

    let max = max;
    if max > i32::MIN {
        for energy in <&mut Energy>::query().iter_mut(world) {
            if energy.energy < 0 {
                energy.energy += -max;
            }
        }
        time.tick_time += -max as i64;
    }
}
