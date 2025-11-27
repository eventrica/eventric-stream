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
        append::AppendQuery,
        iterate::Iterate,
        query::{
            Query,
            Selector,
        },
    },
    temp_path,
};

// =================================================================================================
// Append Query
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

// Append Query Trait

#[test]
fn append_query_with_no_conflict() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &["tag:1"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let (position, _query_opt) = stream.append_query(
        [create_event("event2", "EventA", &["tag:2"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    assert_eq!(position, Position::new(1));

    Ok(())
}

#[test]
fn append_query_detects_conflict() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &["tag:1"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event("event2", "EventB", &["tag:2"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventB")?,
    )])?])?;

    let result = stream.append_query(
        [create_event("event3", "EventC", &[], 0)?],
        query,
        Some(Position::new(0)),
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventB exists after position 0"
    );

    Ok(())
}

#[test]
fn append_query_with_identifier_check() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("first", "StudentEnrolled", &["student:100"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let result = stream.append_query(
        [create_event("second", "CourseCreated", &[], 0)?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect StudentEnrolled event exists"
    );

    Ok(())
}

#[test]
fn append_query_with_tag_filter() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("dummy", "EventA", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event("event1", "EventB", &["course:200"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("EventB")?)],
        vec![Tag::new("course:200")?],
    )?])?;

    let result = stream.append_query(
        [create_event("event2", "EventC", &[], 0)?],
        query,
        Some(Position::new(0)),
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventB with course:200 exists after position 0"
    );

    Ok(())
}

#[test]
fn append_query_with_multiple_tags() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("dummy", "EventA", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event(
            "event1",
            "StudentEnrolled",
            &["student:100", "course:200"],
            0,
        )?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
        vec![Tag::new("student:100")?, Tag::new("course:200")?],
    )?])?;

    let result = stream.append_query(
        [create_event("event2", "EventC", &[], 0)?],
        query,
        Some(Position::new(0)),
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect event with both tags exists"
    );

    Ok(())
}

#[test]
fn append_query_position_at_stream_head() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let (position1, _) = stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (position2, _) = stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        query,
        Some(position1),
    )?;

    assert_eq!(position2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_returns_optimized_query() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (_position, query_optimized) = stream.append_query(
        [create_event("event1", "EventB", &[], 0)?],
        query,
        None,
    )?;

    assert!(
        !format!("{query_optimized:?}").is_empty(),
        "Should return non-empty optimized query"
    );

    Ok(())
}

#[test]
fn append_query_reuses_optimized_query() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (position1, query_optimized) = stream.append_query(
        [create_event("event1", "EventB", &[], 0)?],
        query,
        None,
    )?;

    assert_eq!(position1, Position::new(0));

    let (position2, _) = stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        query_optimized,
        None,
    )?;

    assert_eq!(position2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_with_multiple_identifiers() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![
        Specifier::new(Identifier::new("EventA")?),
        Specifier::new(Identifier::new("EventB")?),
    ])?])?;

    let result = stream.append_query(
        [create_event("event3", "EventC", &[], 0)?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventA or EventB exists"
    );

    Ok(())
}

#[test]
fn append_query_with_multiple_selectors() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &["tag:1"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event("event2", "EventB", &["tag:2"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([
        Selector::specifiers(vec![Specifier::new(Identifier::new("EventA")?)])?,
        Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("EventB")?)],
            vec![Tag::new("tag:2")?],
        )?,
    ])?;

    let result = stream.append_query(
        [create_event("event3", "EventC", &[], 0)?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventA OR (EventB with tag:2) exists"
    );

    Ok(())
}

#[test]
fn append_query_appends_successfully() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("ConflictEvent")?,
    )])?])?;

    let (position, _) = stream.append_query(
        [
            create_event("event1", "EventA", &[], 0)?,
            create_event("event2", "EventB", &[], 0)?,
        ],
        query,
        None,
    )?;

    assert_eq!(position, Position::new(1));

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(1));

    Ok(())
}

#[test]
fn append_query_preserves_existing_events() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("existing", "ExistingEvent", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("ExistingEvent")?,
    )])?])?;

    let result = stream.append_query(
        [create_event("new", "NewEvent", &[], 0)?],
        query,
        None,
    );

    assert!(matches!(result, Err(Error::Concurrency)));

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1, "Should preserve existing event");

    Ok(())
}

#[test]
fn append_query_empty_stream() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("AnyEvent")?,
    )])?])?;

    let (position, _) = stream.append_query(
        [create_event("first", "FirstEvent", &[], 0)?],
        query,
        None,
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_query_with_position_boundary() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let pos1 = stream
        .append_query(
            [create_event("event2", "EventB", &[], 0)?],
            Query::new([Selector::specifiers(vec![Specifier::new(
                Identifier::new("EventC")?,
            )])?])?,
            None,
        )?
        .0;

    stream.append_query(
        [create_event("event3", "EventA", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let result = stream.append_query(
        [create_event("event4", "EventD", &[], 0)?],
        query,
        Some(pos1),
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventA exists after position 1"
    );

    Ok(())
}

#[test]
fn append_query_sequential_operations() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("ConflictEvent")?,
    )])?])?;

    let (pos1, query_opt) = stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        query,
        None,
    )?;

    assert_eq!(pos1, Position::new(0));

    let (pos2, _) = stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        query_opt,
        None,
    )?;

    assert_eq!(pos2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_no_false_positives() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &["tag:1"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("EventA")?)],
        vec![Tag::new("tag:2")?],
    )?])?;

    let (position, _) = stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        query,
        None,
    )?;

    assert_eq!(position, Position::new(1));

    Ok(())
}

