pub mod event;
pub mod query;
pub mod stream;

use std::{
    error::Error,
    ops::Range,
    path::Path,
};

use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    model::event::{
        Identifier,
        InsertionEvent,
        Version,
    },
    persistence::{
        self,
        Context,
        Keyspaces,
        Read,
        Write,
        data,
    },
};

// =================================================================================================
// Model
// =================================================================================================

// -------------------------------------------------------------------------------------------------

// Stream

#[derive(new, Debug)]
#[new(name(inner_new), vis())]
pub struct Stream {
    context: Context,
    keyspaces: Keyspaces,
    position: Position,
}

impl Stream {
    pub fn new<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let context = persistence::context(path)?;
        let keyspaces = persistence::keyspaces(&context)?;

        let len = data::len(&Read::new(&keyspaces))?;
        let position = len.into();

        Ok(Self::inner_new(context, keyspaces, position))
    }
}

impl Stream {
    pub fn append<E>(&mut self, events: E) -> Result<(), Box<dyn Error>>
    where
        E: IntoIterator<Item = InsertionEvent>,
    {
        let mut batch = self.context.as_ref().batch();

        {
            let mut write = Write::new(&mut batch, &self.keyspaces);

            for event in events {
                persistence::insert(&mut write, self.position, event);

                self.position.increment();
            }
        }

        batch.commit()?;

        Ok(())
    }
}

impl Stream {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        data::is_empty(&Read::new(&self.keyspaces))
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        data::len(&Read::new(&self.keyspaces))
    }
}

// -------------------------------------------------------------------------------------------------

// Position

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis())]
pub struct Position(#[new(into)] u64);

impl Position {
    pub(crate) fn increment(&mut self) {
        self.0 += 1;
    }

    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl<T> From<T> for Position
where
    T: Into<u64>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// Specifier

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Specifier(#[new(into)] Identifier, #[new(into)] Option<Range<Version>>);

impl Specifier {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&Range<Version>> {
        self.1.as_ref()
    }

    #[must_use]
    pub fn take(self) -> (Identifier, Option<Range<Version>>) {
        (self.0, self.1)
    }
}

impl<T, U> From<(T, U)> for Specifier
where
    T: Into<Identifier>,
    U: Into<Option<Range<Version>>>,
{
    fn from(value: (T, U)) -> Self {
        Self::new(value.0, value.1)
    }
}
