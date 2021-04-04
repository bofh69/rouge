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

use crate::components::Player;
use crate::positions::{Direction, MapPosition, ScreenPosition};
use crate::resources::OutputQueue;
use crate::resources::{Camera, GameLog, PlayerEntity};
use ::bracket_lib::prelude::*;
use ::legion::Entity;
use ::legion_typeuuid::collect_registry;
use bincode::Options;
use legion::IntoQuery;
use legion_typeuuid::SerializableTypeUuid;
use std::collections::VecDeque;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;

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

struct OuterState {
    state: State,
    scene_manager: scenes::SceneManager<State>,
}

pub(crate) struct State {
    ecs: ecs::Ecs,
    registry: legion::Registry<SerializableTypeUuid>,
    old_shift: bool,
}

impl GameState for OuterState {
    fn tick(&mut self, ctx: &mut BTerm) {
        {
            let mut time = self
                .state
                .ecs
                .resources
                .get_mut_or_default::<resources::Time>();
            time.last_real_time_ms = time.real_time_ms;
            time.real_time_ms += ctx.frame_time_ms as i64;
        }

        {
            let mut input = INPUT.lock();

            ctx.shift = input.key_pressed_set().contains(&VirtualKeyCode::LShift)
                || input.key_pressed_set().contains(&VirtualKeyCode::RShift);
            if !ctx.shift && self.state.old_shift != ctx.shift && ctx.key.is_none() {
                ctx.key = Some(VirtualKeyCode::LShift);
            }
            self.state.old_shift = ctx.shift;

            let ctrl = input.key_pressed_set().contains(&VirtualKeyCode::LControl)
                || input.key_pressed_set().contains(&VirtualKeyCode::RControl);

            if let Some(VirtualKeyCode::P) = ctx.key {
                if ctrl {
                    ctx.screenshot("screenshot.png");
                    let mut gamelog = resource_get_mut!(self.state.ecs, GameLog);
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

        self.scene_manager.tick(&mut self.state, ctx);
    }
}

impl State {}

const LAYERS: usize = 7;
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

fn main() -> Result<()> {
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

    let mut gs = OuterState {
        state: State {
            ecs: ecs::Ecs::new(),
            registry: collect_registry(),
            old_shift: false,
        },
        scene_manager: scenes::SceneManager::new(),
    };

    gs.state
        .ecs
        .resources
        .insert(Camera::new(SCREEN_WIDTH, SCREEN_HEIGHT - 7));

    gs.scene_manager
        .push(Box::new(scenes::MainMenuScene::new()));

    main_loop(context, gs)?;
    Ok(())
}

fn bincode_options() -> bincode::DefaultOptions {
    bincode::DefaultOptions::default()
}

pub(crate) fn new(ecs: &mut ecs::Ecs) -> Result<()> {
    resources::new(ecs)?;

    Ok(())
}

pub(crate) fn save(gs: &State, writer: &mut dyn Write) -> Result<()> {
    resources::save(&gs.ecs, writer)?;

    let entity_serializer = legion::serialize::Canon::default();

    let serializable =
        gs.ecs
            .world
            .as_serializable(legion::query::any(), &gs.registry, &entity_serializer);
    // let encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::fast());
    bincode_options().serialize_into(writer, &serializable)?;

    Ok(())
}

pub(crate) fn load(ecs: &mut ecs::Ecs, reader: &mut dyn Read) -> Result<()> {
    // let mut decoder = flate2::read::GzDecoder::new(reader);

    resources::load(ecs, reader)?;
    queues::register_queues(&mut ecs.resources);

    let mut deser = bincode::Deserializer::with_reader(reader, bincode_options());
    let registry = collect_registry();
    use serde::de::DeserializeSeed;
    let entity_serializer = legion::serialize::Canon::default();
    let world = registry
        .as_deserialize(&entity_serializer)
        .deserialize(&mut deser)
        .unwrap();
    ecs.world = world;

    let mut query = <(Entity, &Player)>::query();

    let (entity, _player) = query.iter(&ecs.world).next().expect("Player");

    let output_queue = OutputQueue::new(Mutex::new(VecDeque::new()), *entity);
    output_queue.s("Welcome back.");
    ecs.resources.insert(output_queue);

    ecs.resources.insert(PlayerEntity(*entity));

    dbg!("All Loaded");

    Ok(())
}
