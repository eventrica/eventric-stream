use std::path::Path;

use assertables::assert_some_as_result;
use eventric_stream::{
    error::Error,
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Position,
        Specifier,
        Tag,
        Version,
    },
    stream::{
        Stream,
        append::{
            Append as _,
            AppendQuery as _,
        },
        iterate::{
            Iterate as _,
            IterateQuery as _,
        },
        query::{
            Query,
            Selector,
        },
    },
};

// =================================================================================================
// Append
// =================================================================================================

// Fixtures

/// Creates a test stream at the given path.
/// If `populate` is true, adds 4 test events to the stream.
fn stream<P>(path: P, populate: bool) -> Result<Stream, Error>
where
    P: AsRef<Path>,
{
    let mut stream = Stream::builder(path).temporary(true).open()?;

    if populate {
        stream.append(
            [
                EphemeralEvent::new(
                    Data::new("data_0")?,
                    Identifier::new("id_0")?,
                    [Tag::new("tag_1")?, Tag::new("tag_2")?, Tag::new("tag_3")?],
                    Version::new(0),
                ),
                EphemeralEvent::new(
                    Data::new("data_1")?,
                    Identifier::new("id_1")?,
                    [Tag::new("tag_2")?, Tag::new("tag_3")?, Tag::new("tag_4")?],
                    Version::new(1),
                ),
                EphemeralEvent::new(
                    Data::new("data_2")?,
                    Identifier::new("id_2")?,
                    [Tag::new("tag_3")?, Tag::new("tag_4")?, Tag::new("tag_5")?],
                    Version::new(0),
                ),
                EphemeralEvent::new(
                    Data::new("data_3")?,
                    Identifier::new("id_2")?,
                    [Tag::new("tag_4")?, Tag::new("tag_5")?, Tag::new("tag_6")?],
                    Version::new(1),
                ),
            ],
            None,
        )?;
    }

    Ok(stream)
}

/// Creates a single test event with the given suffix for data and identifier.
fn event(suffix: &str) -> Result<EphemeralEvent, Error> {
    Ok(EphemeralEvent::new(
        Data::new(format!("data_{suffix}"))?,
        Identifier::new(format!("id_{suffix}"))?,
        [Tag::new(format!("tag_{suffix}"))?],
        Version::new(0),
    ))
}

// Append

#[test]
fn append_single_event_to_empty_stream() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), false)?;

    let result = stream.append([event("test")?], None);

    assert!(result.is_ok());

    let position = result.unwrap();

    assert_eq!(Position::new(0), position);

    // Verify the event was appended
    let mut events = stream.iterate(None);

    assert_some_as_result!(events.next()).unwrap()?;
    assert!(events.next().is_none());

    Ok(())
}

#[test]
fn append_multiple_events_to_empty_stream() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), false)?;

    let result = stream.append([event("a")?, event("b")?, event("c")?], None);

    assert!(result.is_ok());

    let position = result.unwrap();

    // Returns position of last event appended
    assert_eq!(Position::new(2), position);

    // Verify all events were appended
    let mut events = stream.iterate(None);

    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert!(events.next().is_none());

    Ok(())
}

#[test]
fn append_to_populated_stream() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Stream already has 4 events (positions 0-3)
    let result = stream.append([event("new")?], None);

    assert!(result.is_ok());

    let position = result.unwrap();

    assert_eq!(Position::new(4), position);

    // Verify we now have 5 events
    let mut events = stream.iterate(None);

    for _ in 0..5 {
        assert_some_as_result!(events.next()).unwrap()?;
    }

    assert!(events.next().is_none());

    Ok(())
}

#[test]
fn append_with_after_position_at_head() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Stream has events at positions 0-3, so next position is 4
    // Using after=3 means "append if no events after position 3"
    let result = stream.append([event("new")?], Some(Position::new(3)));

    assert!(result.is_ok());

    let position = result.unwrap();

    assert_eq!(Position::new(4), position);

    Ok(())
}

#[test]
fn append_with_after_position_beyond_head() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Stream has events at positions 0-3
    // Using after=10 (beyond current head) should succeed immediately
    let result = stream.append([event("new")?], Some(Position::new(10)));

    assert!(result.is_ok());

    let position = result.unwrap();

    assert_eq!(Position::new(4), position);

    Ok(())
}

#[test]
fn append_with_after_position_in_middle() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Stream has events at positions 0-3
    // Using after=1 means "append if no events after position 1"
    // This should fail because we're appending before tail
    let result = stream.append([event("new")?], Some(Position::new(1)));

    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_returns_last_position() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), false)?;

    let position1 = stream.append([event("a")?], None)?;

    assert_eq!(Position::new(0), position1);

    let position2 = stream.append([event("b")?, event("c")?], None)?;

    assert_eq!(Position::new(2), position2);

    let position3 = stream.append([event("d")?], None)?;

    assert_eq!(Position::new(3), position3);

    Ok(())
}

