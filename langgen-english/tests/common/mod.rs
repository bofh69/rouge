use langgen_english::*;
use std::cell::Cell;

#[derive(Debug, Clone, Copy)]
pub struct EntNum(pub i32);

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
    pub mock_has_short_plural_proper: bool,
    pub mock_short_plural_name: &'static str,
    pub mock_has_long_plural_proper: bool,
    pub mock_long_plural_name: &'static str,

    pub last_who: Cell<EntNum>,
    pub last_obj: Cell<EntNum>,
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
            mock_has_short_plural_proper: true,
            mock_short_plural_name: "Kims",
            mock_has_long_plural_proper: false,
            mock_long_plural_name: "spirits of Kim",
            last_obj: Cell::new(EntNum(-1)),
            last_who: Cell::new(EntNum(-2)),
        }
    }
}

impl<'a> EntityAdapter<'a, EntNum> for DebugEntityAdapter {
    fn can_see(&self, who: EntNum, obj: EntNum) -> bool {
        self.last_who.set(who);
        self.last_obj.set(obj);
        self.mock_can_see
    }

    fn is_me(&self, obj: EntNum) -> bool {
        self.last_obj.set(obj);
        self.mock_is_me
    }

    fn gender(&self, obj: EntNum) -> Gender {
        self.last_obj.set(obj);
        self.mock_gender.clone()
    }

    fn is_thing(&self, obj: EntNum) -> bool {
        self.last_obj.set(obj);
        self.mock_is_thing
    }

    fn has_short_proper(&self, obj: EntNum) -> bool {
        self.last_obj.set(obj);
        self.mock_has_short_proper
    }

    fn short_name(&self, obj: EntNum) -> &'a str {
        self.last_obj.set(obj);
        self.mock_short_name
    }

    fn has_long_proper(&self, obj: EntNum) -> bool {
        self.last_obj.set(obj);
        self.mock_has_long_proper
    }
    fn long_name(&self, obj: EntNum) -> &'a str {
        self.last_obj.set(obj);
        self.mock_long_name
    }

    fn has_short_plural_proper(&self, obj: EntNum) -> bool {
        self.last_obj.set(obj);
        self.mock_has_short_plural_proper
    }

    fn short_plural_name(&self, obj: EntNum) -> &'a str {
        self.last_obj.set(obj);
        self.mock_short_plural_name
    }

    fn has_long_plural_proper(&self, obj: EntNum) -> bool {
        self.last_obj.set(obj);
        self.mock_has_long_plural_proper
    }

    fn long_plural_name(&self, obj: EntNum) -> &'a str {
        self.last_obj.set(obj);
        self.mock_long_plural_name
    }

    fn write_text(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    fn set_color(&mut self, color: (i32, i32, i32)) {
        self.write_text(&format!("{:?}", color));
    }

    fn done(&mut self) {
        self.write_text(&"\n");
    }
}
