use std::{
    path::Path,
    sync::Arc,
};

use assertables::{
    assert_none,
    assert_some_as_result,
};
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
        append::Append as _,
        iterate::{
            Cache,
            Iterate as _,
            IterateQuery as _,
            Options,
        },
        query::{
            Query,
            Selector,
        },
    },
};

// =================================================================================================
// Iterate
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

// Iterate

#[test]
fn iterate_empty_stream() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), false)?;

    let mut events = stream.iterate(None);

    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_all_events() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let mut events = stream.iterate(None);

    // Should return all 4 events in order
    let event = assert_some_as_result!(events.next()).unwrap()?;
    assert_eq!(&Data::new("data_0")?, event.data());
    assert_eq!(&Position::new(0), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;
    assert_eq!(&Data::new("data_1")?, event.data());
    assert_eq!(&Position::new(1), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;
    assert_eq!(&Data::new("data_2")?, event.data());
    assert_eq!(&Position::new(2), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;
    assert_eq!(&Data::new("data_3")?, event.data());
    assert_eq!(&Position::new(3), event.position());

    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_from_position_zero() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let mut events = stream.iterate(Some(Position::new(0)));

    // Should return all events starting from position 0
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_from_middle_position() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let mut events = stream.iterate(Some(Position::new(2)));

    // Should return events starting from position 2
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(2), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(3), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_from_last_position() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let mut events = stream.iterate(Some(Position::new(3)));

    // Should return only the last event
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(3), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_from_beyond_last_position() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let mut events = stream.iterate(Some(Position::new(10)));

    // Should return no events
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_with_default_options() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let options = Options::default();

    let mut events = stream.iterate_with_options(None, options);

    // Should behave the same as iterate()
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_with_custom_options() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let cache = Arc::new(Cache::default());
    let options = Options::default().with_shared_cache(cache.clone());

    let mut events = stream.iterate_with_options(None, options);

    // Should work with custom options
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_some_as_result!(events.next()).unwrap()?;
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_returns_correct_event_data() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let mut events = stream.iterate(None);

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Data::new("data_0")?, event.data());
    assert_eq!(&Identifier::new("id_0")?, event.identifier());
    assert_eq!(&Version::new(0), event.version());
    assert_eq!(&Position::new(0), event.position());

    Ok(())
}

#[test]
fn iterate_multiple_times() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // First iteration
    let mut events1 = stream.iterate(None);

    assert_some_as_result!(events1.next()).unwrap()?;
    assert_some_as_result!(events1.next()).unwrap()?;

    // Second iteration should start from beginning again
    let mut events2 = stream.iterate(None);

    let event = assert_some_as_result!(events2.next()).unwrap()?;

    assert_eq!(&Position::new(0), event.position());

    Ok(())
}

// Iterate Query

#[test]
fn iterate_query_empty_stream() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), false)?;
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _query_hash) = stream.iterate_query(query, None);

    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_returns_query_hash() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _query_hash) = stream.iterate_query(query, None);

    // QueryHash should be returned and events should work
    assert_some_as_result!(events.next()).unwrap()?;

    Ok(())
}

#[test]
fn iterate_query_single_identifier() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return only events with id_0
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Data::new("data_0")?, event.data());
    assert_eq!(&Position::new(0), event.position());

    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_multiple_identifiers() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id1 = Identifier::new("id_0")?;
    let id2 = Identifier::new("id_1")?;
    let spec1 = Specifier::new(id1);
    let spec2 = Specifier::new(id2);
    let selector = Selector::specifiers([spec1, spec2])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return events with id_0 OR id_1
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(0), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(1), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_with_version_range() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_2")?;
    let range = Version::new(1)..Version::MAX;
    let spec = Specifier::new(id).range(range);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return only id_2 with version >= 1 (position 3)
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Data::new("data_3")?, event.data());
    assert_eq!(&Position::new(3), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_with_tags() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let tag = Tag::new("tag_2")?;
    let selector = Selector::specifiers_and_tags([spec], [tag])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return events with id_0 AND tag_2
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Data::new("data_0")?, event.data());
    assert_eq!(&Position::new(0), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_with_multiple_tags() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_2")?;
    let spec = Specifier::new(id);
    let tag1 = Tag::new("tag_4")?;
    let tag2 = Tag::new("tag_5")?;
    let selector = Selector::specifiers_and_tags([spec], [tag1, tag2])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return events with id_2 AND tag_4 AND tag_5 (positions 2 and 3)
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(2), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(3), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_no_matching_events() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("nonexistent")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_from_position() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_2")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, Some(Position::new(3)));

    // id_2 exists at positions 2 and 3, but we start from position 3
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(3), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_from_position_beyond_matches() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, Some(Position::new(5)));

    // id_0 only exists at position 0, so starting from position 5 returns nothing
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_multiple_selectors() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let sel_0 = Selector::specifiers([spec_0])?;

    let id_1 = Identifier::new("id_1")?;
    let spec_1 = Specifier::new(id_1);
    let sel_1 = Selector::specifiers([spec_1])?;

    let query = Query::new([sel_0, sel_1])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return events matching either selector (OR logic)
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(0), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(1), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_with_default_options() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;
    let options = Options::default();

    let (mut events, _) = stream.iterate_query_with_options(query, None, options);

    assert_some_as_result!(events.next()).unwrap()?;
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_with_shared_cache() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let cache = Arc::new(Cache::default());
    let options = Options::default().with_shared_cache(cache.clone());

    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query_with_options(query, None, options);

    // Cache should be populated during iteration
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(0), event.position());

    Ok(())
}

