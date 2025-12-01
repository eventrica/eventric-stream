use std::time::Instant;

use eventric_stream::{
    error::Error,
    event::{
        CandidateEvent,
        Data,
        Identifier,
        Tag,
        Version,
    },
    stream::{
        Stream,
        append::Append as _,
    },
};

// =================================================================================================
// Multiple Append
// =================================================================================================

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn main() -> Result<(), Error> {
    let mut stream = Stream::builder(eventric_stream::temp_path())
        .temporary(true)
        .open()?;

    let count = 10f64;
    let events = (0..count as u64)
        .map(|_| {
            CandidateEvent::new(
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

    let iterations = 100_000f64;
    let start = Instant::now();

    for _ in 0..iterations as u64 {
        stream.append(events.clone(), None).unwrap();
    }

    let duration = Instant::now().duration_since(start);
    let events_per_sec = ((iterations * count) / duration.as_secs_f64()) as u64;

    println!("time: {duration:?} ({events_per_sec} events/sec)",);

    Ok(())
}
