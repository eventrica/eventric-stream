use std::ops::{
    ControlFlow,
    Range,
};

use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::{
    Debug,
    From,
};
use error_stack::{
    Report,
    ResultExt,
};
use fancy_constructor::new;
use fjall::{
    Database,
    Guard,
    Keyspace,
    KeyspaceCreateOptions,
    OwnedWriteBatch as Batch,
    Slice,
};

use crate::{
    error::{
        Error,
        Result,
    },
    event::{
        Event,
        Name,
        Tag,
        Version,
    },
    iter::{
        Seek,
        intersection::Intersection,
        union::Union,
    },
    stream::{
        Metadata,
        Position,
        Timestamp,
        operate::select::{
            Selector,
            TypeSelector,
        },
        store::{
            HASH_LEN,
            ID_LEN,
            POSITION_LEN,
        },
    },
};

// =================================================================================================
// Indices
// =================================================================================================

#[derive(new, Clone, Debug)]
#[new(const_fn, vis())]
pub struct Indices {
    tags: Tags,
    timestamps: Timestamps,
    types: Types,
}

impl Indices {
    pub fn open(database: &Database) -> Result<Self> {
        let keyspace = database
            .keyspace("indices", KeyspaceCreateOptions::default)
            .change_context(Error)
            .attach("failed to open indices keyspace")?;

        let tags = Tags::new(keyspace.clone());
        let timestamps = Timestamps::new(keyspace.clone());
        let types = Types::new(keyspace);

        Ok(Self::new(tags, timestamps, types))
    }
}

impl Indices {
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, meta: &Metadata) {
        self.tags.insert(batch, event, meta);
        self.timestamps.insert(batch, meta);
        self.types.insert(batch, event, meta);
    }
}

impl Indices {
    pub fn iterate<'a, S>(&self, selectors: S, from: Option<Position>) -> IndicesIter
    where
        S: IntoIterator<Item = &'a Selector<u64>>,
    {
        Union::iter(selectors.into_iter().map(|selector| match selector {
            Selector(types, None) => self.types.iterate(types.iter(), from),
            Selector(types, Some(tags)) => Intersection::iter([
                self.types.iterate(types.iter(), from),
                self.tags.iterate(tags.iter(), from),
            ]),
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Indices Iterator

#[derive(Debug, From)]
pub enum IndicesIter {
    Intersection(Intersection<IndicesIter, Position, Report<Error>>),
    Union(Union<IndicesIter, Position, Report<Error>>),
    Tags(TagsIter),
    Types(TypesIter),
}

impl DoubleEndedIterator for IndicesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Intersection(iter) => iter.next_back(),
            Self::Union(iter) => iter.next_back(),
            Self::Tags(iter) => iter.next_back(),
            Self::Types(iter) => iter.next_back(),
        }
    }
}

impl Iterator for IndicesIter {
    type Item = Result<Position>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Intersection(iter) => iter.next(),
            Self::Union(iter) => iter.next(),
            Self::Tags(iter) => iter.next(),
            Self::Types(iter) => iter.next(),
        }
    }
}

impl Seek<Position> for IndicesIter {
    // Skip every node forward to `target`: combinators seek their children, leaf
    // scans re-seek the underlying fjall range. This is what `Intersection` calls
    // to leapfrog a lagging child past a run of non-matching positions.
    fn seek(&mut self, target: Position) {
        match self {
            Self::Intersection(iter) => iter.seek(target),
            Self::Union(iter) => iter.seek(target),
            Self::Tags(iter) => iter.seek(target),
            Self::Types(iter) => iter.seek(target),
        }
    }

    fn seek_back(&mut self, target: Position) {
        match self {
            Self::Intersection(iter) => iter.seek_back(target),
            Self::Union(iter) => iter.seek_back(target),
            Self::Tags(iter) => iter.seek_back(target),
            Self::Types(iter) => iter.seek_back(target),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tag Constants

static TAG_INDEX_ID: u8 = 0;
static TAG_KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static TAG_PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Tag Key Writer

type TagKey = [u8; TAG_KEY_LEN];

struct TagKeyWriter<'a>(&'a Tag<u64>, &'a Position);

impl From<TagKeyWriter<'_>> for TagKey {
    fn from(TagKeyWriter(tag, position): TagKeyWriter<'_>) -> Self {
        let mut key = TagKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TAG_INDEX_ID);
            key.put_u64(tag.0); // Tag
            key.put_u64(position.0); // Position
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Tag Prefix Writer

type TagPrefix = [u8; TAG_PREFIX_LEN];

struct TagPrefixWriter<'a>(&'a Tag<u64>);

impl From<TagPrefixWriter<'_>> for TagPrefix {
    fn from(TagPrefixWriter(tag): TagPrefixWriter<'_>) -> Self {
        let mut prefix = TagPrefix::default();

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(TAG_INDEX_ID);
            prefix.put_u64(tag.0);
        }

        prefix
    }
}

