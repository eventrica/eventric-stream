use std::sync::Arc;

use fancy_constructor::new;
use smallvec::SmallVec;

use crate::stream::{
    iterate::{
        cache::Cache,
        iter::Iter,
    },
    select::{
        Selection,
        SelectionHash,
        SelectionHashAndValue,
        Selections,
        filter::Filter,
        source::Source,
    },
};

// =================================================================================================
// Preparation
// =================================================================================================

// Data

pub(crate) trait Data {
    type Data;
}

impl Data for Selection {
    type Data = ();
}

impl Data for Selections {
    type Data = Arc<SmallVec<[Filter; 8]>>;
}

// -------------------------------------------------------------------------------------------------

// Prepared

/// .
#[allow(private_bounds)]
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Prepared<T>
where
    T: Data,
{
    pub(crate) cache: Arc<Cache>,
    pub(crate) data: T::Data,
    pub(crate) selection: SelectionHash,
}

impl<T> AsRef<SelectionHash> for Prepared<T>
where
    T: Data,
{
    fn as_ref(&self) -> &SelectionHash {
        &self.selection
    }
}

// Selection

impl From<Selection> for Prepared<Selection> {
    fn from(selection: Selection) -> Self {
        let cache = Arc::new(Cache::default());
        let selection_hash_and_value: SelectionHashAndValue = selection.into();

        cache.populate(&selection_hash_and_value);

        let selection_hash: SelectionHash = selection_hash_and_value.into();

        Self::new(cache, (), selection_hash)
    }
}

impl Source for Prepared<Selection> {
    type Iterator = Iter<Selection>;
    type Prepared = Self;

    fn prepare(self) -> Self::Prepared {
        self
    }
}

// Selections

impl From<Selections> for Prepared<Selections> {
    fn from(selections: Selections) -> Self {
        let cache = Arc::new(Cache::default());

        let selection_hashes = selections
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

impl Source for Prepared<Selections> {
    type Iterator = Iter<Selections>;
    type Prepared = Self;

    fn prepare(self) -> Self::Prepared {
        self
    }
}
