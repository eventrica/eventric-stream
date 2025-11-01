//! The [`event`][self] module contains the constituent components for events,
//! both pre- and post- stream append, as well as types related to specifying
//! events within queries.

pub(crate) mod data;
pub(crate) mod identifier;
pub(crate) mod position;
pub(crate) mod specifier;
pub(crate) mod tag;
pub(crate) mod timestamp;
pub(crate) mod version;

use fancy_constructor::new;

use crate::event::{
    identifier::{
        IdentifierHash,
        IdentifierHashRef,
    },
    tag::{
        TagHash,
        TagHashRef,
    },
};

// =================================================================================================
// Event
// =================================================================================================

// Ephemeral

/// The [`EphemeralEvent`] type represents an event which has not yet been
/// persisted (by appending it to an event stream). It is effectively a pending
/// event from the perspective of an event stream system, and as such doesn't
/// yet have some of the properties of an event which has been persisted (in
/// this case a [`PersistentEvent`]) such as a [`Position`] within the stream or
/// a [`Timestamp`].
#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn, name(new_inner), vis())]
pub struct EphemeralEvent {
    data: Data,
    identifier: Identifier,
    tags: Vec<Tag>,
    version: Version,
}

impl EphemeralEvent {
    /// Constructs a new [`EphemeralEvent`] instance, given appropriate event
    /// components. Each of the event components is validated (where relevant)
    /// on construction, so the constructor function is guaranteed to succeed.
    ///
    /// Note that while an event must have an [`Identifier`] and [`Version`],
    /// the supplied value of `T` which may be converted to an iterator of
    /// [`Tag`] instances may be empty, which is valid, as events may have
    /// zero or more logical tags.
    #[must_use]
    pub fn new<T>(data: Data, identifier: Identifier, tags: T, version: Version) -> Self
    where
        T: IntoIterator<Item = Tag>,
    {
        Self::new_inner(data, identifier, tags.into_iter().collect(), version)
    }
}

impl EphemeralEvent {
    /// Returns a reference to the [`Data`] value of the event.
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns a reference to the [`Identifier`] value of the event.
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// Returns a reference to the collection of [`Tag`] values of the event
    /// (which may be empty).
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Returns a reference to the [`Version`] value of the event.
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct EphemeralEventHashRef<'a> {
    pub data: &'a Data,
    pub identifier: IdentifierHashRef<'a>,
    pub tags: Vec<TagHashRef<'a>>,
    pub version: Version,
}

impl<'a> From<&'a EphemeralEvent> for EphemeralEventHashRef<'a> {
    fn from(event: &'a EphemeralEvent) -> Self {
        Self::new(
            event.data(),
            event.identifier().into(),
            event.tags().iter().map(Into::into).collect(),
            *event.version(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Persistent

/// The [`PersistentEvent`] type represents an event which has been appended to
/// a [`Stream`][stream], and has now been returned by a query or similar
/// operation. A [`PersistentEvent`] is immutable from the perspective of a
/// logical stream of events - the stream is an append-only data structure, and
/// once an event is part of a stream, it will always exist in that form and at
/// that [`Position`];
///
/// Note that the [`Timestamp`] is added during the append operation, and the
/// [`Position`] is also determined at this stage. The same event will always be
/// returned from a stream given the same [`Position`].
///
/// [stream]: crate::stream::Stream
#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn, vis(pub(crate)))]
pub struct PersistentEvent {
    data: Data,
    identifier: Identifier,
    position: Position,
    tags: Vec<Tag>,
    timestamp: Timestamp,
    version: Version,
}

impl PersistentEvent {
    /// Returns a reference to the [`Data`] value of the event.
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns a reference to the [`Identifier`] value of the event.
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// Returns a reference to the [`Position`] value of the event, which is
    /// ordinal position of the event in the relevant [`Stream`][stream].
    ///
    /// Note that this is **NOT** the position of the event within an iteration
    /// over the stream given by a [`Stream::query`][query].
    ///
    /// [stream]: crate::stream::Stream
    /// [query]: crate::stream::Stream::query
    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Returns a reference to the collection of [`Tag`] values of the event
    /// (which may be empty).
    ///
    /// Note that a retrieved collection of [`Tag`] values may be empty or
    /// incomplete even if the event had tag values when appended. This is
    /// dependent on the configured query behaviour when retrieving the
    /// event. See [`Stream::query`][query] and [`query::Options`][options] for
    /// more detail.
    ///
    /// [query]: crate::stream::Stream::query
    /// [options]: crate::stream::query::Options
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Returns a reference to the [`Timestamp`] value of the event, which was
    /// generated when the event was appended to the stream.
    #[must_use]
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    /// Returns a reference to the [`Version`] value of the event.
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct PersistentEventHash {
    pub data: Data,
    pub identifier: IdentifierHash,
    pub position: Position,
    pub tags: Vec<TagHash>,
    pub timestamp: Timestamp,
    pub version: Version,
}

impl PersistentEventHash {
    #[must_use]
    #[rustfmt::skip]
    pub fn take(self) -> (Data, IdentifierHash, Position, Vec<TagHash>, Timestamp, Version) {
        (
            self.data,
            self.identifier,
            self.position,
            self.tags,
            self.timestamp,
            self.version,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    data::Data,
    identifier::Identifier,
    position::Position,
    specifier::{
        AnyRange,
        Specifier,
    },
    tag::Tag,
    timestamp::Timestamp,
    version::Version,
};
