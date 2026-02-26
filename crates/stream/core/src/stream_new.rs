mod operations;
mod storage;

use std::{
    path::Path,
    result,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use derive_more::{
    Debug,
    Display,
    Error,
    with_trait::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
};
use error_stack::{
    Report,
    ResultExt,
};
use fancy_constructor::new;
use fjall::Database;

use crate::{
    event_new::Event,
    stream_new::storage::Storage,
};

// =================================================================================================
// Stream
// =================================================================================================

// Builder

#[derive(new, Debug)]
#[new(vis())]
pub struct Builder<P>
where
    P: AsRef<Path>,
{
    path: P,
    #[new(default)]
    temporary: Option<bool>,
}

impl<P> Builder<P>
where
    P: AsRef<Path>,
{
    pub fn open(self) -> Result<Stream> {
        let database = Database::builder(self.path)
            .temporary(self.temporary.unwrap_or_default())
            .open()
            .change_context(Error)
            .attach("failed to open database")?;

        let storage = Storage::open(&database)?;
        let next = storage.len().map(Position::new)?;

        Ok(Stream::new(database, next, storage))
    }
}

impl<P> Builder<P>
where
    P: AsRef<Path>,
{
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Error

#[derive(Debug, Display, Error)]
#[display("stream error")]
pub struct Error;

// -------------------------------------------------------------------------------------------------

// Facets

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Facets(
    #[new(name(position))] pub(crate) Position,
    #[new(name(timestamp))] pub(crate) Timestamp,
);

// -------------------------------------------------------------------------------------------------

// Stream

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    #[debug("Database")]
    database: Database,
    next: Position,
    storage: Storage,
}

impl Stream {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn len(&self) -> u64 {
        self.next.0
    }
}

impl Append for Stream {
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        (&mut || self.database.batch(), &mut self.next, &self.storage).append(events, after)
    }
}

// -------------------------------------------------------------------------------------------------

// Position

#[rustfmt::skip]
#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[derive(Add, AddAssign, Sub, SubAssign)]
#[new(const_fn)]
pub struct Position(#[new(name(position))] pub(crate) u64);

impl Position {
    pub const MAX: Self = Self::new(u64::MAX);
    pub const MIN: Self = Self::new(u64::MIN);
}

impl Add<u64> for Position {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for Position {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::MIN
    }
}

impl Sub<u64> for Position {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<u64> for Position {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

// -------------------------------------------------------------------------------------------------

// Result

pub type Result<T, E = Error> = result::Result<T, Report<E>>;

// -------------------------------------------------------------------------------------------------

// Timestamp

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(#[new(name(nanos))] pub(crate) u64);

impl Timestamp {
    pub fn now() -> Result<Self> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .change_context(Error)
            .attach("failed to get epoch duration")?;

        let nanos = u64::try_from(duration.as_nanos())
            .change_context(Error)
            .attach("failed to get epoch duration as nanos")?;

        Ok(Self::new(nanos))
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::operations::{
    Append,
    Select,
};
