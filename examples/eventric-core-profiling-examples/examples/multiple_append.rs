use std::time::Instant;

use eventric_core::{
    Data,
    Error,
    Event,
    Identifier,
    Stream,
    Tag,
    Version,
};

// =================================================================================================
// Multiple Append
// =================================================================================================

// Configuration

static PATH: &str = "./temp";

// -------------------------------------------------------------------------------------------------

// Profile

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn main() -> Result<(), Error> {
    let mut stream = Stream::builder(PATH).temporary(true).open()?;

    let count = 10f64;
    let events = (0..count as u64)
        .map(|_| {
            Event::new(
                Data::new("Hello World".bytes().collect()),
                Identifier::new("test_identifier".into()),
                Vec::from_iter([Tag::new("test_tag_a".into()), Tag::new("test_tag_b".into())]),
                Version::new(0),
            )
        })
        .collect::<Vec<_>>();

    let iterations = 100_000f64;
    let start = Instant::now();

    for _ in 0..iterations as u64 {
        stream.append(&events, None).unwrap();
    }

    let duration = Instant::now().duration_since(start);
    let events_per_sec = ((iterations * count) / duration.as_secs_f64()) as u64;

    println!("time: {duration:?} ({events_per_sec} events/sec)",);

    Ok(())
}
