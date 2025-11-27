use std::sync::Arc;

use crate::stream::{
    iterate::{
        build::Build,
        cache::Cache,
        iter::{
            Iter,
            IterMulti,
        },
    },
    query::{
        Query,
        QueryHash,
        QueryHashRef,
        QueryMultiOptimized,
        QueryOptimized,
        filter::Filter,
    },
};

// =================================================================================================
// Source
// =================================================================================================

/// .
pub trait Source
where
    Self::Iterator: Build<Self::Optimized> + DoubleEndedIterator + Iterator,
    Self::Optimized: AsRef<QueryHash>,
{
    /// .
    type Iterator;
    /// .
    #[allow(private_bounds)]
    type Optimized;

    /// .
    fn optimize(self) -> Self::Optimized;
}

impl Source for Query {
    type Iterator = Iter;
    type Optimized = QueryOptimized;

    fn optimize(self) -> Self::Optimized {
        let cache = Arc::new(Cache::default());
        let query_hash_ref: QueryHashRef<'_> = (&self).into();
        let query_hash: QueryHash = (&query_hash_ref).into();

        cache.populate(&query_hash_ref);

        QueryOptimized::new(cache, query_hash)
    }
}

impl Source for Vec<Query> {
    type Iterator = IterMulti;
    type Optimized = QueryMultiOptimized;

    fn optimize(self) -> Self::Optimized {
        let cache = Arc::new(Cache::default());
        let query_hashes = self
            .iter()
            .map(Into::<QueryHashRef<'_>>::into)
            .inspect(|query_hash_ref| cache.populate(query_hash_ref))
            .map(Into::<QueryHash>::into)
            .collect::<Vec<_>>();

        let filters = query_hashes.iter().map(Filter::new).collect::<Vec<_>>();
        let filters = Arc::new(filters);

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selector hashess together, even though that will technically work,
        // it could be horribly inefficient.

        let query_hash = QueryHash::new(
            query_hashes
                .into_iter()
                .flat_map(|query_hash| query_hash.0)
                .collect::<Vec<_>>(),
        );

        QueryMultiOptimized::new(cache, filters, query_hash)
    }
}

impl Source for QueryOptimized {
    type Iterator = Iter;
    type Optimized = Self;

    fn optimize(self) -> Self::Optimized {
        self
    }
}

impl Source for QueryMultiOptimized {
    type Iterator = IterMulti;
    type Optimized = Self;

    fn optimize(self) -> Self::Optimized {
        self
    }
}
