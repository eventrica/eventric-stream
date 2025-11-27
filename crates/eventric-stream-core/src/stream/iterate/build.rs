use crate::stream::data::{
    events::PersistentEventHashIterator,
    references::References,
};

// =================================================================================================
// Build
// =================================================================================================

/// .
pub trait Build<T>
where
    Self: DoubleEndedIterator + Iterator,
{
    /// .
    #[allow(private_interfaces)]
    fn build(iter: PersistentEventHashIterator, prepared: &T, references: References) -> Self;
}
