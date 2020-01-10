use crate::camera::Camera;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::{Direction, MapPosition, PlayerPosition, ScreenPosition};
use crate::{InventoryType, PlayerEntity};
use rltk::{Console, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

const BOTTOM_HEIGHT: i32 = 7;

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuState {
    New,
    Load,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    Selected(MainMenuState),
    NoSelection(MainMenuState),
}

pub fn key_to_dir(key: VirtualKeyCode) -> Option<Direction> {
    match key {
        VirtualKeyCode::H | VirtualKeyCode::Left => Some(Direction::West),
        VirtualKeyCode::L | VirtualKeyCode::Right => Some(Direction::East),
        VirtualKeyCode::K | VirtualKeyCode::Up => Some(Direction::North),
        VirtualKeyCode::J | VirtualKeyCode::Down => Some(Direction::South),
        VirtualKeyCode::Y => Some(Direction::NorthWest),
        VirtualKeyCode::U => Some(Direction::NorthEast),
        VirtualKeyCode::B => Some(Direction::SouthWest),
        VirtualKeyCode::N => Some(Direction::SouthEast),
        _ => None,
    }
}

pub fn index_to_letter(idx: u8) -> char {
    if idx > 25 {
        index_to_letter(idx - 26).to_ascii_uppercase()
    } else {
        match idx {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            8 => 'i',
            9 => 'j',
            10 => 'k',
            11 => 'l',
            12 => 'm',
            13 => 'n',
            14 => 'o',
            15 => 'p',
            16 => 'q',
            17 => 'r',
            18 => 's',
            19 => 't',
            20 => 'u',
            21 => 'b',
            22 => 'w',
            23 => 'x',
            24 => 'y',
            25 => 'z',
            _ => panic!("Too large index"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct TargetingInfo {
    range: i32,
    last_mouse_pos: (i32, i32),
    current_pos: (i32, i32),
}

impl TargetingInfo {
    pub fn new(range: i32, start_pos: ScreenPosition, ctx: &mut Rltk) -> Self {
        Self {
            range,
            last_mouse_pos: ctx.mouse_pos(),
            current_pos: start_pos.into(),
        }
    }

    pub fn show_targeting(
        &mut self,
        ecs: &mut World,
        ctx: &mut Rltk,
    ) -> (ItemMenuResult, Option<MapPosition>) {
        if Some(VirtualKeyCode::Escape) == ctx.key {
            return (ItemMenuResult::Cancel, None);
        }

        if self.last_mouse_pos != ctx.mouse_pos() {
            self.last_mouse_pos = ctx.mouse_pos();
            self.current_pos = ctx.mouse_pos();
        } else if ctx.left_click {
            self.current_pos = ctx.mouse_pos();
        } else if let Some(key) = ctx.key {
            if let Some(dir) = key_to_dir(key) {
                let (dx, dy) = dir.into();
                let temp_pos = (self.current_pos.0 + dx, self.current_pos.1 + dy);
                let (screen_width, screen_height) = ctx.get_char_size();
                if temp_pos.0 >= 0
                    && temp_pos.0 < screen_width as i32
                    && temp_pos.1 >= 0
                    && temp_pos.1 < (screen_height as i32 - BOTTOM_HEIGHT)
                {
                    self.current_pos = temp_pos;
                }
            }
        }

        let camera = ecs.fetch::<Camera>();
        let player_pos = ecs.fetch::<PlayerPosition>();
        let player_entity = ecs.fetch::<PlayerEntity>().0;
        let viewsheds = ecs.read_storage::<Viewshed>();

        ctx.print_color(
            5,
            0,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            "Select Target:",
        );

        // Highlight available target cells
        let mut available_cells = Vec::new();
        let visible = viewsheds.get(player_entity);
        if let Some(visible) = visible {
            // We have a viewshed
            for pos in visible.visible_tiles.iter() {
                let point = camera.transform_map_pos(*pos);
                let distance = rltk::DistanceAlg::Pythagoras
                    .distance2d(camera.transform_map_pos(player_pos.0).into(), point.into());
                if distance <= self.range as f32 {
                    ctx.set_bg(point.x, point.y, RGB::named(rltk::BLUE));
                    available_cells.push(point);
                }
            }
        } else {
            return (ItemMenuResult::Cancel, None);
        }

        // Draw mouse cursor
        let mut valid_target = false;
        for idx in available_cells.iter() {
            if idx.x == self.current_pos.0 && idx.y == self.current_pos.1 {
                valid_target = true;
            }
        }
        if valid_target {
            ctx.set_bg(
                self.current_pos.0,
                self.current_pos.1,
                RGB::named(rltk::CYAN),
            );

            match (ctx.key, ctx.left_click) {
                (_, true)
                | (Some(VirtualKeyCode::Return), _)
                | (Some(VirtualKeyCode::Space), _) => {
                    return (
                        ItemMenuResult::Selected,
                        Some(camera.transform_screen_pos(ScreenPosition {
                            x: self.current_pos.0,
                            y: self.current_pos.1,
                        })),
                    );
                }
                _ => (),
            }
        } else {
            ctx.set_bg(
                self.current_pos.0,
                self.current_pos.1,
                RGB::named(rltk::RED),
            );
            match (ctx.key, ctx.left_click) {
                (Some(VirtualKeyCode::Return), _) | (_, true) => {
                    return (ItemMenuResult::Cancel, None)
                }
                _ => (),
            }
        }

        (ItemMenuResult::NoResponse, None)
    }
}

pub fn show_main_menu(ctx: &mut Rltk, current_state: MainMenuState) -> MainMenuResult {
    let (screen_width, screen_height) = ctx.get_char_size();
    let text_width = 7;
    let x = (screen_width / 2 - text_width / 2) as i32;
    let y = (screen_height / 2 - 2) as i32;

    ctx.print_color(
        (80 - 14) / 2,
        11,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Welcome to ...",
    );

    for (y, line) in vec![
        ".########...#######..##.....##..######...########",
        ".##.....##.##.....##.##.....##.##....##..##......",
        ".##.....##.##.....##.##.....##.##........##......",
        ".########..##.....##.##.....##.##...####.######..",
        ".##...##...##.....##.##.....##.##....##..##......",
        ".##....##..##.....##.##.....##.##....##..##......",
        ".##.....##..#######...#######...######...########",
    ]
    .iter()
    .enumerate()
    {
        ctx.print_color(
            15,
            14 + y as i32,
            RGB::named(rltk::ORANGERED2),
            RGB::named(rltk::BLACK),
            line,
        );
    }

    ctx.draw_box_double(
        x,
        y,
        text_width as i32,
        5,
        RGB::named(rltk::DEEPSKYBLUE),
        RGB::named(rltk::BLACK),
    );

    let x = x + 1;

    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);

    let (fg, bg) = if current_state == MainMenuState::New {
        (black, white)
    } else {
        (white, black)
    };
    ctx.print_color(x, y + 1, fg, bg, " New  ");
    let (fg, bg) = if current_state == MainMenuState::Load {
        (black, white)
    } else {
        (white, black)
    };
    ctx.print_color(x, y + 2, fg, bg, " Load ");
    let (fg, bg) = if current_state == MainMenuState::Quit {
        (black, white)
    } else {
        (white, black)
    };
    ctx.print_color(x, y + 4, fg, bg, " Quit ");

    match ctx.key {
        Some(rltk::VirtualKeyCode::Down) | Some(rltk::VirtualKeyCode::J) => {
            use MainMenuState::*;
            let current_state = match current_state {
                New => Load,
                Load => Quit,
                Quit => New,
            };
            MainMenuResult::NoSelection(current_state)
        }
        Some(rltk::VirtualKeyCode::Up) | Some(rltk::VirtualKeyCode::K) => {
            use MainMenuState::*;
            let current_state = match current_state {
                New => Quit,
                Load => New,
                Quit => Load,
            };
            MainMenuResult::NoSelection(current_state)
        }
        Some(VirtualKeyCode::Return) | Some(VirtualKeyCode::Space) => {
            MainMenuResult::Selected(current_state)
        }
        _ => MainMenuResult::NoSelection(current_state),
    }
}

pub fn ask_bool(ctx: &mut Rltk, question: &str) -> (ItemMenuResult, bool) {
    let width = question.len() as i32;

    let (screen_width, screen_height) = ctx.get_char_size();

    ctx.draw_box_double(
        screen_width as i32 / 2 - width - 1,
        screen_height as i32 / 2 - 2,
        width + 3,
        2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
    );

    ctx.print_color(
        screen_width as i32 / 2 - width + 1,
        screen_height as i32 / 2 - 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        question,
    );

    match ctx.key {
        Some(VirtualKeyCode::Y) => (ItemMenuResult::Selected, true),
        Some(VirtualKeyCode::N) => (ItemMenuResult::Selected, false),
        Some(VirtualKeyCode::Escape) => (ItemMenuResult::Cancel, false),
        _ => (ItemMenuResult::NoResponse, false),
    }
}

pub fn show_inventory(
    ecs: &mut World,
    ctx: &mut Rltk,
    inv_type: InventoryType,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = ecs.fetch::<PlayerEntity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let item_index = ecs.read_storage::<ItemIndex>();
    let entities = ecs.entities();

    let mut inventory: Vec<_> = (&entities, &backpack, &item_index, &names)
        .join()
        .filter(|item| item.1.owner == player_entity.0)
        .map(|(entity, _item, idx, name)| (entity, idx.index, name))
        .collect();
    let count = inventory.len() as i32;
    inventory.sort_by(|a, b| a.1.cmp(&b.1));

    if count == 0 {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.log("Your backpack is empty");
        return (ItemMenuResult::Cancel, None);
    }

    let mut y = 25 - (count / 2);
    ctx.draw_box(
        15,
        y - 2,
        31,
        count + 3,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    let title = match inv_type {
        InventoryType::Apply => "Use",
        InventoryType::Drop => "Drop",
    };
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        title,
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut items = std::collections::HashMap::new();
    for (entities, index, name) in inventory {
        items.insert(index, entities);
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(index_to_letter(index)),
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selected = rltk::letter_to_option(key);
                if selected < 0 {
                    (ItemMenuResult::NoResponse, None)
                } else {
                    let mut selected = selected as u8;
                    if ctx.shift {
                        selected += 27u8;
                    }
                    if !items.contains_key(&selected) {
                        (ItemMenuResult::NoResponse, None)
                    } else {
                        (
                            ItemMenuResult::Selected,
                            Some(*items.get(&selected).unwrap()),
                        )
                    }
                }
            }
        },
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let (screen_width, screen_height) = ctx.get_char_size();
    let (screen_width, screen_height) = (screen_width as i32, screen_height as i32);
    let bottom_start = screen_height - BOTTOM_HEIGHT;
    ctx.draw_box(
        0,
        bottom_start,
        screen_width - 1,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            bottom_start,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );

        let bar_color = if stats.hp < stats.max_hp / 3 {
            RGB::named(rltk::RED)
        } else if stats.hp < 3 * stats.max_hp / 4 {
            RGB::named(rltk::YELLOW)
        } else {
            RGB::named(rltk::GREEN)
        };

        ctx.draw_bar_horizontal(
            28,
            bottom_start,
            51,
            stats.hp,
            stats.max_hp,
            bar_color,
            RGB::named(rltk::BLACK),
        );
    }
    let gamelog = ecs.fetch::<GameLog>();
    for (i, entry) in gamelog.entries.iter().rev().enumerate() {
        if i > 4 {
            break;
        }
        ctx.print(1, screen_height - 2 - i as i32, entry);
    }

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let camera = *(ecs.fetch::<Camera>());
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let pos = camera.transform_map_pos(position.0);
        if pos.x == mouse_pos.0 && pos.y == mouse_pos.1 {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    &s.to_string(),
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    &s.to_string(),
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    }
}
