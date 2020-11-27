mod output_builder;
mod output_helper;

use std::sync::Mutex;
use crate::traits::EntityAdapter;
use crate::traits::QueueAdapter;
use crate::FragmentEntry;
pub use output_builder::*;
use output_helper::*;
use std::marker::PhantomData;

/// `OutputQueue` is used to create sentances that should be sent to a player.
///
/// It can adapt the sentances based on the involved objects' genders,
/// plural/uncountability and if the player can see them or not.
///
///
/// The first word in a sentance will become capitalized, unless `supress_capitalize` is called before.
///
/// At the end of a sentance, a dot will be added if the message didn't already end with a punctuation mark.
/// Use `supress_dot` to supress it.
///
/// Between each fragment a space is normally added. The next space can be supressed by called `supress_space`.
/// If the text being added starts with ',' or '"' no space will be added before it.
/// If the text being added ends with '"' no space will be added after it, use "\" ", if needed.
///
pub struct OutputQueue<Entity, QA>
where
    Entity: Copy,
    QA: QueueAdapter<Entity>,
{
    queue_adapter: QA,
    supress_dot: bool,
    supress_capitalize: bool,
    add_space: bool,
    output_string: String,
    clear_output: bool,
    who: Entity,
    lock: Mutex<()>,
}

impl<Entity, QA> OutputQueue<Entity, QA>
where
    QA: QueueAdapter<Entity>,
    Entity: std::fmt::Debug + Copy,
{
    /// Create a new `OutputQueue` for player `who`, using
    /// the given [`QueueAdapter`](QueueAdapter)
    /// to store messages before they are processed
    /// with `process_queue`.
    pub fn new(queue_adapter: QA, who: Entity) -> Self {
        Self {
            queue_adapter,
            supress_dot: false,
            supress_capitalize: false,
            add_space: false,
            output_string: String::new(),
            clear_output: false,
            who,
            lock: Mutex::new(()),
        }
    }

    fn make_output_builder(&self) -> OutputBuilder<'_, QA, Entity> {
        OutputBuilder::new(&self.queue_adapter, self.lock.lock())
    }

    /// Output a/an short-name, Proper-name or something/someone/some/you.
    pub fn a(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().a(who)
    }

    /// Output a/an long name, Proper-long-name or something/someone/some/you.
    pub fn a_(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().a_(who)
    }

    /// Output the short-name, Proper-name or something/someone/some/you.
    pub fn the(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().the(who)
    }

    /// Output the long-name, Proper-name or something/someone/some/you.
    pub fn the_(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().the_(who)
    }

    /// Output "the <object-short-name>'s".
    /// If the viewer can't see it, "something's"/"someone's" is used.
    pub fn thes(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thes(who)
    }

    /// Output "the <object-long-name>'s".
    /// If the viewer can't see it, "something's"/"someone's" is used.
    pub fn thes_(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thes_(who)
    }

    /// Output "yours"/"the <object-short-name>'s".
    /// If the viewer can't see it, "something's"/"someone's" is used.
    pub fn thess(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thess(who)
    }

    /// Output "yours"/"the <object-long-name>'s".
    /// If the viewer can't see it, "something's"/"someone's" is used.
    pub fn thess_(&self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thess_(who)
    }

    /// Output "my/his/her/their/its <object-short-name>".
    /// If the viewer can't see it, a() is used instead.
    pub fn my(&self, who: Entity, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().my(who, obj)
    }

    /// Output "my/his/her/their/its <object-long-name>".
    /// If the viewer can't see it, a_() is used instead.
    pub fn my_(&self, who: Entity, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().my_(who, obj)
    }

    /// Output "you"/<objects-short-name>.
    /// If the viewer can't see it, "something"/"someone" is used.
    pub fn word(&self, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().word(obj)
    }

    /// Output "you"/<objects-long-name>.
    /// If the viewer can't see it, "something"/"someone" is used.
    pub fn word_(&self, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().word_(obj)
    }

    /// Output "is"/"are".
    pub fn is(&self, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().is(obj)
    }

    /// Output "has"/"have".
    pub fn has(&self, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().has(obj)
    }

    /// Output the string.
    pub fn s(&self, s: &'static str) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().s(s)
    }

    /// Output the string.
    pub fn string(&self, s: String) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().string(s)
    }

    /// Output the verb and adds "s"/"es" as needed.
    pub fn v(&self, who: Entity, verb: &'static str) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().v(who, verb)
    }

    /// Output the verb and adds "s"/"es" as needed.
    pub fn verb(&self, who: Entity, verb: String) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().verb(who, verb)
    }

    /// Don't capitalize the next word.
    ///
    /// Only relevant for the first word.
    pub fn supress_capitalize(&self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().supress_capitalize()
    }

    /// Don't add a space before the next word.
    pub fn supress_space(&self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().supress_space()
    }

    /// Capitalize the next word.
    pub fn capitalize(&self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().capitalize()
    }

    /// Supress automatic addition of an dot at the end of the sentance when needed.
    pub fn supress_dot(&self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().supress_dot()
    }

    /// Change the output color.
    pub fn color(&self, color: (u8, u8, u8)) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().color(color)
    }

    fn maybe_clear_output(&mut self) {
        if self.clear_output {
            self.output_string.clear();
            self.clear_output = false;
        }
    }

    fn push_char(&mut self, c: char) {
        self.maybe_clear_output();
        self.output_string.push(c);
    }

    fn push_str(&mut self, s: &str) {
        self.maybe_clear_output();
        self.output_string.push_str(s);
    }

    fn add_string(&mut self, text: &str) {
        if self.add_space && !(text.starts_with(',') || text.starts_with('"')) {
            self.push_char(' ');
        }
        if self.supress_capitalize {
            self.push_str(text);
        } else {
            self.supress_capitalize = true;
            self.maybe_clear_output();
            uppercase_first_char(text, &mut self.output_string);
        }
        self.add_space = !text.ends_with('"');
    }

    fn add_a_word<A: EntityAdapter<Entity>>(
        &mut self,
        entity_adapter: &A,
        obj: Entity,
        name: &str,
        is_prop: bool,
    ) {
        if entity_adapter.is_me(obj) {
            self.add_string("you");
        } else if entity_adapter.can_see(self.who, obj) {
            if !is_prop && is_singular(entity_adapter.gender(obj)) {
                let mut should_be_an = false;
                if let Some(c) = name.chars().next() {
                    if is_vowel(c) {
                        should_be_an = true;
                    }
                }
                if should_be_an {
                    self.add_string("an");
                } else {
                    self.add_string("a");
                }
            } else if !is_prop {
                self.add_string("some");
            }
            self.add_string(name);
        } else if is_prop {
            self.add_string("someone");
        } else {
            self.add_string("something");
        }
    }

    fn add_the_word<A: EntityAdapter<Entity>>(
        &mut self,
        entity_adapter: &A,
        obj: Entity,
        name: &str,
        is_proper: bool,
    ) {
        if entity_adapter.is_me(obj) {
            self.add_string("you");
        } else if entity_adapter.can_see(self.who, obj) {
            if !is_proper {
                self.add_string("the");
            }
            self.add_string(name);
        } else if is_proper {
            self.add_string("someone");
        } else {
            self.add_string("something");
        }
    }

    fn sing_plur<A: EntityAdapter<Entity>>(
        &mut self,
        entity_adapter: &A,
        who: Entity,
        singular: &'static str,
        plural: &'static str,
    ) {
        let mut g = entity_adapter.gender(who);
        if entity_adapter.is_me(who) {
            g = crate::Gender::Plural;
        } else if !entity_adapter.can_see(self.who, who) {
            g = crate::Gender::Male;
        }
        self.add_string(match g {
            crate::Gender::Plural => plural,
            _ => singular,
        });
    }

    fn add_verb<A: EntityAdapter<Entity>>(&mut self, entity_adapter: &A, who: Entity, verb: &str) {
        if !self.supress_capitalize {
            unimplemented!();
        }
        if self.add_space {
            self.push_char(' ');
        }
        self.push_str(verb);
        self.supress_capitalize = true;
        self.add_space = false;
        if is_singular(entity_adapter.gender(who)) && !entity_adapter.is_me(who) {
            add_verb_end_s(&mut self.output_string);
        }
        self.add_string("");
    }

    /// Process all the queued output with the `entity_adapter`.
    pub fn process_queue<A: EntityAdapter<Entity>>(&mut self, entity_adapter: &mut A) {
        while let Some(FragmentEntry(frag)) = self.queue_adapter.pop() {
            use crate::Fragment::*;
            match frag {
                A(obj) => {
                    // TODO; Fix add_a_word to take function.
                    let mut s = String::new();
                    entity_adapter.append_short_name(obj, &mut s);
                    self.add_a_word(
                        entity_adapter,
                        obj,
                        &s,
                        entity_adapter.has_short_proper(obj),
                    );
                }
                A_(obj) => {
                    let mut s = String::new();
                    entity_adapter.append_long_name(obj, &mut s);
                    self.add_a_word(entity_adapter, obj, &s, entity_adapter.has_long_proper(obj));
                }
                The(obj) => {
                    let mut s = String::new();
                    entity_adapter.append_short_name(obj, &mut s);
                    self.add_the_word(
                        entity_adapter,
                        obj,
                        &s,
                        entity_adapter.has_short_proper(obj),
                    );
                }
                The_(obj) => {
                    let mut s = String::new();
                    entity_adapter.append_long_name(obj, &mut s);
                    self.add_the_word(entity_adapter, obj, &s, entity_adapter.has_long_proper(obj));
                }
                Thes(obj) => {
                    if entity_adapter.is_me(self.who) {
                        self.add_string("your");
                    } else if entity_adapter.can_see(self.who, obj) {
                        let mut s = String::new();
                        entity_adapter.append_short_name(obj, &mut s);
                        // TODO; just append string to s.
                        if let Some(ch) = s.chars().rev().next() {
                            let uc = ch.is_uppercase();
                            let add = match ch {
                                's' | 'S' => "'",
                                _ => {
                                    if uc {
                                        "'S"
                                    } else {
                                        "'s"
                                    }
                                }
                            };
                            self.add_the_word(
                                entity_adapter,
                                obj,
                                &s,
                                entity_adapter.has_short_proper(obj),
                            );
                            self.add_space = false;
                            self.add_string(add);
                        } else {
                            // Error, short_name() == ""
                        }
                    } else if entity_adapter.has_short_proper(obj) {
                        self.add_string("someone's");
                    } else {
                        self.add_string("something's");
                    }
                }
                Thes_(obj) => {
                    if entity_adapter.is_me(self.who) {
                        self.add_string("your");
                    } else if entity_adapter.can_see(self.who, obj) {
                        let mut s = String::new();
                        entity_adapter.append_long_name(obj, &mut s);
                        if let Some(ch) = s.chars().rev().next() {
                            let uc = ch.is_uppercase();
                            let add = match ch {
                                's' | 'S' => "'",
                                _ => {
                                    if uc {
                                        "'S"
                                    } else {
                                        "'s"
                                    }
                                }
                            };
                            self.add_the_word(
                                entity_adapter,
                                obj,
                                &s,
                                entity_adapter.has_long_proper(obj),
                            );
                            self.add_space = false;
                            self.add_string(add);
                        } else {
                            // Error, long_name() == ""
                        }
                    } else if entity_adapter.has_long_proper(obj) {
                        self.add_string("someone's");
                    } else {
                        self.add_string("something's");
                    }
                }
                Thess(_obj) => {
                    /* TODO */
                    unimplemented!();
                }
                Thess_(_obj) => {
                    /* TODO */
                    unimplemented!();
                }
                My(_who, _obj) => {
                    /* TODO */
                    unimplemented!();
                }
                My_(_who, _obj) => {
                    /* TODO */
                    unimplemented!();
                }
                Word(_obj) => {
                    /* TODO */
                    unimplemented!();
                }
                Word_(_obj) => {
                    /* TODO */
                    unimplemented!();
                }
                Is(obj) => {
                    self.sing_plur(entity_adapter, obj, "is", "are");
                }
                Has(obj) => {
                    self.sing_plur(entity_adapter, obj, "has", "have");
                }
                TextRef(s) => {
                    self.add_string(s);
                }
                Text(s) => {
                    self.add_string(&s);
                }
                VerbRef(who, verb) => {
                    self.add_verb(entity_adapter, who, verb);
                }
                VerbString(who, verb) => {
                    self.add_verb(entity_adapter, who, &verb);
                }
                Capitalize(capitalize) => {
                    self.supress_capitalize = !capitalize;
                }
                SupressDot(supress_dot) => {
                    self.supress_dot = supress_dot;
                }
                SupressSpace(supress_space) => {
                    self.add_space = !supress_space;
                }
                Color(rgb) => {
                    entity_adapter.write_text(&self.output_string);
                    entity_adapter.set_color(rgb);
                    self.clear_output = true;
                    self.add_space = false;
                }
                EndOfLine => {
                    if !self.supress_dot && needs_dot(&self.output_string) {
                        self.push_char('.');
                    } else {
                        self.maybe_clear_output();
                    }
                    entity_adapter.write_text(&self.output_string);
                    entity_adapter.done();
                    self.supress_dot = false;
                    self.supress_dot = false;
                    self.supress_capitalize = false;
                    self.add_space = false;
                    self.output_string.clear();
                    self.clear_output = false;
                }
            }
        }
    }
}
