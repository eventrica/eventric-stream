use std::sync::Arc;

use fancy_constructor::new;
use smallvec::SmallVec;

use crate::stream::select::{
    Selection,
    SelectionHash,
    Selections,
    filter::Filter,
    lookup::Lookup,
};

// =================================================================================================
// Prepared
// =================================================================================================

// Prepared

/// .
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Prepared {
    pub(crate) lookup: Arc<Lookup>,
    pub(crate) selection: SelectionHash,
}

impl AsRef<SelectionHash> for Prepared {
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

impl From<Selection> for Prepared {
    fn from(selection: Selection) -> Self {
        let mut lookup = Lookup::default();

        let selection_hash_and_value = selection.into();

        lookup.populate(&selection_hash_and_value);

        let lookup = Arc::new(lookup);
        let selection_hash = selection_hash_and_value.into();

        Self::new(lookup, selection_hash)
    }
}

// Prepared (Multiple)

/// .
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct PreparedMultiple {
    pub(crate) filters: Arc<SmallVec<[Filter; 8]>>,
    pub(crate) lookup: Arc<Lookup>,
    pub(crate) selection: SelectionHash,
}

impl AsRef<SelectionHash> for PreparedMultiple {
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

impl From<Selections> for PreparedMultiple {
    fn from(multi_selection: Selections) -> Self {
        let mut lookup = Lookup::default();

        let selection_hashes = multi_selection
            .0
            .into_iter()
            .map(Into::into)
            .inspect(|selection_hash_and_value| lookup.populate(selection_hash_and_value))
            .map(Into::into)
            .collect::<Vec<_>>();

        let lookup = Arc::new(lookup);

        let filters = selection_hashes.iter().map(Filter::new).collect();
        let filters = Arc::new(filters);

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selector hashess together, even though that will technically work,
        // it could be horribly inefficient.

        let selection_hash = SelectionHash::new(
            selection_hashes
                .into_iter()
                .flat_map(|selection_hash| selection_hash.0)
                .collect(),
        );

        Self::new(filters, lookup, selection_hash)
    }
}