// -------------------------------------------------------------------------------------------------

// Tag Position Reader

struct TagPositionReader<'a>(&'a Slice);

impl From<TagPositionReader<'_>> for Position {
    fn from(TagPositionReader(slice): TagPositionReader<'_>) -> Self {
        let mut slice = &slice[..];

        slice.advance(TAG_PREFIX_LEN);

        Position::new(slice.get_u64())
    }
}

// -------------------------------------------------------------------------------------------------

// Tags

#[derive(new, Clone, Debug)]
struct Tags {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Tags {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, meta: &Metadata) {
        for tag in event.facets().tags() {
            let key: TagKey = TagKeyWriter(tag, &meta.0).into(); // Tag & Position
            let value = []; // Empty

            batch.insert(&self.keyspace, key, value);
        }
    }
}

impl Tags {
    fn iterate<'a, T>(&self, tags: T, from: Option<Position>) -> IndicesIter
    where
        T: Iterator<Item = &'a Tag<u64>>,
    {
        Intersection::iter(tags.map(|tag| {
            let lower = from.unwrap_or(Position::new(0));
            let iter = if let Some(from) = from {
                let from: TagKey = TagKeyWriter(tag, &from).into();
                let to: TagKey = TagKeyWriter(tag, &Position::MAX).into();
                let range = from..to;

                self.keyspace.range(range)
            } else {
                let prefix: TagPrefix = TagPrefixWriter(tag).into();

                self.keyspace.prefix(prefix)
            };

            // Retain the keyspace + tag hash so `seek`/`seek_back` can re-range the
            // scan to an arbitrary position (the leapfrog skip); `lower` is the
            // query's lower bound, preserved by the reverse re-range.
            TagsIter::new(self.keyspace.clone(), tag.clone(), lower, iter).into()
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct TagsIter {
    #[debug("Keyspace")]
    keyspace: Keyspace,
    tag: Tag<u64>,
    lower: Position,
    #[debug("Iter")]
    iter: fjall::Iter,
}

impl TagsIter {
    #[rustfmt::skip]
    fn next_map(guard: Guard) -> <Self as Iterator>::Item {
        match guard.key() {
            Ok(key) => Ok(TagPositionReader(&key).into()),
            Err(err) => Err(err).change_context(Error).attach("failed to map next tag"),
        }
    }
}

impl Seek<Position> for TagsIter {
    // Re-range the scan to `[tag, target] .. [tag, MAX]`, so the next item is the
    // first position `>= target` for this tag — one LSM seek instead of stepping.
    fn seek(&mut self, target: Position) {
        let from: TagKey = TagKeyWriter(&self.tag, &target).into();
        let to: TagKey = TagKeyWriter(&self.tag, &Position::MAX).into();

        self.iter = self.keyspace.range(from..to);
    }

    // The reverse: re-range to `[tag, lower] ..= [tag, target]` (inclusive of
    // target, preserving the query's lower bound), so the next `next_back` is the
    // last position `<= target` for this tag.
    fn seek_back(&mut self, target: Position) {
        let from: TagKey = TagKeyWriter(&self.tag, &self.lower).into();
        let to: TagKey = TagKeyWriter(&self.tag, &target).into();

        self.iter = self.keyspace.range(from..=to);
    }
}

impl DoubleEndedIterator for TagsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::next_map)
    }
}

impl Iterator for TagsIter {
    type Item = Result<Position>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::next_map)
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamp Constants

static TIMESTAMP_INDEX_ID: u8 = 1;
static TIMESTAMP_KEY_LEN: usize = ID_LEN + TIMESTAMP_LEN;
static TIMESTAMP_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Timestamp Key Writer

type TimestampKey = [u8; TIMESTAMP_KEY_LEN];

struct TimestampKeyWriter<'a>(&'a Timestamp);

impl From<TimestampKeyWriter<'_>> for TimestampKey {
    fn from(TimestampKeyWriter(timestamp): TimestampKeyWriter<'_>) -> Self {
        let mut key = TimestampKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TIMESTAMP_INDEX_ID);
            key.put_u64(timestamp.0);
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamps

#[derive(new, Clone, Debug)]
struct Timestamps {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Timestamps {
    fn insert(&self, batch: &mut Batch, meta: &Metadata) {
        let key: TimestampKey = TimestampKeyWriter(&meta.1).into(); // Timestamp
        let value = meta.0.0.to_be_bytes(); // Position

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Type Constants

static TYPE_INDEX_ID: u8 = 2;
static TYPE_KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static TYPE_PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Type Key Writer

type TypeKey = [u8; TYPE_KEY_LEN];

struct TypeKeyWriter<'a>(&'a Name<u64>, &'a Position);

impl From<TypeKeyWriter<'_>> for TypeKey {
    fn from(TypeKeyWriter(name, position): TypeKeyWriter<'_>) -> Self {
        let mut key = TypeKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TYPE_INDEX_ID);
            key.put_u64(name.0); // Type Name
            key.put_u64(position.0); // Position
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Type Position Reader

struct TypePositionReader<'a>(&'a Slice);

impl From<TypePositionReader<'_>> for Position {
    fn from(TypePositionReader(slice): TypePositionReader<'_>) -> Self {
        let mut slice = &slice[..];

        slice.advance(TYPE_PREFIX_LEN);

        Position::new(slice.get_u64())
    }
}

// -------------------------------------------------------------------------------------------------

// Type Prefix Writer

type TypePrefix = [u8; TYPE_PREFIX_LEN];

struct TypePrefixWriter<'a>(&'a Name<u64>);

impl From<TypePrefixWriter<'_>> for TypePrefix {
    fn from(TypePrefixWriter(name): TypePrefixWriter<'_>) -> Self {
        let mut prefix = TypePrefix::default();

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(TYPE_INDEX_ID);
            prefix.put_u64(name.0); // Type Name
        }

