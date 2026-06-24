//! The event type and its components: the payload `Data`, the queryable
//! `Facets` (`Type` = `Name` + `Version`, plus tags), and the generic
//! `Event<M, T>` itself (candidate `Event<(), String>` before append, persisted
//! `Event<Metadata, u64>` from a query).

use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ops::Range,
};

use derive_more::AsRef;
use error_stack::ResultExt;
pub use eventric_macros::tag;
use fancy_constructor::new;
use pastey::paste;

use crate::{
    error::{
        Error,
        Result,
    },
    utils::{
        hashing,
        validation::{
            self,
            NoControlCharacters,
            NoPrecedingWhiteSpace,
            NoTrailingWhiteSpace,
            NotEmpty,
            Validate,
        },
    },
};

// =================================================================================================
// Event
// =================================================================================================

// Data

/// A validated, non-empty event payload.
#[derive(new, AsRef, Clone, Debug, Eq, PartialEq)]
#[as_ref([u8])]
#[new(const_fn, name(new_unvalidated))]
pub struct Data(#[new(name(data))] pub(crate) Vec<u8>);

impl Data {
    /// Creates a `Data` payload, validating that it is non-empty.
    pub fn new<D>(data: D) -> Result<Self>
    where
        D: Into<Vec<u8>>,
    {
        Self::new_unvalidated(data.into()).validate()
    }
}

impl Validate for Data {
    fn validate(self) -> Result<Self> {
        validation::validate(&self.0, "data", &[&NotEmpty]).change_context(Error)?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Event

/// A stream event: a payload, its queryable facets, and metadata. Candidate
/// events (pre-append) are `Event<(), String>`; persisted events (from a query)
/// are `Event<Metadata, u64>`.
#[derive(new, Clone, Debug)]
pub struct Event<M, T>(
    #[new(name(data))] pub(crate) Data,
    #[new(name(facets))] pub(crate) Facets<T>,
    #[new(name(meta))] pub(crate) M,
);

macro_rules! event_from {
    ($from:ty, $to:ty) => {
        impl<M> From<Event<M, $from>> for Event<M, $to> {
            fn from(event: Event<M, $from>) -> Self {
                Self(event.0, event.1.into(), event.2)
            }
        }
    };
}

event_from!(String, u64);

impl<M, T> Event<M, T> {
    /// The event payload.
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.0
    }

    /// The event's queryable facets (its type and its tags).
    #[must_use]
    pub fn facets(&self) -> &Facets<T> {
        &self.1
    }

    /// The event's metadata (for a persisted event, its position and
    /// timestamp).
    #[must_use]
    pub fn meta(&self) -> &M {
        &self.2
    }
}

// -------------------------------------------------------------------------------------------------

// Facets

/// An event's queryable facets: its type and its set of tags.
#[derive(new, Clone, Debug)]
pub struct Facets<T>(
    #[new(name(ty))] pub(crate) Type<T>,
    #[new(name(tags))] pub(crate) BTreeSet<Tag<T>>,
);

macro_rules! facets_from {
    ($from:ty, $to:ty) => {
        impl From<Facets<$from>> for Facets<$to> {
            fn from(facets: Facets<$from>) -> Self {
                Self(
                    facets.0.into(),
                    facets.1.into_iter().map(Into::into).collect(),
                )
            }
        }
    };
}

facets_from!(String, u64);

impl<T> Facets<T> {
    /// The event's type (its name and version).
    #[must_use]
    pub fn ty(&self) -> &Type<T> {
        &self.0
    }

    /// The event's tags.
    #[must_use]
    pub fn tags(&self) -> &BTreeSet<Tag<T>> {
        &self.1
    }
}

// -------------------------------------------------------------------------------------------------

// Name & Tag

macro_rules! string_type {
    ($name:ident) => {
        paste! {
            /// A validated string newtype, generic over `T`: the `String` form
            /// holds the original value, the `u64` form its stable hash.
            #[derive(new, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
            #[new(const_fn, name(new_unvalidated), vis())]
            pub struct $name<T>(pub(crate) T);

            impl $name<String> {
                /// Creates the value, validating it is non-empty, free of
                /// control characters, and has no leading/trailing whitespace.
                pub fn new<T>([< $name:lower >]: T) -> Result<Self>
                where
                    T: Into<String>,
                {
                    Self::new_unvalidated([< $name:lower >].into()).validate()
                }
            }

            impl From<$name<String>> for $name<u64> {
                fn from([< $name:lower >]: $name<String>) -> Self {
                    Self(hashing::hash(&[< $name:lower >].0))
                }
            }

            impl Validate for $name<String> {
                fn validate(self) -> Result<Self> {
                    validation::validate(&self.0, stringify!([< $name:snake >]), &[
                        &NotEmpty,
                        &NoControlCharacters,
                        &NoPrecedingWhiteSpace,
                        &NoTrailingWhiteSpace,
                    ])
                    .change_context(Error)?;

                    Ok(self)
                }
            }
        }
    };
}

string_type!(Name);
string_type!(Tag);

impl Tag<String> {
    /// Creates a tag from a `prefix` and a `value`, formatted as `prefix:value`
    /// (e.g. `student:3242`) and then validated. The `prefix:value` shape is
    /// the tag's own convention, owned here rather than by the callers that
    /// build it.
    pub fn prefixed(prefix: impl std::fmt::Display, value: impl std::fmt::Display) -> Result<Self> {
        Self::new(format!("{prefix}:{value}"))
    }
}

// -------------------------------------------------------------------------------------------------

// Type

/// An event's type: a `Name` together with a `Version`.
#[derive(new, Clone, Debug)]
pub struct Type<T>(
    #[new(name(name))] pub(crate) Name<T>,
    #[new(name(version))] pub(crate) Version,
);

macro_rules! type_from {
    ($from:ty, $to:ty) => {
        impl From<Type<$from>> for Type<$to> {
            fn from(ty: Type<$from>) -> Self {
                Self(ty.0.into(), ty.1)
            }
        }
    };
}

type_from!(String, u64);

impl<T> Type<T> {
    /// The type's name.
    #[must_use]
    pub fn name(&self) -> &Name<T> {
        &self.0
    }

    /// The type's version.
    #[must_use]
    pub fn version(&self) -> Version {
        self.1
    }
}

// -------------------------------------------------------------------------------------------------

// Version

/// A `u8` schema version for an event type.
#[derive(new, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub struct Version(#[new(name(version))] pub(crate) u8);

impl Version {
    /// The maximum version.
    pub const MAX: Self = Self::new(u8::MAX);
    /// The minimum version.
    pub const MIN: Self = Self::new(u8::MIN);
}

impl PartialEq<Range<Self>> for Version {
    fn eq(&self, other: &Range<Self>) -> bool {
        self >= &other.start && self < &other.end
    }
}

impl PartialOrd<Range<Self>> for Version {
    fn partial_cmp(&self, other: &Range<Self>) -> Option<Ordering> {
        match self {
            _ if self < &other.start => Some(Ordering::Less),
            _ if self >= &other.end => Some(Ordering::Greater),
            _ => Some(Ordering::Equal),
        }
    }
}
