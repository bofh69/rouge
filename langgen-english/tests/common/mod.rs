use langgen_english::*;
use std::cell::Cell;

pub struct DebugEntityAdapter {
    pub buffer: String,
    pub mock_can_see: bool,
    pub mock_is_me: bool,
    pub mock_gender: Gender,
    pub mock_is_thing: bool,
    pub mock_has_short_proper: bool,
    pub mock_short_name: &'static str,
    pub mock_has_long_proper: bool,
    pub mock_long_name: &'static str,
    pub mock_short_plural_name: &'static str,
    pub mock_long_plural_name: &'static str,

    pub last_who: Cell<i32>,
    pub last_obj: Cell<i32>,
}

impl DebugEntityAdapter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            mock_can_see: true,
            mock_is_me: false,
            mock_gender: Gender::Neuter,
            mock_is_thing: false,
            mock_has_short_proper: true,
            mock_short_name: "Kim",
            mock_has_long_proper: false,
            mock_long_name: "spirit of Kim",
            mock_short_plural_name: "Kims",
            mock_long_plural_name: "spirits of Kim",
            last_obj: Cell::new(-1),
            last_who: Cell::new(-2),
        }
    }
}

impl EntityAdapter<i32> for DebugEntityAdapter {
    fn can_see(&self, who: i32, obj: i32) -> bool {
        self.last_who.set(who);
        self.last_obj.set(obj);
        self.mock_can_see
    }

    fn is_me(&self, obj: i32) -> bool {
        self.last_obj.set(obj);
        self.mock_is_me
    }

    fn gender(&self, obj: i32) -> Gender {
        self.last_obj.set(obj);
        self.mock_gender.clone()
    }

    fn is_thing(&self, obj: i32) -> bool {
        self.last_obj.set(obj);
        self.mock_is_thing
    }

    fn has_short_proper(&self, obj: i32) -> bool {
        self.last_obj.set(obj);
        self.mock_has_short_proper
    }

    fn append_short_name(&self, obj: i32, s: &mut String) {
        self.last_obj.set(obj);

        s.push_str(self.mock_short_name);
    }

    fn has_long_proper(&self, obj: i32) -> bool {
        self.last_obj.set(obj);
        self.mock_has_long_proper
    }
    fn append_long_name(&self, obj: i32, s: &mut String) {
        self.last_obj.set(obj);
        s.push_str(self.mock_long_name);
    }

    fn append_short_plural_name(&self, obj: i32, s: &mut String) {
        self.last_obj.set(obj);
        s.push_str(self.mock_short_plural_name);
    }

    fn append_long_plural_name(&self, obj: i32, s: &mut String) {
        self.last_obj.set(obj);
        s.push_str(self.mock_long_plural_name);
    }

    fn write_text(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    fn set_color(&mut self, color: (u8, u8, u8)) {
        self.write_text(&format!("{:?}", color));
    }

    fn done(&mut self) {
        self.write_text(&"\n");
    }
}
