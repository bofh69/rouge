use langgen_english::*;

#[derive(Debug, Clone, Copy)]
pub struct EntNum(pub i32);

pub struct DebugEntityAdapter {
    pub buffer: String,
}

impl<'a> EntityAdapter<'a, EntNum> for DebugEntityAdapter {
    fn can_see(&self, who: EntNum, obj: EntNum) -> bool {
        who.0 & 7 < obj.0 & 7
    }

    fn is_me(&self, obj: EntNum) -> bool {
        2 == obj.0 % 7
    }

    fn gender(&self, obj: EntNum) -> Gender {
        match obj.0 & 7 {
            0 => Gender::Male,
            1 => Gender::Female,
            2 => Gender::Neuter,
            3 => Gender::Plural,
            4 => Gender::Uncountable,
            _ => panic!("Impossible!"),
        }
    }

    fn is_thing(&self, obj: EntNum) -> bool {
        obj.0 & 8 == 8
    }

    fn has_short_proper(&self, obj: EntNum) -> bool {
        obj.0 & 16 == 16
    }

    fn short_name(&self, obj: EntNum) -> &'a str {
        if self.is_thing(obj) {
            if let Gender::Plural = self.gender(obj) {
                &"ball"
            } else if let Gender::Uncountable = self.gender(obj) {
                &"water"
            } else {
                &"apple"
            }
        } else {
            match obj.0 % 6 {
                0 => &"Ove",
                1 => &"Eva",
                2 => &"Kim",
                3 => &"gnomes",
                4 => &"water",
                _ => panic!("Impossible!"),
            }
        }
    }

    fn has_long_proper(&self, obj: EntNum) -> bool {
        obj.0 & 64 == 64
    }
    fn long_name(&self, obj: EntNum) -> &'a str {
        if self.is_thing(obj) {
            if let Gender::Plural = self.gender(obj) {
                &"blue ball"
            } else if let Gender::Uncountable = self.gender(obj) {
                &"calm water"
            } else {
                &"red apple"
            }
        } else {
            match obj.0 % 6 {
                0 => &"Ove Thörnkvist",
                1 => &"Eva Adamsfru",
                2 => &"Kim Kimmy",
                3 => &"band of gnomes",
                4 => &"gold",
                _ => panic!("Impossible!"),
            }
        }
    }

    fn has_short_plural_proper(&self, obj: EntNum) -> bool {
        if self.is_thing(obj) {
            false
        } else {
            match obj.0 % 6 {
                0 => true,
                1 => true,
                2 => true,
                3 => false,
                4 => false,
                _ => panic!("Impossible!"),
            }
        }
    }

    fn short_plural_name(&self, obj: EntNum) -> &'a str {
        if self.is_thing(obj) {
            if let Gender::Plural = self.gender(obj) {
                &"balls"
            } else if let Gender::Uncountable = self.gender(obj) {
                &"waters"
            } else {
                &"apples"
            }
        } else {
            match obj.0 % 6 {
                0 => &"Ovar",
                1 => &"Evor",
                2 => &"Kimies",
                3 => &"gnomes",
                4 => &"gold",
                _ => panic!("Impossible!"),
            }
        }
    }

    fn has_long_plural_proper(&self, obj: EntNum) -> bool {
        if self.is_thing(obj) {
            if let Gender::Plural = self.gender(obj) {
                true
            } else if let Gender::Uncountable = self.gender(obj) {
                false
            } else {
                false
            }
        } else {
            match obj.0 % 6 {
                0 => false,
                1 => false,
                2 => false,
                3 => true,
                4 => false,
                _ => panic!("Impossible!"),
            }
        }
    }

    fn long_plural_name(&self, obj: EntNum) -> &'a str {
        if self.is_thing(obj) {
            if let Gender::Plural = self.gender(obj) {
                &"blue blue balls"
            } else if let Gender::Uncountable = self.gender(obj) {
                &"calm waters"
            } else {
                &"red apples"
            }
        } else {
            match obj.0 % 6 {
                0 => &"Ove Thörnkvistar",
                1 => &"Eva Adamsfruar",
                2 => &"Kim Kimmies",
                3 => &"bands of gnomes",
                4 => &"gold",
                _ => panic!("Impossible!"),
            }
        }
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
