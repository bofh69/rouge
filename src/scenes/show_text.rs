use bracket_lib::prelude::*;

pub(crate) struct ShowText {
    text: &'static str,
}

impl<T> super::Scene<T> for ShowText {
    fn tick(&mut self, _: &mut T, ctx: &mut BTerm) -> super::SceneResult<T> {
        let (width, height) = ctx.get_char_size();

        let mut draw_batch = DrawBatch::new();

        draw_batch.draw_double_box(
            Rect::with_size(8, 1, width as u32 - 18, height as u32 - 3),
            ColorPair::new(BLUE, BLACK),
        );
        draw_batch.print_color_centered(1, "HELP", ColorPair::new(YELLOW, BLUE));
        let mut block = TextBlock::new(10, 3, width as i32 - 20, height as i32 - 5);
        let mut buf = TextBuilder::empty();
        buf.fg(GREEN).bg(BLACK);

        for line in self.text.split('\n') {
            buf.append(line).ln();
        }
        let _ = block.print(&buf);
        block.render_to_draw_batch(&mut draw_batch);

        draw_batch.print_color_centered(
            height as i32 - 2,
            "Press ENTER",
            ColorPair::new(YELLOW, BLUE),
        );

        draw_batch.submit(0).unwrap();
        render_draw_buffer(ctx).unwrap();

        if let Some(VirtualKeyCode::Return) = ctx.key {
            super::SceneResult::Pop
        } else {
            super::SceneResult::Continue
        }
    }
}

impl ShowText {
    pub(crate) fn new(text: &'static str) -> Self {
        Self { text }
    }
}
