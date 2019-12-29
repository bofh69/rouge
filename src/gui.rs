use crate::components::*;
use rltk::{Console, Rltk, RGB};
use specs::prelude::*;

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
}
