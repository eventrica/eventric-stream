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
        operations::{
            AndIter,
            OrIter,
            Selector,
            TypeSelector,
        },
        storage::{
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
    pub fn iterate(&self, selection: &[Selector<u64>], from: Option<Position>) -> IndicesIter {
        OrIter::iter(selection.iter().map(|selector| match selector {
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

// Tags

#[derive(new, Debug)]
struct Tags {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Tags {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        for tag in &event.1.1 {
            let key: TagsKey = TagsKeyConverter(tag, &facets.0).into(); // Tag & Position
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
                let from: TagsKey = TagsKeyConverter(tag, &from).into();
                let to: TagsKey = TagsKeyConverter(tag, &Position::MAX).into();
                let range = from..to;

                self.keyspace.range(range)
            } else {
                let prefix: TagsPrefix = TagsPrefixConverter(tag).into();

                self.keyspace.prefix(prefix)
            };

            TagsIter::new(iter).into()
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Constants

static TAGS_INDEX_ID: u8 = 0;
static TAGS_KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static TAGS_PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Tags Converters

struct TagsKeyConverter<'a>(&'a Tag<u64>, &'a Position);

impl From<TagsKeyConverter<'_>> for TagsKey {
    fn from(TagsKeyConverter(tag, position): TagsKeyConverter<'_>) -> Self {
        let mut key = TagsKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TAGS_INDEX_ID);
            key.put_u64(tag.0); // Tag
            key.put_u64(position.0); // Position
        }

        key
    }
}

struct TagsPositionConverter<'a>(&'a Slice);

impl From<TagsPositionConverter<'_>> for Position {
    fn from(TagsPositionConverter(slice): TagsPositionConverter<'_>) -> Self {
        let mut slice = &slice[..];

        slice.advance(TAGS_PREFIX_LEN);

        Position::new(slice.get_u64())
    }
}

struct TagsPrefixConverter<'a>(&'a Tag<u64>);

impl From<TagsPrefixConverter<'_>> for TagsPrefix {
    fn from(TagsPrefixConverter(tag): TagsPrefixConverter<'_>) -> Self {
        let mut prefix = TagsPrefix::default();

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(TAGS_INDEX_ID);
            prefix.put_u64(tag.0);
        }

        prefix
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
            Ok(key) => Ok(TagsPositionConverter(&key).into()),
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

// Tags Types

type TagsKey = [u8; TAGS_KEY_LEN];
type TagsPrefix = [u8; TAGS_PREFIX_LEN];

// -------------------------------------------------------------------------------------------------

// Timestamps

#[derive(new, Debug)]
struct Timestamps {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Timestamps {
    fn insert(&self, batch: &mut Batch, facets: &Facets) {
        let key: TimestampsKey = TimestampsKeyConverter(&facets.1).into(); // Timestamp
        let value = facets.0.0.to_be_bytes(); // Position

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamps Constants

static TIMESTAMPS_INDEX_ID: u8 = 1;
static TIMESTAMPS_KEY_LEN: usize = ID_LEN + TIMESTAMPS_LEN;
static TIMESTAMPS_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Timestamps Converters

struct TimestampsKeyConverter<'a>(&'a Timestamp);

impl From<TimestampsKeyConverter<'_>> for TimestampsKey {
    fn from(TimestampsKeyConverter(timestamp): TimestampsKeyConverter<'_>) -> Self {
        let mut key = TimestampsKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TIMESTAMPS_INDEX_ID);
            key.put_u64(timestamp.0);
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamps Types

type TimestampsKey = [u8; TIMESTAMPS_KEY_LEN];

// -------------------------------------------------------------------------------------------------

// Types

#[derive(new, Debug)]
struct Types {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Types {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        let key: TypesKey = TypesKeyConverter(&event.1.0.0, &facets.0).into(); // Type Name & Position
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
                let from: TypesKey = TypesKeyConverter(&ty.0, &from).into();
                let to: TypesKey = TypesKeyConverter(&ty.0, &Position::MAX).into();
                let range = from..to;

                self.keyspace.range(range)
            } else {
                let prefix: TypesPrefix = TypesPrefixConverter(&ty.0).into();

                self.keyspace.prefix(prefix)
            };

            let range = ty.1.clone();

            TypesIter::new(iter, range).into()
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Types Constants

static TYPES_INDEX_ID: u8 = 2;
static TYPES_KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static TYPES_PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Types Converters

struct TypesKeyConverter<'a>(&'a Name<u64>, &'a Position);

impl From<TypesKeyConverter<'_>> for TypesKey {
    fn from(TypesKeyConverter(name, position): TypesKeyConverter<'_>) -> Self {
        let mut key = TypesKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TYPES_INDEX_ID);
            key.put_u64(name.0); // Type Name
            key.put_u64(position.0); // Position
        }

        key
    }
}

struct TypesPositionConverter<'a>(&'a Slice);

impl From<TypesPositionConverter<'_>> for Position {
    fn from(TypesPositionConverter(slice): TypesPositionConverter<'_>) -> Self {
        let mut slice = &slice[..];

        slice.advance(TYPES_PREFIX_LEN);

        Position::new(slice.get_u64())
    }
}

struct TypesPrefixConverter<'a>(&'a Name<u64>);

impl From<TypesPrefixConverter<'_>> for TypesPrefix {
    fn from(TypesPrefixConverter(name): TypesPrefixConverter<'_>) -> Self {
        let mut prefix = TypesPrefix::default();

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(TYPES_INDEX_ID);
            prefix.put_u64(name.0); // Type Name
        }

        prefix
    }
}

struct TypesVersionConverter<'a>(&'a Slice);

impl From<TypesVersionConverter<'_>> for Version {
    fn from(TypesVersionConverter(slice): TypesVersionConverter<'_>) -> Self {
        Version::new(slice.as_ref().get_u8())
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
                .contains::<Version>(&TypesVersionConverter(&value).into())
                .then(|| Ok(TypesPositionConverter(&key).into())),
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

// -------------------------------------------------------------------------------------------------

// Types Types

type TypesKey = [u8; TYPES_KEY_LEN];
type TypesPrefix = [u8; TYPES_PREFIX_LEN];
