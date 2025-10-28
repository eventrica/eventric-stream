#![allow(clippy::missing_panics_doc)]

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use eventric_core::{
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Tag,
        Version,
    },
    stream::Stream,
};

// =================================================================================================
// Append
// =================================================================================================

// Single

pub fn single_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("append");

    group.sample_size(10).bench_function("1000 x 1 event", |b| {
        let mut stream = Stream::builder(eventric_core::temp_path())
            .temporary(true)
            .open()
            .unwrap();

        let events = [EphemeralEvent::new(
            Data::new("Hello World").unwrap(),
            Identifier::new("test_identifier").unwrap(),
            Vec::from_iter([
                Tag::new("test_tag_a").unwrap(),
                Tag::new("test_tag_b").unwrap(),
            ]),
            Version::new(0),
        )];

        b.iter_with_large_drop(|| {
            for _ in 0..1_000 {
                stream.append(&events, None).unwrap();
            }
        });

        drop(stream);
    });

    group.finish();
}

// Multiple

pub fn multiple_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("append");

    group
        .sample_size(10)
        .bench_function("1000 x 10 events", |b| {
            let mut stream = Stream::builder(eventric_core::temp_path())
                .temporary(true)
                .open()
                .unwrap();

            let events = (0..10)
                .map(|_| {
                    EphemeralEvent::new(
                        Data::new("Hello World").unwrap(),
                        Identifier::new("test_identifier").unwrap(),
                        Vec::from_iter([
                            Tag::new("test_tag_a").unwrap(),
                            Tag::new("test_tag_b").unwrap(),
                        ]),
                        Version::new(0),
                    )
                })
                .collect::<Vec<_>>();

            b.iter_with_large_drop(|| {
                for _ in 0..1_000 {
                    stream.append(&events, None).unwrap();
                }
            });

            drop(stream);
        });

    group.finish();
}

criterion_group!(benches, single_append, multiple_append);
criterion_main!(benches);