#[test]
fn append_query_complex_scenario() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event(
            "enrollment1",
            "StudentEnrolled",
            &["student:100", "course:200"],
            0,
        )?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseDeleted")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event("course_created", "CourseCreated", &["course:200"], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseDeleted")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
        vec![Tag::new("course:200")?],
    )?])?;

    let result = stream.append_query(
        [create_event("enrollment2", "StudentEnrolled", &["student:101"], 0)?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect existing StudentEnrolled with course:200"
    );

    Ok(())
}

#[test]
fn append_query_with_version_variants() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("v0", "Event", &[], 0)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        None,
    )?;

    stream.append_query(
        [create_event("v1", "Event", &[], 1)?],
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("Event")?,
    )])?])?;

    let result = stream.append_query(
        [create_event("v2", "Event", &[], 2)?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect Event exists regardless of version"
    );

    Ok(())
}

// Vec<Query> Tests

#[test]
fn append_query_with_vec_query_basic_usage() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let (position, _query_multi_opt) = stream.append_query(
        [create_event("event1", "EventC", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_query_with_vec_query_returns_multi_optimized() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let (_position, query_multi_opt) = stream.append_query(
        [create_event("event1", "EventC", &[], 0)?],
        queries,
        None,
    )?;

    assert!(
        !format!("{query_multi_opt:?}").is_empty(),
        "Should return non-empty QueryMultiOptimized"
    );

    Ok(())
}

#[test]
fn append_query_reuses_multi_optimized_query() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let (position1, query_multi_opt) = stream.append_query(
        [create_event("event1", "EventC", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position1, Position::new(0));

    let (position2, _) = stream.append_query(
        [create_event("event2", "EventD", &[], 0)?],
        query_multi_opt,
        None,
    )?;

    assert_eq!(position2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_with_vec_query_appends_events() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictB")?,
        )])?])?,
    ];

    stream.append_query(
        [
            create_event("event1", "EventA", &[], 0)?,
            create_event("event2", "EventB", &[], 0)?,
        ],
        queries,
        None,
    )?;

    let events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);

    Ok(())
}

#[test]
fn append_query_vec_query_with_single_query() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let queries = vec![Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?];

    let (position, _) = stream.append_query(
        [create_event("event1", "EventB", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_query_vec_query_sequential_operations() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("BlockerEvent")?,
        )])?])?,
    ];

    let (pos1, _) = stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        queries.clone(),
        None,
    )?;

    assert_eq!(pos1, Position::new(0));

    let (pos2, _) = stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        queries.clone(),
        None,
    )?;

    assert_eq!(pos2, Position::new(1));

    let (pos3, _) = stream.append_query(
        [create_event("event3", "EventC", &[], 0)?],
        queries,
        Some(pos2),
    )?;

    assert_eq!(pos3, Position::new(2));

    Ok(())
}

#[test]
fn append_query_vec_query_fails_if_first_query_matches() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let result = stream.append_query(
        [create_event("event2", "EventC", &[], 0)?],
        queries,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should fail when first query matches (EventA exists)"
    );

    Ok(())
}

#[test]
fn append_query_vec_query_fails_if_second_query_matches() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventB", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let result = stream.append_query(
        [create_event("event2", "EventC", &[], 0)?],
        queries,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should fail when second query matches (EventB exists)"
    );

    Ok(())
}

#[test]
fn append_query_vec_query_fails_if_any_query_matches() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    stream.append_query(
        [create_event("event3", "EventC", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
    ];

    let result = stream.append_query(
        [create_event("event4", "EventD", &[], 0)?],
        queries,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should fail when any query matches (EventA, EventB, or EventC exists)"
    );

    Ok(())
}

#[test]
fn append_query_vec_query_succeeds_if_no_queries_match() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventD")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventE")?,
        )])?])?,
    ];

    let (position, _) = stream.append_query(
        [create_event("event3", "EventF", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position, Position::new(2));

    Ok(())
}

#[test]
fn append_query_vec_query_with_tags_any_match() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &["tag:1"], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("EventA")?)],
            vec![Tag::new("tag:1")?],
        )?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let result = stream.append_query(
        [create_event("event2", "EventC", &[], 0)?],
        queries,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should fail when first query matches (EventA with tag:1 exists)"
    );

    Ok(())
}

#[test]
fn append_query_vec_query_position_check_any_match() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("event1", "EventA", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    stream.append_query(
        [create_event("event2", "EventB", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
    ];

    let result = stream.append_query(
        [create_event("event3", "EventD", &[], 0)?],
        queries,
        Some(Position::new(0)),
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should fail when first query matches after position (EventB exists after position 0)"
    );

    Ok(())
}

#[test]
fn append_query_vec_query_or_semantics_demonstration() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append_query(
        [create_event("only_a", "EventA", &[], 0)?],
        vec![Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?],
        None,
    )?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ];

    let result = stream.append_query(
        [create_event("new_event", "EventC", &[], 0)?],
        queries,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Vec<Query> uses OR semantics: fails if EventA OR EventB exists (only EventA exists, so it fails)"
    );

    Ok(())
}
