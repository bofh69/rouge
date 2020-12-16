use ::serde::*;
use bracket_lib::prelude::{BTerm, BLACK, WHITE};
use legion::Entity;
use std::collections::VecDeque;
use std::sync::Mutex;

pub(crate) type QueueAdapter = Mutex<VecDeque<langgen_english::FragmentEntry<Entity>>>;
pub(crate) type OutputQueue = langgen_english::OutputQueue<Entity, QueueAdapter>;

#[derive(Clone, Serialize, Deserialize)]
enum LogEntry {
    Text(String),
    Color((u8, u8, u8)),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GameLog {
    entries: Vec<Vec<LogEntry>>,
    current_line: Vec<LogEntry>,
}

impl GameLog {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            current_line: vec![],
        }
    }

    pub fn write_text<T: Into<String>>(&mut self, s: T) {
        self.current_line.push(LogEntry::Text(s.into()));
    }

    pub fn set_color(&mut self, color: (u8, u8, u8)) {
        self.current_line.push(LogEntry::Color(color));
    }

    pub fn end_of_line(&mut self) {
        self.entries.push(self.current_line.clone());
        self.current_line.clear();
    }

    pub fn draw_log(&self, ctx: &mut BTerm, last_line: u32, rows: u32) {
        let rows = rows as usize;
        let l = self.entries.len();
        let entries = if rows < l {
            let (_, entries) = self.entries.split_at(l - rows);
            entries
        } else {
            &self.entries
        };
        for (i, line) in entries.iter().rev().enumerate() {
            let y = last_line as i32 - i as i32;
            let mut x = 1;
            let mut color = WHITE;

            for entry in line {
                match entry {
                    LogEntry::Text(text) => {
                        ctx.print_color(x, y, color, BLACK, text);
                        x += text.chars().count();
                    }
                    LogEntry::Color(c) => {
                        color = *c;
                    }
                }
            }
        }
    }
}
