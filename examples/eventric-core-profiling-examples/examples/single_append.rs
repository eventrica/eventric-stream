use std::time::Instant;

use eventric_core::{
    error::Error,
    event::{
        Data,
        Event,
        Identifier,
        Tag,
        Version,
    },
    stream::Stream,
};

// =================================================================================================
// Single Append
// =================================================================================================

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn main() -> Result<(), Error> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    let events = [Event::new(
        Data::new("Hello World").unwrap(),
        Identifier::new("test_identifier").unwrap(),
        Vec::from_iter([
            Tag::new("test_tag_a").unwrap(),
            Tag::new("test_tag_b").unwrap(),
        ]),
        Version::new(0),
    )];

    let iterations = 1_000_000f64;
    let start = Instant::now();

    for _ in 0..iterations as u64 {
        stream.append(&events, None).unwrap();
    }

    let duration = Instant::now().duration_since(start);
    let events_per_sec = (iterations / duration.as_secs_f64()) as u64;

    println!("time: {duration:?} ({events_per_sec} events/sec)",);

    Ok(())
}
