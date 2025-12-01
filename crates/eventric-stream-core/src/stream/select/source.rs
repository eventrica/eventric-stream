use crate::stream::{
    iterate::build::Build,
    select::SelectionHash,
};

// =================================================================================================
// Source
// =================================================================================================

/// .
#[allow(private_bounds)]
pub trait Source
where
    Self::Iterator: Build<Self::Prepared> + DoubleEndedIterator + Iterator,
    Self::Prepared: AsRef<SelectionHash>,
{
    /// .
    type Iterator;
    /// .
    #[allow(private_bounds)]
    type Prepared;

    /// .
    fn prepare(self) -> Self::Prepared;
}
