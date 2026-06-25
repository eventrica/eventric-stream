#![allow(clippy::missing_panics_doc)]

//! Retrieval benchmark for the dense-AND-sparse query shape.
//!
//! A selector that OR-combines two (dense) type names and AND-combines two
//! (sparse) tags — `(registered OR updated) AND account:42 AND region:eu` —
//! exercising the full index combinator tree: the type-side `OrIter`, the
//! tag-side `AndIter`, and the top `AndIter` that intersects the dense type
//! union with the sparse tag intersection.
//!
//! The stream is seeded with `n` events (parameterised) of which ~20 match,
//! spread evenly so the deepest match sits near position `n` — the worst case
//! for the single-step merge, which walks the dense union up to that last
//! match. The result is always 20, so the per-size wall-clock isolates
//! retrieval cost: a linear merge grows O(n); a seeking merge should stay
//! ~flat.

use std::collections::BTreeSet;

use criterion::{
    BenchmarkId,
    Criterion,
    criterion_group,
    criterion_main,
};
use eventric_stream::{
    event::{
        Data,
        Event,
        Facets,
        Name,
        Tag,
        Type,
        Version,
    },
    stream::{
        Stream,
        operate::{
            Condition,
            Selection,
            append::Append as _,
            select::{
                Select as _,
                Selector,
                TypeSelector,
            },
        },
    },
};

// =================================================================================================
// Select
// =================================================================================================

// The fixed result size across every stream size: exactly the events tagged
// both `account:42` and `region:eu`.
const MATCHES: usize = 20;

fn event(ty: &str, tags: Vec<String>) -> Event<(), String> {
    let tags = tags
        .into_iter()
        .map(|tag| Tag::new(tag).unwrap())
        .collect::<BTreeSet<_>>();

    Event::new(
        Data::new("payload").unwrap(),
        Facets::new(Type::new(Name::new(ty).unwrap(), Version::new(0)), tags),
        (),
    )
}

// Seed `n` events. Across each `n / 20` block exactly one event matches the
// query (`account:42` + `region:eu`), one carries only `account:42`, and one
// only `region:eu` (so each sparse tag stream is longer than their intersection
// — a non-trivial tag-side AND); the rest match neither tag. Types alternate so
// both type scans are dense. Deterministic, so the benchmark is reproducible.
fn seed(n: u64) -> Vec<Event<(), String>> {
    let step = n / MATCHES as u64;

    (0..n)
        .map(|i| {
            let ty = if i % 2 == 0 { "registered" } else { "updated" };

            let tags = match i % step {
                0 => vec!["account:42".to_owned(), "region:eu".to_owned()], // match
                1 => vec!["account:42".to_owned(), "region:us".to_owned()], // account only
                2 => vec!["account:99".to_owned(), "region:eu".to_owned()], // region only
                _ => vec![format!("account:acct_{i}"), format!("region:r{}", i % 37)], // neither
            };

            event(ty, tags)
        })
        .collect()
}

// `(registered OR updated) AND account:42 AND region:eu`.
fn query() -> Condition {
    let selector = Selector::types_and_tags(
        [
            TypeSelector::new("registered").unwrap(),
            TypeSelector::new("updated").unwrap(),
        ],
        [
            Tag::new("account:42").unwrap(),
            Tag::new("region:eu").unwrap(),
        ],
    );

    Condition::new().selections([Selection::new([selector])])
}

pub fn select_dense_and_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_dense_and_sparse");
    group.sample_size(20);

    for n in [10_000u64, 50_000, 100_000, 200_000] {
        let mut stream = Stream::builder(eventric_stream::utils::temp_path())
            .temporary(true)
            .open()
            .unwrap();

        stream.append(seed(n), Condition::new()).unwrap();

        // Fail fast (before timing) if the seed/query ever drift out of sync.
        let count = stream.select(query()).map(Result::unwrap).count();
        assert_eq!(count, MATCHES, "seed/query mismatch at n={n}");

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let count = stream.select(query()).map(Result::unwrap).count();
                assert_eq!(count, MATCHES);
                count
            });
        });

        drop(stream);
    }

    group.finish();
}

// The same query consumed in reverse (`.rev()`), to confirm the reverse path
// leapfrogs symmetrically with the forward one rather than degrading to the old
// single-step O(n).
pub fn select_dense_and_sparse_reverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_dense_and_sparse_reverse");
    group.sample_size(20);

    for n in [10_000u64, 50_000, 100_000, 200_000] {
        let mut stream = Stream::builder(eventric_stream::utils::temp_path())
            .temporary(true)
            .open()
            .unwrap();

        stream.append(seed(n), Condition::new()).unwrap();

        let count = stream.select(query()).rev().map(Result::unwrap).count();
        assert_eq!(count, MATCHES, "seed/query mismatch (reverse) at n={n}");

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let count = stream.select(query()).rev().map(Result::unwrap).count();
                assert_eq!(count, MATCHES);
                count
            });
        });

        drop(stream);
    }

    group.finish();
}

criterion_group!(
    benches,
    select_dense_and_sparse,
    select_dense_and_sparse_reverse
);
criterion_main!(benches);
