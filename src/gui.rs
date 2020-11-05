use crate::camera::Camera;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::Ecs;
use crate::{Direction, MapPosition, PlayerPosition, ScreenPosition};
use crate::{InventoryType, PlayerEntity};
use bracket_lib::prelude::*;
use legion::*;

const BOTTOM_HEIGHT: i32 = 7;

#[derive(PartialEq, Copy, Clone)]
pub(crate) enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub(crate) enum MainMenuState {
    New,
    Load,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub(crate) enum MainMenuResult {
    Selected(MainMenuState),
    NoSelection(MainMenuState),
}

pub(crate) fn key_to_dir(key: VirtualKeyCode) -> Option<Direction> {
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

pub(crate) fn index_to_letter(idx: u8) -> char {
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
pub(crate) struct TargetingInfo {
    range: i32,
    last_mouse_pos: (i32, i32),
    current_pos: (i32, i32),
}

impl TargetingInfo {
    pub fn new(range: i32, start_pos: ScreenPosition, ctx: &mut BTerm) -> Self {
        Self {
            range,
            last_mouse_pos: ctx.mouse_pos(),
            current_pos: start_pos.into(),
        }
    }

    pub fn show_targeting(
        &mut self,
        ecs: &mut Ecs,
        ctx: &mut BTerm,
    ) -> (ItemMenuResult, Option<MapPosition>) {
        if Some(VirtualKeyCode::Escape) == ctx.key {
            return (ItemMenuResult::Cancel, None);
        }

        if self.last_mouse_pos != ctx.mouse_pos() {
            println!("Mouse pos {:?} - {:?}", ctx.mouse_pos, ctx.width_pixels);
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

        ctx.print_color(
            5,
            0,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            "Select Target:",
        );

        let camera = *ecs.resources.get::<Camera>().unwrap();
        let player_entity = ecs.resources.get::<PlayerEntity>().unwrap().0;
        let player_entry = ecs.ecs.entry(player_entity).unwrap();

        // Highlight available target cells
        let mut available_cells = Vec::new();
        if let Ok(visible) = player_entry.into_component::<Viewshed>() {
            let player_pos = *ecs.resources.get::<PlayerPosition>().unwrap();
            // We have a viewshed
            for pos in visible.visible_tiles.iter() {
                let point = camera.transform_map_pos(*pos);
                let distance = DistanceAlg::Pythagoras
                    .distance2d(camera.transform_map_pos(player_pos.0).into(), point.into());
                if distance <= self.range as f32 {
                    ctx.set_bg(point.x, point.y, RGB::named(BLUE));
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
            ctx.set_bg(self.current_pos.0, self.current_pos.1, RGB::named(CYAN));

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
            ctx.set_bg(self.current_pos.0, self.current_pos.1, RGB::named(RED));
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

pub(crate) fn show_main_menu(ctx: &mut BTerm, current_state: MainMenuState) -> MainMenuResult {
    let (screen_width, screen_height) = ctx.get_char_size();
    let text_width = 7;
    let x = (screen_width / 2 - text_width / 2) as i32;
    let y = (screen_height / 2 - 2) as i32;

    ctx.print_color(
        (80 - 14) / 2,
        11,
        RGBA::named(YELLOW),
        RGBA::named(BLACK),
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
            RGB::named(ORANGERED2),
            RGB::named(BLACK),
            line,
        );
    }

    ctx.draw_box_double(
        x,
        y,
        text_width as i32,
        5,
        RGB::named(DEEPSKYBLUE),
        RGB::named(BLACK),
    );

    let x = x + 1;

    let black = RGB::named(BLACK);
    let white = RGB::named(WHITE);

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
        Some(VirtualKeyCode::Down) | Some(VirtualKeyCode::J) => {
            use MainMenuState::*;
            let current_state = match current_state {
                New => Load,
                Load => Quit,
                Quit => New,
            };
            MainMenuResult::NoSelection(current_state)
        }
        Some(VirtualKeyCode::Up) | Some(VirtualKeyCode::K) => {
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

pub(crate) fn ask_bool(ctx: &mut BTerm, question: &str) -> (ItemMenuResult, bool) {
    let width = question.len() as i32;

    let (screen_width, screen_height) = ctx.get_char_size();

    ctx.draw_box_double(
        screen_width as i32 / 2 - width - 1,
        screen_height as i32 / 2 - 2,
        width + 3,
        2,
        RGB::named(YELLOW),
        RGB::named(BLACK),
    );

    ctx.print_color(
        screen_width as i32 / 2 - width + 1,
        screen_height as i32 / 2 - 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        question,
    );

    match ctx.key {
        Some(VirtualKeyCode::Y) => (ItemMenuResult::Selected, true),
        Some(VirtualKeyCode::N) => (ItemMenuResult::Selected, false),
        Some(VirtualKeyCode::Escape) => (ItemMenuResult::Cancel, false),
        _ => (ItemMenuResult::NoResponse, false),
    }
}

pub(crate) fn show_inventory(
    ecs: &mut Ecs,
    ctx: &mut BTerm,
    inv_type: InventoryType,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = ecs.resources.get::<PlayerEntity>().unwrap();

    let mut query = <(Entity, &Name, &InBackpack, &ItemIndex)>::query();

    let mut inventory: Vec<_> = query
        .iter(&ecs.ecs)
        .filter(|item| item.2.owner == player_entity.0)
        .map(|(entity, name, _inbackpack, idx)| (*entity, idx.index, name))
        .collect();

    let count = inventory.len() as i32;
    inventory.sort_by(|a, b| a.1.cmp(&b.1));

    if count == 0 {
        let mut gamelog = ecs.resources.get_mut::<GameLog>().unwrap();
        gamelog.log("Your backpack is empty");
        return (ItemMenuResult::Cancel, None);
    }

    let mut y = 25 - (count / 2);
    ctx.draw_box(
        15,
        y - 2,
        31,
        count + 3,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    let title = match inv_type {
        InventoryType::Apply => "Use",
        InventoryType::Drop => "Drop",
    };
    ctx.print_color(18, y - 2, RGB::named(YELLOW), RGB::named(BLACK), title);
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    let mut items = std::collections::HashMap::new();
    for (entities, index, name) in inventory {
        items.insert(index, entities);
        ctx.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            to_cp437(index_to_letter(index)),
        );
        ctx.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selected = letter_to_option(key);
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

pub(crate) fn draw_ui(ecs: &Ecs, ctx: &mut BTerm) {
    let (screen_width, screen_height) = ctx.get_char_size();
    let (screen_width, screen_height) = (screen_width as i32, screen_height as i32);
    let bottom_start = screen_height - BOTTOM_HEIGHT;
    ctx.draw_box(
        0,
        bottom_start,
        screen_width - 1,
        6,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );

    let mut query = <(&CombatStats, &Player)>::query();
    for (stats, _player) in query.iter(&ecs.ecs) {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            bottom_start,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            &health,
        );

        let bar_color = if stats.hp < stats.max_hp / 3 {
            RGB::named(RED)
        } else if stats.hp < 3 * stats.max_hp / 4 {
            RGB::named(YELLOW)
        } else {
            RGB::named(GREEN)
        };

        ctx.draw_bar_horizontal(
            28,
            bottom_start,
            51,
            stats.hp,
            stats.max_hp,
            bar_color,
            RGB::named(BLACK),
        );
    }

    let gamelog = ecs.resources.get::<GameLog>().unwrap();
    for (i, entry) in gamelog.entries.iter().rev().enumerate() {
        if i > 4 {
            break;
        }
        ctx.print(1, screen_height - 2 - i as i32, entry);
    }

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &Ecs, ctx: &mut BTerm) {
    let camera = *ecs.resources.get::<Camera>().unwrap();
    let map = ecs.resources.get::<Map>().unwrap();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    let mut query = <(&Name, &Position)>::query();
    for (name, position) in query.iter(&ecs.ecs) {
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
                    RGB::named(WHITE),
                    RGB::named(GREY),
                    &s.to_string(),
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
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
                    RGB::named(WHITE),
                    RGB::named(GREY),
                    &s.to_string(),
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"<-".to_string(),
            );
        }
    }
}
