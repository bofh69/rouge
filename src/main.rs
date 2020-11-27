bracket_lib::prelude::add_wasm_support!();

#[macro_use]
mod ecs;

mod components;
mod entity_adapter;
mod gui;
mod messages;
mod player;
mod positions;
mod queues;
mod resources;
mod scenes;
mod spawner;
mod systems;

use crate::positions::{Direction, MapPosition, ScreenPosition};
use crate::resources::{Camera, GameLog, OutputQueue, PlayerEntity, PlayerPosition, PlayerTarget};
use ::bracket_lib::prelude::*;
use ::legion::Entity;
use ::legion_typeuuid::collect_registry;
use ::std::collections::VecDeque;
use ::std::sync::Mutex;
use legion_typeuuid::SerializableTypeUuid;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static + Send + Sync>>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum InventoryType {
    Apply,
    Drop,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum RunState {
    AwaitingInput,
    ReallyQuit,
    PreRun,
    Tick,
    EnergylessTick,
    ShowInventory(InventoryType),
    ShowTargeting(gui::TargetingInfo, Entity),
    SaveGame,
}

pub(crate) struct State {
    ecs: ecs::Ecs,
    registry: legion::Registry<SerializableTypeUuid>,
    scene_manager: scenes::SceneManager<ecs::Ecs>,
    old_shift: bool,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        {
            let mut time = self.ecs.resources.get_mut_or_default::<resources::Time>();
            time.last_real_time_ms = time.real_time_ms;
            time.real_time_ms += ctx.frame_time_ms as i64;
        }

        {
            let mut input = INPUT.lock();

            ctx.shift = input.key_pressed_set().contains(&VirtualKeyCode::LShift)
                || input.key_pressed_set().contains(&VirtualKeyCode::RShift);
            if !ctx.shift && self.old_shift != ctx.shift && ctx.key.is_none() {
                ctx.key = Some(VirtualKeyCode::LShift);
            }
            self.old_shift = ctx.shift;

            let ctrl = input.key_pressed_set().contains(&VirtualKeyCode::LControl)
                || input.key_pressed_set().contains(&VirtualKeyCode::RControl);

            if let Some(VirtualKeyCode::P) = ctx.key {
                if ctrl {
                    ctx.screenshot("screenshot.png");
                    let mut gamelog = resource_get_mut!(self.ecs, GameLog);
                    gamelog.set_color(GREEN);
                    gamelog.write_text("Screenshot taken.");
                    gamelog.end_of_line();
                }
            }

            #[allow(clippy::single_match)]
            input.for_each_message(|event| match event {
                BEvent::CloseRequested => ctx.quitting = true,
                _ => (),
            });
        }

        self.scene_manager.tick(&mut self.ecs, ctx);
    }
}

impl State {}

const LAYERS: usize = 7;

fn main() -> Result<()> {
    const SCREEN_WIDTH: i32 = 80;
    const SCREEN_HEIGHT: i32 = 50;

    let map = resources::Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();

    let mut builder = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)?
        .with_title("Rouge World")
        .with_font("terminal8x8.png", 8, 8)
        .with_advanced_input(true)
        .with_resource_path("resources")
        .with_vsync(true);
    // Add layers for walls.
    for _i in 0..LAYERS - 1 {
        builder = builder.with_sparse_console_no_bg(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png");
    }
    // Layer for GUI:
    builder = builder.with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png");
    let mut context = builder.build()?;
    context.set_active_console(LAYERS);

    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: ecs::Ecs::new(),
        scene_manager: scenes::SceneManager::new(),
        registry: collect_registry(),
        old_shift: false,
    };

    gs.ecs.resources.insert(RandomNumberGenerator::new());
    gs.ecs.resources.insert(GameLog::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    let player_entity = spawner::player(&mut gs.ecs, player_pos.x, player_pos.y);

    let output_queue = OutputQueue::new(Mutex::new(VecDeque::new()), player_entity);
    output_queue.s("Welcome to ").color(RED).s("Rouge");
    gs.ecs.resources.insert(output_queue);

    let player_pos = PlayerPosition(MapPosition {
        x: player_pos.x,
        y: player_pos.y,
    });
    gs.ecs.resources.insert(Camera::new(
        player_pos,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32 - 7,
    ));
    gs.ecs.resources.insert(map);
    gs.ecs.resources.insert(player_pos);
    gs.ecs.resources.insert(PlayerEntity(player_entity));
    gs.ecs.resources.insert(PlayerTarget::None);
    queues::register_queues(&mut gs.ecs.resources);

    gs.scene_manager
        .push(Box::new(scenes::MainMenuScene::new()));

    {
        let mut fil = std::fs::File::create("save.dat")?;
        save(&gs, &mut fil)?;
    }

    /*
    {
        let mut fil = std::fs::File::open("save.dat")?;
        load(&mut gs, &mut fil)?;
    }
    */

    main_loop(context, gs)?;
    Ok(())
}

pub(crate) fn save(gs: &State, writer: &mut dyn Write) -> Result<()> {
    let bytes = serde_cbor::to_vec(
        &gs.ecs
            .world
            .as_serializable(legion::query::any(), &gs.registry),
    )?;
    writer.write_all(&bytes).unwrap();

    // Write resources.
    // TODO: This probably doesn't work to deserialize later, it will be joined with the above code:
    serde_cbor::to_writer(writer, &*resource_get!(gs.ecs, crate::resources::Map)).unwrap();

    Ok(())
}

/*
pub(crate) fn load(gs: &mut State, reader: &mut dyn Read) -> Result<()> {
    use serde::de::DeserializeSeed;
    let obj = serde_cbor::de::from_reader(reader)?;
    let world: ::legion::World = gs.registry.as_deserialize().deserialize(obj)?;

    // Read resources
    //serde_cbor::to_writer(writer, &*resource_get!(gs.ecs, crate::resources::Map)).unwrap();

    Ok(())
}
*/
