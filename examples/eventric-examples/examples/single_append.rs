use std::{
    collections::BTreeSet,
    time::Instant,
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
            append::Append as _,
        },
    },
};

// =================================================================================================
// Single Append
// =================================================================================================

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn main() {
    let mut stream = Stream::builder(eventric_stream::utils::temp_path())
        .temporary(true)
        .open()
        .unwrap();

    let events = [Event::new(
        Data::new("Hello World").unwrap(),
        Facets::new(
            Type::new(Name::new("test_identifier").unwrap(), Version::new(0)),
            BTreeSet::from([
                Tag::new("test_tag_a").unwrap(),
                Tag::new("test_tag_b").unwrap(),
            ]),
        ),
        (),
    )];

    let iterations = 1_000_000f64;
    let start = Instant::now();

    for _ in 0..iterations as u64 {
        stream.append(events.clone(), Condition::new()).unwrap();
    }

    let duration = Instant::now().duration_since(start);
    let events_per_sec = (iterations / duration.as_secs_f64()) as u64;

    println!("time: {duration:?} ({events_per_sec} events/sec)");
}
