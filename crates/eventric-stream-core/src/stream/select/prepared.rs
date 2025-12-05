use std::sync::Arc;

use fancy_constructor::new;
use smallvec::SmallVec;

use crate::stream::{
    iterate::cache::Cache,
    select::{
        MultiSelection,
        Selection,
        SelectionHash,
        SelectionHashAndValue,
        filter::Filter,
    },
};

// =================================================================================================
// Preparation
// =================================================================================================

// Prepared

/// .
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Prepared {
    pub(crate) cache: Arc<Cache>,
    pub(crate) selection: SelectionHash,
}

impl AsRef<SelectionHash> for Prepared {
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

// Selection

impl From<Selection> for Prepared {
    fn from(selection: Selection) -> Self {
        let cache = Arc::new(Cache::default());
        let selection_hash_and_value: SelectionHashAndValue = selection.into();

        cache.populate(&selection_hash_and_value);

        let selection_hash: SelectionHash = selection_hash_and_value.into();

        Self::new(cache, selection_hash)
    }
}

// Multi

/// .
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct MultiPrepared {
    pub(crate) cache: Arc<Cache>,
    pub(crate) filters: Arc<SmallVec<[Filter; 8]>>,
    pub(crate) selection: SelectionHash,
}

impl AsRef<SelectionHash> for MultiPrepared {
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

impl From<MultiSelection> for MultiPrepared {
    fn from(multi_selection: MultiSelection) -> Self {
        let cache = Arc::new(Cache::default());

        let selection_hashes = multi_selection
            .0
            .into_iter()
            .map(Into::into)
            .inspect(|query_hash_ref| cache.populate(query_hash_ref))
            .map(Into::into)
            .collect::<Vec<_>>();

        let filters = selection_hashes.iter().map(Filter::new).collect();
        let filters = Arc::new(filters);

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selector hashess together, even though that will technically work,
        // it could be horribly inefficient.

        let selection = SelectionHash::new(
            selection_hashes
                .into_iter()
                .flat_map(|selection_hash| selection_hash.0)
                .collect(),
        );

        Self::new(cache, filters, selection)
    }
}
