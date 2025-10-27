pub mod append;
pub mod query;

use std::path::Path;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::Database;
use include_utils::include_md;

use crate::{
    data::Data,
    error::Error,
    model::stream::position::Position,
};

// =================================================================================================
// Stream
// =================================================================================================

#[doc = include_md!("README.md:stream")]
#[doc = include_md!("README.md:open_stream")]
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
    /// Constructs a new [`StreamBuilder`] which which can be used to configure
    /// the properties of a new [`Stream`] to be opened..
    pub fn builder<P>(path: P) -> StreamBuilder<P>
    where
        P: AsRef<Path>,
    {
        StreamBuilder::new(path)
    }
}

// Properties

impl Stream {
    /// Returns whether this [`Stream`] is empty (i.e. has no events stored
    /// within it).
    ///
    /// # Errors
    ///
    /// Returns an error if an underlying database IO error occurred.
    pub fn is_empty(&self) -> Result<bool, Error> {
        self.data.events.is_empty()
    }

    /// Returns the current length of this [`Stream`] (i.e. how many events are
    /// contained within it).
    ///
    /// # Errors
    ///
    /// Returns an error if an underlying database IO error occurred.
    pub fn len(&self) -> Result<u64, Error> {
        self.data.events.len()
    }
}

// -------------------------------------------------------------------------------------------------

// Stream Builder

/// The [`StreamBuilder`] type configures and creates new [`Stream`] instances.
/// An instance of [`StreamBuilder`] can be obtained by calling
/// [`Stream::builder`] with a chosen path for stream storage.
#[derive(new, Debug)]
#[new(vis())]
pub struct StreamBuilder<P>
where
    P: AsRef<Path>,
{
    path: P,
    #[new(default)]
    temporary: Option<bool>,
}

impl<P> StreamBuilder<P>
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

impl<P> StreamBuilder<P>
where
    P: AsRef<Path>,
{
    /// Sets whether or not the [`Stream`] should be temporary (temporary
    /// streams delete their underlying data storage when dropped, and are thus
    /// ephemeral - this is only generally required when developing/testing, and
    /// should generally never be set for a production system).
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}
