//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub mod append;
pub mod iterate;
pub mod select;

pub(crate) mod data;

use std::path::Path;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::Database;

use crate::{
    error::Error,
    event::{
        CandidateEvent,
        position::Position,
    },
    stream::{
        append::{
            Append,
            AppendSelect,
        },
        data::Data,
        iterate::{
            Iterate,
            IterateSelect,
            build::Build,
            iter::Iter,
        },
        select::Source,
    },
};

// =================================================================================================
// Stream
// =================================================================================================

/// The [`Stream`] type is the central element of Eventric Stream. All
/// interactions happen relative to a [`Stream`] instance, whether appending new
/// events or querying existing events, and any higher-level libraries are built
/// on this underlying abstraction.
///
/// To open a new [`Stream`] instance use a [`Builder`], which can be obtained
/// using the [`Stream::builder`] function. Once a new [`Stream`] instance has
/// been opened, see [`Stream::append`] and [`Stream::query`] for information on
/// how to work with the stream and related events.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    #[debug("Database")]
    database: Database,
    data: Data,
    next: Position,
}

// Builder

impl Stream {
    /// Constructs a new [`Builder`] which which can be used to configure the
    /// properties of a new [`Stream`] to be opened..
    pub fn builder<P>(path: P) -> Builder<P>
    where
        P: AsRef<Path>,
    {
        Builder::new(path)
    }
}

// Properties

impl Stream {
    /// Returns whether this [`Stream`] is empty (i.e. has no events stored
    /// within it).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the current length of this [`Stream`] (i.e. how many events are
    /// contained within it).
    #[must_use]
    pub fn len(&self) -> u64 {
        self.next.value
    }
}

// Split

impl Stream {
    /// .
    #[must_use]
    pub fn split(self) -> (StreamReader, StreamWriter) {
        (
            StreamReader::new(self.data.clone()),
            StreamWriter::new(self.database, self.data, self.next),
        )
    }
}

// Append

impl Append for Stream {
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
    {
        append::append(&self.database, &self.data, &mut self.next, events, after)
    }
}

impl AppendSelect for Stream {
    #[rustfmt::skip]
    fn append_select<E, S>(&mut self, events: E, source: S, after: Option<Position>) -> Result<(Position, S::Prepared), Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        S: Source,
    {
        append::append_select(&self.database, &self.data, &mut self.next, events, source, after)
    }
}

// Iterate

impl Iterate for Stream {
    fn iter(&self, from: Option<Position>) -> Iter<()> {
        iterate::iter(&self.data, from)
    }
}

#[allow(private_bounds)]
impl IterateSelect for Stream {
    fn iter_select<S>(&self, source: S, from: Option<Position>) -> (S::Iterator, S::Prepared)
    where
        S: Source,
        S::Iterator: Build<S::Prepared>,
    {
        iterate::iter_select(&self.data, source, from)
    }
}

// -------------------------------------------------------------------------------------------------

// Reader

/// .
#[derive(new, Clone, Debug)]
#[new(const_fn, vis())]
pub struct StreamReader {
    data: Data,
}

// Iterate

impl Iterate for StreamReader {
    fn iter(&self, from: Option<Position>) -> Iter<()> {
        iterate::iter(&self.data, from)
    }
}

#[allow(private_bounds)]
impl IterateSelect for StreamReader {
    fn iter_select<S>(&self, source: S, from: Option<Position>) -> (S::Iterator, S::Prepared)
    where
        S: Source,
        S::Iterator: Build<S::Prepared>,
    {
        iterate::iter_select(&self.data, source, from)
    }
}

// -------------------------------------------------------------------------------------------------

// Writer

/// .
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct StreamWriter {
    #[debug("Database")]
    database: Database,
    data: Data,
    next: Position,
}

// Append

impl Append for StreamWriter {
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
    {
        append::append(&self.database, &self.data, &mut self.next, events, after)
    }
}

impl AppendSelect for StreamWriter {
    #[rustfmt::skip]
    fn append_select<E, S>(&mut self, events: E, source: S, after: Option<Position>) -> Result<(Position, S::Prepared), Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        S: Source,
    {
        append::append_select(&self.database, &self.data, &mut self.next, events, source, after)
    }
}

// -------------------------------------------------------------------------------------------------

//  Builder

/// The [`Builder`] type configures and creates new [`Stream`] instances.
/// An instance of [`Builder`] can be obtained by calling [`Stream::builder`]
/// with a chosen path for stream storage.
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
    /// Attempts to open a new [`Stream`] instance given the configured path and
    /// options.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an IO error, or if the underlying database
    /// cannot be opened/read.
    pub fn open(self) -> Result<Stream, Error> {
        let database = Database::builder(self.path)
            .temporary(self.temporary.unwrap_or_default())
            .open()?;

        let data = Data::open(&database)?;
        let position = Position::new(data.events.len()?);

        Ok(Stream::new(database, data, position))
    }
}

impl<P> Builder<P>
where
    P: AsRef<Path>,
{
    /// Sets whether or not the [`Stream`] should be temporary (temporary
    /// streams delete their underlying data storage when dropped, and are thus
    /// ephemeral - this is only generally required when developing/testing, and
    /// should never be set for a production system).
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}
