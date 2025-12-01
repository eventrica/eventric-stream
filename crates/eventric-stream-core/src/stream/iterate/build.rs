use crate::stream::data::{
    events::EventHashIter,
    references::References,
};

// =================================================================================================
// Build
// =================================================================================================

/// .
pub(crate) trait Build<T>
where
    Self: DoubleEndedIterator + Iterator,
{
    /// .
    #[allow(private_interfaces)]
    fn build(iter: EventHashIter, prepared: &T, references: References) -> Self;
}