#[test]
fn iterate_query_reuses_shared_cache() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let cache = Arc::new(Cache::default());

    // First query
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query1 = Query::new([selector])?;
    let options1 = Options::default().with_shared_cache(cache.clone());

    let (mut events1, _) = stream.iterate_query_with_options(query1, None, options1);

    assert_some_as_result!(events1.next()).unwrap()?;

    // Second query with same cache
    let id = Identifier::new("id_1")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query2 = Query::new([selector])?;
    let options2 = Options::default().with_shared_cache(cache.clone());

    let (mut events2, _) = stream.iterate_query_with_options(query2, None, options2);

    assert_some_as_result!(events2.next()).unwrap()?;

    // Both queries should complete successfully with shared cache
    // The cache is shared and reused across queries
    assert!(Arc::strong_count(&cache) >= 2);

    Ok(())
}

#[test]
fn iterate_query_complex_version_and_tags() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;
    let id = Identifier::new("id_2")?;
    let range = Version::new(1)..Version::MAX;
    let spec = Specifier::new(id).range(range);
    let tag1 = Tag::new("tag_5")?;
    let tag2 = Tag::new("tag_6")?;
    let selector = Selector::specifiers_and_tags([spec], [tag1, tag2])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    // Should return id_2 with version >= 1 AND tag_5 AND tag_6 (position 3 only)
    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Data::new("data_3")?, event.data());
    assert_eq!(&Position::new(3), event.position());
    assert_none!(events.next());

    Ok(())
}

// Integration

#[test]
fn iterate_after_append() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Append a new event
    stream.append([event("new")?], None)?;

    // Iterate should now return 5 events
    let mut events = stream.iterate(None);

    for _ in 0..5 {
        assert_some_as_result!(events.next()).unwrap()?;
    }

    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_query_after_append() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    // Append a new event with id_0
    let new_event = EphemeralEvent::new(
        Data::new("data_new")?,
        Identifier::new("id_0")?,
        [Tag::new("tag_new")?],
        Version::new(0),
    );

    stream.append([new_event], None)?;

    // Query for id_0 should now return 2 events
    let id = Identifier::new("id_0")?;
    let spec = Specifier::new(id);
    let selector = Selector::specifiers([spec])?;
    let query = Query::new([selector])?;

    let (mut events, _) = stream.iterate_query(query, None);

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(0), event.position());

    let event = assert_some_as_result!(events.next()).unwrap()?;

    assert_eq!(&Position::new(4), event.position());
    assert_none!(events.next());

    Ok(())
}

#[test]
fn iterate_and_iterate_query_consistency() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Count all events using iterate
    let all_events = stream.iterate(None).count();

    // Count all events using iterate_query with a broad query
    let id_0 = Identifier::new("id_0")?;
    let id_1 = Identifier::new("id_1")?;
    let id_2 = Identifier::new("id_2")?;
    let spec_0 = Specifier::new(id_0);
    let spec_1 = Specifier::new(id_1);
    let spec_2 = Specifier::new(id_2);
    let selector = Selector::specifiers([spec_0, spec_1, spec_2])?;
    let query = Query::new([selector])?;

    let (query_events, _) = stream.iterate_query(query, None);
    let query_count = query_events.count();

    // Both should return 4 events
    assert_eq!(all_events, query_count);
    assert_eq!(4, all_events);

    Ok(())
}
