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
use error_stack::ResultExt;
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
    event_new::{
        Event,
        Name,
        Tag,
        Version,
    },
    stream_new::{
        Error,
        Facets,
        Position,
        Result,
        Timestamp,
        iterate::{
            AndIter,
            OrIter,
        },
        operate::{
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

#[derive(new, Debug)]
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
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        self.tags.insert(batch, event, facets);
        self.timestamps.insert(batch, facets);
        self.types.insert(batch, event, facets);
    }
}

impl Indices {
    pub fn iterate<'a, S>(&self, selectors: S, from: Option<Position>) -> IndicesIter
    where
        S: IntoIterator<Item = &'a Selector<u64>>,
    {
        OrIter::iter(selectors.into_iter().map(|selector| match selector {
            Selector(types, None) => self.types.iterate(types.iter(), from),
            Selector(types, Some(tags)) => AndIter::iter([
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
    And(AndIter<IndicesIter, Position>),
    Or(OrIter<IndicesIter, Position>),
    Tags(TagsIter),
    Types(TypesIter),
}

impl DoubleEndedIterator for IndicesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next_back(),
            Self::Or(iter) => iter.next_back(),
            Self::Tags(iter) => iter.next_back(),
            Self::Types(iter) => iter.next_back(),
        }
    }
}

impl Iterator for IndicesIter {
    type Item = Result<Position>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next(),
            Self::Or(iter) => iter.next(),
            Self::Tags(iter) => iter.next(),
            Self::Types(iter) => iter.next(),
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

#[derive(new, Debug)]
struct Tags {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Tags {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        for tag in &event.1.1 {
            let key: TagKey = TagKeyWriter(tag, &facets.0).into(); // Tag & Position
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
        AndIter::iter(tags.map(|tag| {
            let iter = if let Some(from) = from {
                let from: TagKey = TagKeyWriter(tag, &from).into();
                let to: TagKey = TagKeyWriter(tag, &Position::MAX).into();
                let range = from..to;

                self.keyspace.range(range)
            } else {
                let prefix: TagPrefix = TagPrefixWriter(tag).into();

                self.keyspace.prefix(prefix)
            };

            TagsIter::new(iter).into()
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct TagsIter {
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

#[derive(new, Debug)]
struct Timestamps {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Timestamps {
    fn insert(&self, batch: &mut Batch, facets: &Facets) {
        let key: TimestampKey = TimestampKeyWriter(&facets.1).into(); // Timestamp
        let value = facets.0.0.to_be_bytes(); // Position

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

#[derive(new, Debug)]
struct Types {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Types {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        let key: TypeKey = TypeKeyWriter(&event.1.0.0, &facets.0).into(); // Type Name & Position
        let value = event.1.0.1.0.to_be_bytes(); // Version

        batch.insert(&self.keyspace, key, value);
    }
}

impl Types {
    fn iterate<'a, T>(&self, types: T, from: Option<Position>) -> IndicesIter
    where
        T: Iterator<Item = &'a TypeSelector<u64>>,
    {
        OrIter::iter(types.map(|ty| {
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

            TypesIter::new(iter, range).into()
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Types Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct TypesIter {
    #[debug("Iter")]
    iter: fjall::Iter,
    range: Range<Version>,
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
