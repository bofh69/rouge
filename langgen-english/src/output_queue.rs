use crate::traits::EntityAdapter;
use crate::traits::QueueAdapter;
use crate::FragmentEntry;
use std::marker::PhantomData;

pub struct OutputQueue<'a, Entity, A, QA>
where
    A: EntityAdapter<'a, Entity>,
    QA: QueueAdapter<Entity>,
{
    queue_adapter: QA,
    supress_dot: bool,
    supress_capitalize: bool,
    _entity: PhantomData<Entity>,
    _entity_adapter: PhantomData<&'a A>,
}

impl<'a, Entity, A, QA> OutputQueue<'a, Entity, A, QA>
where
    A: EntityAdapter<'a, Entity>,
    QA: QueueAdapter<Entity>,
    Entity: std::fmt::Debug,
{
    pub fn new(queue_adapter: QA) -> Self {
        Self {
            queue_adapter,
            supress_dot: false,
            supress_capitalize: false,
            _entity: Default::default(),
            _entity_adapter: Default::default(),
        }
    }

    fn make_output_builder(&mut self) -> OutputBuilder<'_, QA, Entity> {
        OutputBuilder::new(&mut self.queue_adapter)
    }

    pub fn a(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().a(who)
    }

    pub fn a_(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().a_(who)
    }

    pub fn the(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().the(who)
    }

    pub fn the_(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().the_(who)
    }

    pub fn thes(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thes(who)
    }

    pub fn thes_(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thes_(who)
    }

    pub fn thess(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thess(who)
    }

    pub fn thess_(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().thess_(who)
    }

    pub fn my(&mut self, who: Entity, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().my(who, obj)
    }

    pub fn my_(&mut self, who: Entity, obj: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().my_(who, obj)
    }

    pub fn word(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().word(who)
    }

    pub fn word_(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().word_(who)
    }

    pub fn is(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().is(who)
    }

    pub fn has(&mut self, who: Entity) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().has(who)
    }

    pub fn s(&mut self, s: &'static str) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().s(s)
    }

    pub fn string(&mut self, s: String) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().string(s)
    }

    pub fn v(&mut self, who: Entity, verb: &'static str) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().v(who, verb)
    }

    pub fn verb(&mut self, who: Entity, verb: String) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().verb(who, verb)
    }

    pub fn supress_capitalize(&mut self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().supress_capitalize()
    }

    pub fn capitalize(&mut self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().capitalize()
    }

    pub fn supress_dot(&mut self) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().supress_dot()
    }

    pub fn color(&mut self, color: (i32, i32, i32)) -> OutputBuilder<'_, QA, Entity> {
        self.make_output_builder().color(color)
    }

    pub fn process_queue(&mut self, entity_adapter: &mut A) {
        while let Some(FragmentEntry(frag)) = self.queue_adapter.pop() {
            use crate::Fragment::*;
            match frag {
                A(who) => {
                    /* TODO */
                    entity_adapter.write_text(entity_adapter.short_name(who));
                }
                A_(who) => {
                    /* TODO */
                    entity_adapter.write_text(entity_adapter.long_name(who));
                }
                The(who) => {
                    /* todo */
                    entity_adapter.write_text(entity_adapter.short_name(who));
                }
                The_(who) => {
                    /* todo */
                    entity_adapter.write_text(entity_adapter.long_name(who));
                }
                Thes(who) => {
                    /* todo */
                    entity_adapter.write_text(entity_adapter.short_name(who));
                }
                Thes_(who) => {
                    /* todo */
                    entity_adapter.write_text(entity_adapter.long_name(who));
                }
                Thess(who) => {
                    /* todo */
                    entity_adapter.write_text(entity_adapter.short_name(who));
                }
                Thess_(who) => {
                    /* todo */
                    entity_adapter.write_text(entity_adapter.long_name(who));
                }
                My(who, obj) => {
                    /* TODO */
                    entity_adapter.write_text(entity_adapter.short_name(who));
                    entity_adapter.write_text("'s");
                    entity_adapter.write_text(entity_adapter.short_name(obj));
                }
                My_(who, obj) => {
                    /* TODO */
                    entity_adapter.write_text(entity_adapter.long_name(who));
                    entity_adapter.write_text("'s");
                    entity_adapter.write_text(entity_adapter.long_name(obj));
                }
                Word(who) => {
                    /* TODO */
                    entity_adapter.write_text(entity_adapter.short_name(who));
                }
                Word_(who) => {
                    /* TODO */
                    entity_adapter.write_text(entity_adapter.long_name(who));
                }
                Is(who) => {
                    if entity_adapter.is_me(who) {
                        /* TODO */
                        entity_adapter.write_text("are");
                    } else {
                        /* TODO */
                        entity_adapter.write_text("is");
                    }
                }
                Has(who) => {
                    if entity_adapter.is_me(who) {
                        /* TODO */
                        entity_adapter.write_text("have");
                    } else {
                        /* TODO */
                        entity_adapter.write_text("has");
                    }
                }
                TextRef(s) => {
                    entity_adapter.write_text(s);
                }
                Text(s) => {
                    entity_adapter.write_text(&s);
                }
                VerbRef(who, s) => {
                    entity_adapter.write_text(s);
                    if entity_adapter.is_me(who) {
                        /* TODO */
                        entity_adapter.write_text("");
                    } else {
                        /* TODO */
                        entity_adapter.write_text("s");
                    }
                }
                VerbString(who, s) => {
                    entity_adapter.write_text(&s);
                    if entity_adapter.is_me(who) {
                        /* TODO */
                        entity_adapter.write_text("");
                    } else {
                        /* TODO */
                        entity_adapter.write_text("s");
                    }
                }
                Capitalize(capitalize) => {
                    self.supress_capitalize = !capitalize;
                }
                SupressDot(supress_dot) => {
                    self.supress_dot = supress_dot;
                }
                Color(rgb) => {
                    entity_adapter.set_color(rgb);
                }
                EndOfLine => {
                    if !self.supress_dot {
                        entity_adapter.write_text(".");
                    }
                    entity_adapter.done();
                    self.supress_dot = false;
                }
            }
        }
    }
}

