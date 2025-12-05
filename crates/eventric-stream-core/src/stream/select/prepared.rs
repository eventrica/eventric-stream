use std::sync::Arc;

use fancy_constructor::new;
use smallvec::SmallVec;

use crate::stream::select::{
    Selection,
    SelectionHash,
    SelectionHashAndValue,
    Selections,
    filter::Filter,
    iter::IterDefinition,
    lookup::Lookup,
};

// =================================================================================================
// Preparation
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct PreparedGen<T>
where
    T: IterDefinition,
{
    pub(crate) data: T::Data,
    pub(crate) lookup: Arc<Lookup>,
    pub(crate) selection: SelectionHash,
}

impl From<Selection> for PreparedGen<Selection> {
    fn from(selection: Selection) -> Self {
        let mut lookup = Lookup::default();

        let selection_hash_and_value: SelectionHashAndValue = selection.into();

        lookup.populate(&selection_hash_and_value);

        let lookup = Arc::new(lookup);
        let selection_hash: SelectionHash = selection_hash_and_value.into();

        Self::new((), lookup, selection_hash)
    }
}

impl From<Selections> for PreparedGen<Selections> {
    fn from(selections: Selections) -> Self {
        let mut lookup = Lookup::default();

        let selection_hashes = selections
            .0
            .into_iter()
            .map(Into::into)
            .inspect(|selection_hash_and_value| lookup.populate(selection_hash_and_value))
            .map(Into::into)
            .collect::<Vec<_>>();

        let filters = selection_hashes.iter().map(Filter::new).collect();
        let filters = Arc::new(filters);

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selector hashess together, even though that will technically work,
        // it could be horribly inefficient.

        let lookup = Arc::new(lookup);
        let selection_hash = SelectionHash::new(
            selection_hashes
                .into_iter()
                .flat_map(|selection_hash| selection_hash.0)
                .collect(),
        );

        Self::new(filters, lookup, selection_hash)
    }
}

// Prepared

/// .
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct PreparedSelection {
    pub(crate) lookup: Arc<Lookup>,
    pub(crate) selection: SelectionHash,
}

impl AsRef<SelectionHash> for PreparedSelection {
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

// Selection

impl From<Selection> for PreparedSelection {
    fn from(selection: Selection) -> Self {
        let mut lookup = Lookup::default();

        let selection_hash_and_value: SelectionHashAndValue = selection.into();

        lookup.populate(&selection_hash_and_value);

        let lookup = Arc::new(lookup);
        let selection_hash: SelectionHash = selection_hash_and_value.into();

        Self::new(lookup, selection_hash)
    }
}

// Multi

/// .
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct PreparedSelections {
    pub(crate) filters: Arc<SmallVec<[Filter; 8]>>,
    pub(crate) lookup: Arc<Lookup>,
    pub(crate) selection: SelectionHash,
}

impl AsRef<SelectionHash> for PreparedSelections {
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

impl From<Selections> for PreparedSelections {
    fn from(multi_selection: Selections) -> Self {
        let mut lookup = Lookup::default();

        let selection_hashes = multi_selection
            .0
            .into_iter()
            .map(Into::into)
            .inspect(|selection_hash_and_value| lookup.populate(selection_hash_and_value))
            .map(Into::into)
            .collect::<Vec<_>>();

        let filters = selection_hashes.iter().map(Filter::new).collect();
        let filters = Arc::new(filters);

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selector hashess together, even though that will technically work,
        // it could be horribly inefficient.

        let lookup = Arc::new(lookup);
        let selection_hash = SelectionHash::new(
            selection_hashes
                .into_iter()
                .flat_map(|selection_hash| selection_hash.0)
                .collect(),
        );

        Self::new(filters, lookup, selection_hash)
    }
}
