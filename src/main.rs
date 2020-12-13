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
use bincode::Options;
use legion_typeuuid::SerializableTypeUuid;
use std::io::Read;
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

    let mut gs = State {
        ecs: ecs::Ecs::new(),
        scene_manager: scenes::SceneManager::new(),
        registry: collect_registry(),
        old_shift: false,
    };

    gs.ecs
        .resources
        .insert(Camera::new(SCREEN_WIDTH, SCREEN_HEIGHT - 7));

    // new(&mut gs.ecs, SCREEN_WIDTH, SCREEN_HEIGHT)?;

    gs.scene_manager
        .push(Box::new(scenes::MainMenuScene::new()));

    /*
    {
        let mut fil = std::fs::File::create("save.dat")?;
        save(&gs, &mut fil)?;
    }

    {
        let mut fil = std::fs::File::open("save.dat")?;
        load(&mut gs, &mut fil)?;
    }
    */

    main_loop(context, gs)?;
    Ok(())
}

fn bincode_options() -> bincode::DefaultOptions {
    bincode::DefaultOptions::default()
}

pub(crate) fn new(ecs: &mut ecs::Ecs) -> Result<()> {
    let map = resources::Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();

    ecs.resources.insert(RandomNumberGenerator::new());
    ecs.resources.insert(GameLog::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, room);
    }
    let player_entity = spawner::player(ecs, player_pos.x, player_pos.y);

    let output_queue = OutputQueue::new(Mutex::new(VecDeque::new()), player_entity);
    output_queue.s("Welcome to ").color(RED).s("Rouge");
    ecs.resources.insert(output_queue);

    let player_pos = PlayerPosition(MapPosition {
        x: player_pos.x,
        y: player_pos.y,
    });
    {
        let mut camera = resource_get_mut!(ecs, Camera);
        camera.center(player_pos);
    }

    ecs.resources.insert(map);
    ecs.resources.insert(player_pos);
    ecs.resources.insert(PlayerEntity(player_entity));
    ecs.resources.insert(PlayerTarget::None);
    queues::register_queues(&mut ecs.resources);

    Ok(())
}

pub(crate) fn save(gs: &State, writer: &mut dyn Write) -> Result<()> {
    let map = &*resource_get!(gs.ecs, crate::resources::Map);
    let data = bincode::serialize(&map)?;
    writer.write_all(&data.len().to_le_bytes())?;
    writer.write_all(&data)?;

    let serializable = gs
        .ecs
        .world
        .as_serializable(legion::query::any(), &gs.registry);
    // let encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::fast());
    bincode_options().serialize_into(writer, &serializable)?;

    Ok(())
}

pub(crate) fn load(gs: &mut State, reader: &mut dyn Read) -> Result<()> {
    // let mut decoder = flate2::read::GzDecoder::new(reader);

    let map = {
        let mut data = [0_u8; 8];
        reader.read_exact(&mut data)?;
        let len = usize::from_le_bytes(data);
        let mut data = vec![0_u8; len];
        reader.read_exact(&mut data)?;
        bincode::deserialize(&data)?
    };

    gs.ecs.resources.insert(map);

    let mut deser = bincode::Deserializer::with_reader(reader, bincode_options());
    let registry = collect_registry();
    use serde::de::DeserializeSeed;
    let mut _world = registry.as_deserialize().deserialize(&mut deser).unwrap();

    Ok(())
}
