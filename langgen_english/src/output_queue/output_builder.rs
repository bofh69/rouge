use super::{FragmentEntry, PhantomData, QueueAdapter};

#[allow(missing_docs)]
/// `OutputBuilder` helps output queue building whole sentences.
///
/// This struct has a subset of [`OutputQueue`](crate::OutputQueue)'s functions
/// and they are described there.
pub struct OutputBuilder<'a, QA, Entity>
where
    QA: QueueAdapter<Entity>,
{
    queue_adapter: &'a QA,
    _lock: std::sync::LockResult<std::sync::MutexGuard<'a, ()>>,
    _entity: PhantomData<Entity>,
}

#[allow(missing_docs)]
impl<'a, Entity, QA> OutputBuilder<'a, QA, Entity>
where
    QA: QueueAdapter<Entity>,
{
    pub(crate) fn new(queue_adapter: &'a QA, lock: std::sync::LockResult<std::sync::MutexGuard<'a, ()>>) -> Self {
        Self {
            queue_adapter,
            _lock: lock,
            _entity: PhantomData::default(),
        }
    }

    fn push_fragment(self, frag: crate::Fragment<Entity>) -> Self {
        self.queue_adapter.push(FragmentEntry::<Entity>(frag));
        self
    }

    pub fn a(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::A(obj))
    }

    pub fn a_(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::A_(obj))
    }

    pub fn the(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::The(obj))
    }

    pub fn the_(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::The_(obj))
    }

    pub fn thes(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thes(obj))
    }

    pub fn thes_(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thes_(obj))
    }

    pub fn thess(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thess(obj))
    }

    pub fn thess_(self, obj: Entity) -> Self {
        self.push_fragment(crate::Fragment::Thess_(obj))
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

    pub fn supress_space(self) -> Self {
        self.push_fragment(crate::Fragment::SupressSpace(true))
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

    pub fn color(self, color: (u8, u8, u8)) -> Self {
        self.push_fragment(crate::Fragment::Color(color))
    }
}

/// Drop implementation to send the end of line message.
impl<'a, Entity, QA> Drop for OutputBuilder<'a, QA, Entity>
where
    QA: QueueAdapter<Entity>,
{
    fn drop(&mut self) {
        self.queue_adapter
            .push(FragmentEntry::<Entity>(crate::Fragment::EndOfLine));
    }
}
