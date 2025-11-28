mod fixtures;

use std::collections::BTreeSet;

use assertables::{
    assert_gt,
    assert_lt,
};
use eventric_stream::{
    error::Error,
    event::{
        Data,
        Identifier,
        Position,
        Tag,
        Timestamp,
        Version,
    },
    stream::{
        append::Append,
        iterate::Iterate,
    },
};

// =================================================================================================
// Iterate
// =================================================================================================

#[test]
fn iterate() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // Iterate on empty stream should return no events

    assert_eq!(stream.iterate(None).next(), None);

    // Iterate on stream after single append return a single event with
    // correct properties

    stream.append(fixtures::event("one", "id_one", &["tag:a"], 1), None)?;

    let events = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);

    assert_eq!(events[0].data(), &Data::new("one")?);
    assert_eq!(events[0].identifier(), &Identifier::new("id_one")?);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[0].tags(), &BTreeSet::from_iter([Tag::new("tag:a")?]));
    assert_gt!(events[0].timestamp(), &Timestamp::new(0));
    assert_lt!(events[0].timestamp(), &Timestamp::now()?);
    assert_eq!(events[0].version(), &Version::new(1));

    // Iterate on stream after batch append (batch size 7) should return 8 events
    // with correct position properties (other properties assumed correct)

    stream.append(fixtures::events()?, None)?;

    let events = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 8);

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.position(), &Position::new(i as u64));
    }

    // Iterate on stream from a specified position (4) should return 4 events with
    // correct position properties

    let events = stream
        .iterate(Some(Position::new(4)))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 4);

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.position(), &Position::new(4 + i as u64));
    }

    // Iterate on stream from the head position should return a single event with
    // corretc Position property

    let events = stream
        .iterate(Some(Position::new(7)))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(7));

    // Iterate on stream from a position after the head position should return no
    // events

    assert_eq!(stream.iterate(Some(Position::new(8))).next(), None);

    // Iterate on a stream maintains the order of appended events

    stream.append(fixtures::event("a", "id_a", &[], 0), None)?;
    stream.append(fixtures::event("b", "id_b", &[], 0), None)?;
    stream.append(fixtures::event("c", "id_c", &[], 0), None)?;

    let events = stream
        .iterate(Some(Position::new(8)))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].identifier(), &Identifier::new("id_a")?);
    assert_eq!(events[1].identifier(), &Identifier::new("id_b")?);
    assert_eq!(events[2].identifier(), &Identifier::new("id_c")?);

    // Iterate on an unchanged stream returns the same events if called multiple
    // times

    let events_a = stream.iterate(None).collect::<Result<Vec<_>, _>>();
    let events_b = stream.iterate(None).collect::<Result<Vec<_>, _>>();

    assert_eq!(events_a, events_b);

    // Iterate on a reversed stream, including from a position, returns events in
    // reverse order

    let events = stream
        .iterate(Some(Position::new(8)))
        .rev()
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].identifier(), &Identifier::new("id_c")?);
    assert_eq!(events[1].identifier(), &Identifier::new("id_b")?);
    assert_eq!(events[2].identifier(), &Identifier::new("id_a")?);

    Ok(())
}