// Append Query

#[test]
fn append_query_with_no_matching_events() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Query for an identifier that doesn't exist
    let id = Identifier::new("nonexistent")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // Should succeed because no events match the query
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_ok());

    let position = result.unwrap();

    assert_eq!(Position::new(4), position);

    Ok(())
}

#[test]
fn append_query_fails_when_events_match() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Query for id_0 which exists at position 0
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // Should fail because events match the query
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_query_with_after_position_no_match() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // id_0 exists at position 0
    // Using after=0 means "check for events after position 0"
    // So position 0 itself won't cause a conflict
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let result = stream.append_query([event("new")?], &query, Some(Position::new(0)));

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn append_query_with_after_position_has_match() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // id_2 exists at positions 2 and 3
    // Using after=1 means "check for events after position 1"
    // Positions 2 and 3 are after position 1, so this should fail
    let id = Identifier::new("id_2")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let result = stream.append_query([event("new")?], &query, Some(Position::new(1)));

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_query_with_after_at_head() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Stream has events 0-3
    // Query for id_0 (exists at position 0)
    // Using after=3 means no events after position 3, so should succeed
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let result = stream.append_query([event("new")?], &query, Some(Position::new(3)));

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn append_query_with_after_beyond_head() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Using after position beyond current head should succeed immediately
    // without checking query
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let result = stream.append_query([event("new")?], &query, Some(Position::new(100)));

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn append_query_with_tags() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Query for tag that exists in multiple events
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let tag = Tag::new("tag_2")?;
    let selector = Selector::specifiers_and_tags([spec], [tag])?;
    let query = Query::new([selector])?;

    // id_0 with tag_2 exists at position 0, should fail
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_query_with_version_range() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Query for id_2 with version >= 1
    // id_2 v0 at position 2, id_2 v1 at position 3
    let id = Identifier::new("id_2")?;
    let range = Version::new(1)..Version::MAX;
    let spec = Specifier::new(id).range(range);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // Should fail because id_2 v1 exists at position 3
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_query_with_version_range_no_match() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Query for id_2 with version >= 5 (doesn't exist)
    let id = Identifier::new("id_2")?;
    let range = Version::new(5)..Version::MAX;
    let spec = Specifier::new(id).range(range);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // Should succeed because no matching events
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn append_query_multiple_selectors() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Query with two selectors (OR condition)
    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let sel_0 = Selector::specifiers([spec_0])?;

    let id_1 = Identifier::new("id_1")?;
    let spec_1 = Specifier::new(id_1);
    let sel_1 = Selector::specifiers([spec_1])?;

    let query = Query::new([sel_0, sel_1])?;

    // Should fail because both id_0 and id_1 exist in the stream
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_query_borrowed_query() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    let id = Identifier::new("nonexistent")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // Test with borrowed query
    let result = stream.append_query([event("new")?], &query, None);

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn append_query_owned_query() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    let id = Identifier::new("nonexistent")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // Test with owned query
    let result = stream.append_query([event("new")?], query, None);

    assert!(result.is_ok());

    Ok(())
}

#[test]
fn append_query_returns_correct_position() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), false)?;

    let id = Identifier::new("nonexistent")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // First append
    let position1 = stream.append_query([event("a")?], &query, None)?;

    assert_eq!(Position::new(0), position1);

    // Second append with multiple events
    let position2 = stream.append_query([event("b")?, event("c")?], &query, None)?;

    assert_eq!(Position::new(2), position2);

    Ok(())
}

// Integration

#[test]
fn append_and_query_integration() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), false)?;

    // Append initial event
    stream.append([event("initial")?], None)?;

    // Query to verify it was appended
    let id = Identifier::new("id_initial")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query.clone(), None);
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Data::new("data_initial")?, event.data());

    Ok(())
}

#[test]
fn concurrent_append_simulation() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), false)?;

    // Simulate two "concurrent" operations:
    // 1. First operation reads stream (empty), prepares to append
    let position_before_first = None; // Stream is empty, no previous position

    // 2. Second operation appends an event
    stream.append([event("second")?], None)?;

    // 3. First operation tries to append with its "before" position This should
    //    fail because events were added after its read
    let id = Identifier::new("id_second")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let result = stream.append_query([event("first")?], &query, position_before_first);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Concurrency));

    Ok(())
}

#[test]
fn append_after_successful_concurrency_check() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Get current head position
    let current_head = Position::new(3);

    // Create query that won't match anything after current head
    let id = Identifier::new("future_event")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    // This should succeed
    let result = stream.append_query([event("new")?], &query, Some(current_head));

    assert!(result.is_ok());

    let position = result.unwrap();

    assert_eq!(Position::new(4), position);

    Ok(())
}