pub struct OutputBuilder<'a, QA, Entity>
where
    QA: QueueAdapter<Entity>,
{
    queue_adapter: &'a mut QA,
    _entity: PhantomData<Entity>,
}

impl<'a, Entity, QA> OutputBuilder<'a, QA, Entity>
where
    QA: QueueAdapter<Entity>,
{
    pub fn new(queue_adapter: &'a mut QA) -> Self {
        Self {
            queue_adapter,
            _entity: Default::default(),
        }
    }

    fn push_fragment(self, frag: crate::Fragment<Entity>) -> Self {
        self.queue_adapter.push(FragmentEntry::<Entity>(frag));
        self
    }

    pub fn a(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::A(who))
    }

    pub fn a_(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::A_(who))
    }

    pub fn the(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::The(who))
    }

    pub fn the_(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::The_(who))
    }

    pub fn thes(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thes(who))
    }

    pub fn thes_(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thes_(who))
    }

    pub fn thess(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thess(who))
    }

    pub fn thess_(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thess_(who))
    }

    pub fn my(self, who: Entity, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::My(who, obj))
    }

    pub fn my_(self, who: Entity, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::My_(who, obj))
    }

    pub fn word(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Word(who))
    }

    pub fn word_(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Word_(who))
    }

    pub fn is(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Is(who))
    }

    pub fn has(self, who: Entity) -> Self {
        self.push_fragment(crate::Fragment::Has(who))
    }

    pub fn s(self, s: &'static str) -> Self {
        self.push_fragment(crate::Fragment::TextRef(s))
    }

    pub fn string(self, s: String) -> Self {
        self.push_fragment(crate::Fragment::Text(s))
    }

    pub fn v(self, who: Entity, verb: &'static str) -> Self {
        self.push_fragment(crate::Fragment::VerbRef(who, verb))
    }

    pub fn verb(self, who: Entity, verb: String) -> Self {
        self.push_fragment(crate::Fragment::VerbString(who, verb))
    }

    pub fn supress_capitalize(self) -> Self {
        self.push_fragment(crate::Fragment::Capitalize(false))
    }

    pub fn capitalize(self) -> Self {
        self.push_fragment(crate::Fragment::Capitalize(true))
    }

    pub fn supress_dot(self) -> Self {
        self.push_fragment(crate::Fragment::SupressDot(true))
    }

    pub fn color(self, color: (i32, i32, i32)) -> Self {
        self.push_fragment(crate::Fragment::Color(color))
    }
}

impl<'a, Entity, QA> Drop for OutputBuilder<'a, QA, Entity>
where
    QA: QueueAdapter<Entity>,
{
    fn drop(&mut self) {
        self.queue_adapter
            .push(FragmentEntry::<Entity>(crate::Fragment::EndOfLine));
    }
}
