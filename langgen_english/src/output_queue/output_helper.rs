use crate::Gender;

pub(crate) fn last_char(s: &str) -> Option<char> {
    s.chars().next_back()
}

pub(crate) fn needs_dot(s: &str) -> bool {
    if let Some(c) = last_char(s) {
        !matches!(c, '.' | '?' | '!' | ':' | ';' | '"')
    } else {
        false
    }
}

// Used to decide between a/an.
pub(crate) fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U')
    // y is usually not pronounced like a vowel.
}

pub(crate) fn uppercase_first_char(s: &str, to: &mut String) {
    let mut c = s.chars();
    if let Some(ch) = c.next() {
        for ch in ch.to_uppercase() {
            to.push(ch);
        }
        to.push_str(c.as_str());
    }
}

pub(crate) fn is_singular(gender: Gender) -> bool {
    !matches!(gender, Gender::Plural | Gender::Uncountable)
}

pub(crate) fn add_verb_end_s(str: &mut String) {
    let mut add: &str = "";
    let mut uc = false;
    let mut remove = 0;

    {
        let mut ci = str.chars().rev();
        if let Some(ch) = ci.next() {
            if ch.is_uppercase() {
                uc = true;
            }
            add = match ch {
                's' | 'o' | 'z' | 'x' | 'S' | 'O' | 'Z' | 'X' => "es",
                'y' | 'Y' => {
                    remove = 1;

                    ci.next().map_or("ies", |c2| if is_vowel(c2) {
                            remove = 0;
                            "s"
                        } else {
                            "ies"
                        })
                }
                'h' | 'H' => {

                    ci.next().map_or("s", |c2| if c2 == 'c' || c2 == 's' || c2 == 'C' || c2 == 'S' {
                            "es"
                        } else {
                            "s"
                        })
                }
                _ => "s",
            }
        }
        while remove > 0 {
            str.pop();
            remove -= 1;
        }
        if uc {
            str.push_str(&add.to_uppercase());
        } else {
            str.push_str(add);
        }
    }
}
