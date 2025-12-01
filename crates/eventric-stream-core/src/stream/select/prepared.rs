use std::{
    mem::MaybeUninit,
    sync::Arc,
};

use fancy_constructor::new;

use crate::stream::{
    iterate::{
        cache::Cache,
        iter::Iter,
    },
    select::{
        Selection,
        SelectionHash,
        SelectionHashRef,
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

impl<const N: usize> Data for Selections<N> {
    type Data = Arc<[Filter; N]>;
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
        let selection_hash_ref: SelectionHashRef<'_> = (&selection).into();
        let selection_hash: SelectionHash = (&selection_hash_ref).into();

        cache.populate(&selection_hash_ref);

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

#[allow(unsafe_code)]
#[rustfmt::skip]
impl<const N: usize> From<Selections<N>> for Prepared<Selections<N>> {
    fn from(selections: Selections<N>) -> Self {
        let cache = Arc::new(Cache::default());

        let mut filters: [MaybeUninit<Filter>; N] = [const { MaybeUninit::uninit() }; N];
        let mut selection_hashes: [MaybeUninit<SelectionHash>; N] = [const { MaybeUninit::uninit() }; N];

        for (i, selection) in selections.0.iter().enumerate() {
            let selection_hash_ref: SelectionHashRef<'_> = selection.into();
            let selection_hash: SelectionHash = (&selection_hash_ref).into();
            let filter = Filter::new(&selection_hash);

            cache.populate(&selection_hash_ref);

            filters[i].write(filter);
            selection_hashes[i].write(selection_hash);
        }

        let filters = Arc::new(filters.map(|filter| unsafe { filter.assume_init() }));
        let selection_hashes = selection_hashes.map(|selection_hash| unsafe { selection_hash.assume_init() });

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

impl<const N: usize> Source for Prepared<Selections<N>> {
    type Iterator = Iter<Selections<N>>;
    type Prepared = Self;

    fn prepare(self) -> Self::Prepared {
        self
    }
}
