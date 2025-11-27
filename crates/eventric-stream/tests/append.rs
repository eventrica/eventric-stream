use eventric_stream::{
    error::Error,
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Position,
        Tag,
        Version,
    },
    stream::{
        Stream,
        append::Append,
        iterate::Iterate,
    },
    temp_path,
};

// =================================================================================================
// Append
// =================================================================================================

/// Creates a new temporary test stream that will be automatically cleaned up
fn create_test_stream() -> Result<Stream, Error> {
    Stream::builder(temp_path()).temporary(true).open()
}

/// Creates a sample `EphemeralEvent` for testing
fn create_event(
    data: &str,
    identifier: &str,
    tags: &[&str],
    version: u8,
) -> Result<EphemeralEvent, Error> {
    Ok(EphemeralEvent::new(
        Data::new(data)?,
        Identifier::new(identifier)?,
        tags.iter()
            .map(|tag| Tag::new(*tag))
            .collect::<Result<Vec<_>, _>>()?,
        Version::new(version),
    ))
}

/// Creates multiple sample events for testing
fn create_sample_events() -> Result<[EphemeralEvent; 3], Error> {
    Ok([
        create_event(
            "student subscribed",
            "StudentSubscribedToCourse",
            &["student:100", "course:200"],
            0,
        )?,
        create_event(
            "capacity changed",
            "CourseCapacityChanged",
            &["course:200"],
            0,
        )?,
        create_event(
            "another student subscribed",
            "StudentSubscribedToCourse",
            &["student:101", "course:201"],
            1,
        )?,
    ])
}

// Append Trait

#[test]
fn append_single_event() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    let event = create_event("test data", "TestEvent", &["tag:1"], 0)?;
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
    let mut stream = create_test_stream()?;

    let events = create_sample_events()?;
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
    let mut stream = create_test_stream()?;

    let position1 = stream.append([create_event("first", "Event1", &[], 0)?], None)?;

    assert_eq!(position1, Position::new(0));

    let position2 = stream.append([create_event("second", "Event2", &[], 0)?], None)?;

    assert_eq!(position2, Position::new(1));

    let position3 = stream.append(
        [
            create_event("third", "Event3", &[], 0)?,
            create_event("fourth", "Event4", &[], 0)?,
        ],
        None,
    )?;

    assert_eq!(position3, Position::new(3));

    Ok(())
}

#[test]
fn append_with_concurrency_check_passes() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    let position1 = stream.append(create_sample_events()?, None)?;
    let new_event = create_event("new event", "NewEvent", &["tag:new"], 0)?;
    let position2 = stream.append([new_event], Some(position1))?;

    assert_eq!(position2, Position::new(3));

    Ok(())
}

#[test]
fn append_with_concurrency_check_fails() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("first", "Event1", &[], 0)?], None)?;
    stream.append([create_event("second", "Event2", &[], 0)?], None)?;

    let result = stream.append(
        [create_event("concurrent", "Event3", &[], 0)?],
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
    let mut stream = create_test_stream()?;

    let position = stream.append(
        [create_event("first", "Event1", &[], 0)?],
        Some(Position::new(100)),
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_events_can_be_retrieved() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    let events = create_sample_events()?;
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
    let mut stream = create_test_stream()?;

    let event = create_event("no tags", "EventWithoutTags", &[], 0)?;
    let position = stream.append([event], None)?;

    assert_eq!(position, Position::new(0));

    let retrieved: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(retrieved.len(), 1);
    assert!(retrieved[0].tags().is_empty(), "Event should have no tags");

    Ok(())
}

#[test]
fn append_with_multiple_tags() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    let event = create_event(
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
