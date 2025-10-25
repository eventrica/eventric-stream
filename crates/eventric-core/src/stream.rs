pub mod append;
pub mod query;

use std::path::Path;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::Database;

use crate::{
    data::Data,
    error::Error,
    model::stream::position::Position,
};

// =================================================================================================
// Stream
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    #[debug("Database")]
    database: Database,
    data: Data,
    position: Position,
}

// Configuration

impl Stream {
    pub fn builder<P>(path: P) -> StreamBuilder<P>
    where
        P: AsRef<Path>,
    {
        StreamBuilder::new(path)
    }
}

// Properties

impl Stream {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.events.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> u64 {
        self.data.events.len()
    }
}

// -------------------------------------------------------------------------------------------------

// Stream Builder

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
    pub fn open(self) -> Stream {
        let database = Database::builder(self.path)
            .temporary(self.temporary.unwrap_or_default())
            .open()
            .map_err(Error::from)
            .expect("database open: database error");

        let data = Data::open(&database);
        let position = Position::new(data.events.len());

        Stream::new(database, data, position)
    }
}

impl<P> StreamBuilder<P>
where
    P: AsRef<Path>,
{
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}
