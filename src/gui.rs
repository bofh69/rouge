use crate::camera::Camera;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::MapPosition;
use crate::PlayerPosition;
use crate::ScreenPosition;
use crate::{InventoryType, PlayerEntity, State};
use rltk::{Console, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
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

pub fn show_targeting(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<MapPosition>) {
    if Some(VirtualKeyCode::Escape) == ctx.key {
        return (ItemMenuResult::Cancel, None);
    }

    let camera = gs.ecs.fetch::<Camera>();
    let player_pos = gs.ecs.fetch::<PlayerPosition>();
    let player_entity = gs.ecs.fetch::<PlayerEntity>().0;
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

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
            let point = camera.transform_map_pos(pos);
            let distance = rltk::DistanceAlg::Pythagoras
                .distance2d(camera.transform_map_pos(&player_pos.0).into(), point.into());
            if distance <= range as f32 {
                ctx.set_bg(point.x, point.y, RGB::named(rltk::BLUE));
                available_cells.push(point);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(camera.transform_screen_pos(&ScreenPosition {
                    x: mouse_pos.0,
                    y: mouse_pos.1,
                })),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn show_inventory(
    gs: &mut State,
    ctx: &mut Rltk,
    inv_type: InventoryType,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<PlayerEntity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == player_entity.0);
    let count = inventory.count() as i32;

    if count == 0 {
        let mut gamelog = gs.ecs.fetch_mut::<GameLog>();
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

    let mut j = 0;
    let mut items = vec![];
    for (entities, _pack, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == player_entity.0)
    {
        items.push(entities);
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
            97 + j as u8,
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
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selected = rltk::letter_to_option(key);
                if selected < 0 || selected >= count as i32 {
                    (ItemMenuResult::NoResponse, None)
                } else {
                    (ItemMenuResult::Selected, Some(items[selected as usize]))
                }
            }
        },
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
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
            43,
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
            43,
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
        ctx.print(1, 48 - i as i32, entry);
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
        let pos = camera.transform_map_pos(&position.0);
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
