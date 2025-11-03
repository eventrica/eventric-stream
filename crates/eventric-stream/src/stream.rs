//! The [`stream`][self] module contains the core stream abstraction,
//! along with support for configuring and opening stream instances. Sub-modules
//! contain types related to appending events to the stream, and querying the
//! stream for previously appended events.

pub mod append;
pub mod query;

pub(crate) mod data;

use std::path::Path;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::Database;

use crate::{
    error::Error,
    event::position::Position,
    stream::data::Data,
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

// Building

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
        *self.next
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
