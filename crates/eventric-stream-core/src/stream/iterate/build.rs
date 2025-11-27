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
    fn build(optimization: &T, iter: PersistentEventHashIterator, references: References) -> Self;
}
