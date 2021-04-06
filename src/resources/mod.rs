mod camera;
mod gamelog;
mod map;

pub(crate) use camera::*;
pub(crate) use gamelog::*;
pub(crate) use map::*;

use crate::ecs::*;

use ::bracket_lib::prelude::RandomNumberGenerator;
use ::bracket_lib::prelude::RED;
use ::legion::Entity;
use ::serde::*;
use ::std::collections::VecDeque;
use ::std::io::Read;
use ::std::io::Write;
use ::std::sync::Mutex;

use crate::components::Position;
use crate::positions::{Direction, MapPosition};

#[derive(Serialize, Deserialize)]
pub(crate) struct PlayerEntity(pub Entity);

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum PlayerTarget {
    None,
    Position(MapPosition),
    Dir(Direction),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) struct PlayerPosition(pub MapPosition);

impl From<PlayerPosition> for Position {
    fn from(pos: PlayerPosition) -> Self {
        Self(pos.0)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct Time {
    pub real_time_ms: i64,
    pub last_real_time_ms: i64,
    pub tick: i64,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static + Send + Sync>>;

pub(crate) fn new(ecs: &mut Ecs) {
    ecs.resources.insert(PlayerTarget::None);

    let map = Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();

    ecs.resources.insert(RandomNumberGenerator::new());
    ecs.resources.insert(GameLog::new());

    for room in map.rooms.iter().skip(1) {
        crate::spawner::spawn_room(ecs, room);
    }
    let player_entity = crate::spawner::player(ecs, player_pos.x, player_pos.y);

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
    crate::queues::register_queues(&mut ecs.resources);
}

fn save_resource<T: 'static + serde::Serialize>(ecs: &Ecs, writer: &mut dyn Write) -> Result<()> {
    let obj = &*resource_get!(ecs, T);
    let data = bincode::serialize(&obj)?;
    writer.write_all(&data.len().to_le_bytes())?;
    writer.write_all(&data)?;

    Ok(())
}

pub(crate) fn save(ecs: &Ecs, writer: &mut dyn Write) -> Result<()> {
    save_resource::<PlayerTarget>(ecs, writer)?;
    save_resource::<Map>(ecs, writer)?;
    save_resource::<RandomNumberGenerator>(ecs, writer)?;
    save_resource::<GameLog>(ecs, writer)?;
    // save_resource::<'static, OutputQueue>(ecs, writer)?;
    save_resource::<Camera>(ecs, writer)?;
    save_resource::<PlayerPosition>(ecs, writer)?;
    // save_resource::<PlayerEntity>(ecs, writer)?;

    // crate::queues::register_queues(&mut ecs.resources);

    Ok(())
}

fn load_resource<'de, T: 'static + Deserialize<'de>>(
    ecs: &mut Ecs,
    reader: &mut dyn Read,
) -> Result<()> {
    let mut data = [0_u8; 8];
    reader.read_exact(&mut data)?;
    let len = usize::from_le_bytes(data);
    let mut data = Box::new(vec![0_u8; len]);
    reader.read_exact(&mut data)?;
    dbg!("Loading resource ");
    let obj = bincode::deserialize::<T>(data.leak())?;

    dbg!("Loaded resource ");

    ecs.resources.insert::<T>(obj);

    Ok(())
}

pub(crate) fn load(ecs: &mut Ecs, reader: &mut dyn Read) -> Result<()> {
    load_resource::<PlayerTarget>(ecs, reader)?;
    load_resource::<Map>(ecs, reader)?;
    load_resource::<RandomNumberGenerator>(ecs, reader)?;
    load_resource::<GameLog>(ecs, reader)?;
    load_resource::<Camera>(ecs, reader)?;
    load_resource::<PlayerPosition>(ecs, reader)?;

    Ok(())
}
