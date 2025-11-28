mod fixtures;

use eventric_stream::{
    error::Error,
    event::Position,
    stream::{
        append::Append,
        iterate::Iterate,
    },
};

// =================================================================================================
// Append
// =================================================================================================

#[test]
fn append_single_event() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let event = fixtures::event("test data", "TestEvent", &["tag:1"], 0)?;
    let position = stream.append([event], None)?;

    assert_eq!(
        position,
        Position::new(0),
        "First event should be at position 0"
    );
    Ok(())
}

#[test]
fn append_multiple_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let events = fixtures::create_domain_events()?;
    let position = stream.append(events, None)?;

    assert_eq!(
        position,
        Position::new(2),
        "Last event should be at position 2"
    );
    Ok(())
}

#[test]
fn append_sequential_batches() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let position1 = stream.append([fixtures::event("first", "Event1", &[], 0)?], None)?;

    assert_eq!(position1, Position::new(0));

    let position2 = stream.append([fixtures::event("second", "Event2", &[], 0)?], None)?;

    assert_eq!(position2, Position::new(1));

    let position3 = stream.append(
        [
            fixtures::event("third", "Event3", &[], 0)?,
            fixtures::event("fourth", "Event4", &[], 0)?,
        ],
        None,
    )?;

    assert_eq!(position3, Position::new(3));

    Ok(())
}

#[test]
fn append_with_concurrency_check_passes() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let position1 = stream.append(fixtures::create_domain_events()?, None)?;
    let new_event = fixtures::event("new event", "NewEvent", &["tag:new"], 0)?;
    let position2 = stream.append([new_event], Some(position1))?;

    assert_eq!(position2, Position::new(3));

    Ok(())
}

#[test]
fn append_with_concurrency_check_fails() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("first", "Event1", &[], 0)?], None)?;
    stream.append([fixtures::event("second", "Event2", &[], 0)?], None)?;

    let result = stream.append(
        [fixtures::event("concurrent", "Event3", &[], 0)?],
        Some(Position::new(0)),
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Expected concurrency error, got: {result:?}"
    );

    Ok(())
}

#[test]
fn append_empty_stream_with_concurrency_check() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let position = stream.append(
        [fixtures::event("first", "Event1", &[], 0)?],
        Some(Position::new(100)),
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_events_can_be_retrieved() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let events = fixtures::create_domain_events()?;
    let original_identifiers: Vec<_> = events.iter().map(|e| e.identifier().clone()).collect();

    stream.append(events, None)?;

    let retrieved: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(retrieved.len(), 3, "Should retrieve 3 events");

    for (i, event) in retrieved.iter().enumerate() {
        assert_eq!(
            event.identifier(),
            &original_identifiers[i],
            "Event {i} identifier mismatch"
        );
        assert_eq!(
            event.position(),
            &Position::new(i as u64),
            "Event {i} position mismatch"
        );
    }
    Ok(())
}

#[test]
fn append_with_no_tags() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let event = fixtures::event("no tags", "EventWithoutTags", &[], 0)?;
    let position = stream.append([event], None)?;

    assert_eq!(position, Position::new(0));

    let retrieved: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(retrieved.len(), 1);
    assert!(retrieved[0].tags().is_empty(), "Event should have no tags");

    Ok(())
}

#[test]
fn append_with_multiple_tags() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    let event = fixtures::event(
        "multi-tag",
        "MultiTagEvent",
        &["tag:1", "tag:2", "tag:3", "tag:4"],
        0,
    )?;

    stream.append([event], None)?;

    let retrieved: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].tags().len(), 4, "Event should have 4 tags");

    Ok(())
}
