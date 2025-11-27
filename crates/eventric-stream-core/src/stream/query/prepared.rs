use std::sync::Arc;

use fancy_constructor::new;
use itertools::Itertools;

use crate::stream::{
    iterate::{
        cache::Cache,
        iter::Iter,
    },
    query::{
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

impl Data for Vec<Query> {
    type Data = Arc<Vec<Filter>>;
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

// Vec<Query>

impl From<Vec<Query>> for Prepared<Vec<Query>> {
    fn from(queries: Vec<Query>) -> Self {
        let cache = Arc::new(Cache::default());
        let query_hashes = queries
            .iter()
            .map_into()
            .inspect(|query_hash_ref| cache.populate(query_hash_ref))
            .map_into()
            .collect_vec();

        let filters = query_hashes.iter().map(Filter::new).collect();
        let filters = Arc::new(filters);

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

impl Source for Prepared<Vec<Query>> {
    type Iterator = Iter<Vec<Query>>;
    type Prepared = Self;

    fn prepare(self) -> Self::Prepared {
        self
    }
}
