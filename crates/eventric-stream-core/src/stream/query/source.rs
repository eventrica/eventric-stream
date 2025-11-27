use crate::stream::{
    iterate::build::Build,
    query::QueryHash,
};

// =================================================================================================
// Source
// =================================================================================================

/// .
pub trait Source
where
    Self::Iterator: Build<Self::Prepared> + DoubleEndedIterator + Iterator,
    Self::Prepared: AsRef<QueryHash>,
{
    /// .
    type Iterator;
    /// .
    #[allow(private_bounds)]
    type Prepared;

    /// .
    fn prepare(self) -> Self::Prepared;
}
