#![allow(clippy::missing_panics_doc)]

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use eventric_core::{
    Condition,
    Data,
    Event,
    Identifier,
    Stream,
    Tag,
    Version,
};

// =================================================================================================
// Append
// =================================================================================================

// Configuration

static PATH: &str = "./temp";

// -------------------------------------------------------------------------------------------------

// Benches

pub fn single_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("append");

    group.sample_size(10).bench_function("1000 x 1 event", |b| {
        let mut stream = Stream::configure(PATH).temporary(true).open().unwrap();

        let events = [Event::new(
            Data::new("Hello World".bytes().collect()),
            Identifier::new("test_identifier".into()),
            Vec::from_iter([Tag::new("test_tag_a".into()), Tag::new("test_tag_b".into())]),
            Version::new(0),
        )];

        b.iter(|| {
            for _ in 0..1_000 {
                stream.append(&events, Condition::default()).unwrap();
            }
        });
    });

    group.finish();
}

pub fn multiple_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("append");

    group
        .sample_size(10)
        .bench_function("1000 x 10 events", |b| {
            let mut stream = Stream::configure(PATH).temporary(true).open().unwrap();

            let events = (0..10)
                .map(|_| {
                    Event::new(
                        Data::new("Hello World".bytes().collect()),
                        Identifier::new("test_identifier".into()),
                        Vec::from_iter([
                            Tag::new("test_tag_a".into()),
                            Tag::new("test_tag_b".into()),
                        ]),
                        Version::new(0),
                    )
                })
                .collect::<Vec<_>>();

            b.iter(|| {
                for _ in 0..1_000 {
                    stream.append(&events, Condition::default()).unwrap();
                }
            });
        });

    group.finish();
}

criterion_group!(benches, single_append, multiple_append);
criterion_main!(benches);
