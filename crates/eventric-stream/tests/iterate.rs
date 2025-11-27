use eventric_stream::{
    error::Error,
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Position,
        Tag,
        Timestamp,
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
// Iterate
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
fn create_sample_events() -> Result<[EphemeralEvent; 5], Error> {
    Ok([
        create_event("first", "EventA", &["tag:1"], 0)?,
        create_event("second", "EventB", &["tag:2"], 0)?,
        create_event("third", "EventC", &["tag:3"], 0)?,
        create_event("fourth", "EventD", &["tag:4"], 0)?,
        create_event("fifth", "EventE", &["tag:5"], 0)?,
    ])
}

// Iterate Trait

#[test]
fn iterate_empty_stream() -> Result<(), Error> {
    let stream = create_test_stream()?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 0, "Empty stream should yield no events");

    Ok(())
}

#[test]
fn iterate_single_event() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("data", "TestEvent", &["tag:1"], 0)?], None)?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1, "Should retrieve 1 event");
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(
        events[0].identifier(),
        &Identifier::new("TestEvent").unwrap()
    );

    Ok(())
}

#[test]
fn iterate_multiple_events() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_sample_events()?, None)?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 5, "Should retrieve 5 events");

    for (i, event) in events.iter().enumerate() {
        assert_eq!(
            event.position(),
            &Position::new(i as u64),
            "Event {i} should be at position {i}"
        );
    }

    Ok(())
}

#[test]
fn iterate_from_position() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_sample_events()?, None)?;

    let events: Vec<_> = stream
        .iterate(Some(Position::new(2)))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        3,
        "Should retrieve events from position 2 onwards"
    );
    assert_eq!(events[0].position(), &Position::new(2));
    assert_eq!(events[1].position(), &Position::new(3));
    assert_eq!(events[2].position(), &Position::new(4));

    Ok(())
}

#[test]
fn iterate_from_last_position() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_sample_events()?, None)?;

    let events: Vec<_> = stream
        .iterate(Some(Position::new(4)))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1, "Should retrieve only the last event");
    assert_eq!(events[0].position(), &Position::new(4));

    Ok(())
}

#[test]
fn iterate_from_beyond_stream() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_sample_events()?, None)?;

    let events: Vec<_> = stream
        .iterate(Some(Position::new(100)))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 0, "Should retrieve no events");

    Ok(())
}

#[test]
fn iterate_preserves_event_data() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    let original_event = create_event("test data", "TestEvent", &["tag:a", "tag:b"], 3)?;
    let expected_data = original_event.data().clone();
    let expected_identifier = original_event.identifier().clone();
    let expected_tags = original_event.tags().to_vec();
    let expected_version = *original_event.version();

    stream.append([original_event], None)?;

    let events = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data(), &expected_data);
    assert_eq!(events[0].identifier(), &expected_identifier);
    assert_eq!(events[0].tags(), expected_tags.as_slice());
    assert_eq!(events[0].version(), &expected_version);

    Ok(())
}

#[test]
fn iterate_assigns_positions_sequentially() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("first", "Event1", &[], 0)?], None)?;
    stream.append([create_event("second", "Event2", &[], 0)?], None)?;
    stream.append([create_event("third", "Event3", &[], 0)?], None)?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(1));
    assert_eq!(events[2].position(), &Position::new(2));

    Ok(())
}

#[test]
fn iterate_assigns_timestamps() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("data", "TestEvent", &[], 0)?], None)?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert!(
        events[0].timestamp() > &Timestamp::new(0),
        "Timestamp should be assigned"
    );

    Ok(())
}

#[test]
fn iterate_maintains_append_order() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    let identifiers = ["EventA", "EventB", "EventC", "EventD", "EventE"];

    for id in &identifiers {
        stream.append([create_event("data", id, &[], 0)?], None)?;
    }

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), identifiers.len());

    for (i, event) in events.iter().enumerate() {
        assert_eq!(
            event.identifier(),
            &Identifier::new(identifiers[i]).unwrap(),
            "Event {i} identifier mismatch"
        );
    }

    Ok(())
}

#[test]
fn iterate_multiple_times() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_sample_events()?, None)?;

    let events1: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;
    let events2: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events1.len(),
        events2.len(),
        "Both iterations should return same count"
    );
    assert_eq!(events1.len(), 5);

    for (i, (e1, e2)) in events1.iter().zip(events2.iter()).enumerate() {
        assert_eq!(
            e1.position(),
            e2.position(),
            "Event {i} positions should match"
        );
        assert_eq!(
            e1.identifier(),
            e2.identifier(),
            "Event {i} identifiers should match"
        );
    }

    Ok(())
}

#[test]
fn iterate_backward() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_sample_events()?, None)?;

    let events: Vec<_> = stream.iterate(None).rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 5, "Should retrieve 5 events in reverse");
    assert_eq!(
        events[0].position(),
        &Position::new(4),
        "First should be last"
    );
    assert_eq!(
        events[4].position(),
        &Position::new(0),
        "Last should be first"
    );

    for (i, event) in events.iter().enumerate() {
        let expected_position = 4 - i;
        assert_eq!(
            event.position(),
            &Position::new(expected_position as u64),
            "Event at index {i} should have position {expected_position}"
        );
    }

    Ok(())
}

#[test]
fn iterate_with_mixed_tags() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(
        [
            create_event("event1", "Event1", &[], 0)?,
            create_event("event2", "Event2", &["tag:a"], 0)?,
            create_event("event3", "Event3", &["tag:a", "tag:b", "tag:c"], 0)?,
        ],
        None,
    )?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].tags().len(), 0, "First event has no tags");
    assert_eq!(events[1].tags().len(), 1, "Second event has 1 tag");
    assert_eq!(events[2].tags().len(), 3, "Third event has 3 tags");

    Ok(())
}

#[test]
fn iterate_with_different_versions() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(
        [
            create_event("v0", "Event", &[], 0)?,
            create_event("v1", "Event", &[], 1)?,
            create_event("v2", "Event", &[], 2)?,
        ],
        None,
    )?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].version(), &Version::new(0));
    assert_eq!(events[1].version(), &Version::new(1));
    assert_eq!(events[2].version(), &Version::new(2));

    Ok(())
}

#[test]
fn iterate_after_multiple_appends() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("first", "Event1", &[], 0)?], None)?;
    stream.append([create_event("second", "Event2", &[], 0)?], None)?;

    let events_before: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_before.len(), 2);

    stream.append([create_event("third", "Event3", &[], 0)?], None)?;

    let events_after: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_after.len(), 3);
    assert_eq!(events_after[2].position(), &Position::new(2));

    Ok(())
}
