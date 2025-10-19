use eventric_core_model::{
    Event,
    SequencedEventRef,
};

// =================================================================================================
// Event
// =================================================================================================

pub trait Events<'a> = IntoIterator<Item = &'a Event>;

pub trait SequencedEvents<'a> = Iterator<Item = SequencedEventRef<'a>>;