        prefix
    }
}

// -------------------------------------------------------------------------------------------------

// Type Version Reader

struct TypeVersionReader<'a>(&'a Slice);

impl From<TypeVersionReader<'_>> for Version {
    fn from(TypeVersionReader(slice): TypeVersionReader<'_>) -> Self {
        Version::new(slice.as_ref().get_u8())
    }
}

// -------------------------------------------------------------------------------------------------

// Types

#[derive(new, Clone, Debug)]
struct Types {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Types {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, meta: &Metadata) {
        let ty = event.facets().ty();
        let key: TypeKey = TypeKeyWriter(ty.name(), &meta.0).into(); // Type Name & Position
        let value = ty.version().0.to_be_bytes(); // Version

        batch.insert(&self.keyspace, key, value);
    }
}

impl Types {
    fn iterate<'a, T>(&self, types: T, from: Option<Position>) -> IndicesIter
    where
        T: Iterator<Item = &'a TypeSelector<u64>>,
    {
        Union::iter(types.map(|ty| {
            let lower = from.unwrap_or(Position::new(0));
            let iter = if let Some(from) = from {
                let from: TypeKey = TypeKeyWriter(&ty.0, &from).into();
                let to: TypeKey = TypeKeyWriter(&ty.0, &Position::MAX).into();
                let range = from..to;

                self.keyspace.range(range)
            } else {
                let prefix: TypePrefix = TypePrefixWriter(&ty.0).into();

                self.keyspace.prefix(prefix)
            };

            let range = ty.1.clone();

            // Retain the keyspace + type-name hash so `seek`/`seek_back` can
            // re-range (with `lower` the query bound the reverse re-range keeps);
            // the version range rides along and is re-applied to the new scan.
            TypesIter::new(self.keyspace.clone(), ty.0.clone(), lower, iter, range).into()
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Types Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct TypesIter {
    #[debug("Keyspace")]
    keyspace: Keyspace,
    name: Name<u64>,
    lower: Position,
    #[debug("Iter")]
    iter: fjall::Iter,
    range: Range<Version>,
}

impl Seek<Position> for TypesIter {
    // Re-range the scan forward to `target` for this type name; the version filter
    // is unaffected (it is applied per item in `next_map`).
    fn seek(&mut self, target: Position) {
        let from: TypeKey = TypeKeyWriter(&self.name, &target).into();
        let to: TypeKey = TypeKeyWriter(&self.name, &Position::MAX).into();

        self.iter = self.keyspace.range(from..to);
    }

    // The reverse: re-range to `[name, lower] ..= [name, target]` (inclusive of
    // target, preserving the query's lower bound); the version filter rides along.
    fn seek_back(&mut self, target: Position) {
        let from: TypeKey = TypeKeyWriter(&self.name, &self.lower).into();
        let to: TypeKey = TypeKeyWriter(&self.name, &target).into();

        self.iter = self.keyspace.range(from..=to);
    }
}

impl TypesIter {
    #[inline]
    fn check<T, U>(mut f: impl FnMut(T) -> Option<U>) -> impl FnMut((), T) -> ControlFlow<U> {
        move |(), x| match f(x) {
            Some(x) => ControlFlow::Break(x),
            None => ControlFlow::Continue(()),
        }
    }

    fn next_map(guard: Guard, range: &Range<Version>) -> Option<<Self as Iterator>::Item> {
        match guard.into_inner() {
            Ok((key, value)) => range
                .contains::<Version>(&TypeVersionReader(&value).into())
                .then(|| Ok(TypePositionReader(&key).into())),
            Err(err) => Some(
                Err(err)
                    .change_context(Error)
                    .attach("failed to map next type"),
            ),
        }
    }
}

impl DoubleEndedIterator for TypesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .try_rfold((), Self::check(|x| Self::next_map(x, &self.range)))
            .break_value()
    }
}

impl Iterator for TypesIter {
    type Item = Result<Position>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .try_fold((), Self::check(|x| Self::next_map(x, &self.range)))
            .break_value()
    }
}
