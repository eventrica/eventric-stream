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
    query::{
        Queries,
        Query,
        QueryHash,
        QueryHashRef,
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

impl Data for Query {
    type Data = ();
}

impl<const N: usize> Data for Queries<N> {
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
    pub(crate) query_hash: QueryHash,
}

impl<T> AsRef<QueryHash> for Prepared<T>
where
    T: Data,
{
    fn as_ref(&self) -> &QueryHash {
        &self.query_hash
    }
}

// Query

impl From<Query> for Prepared<Query> {
    fn from(query: Query) -> Self {
        let cache = Arc::new(Cache::default());
        let query_hash_ref: QueryHashRef<'_> = (&query).into();
        let query_hash: QueryHash = (&query_hash_ref).into();

        cache.populate(&query_hash_ref);

        Self::new(cache, (), query_hash)
    }
}

impl Source for Prepared<Query> {
    type Iterator = Iter<Query>;
    type Prepared = Self;

    fn prepare(self) -> Self::Prepared {
        self
    }
}

// Queries

#[allow(unsafe_code)]
impl<const N: usize> From<Queries<N>> for Prepared<Queries<N>> {
    fn from(queries: Queries<N>) -> Self {
        let cache = Arc::new(Cache::default());

        let mut filters: [MaybeUninit<Filter>; N] = [const { MaybeUninit::uninit() }; N];
        let mut query_hashes: [MaybeUninit<QueryHash>; N] = [const { MaybeUninit::uninit() }; N];

        for (i, query) in queries.0.iter().enumerate() {
            let query_hash_ref: QueryHashRef<'_> = query.into();
            let query_hash: QueryHash = (&query_hash_ref).into();
            let filter = Filter::new(&query_hash);

            cache.populate(&query_hash_ref);

            filters[i].write(filter);
            query_hashes[i].write(query_hash);
        }

        let filters = Arc::new(filters.map(|filter| unsafe { filter.assume_init() }));
        let query_hashes = query_hashes.map(|query_hash| unsafe { query_hash.assume_init() });

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selector hashess together, even though that will technically work,
        // it could be horribly inefficient.

        let query_hash = QueryHash::new(
            query_hashes
                .into_iter()
                .flat_map(|query_hash| query_hash.0)
                .collect(),
        );

        Self::new(cache, filters, query_hash)
    }
}

impl<const N: usize> Source for Prepared<Queries<N>> {
    type Iterator = Iter<Queries<N>>;
    type Prepared = Self;

    fn prepare(self) -> Self::Prepared {
        self
    }
}
