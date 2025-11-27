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
        append::Append,
        iterate::IterateQuery,
        query::{
            Query,
            Selector,
        },
    },
    temp_path,
};

// =================================================================================================
// Iterate Query
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

/// Creates a diverse set of events for query testing
fn create_diverse_events() -> Result<Vec<EphemeralEvent>, Error> {
    Ok(vec![
        create_event("event1", "StudentEnrolled", &["student:100", "course:200"], 0)?,
        create_event("event2", "CourseCreated", &["course:200"], 0)?,
        create_event("event3", "StudentEnrolled", &["student:101", "course:200"], 0)?,
        create_event("event4", "CourseUpdated", &["course:200"], 0)?,
        create_event("event5", "StudentEnrolled", &["student:102", "course:201"], 0)?,
        create_event("event6", "CourseCreated", &["course:201"], 0)?,
        create_event("event7", "StudentDropped", &["student:100", "course:200"], 0)?,
    ])
}

// Iterate Query Trait

#[test]
fn iterate_query_by_single_identifier() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3, "Should match 3 StudentEnrolled events");
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(2));
    assert_eq!(events[2].position(), &Position::new(4));

    Ok(())
}

#[test]
fn iterate_query_by_multiple_identifiers() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![
        Specifier::new(Identifier::new("CourseCreated")?),
        Specifier::new(Identifier::new("CourseUpdated")?),
    ])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3, "Should match CourseCreated and CourseUpdated events");
    assert_eq!(events[0].position(), &Position::new(1));
    assert_eq!(events[1].position(), &Position::new(3));
    assert_eq!(events[2].position(), &Position::new(5));

    Ok(())
}

#[test]
fn iterate_query_by_identifier_and_tags() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
        vec![Tag::new("course:200")?],
    )?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        2,
        "Should match StudentEnrolled events with course:200"
    );
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(2));

    Ok(())
}

#[test]
fn iterate_query_with_multiple_tags() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
        vec![Tag::new("student:100")?, Tag::new("course:200")?],
    )?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        1,
        "Should match events with both student:100 AND course:200"
    );
    assert_eq!(events[0].position(), &Position::new(0));

    Ok(())
}

#[test]
fn iterate_query_with_no_matches() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("NonExistentEvent")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 0, "Should match no events");

    Ok(())
}

#[test]
fn iterate_query_from_position() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, Some(Position::new(3)));
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        1,
        "Should match StudentEnrolled events from position 3"
    );
    assert_eq!(events[0].position(), &Position::new(4));

    Ok(())
}

#[test]
fn iterate_query_from_beyond_matches() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, Some(Position::new(100)));
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 0, "Should match no events beyond stream");

    Ok(())
}

#[test]
fn iterate_query_with_multiple_selectors() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([
        Selector::specifiers(vec![Specifier::new(Identifier::new("CourseCreated")?)])?,
        Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
            vec![Tag::new("student:100")?],
        )?,
    ])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        3,
        "Should match CourseCreated OR (StudentEnrolled with student:100)"
    );
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(1));
    assert_eq!(events[2].position(), &Position::new(5));

    Ok(())
}

#[test]
fn iterate_query_returns_optimized_query() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (_events, query_optimized) = stream.iterate_query(query, None);

    assert!(
        !format!("{query_optimized:?}").is_empty(),
        "Should return non-empty optimized query"
    );

    Ok(())
}

#[test]
fn iterate_query_preserves_event_data() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let event = create_event("test data", "TestEvent", &["tag:a", "tag:b"], 3)?;
    let expected_identifier = event.identifier().clone();

    stream.append([event], None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("TestEvent")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].identifier(), &expected_identifier);
    assert_eq!(events[0].version(), &Version::new(3));

    Ok(())
}

#[test]
fn iterate_query_multiple_times() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query1 = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;
    let query2 = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events1, _) = stream.iterate_query(query1, None);
    let events1 = events1.collect::<Result<Vec<_>, _>>()?;

    let (events2, _) = stream.iterate_query(query2, None);
    let events2 = events2.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events1.len(), events2.len());
    assert_eq!(events1.len(), 3);

    for (e1, e2) in events1.iter().zip(events2.iter()) {
        assert_eq!(e1.position(), e2.position());
    }

    Ok(())
}

#[test]
fn iterate_query_empty_stream() -> Result<(), Error> {
    let stream = create_test_stream()?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("AnyEvent")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 0, "Empty stream should yield no events");

    Ok(())
}

#[test]
fn iterate_query_maintains_order() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    for i in 0..events.len() - 1 {
        assert!(
            events[i].position() < events[i + 1].position(),
            "Events should be in position order"
        );
    }

    Ok(())
}

#[test]
fn iterate_query_backward() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].position(), &Position::new(4));
    assert_eq!(events[1].position(), &Position::new(2));
    assert_eq!(events[2].position(), &Position::new(0));

    Ok(())
}

#[test]
fn iterate_query_with_version_specifier() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(
        vec![
            create_event("v0", "Event", &["tag:1"], 0)?,
            create_event("v1", "Event", &["tag:1"], 1)?,
            create_event("v2", "Event", &["tag:1"], 2)?,
            create_event("v0_again", "Event", &["tag:1"], 0)?,
        ],
        None,
    )?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("Event")?,
    )])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 4, "Should match all Event instances");

    Ok(())
}

#[test]
fn iterate_query_with_common_tag() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers_and_tags(
        vec![
            Specifier::new(Identifier::new("StudentEnrolled")?),
            Specifier::new(Identifier::new("CourseCreated")?),
            Specifier::new(Identifier::new("CourseUpdated")?),
            Specifier::new(Identifier::new("StudentDropped")?),
        ],
        vec![Tag::new("course:200")?],
    )?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        5,
        "Should match all events tagged with course:200"
    );

    Ok(())
}

#[test]
fn iterate_query_after_additional_appends() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    let initial_events = vec![
        create_event("event1", "EventA", &["tag:1"], 0)?,
        create_event("event2", "EventB", &["tag:2"], 0)?,
    ];

    stream.append(initial_events, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (events_before, _) = stream.iterate_query(query.clone(), None);
    let events_before = events_before.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_before.len(), 1);

    stream.append([create_event("event3", "EventA", &["tag:3"], 0)?], None)?;

    let (events_after, _) = stream.iterate_query(query, None);
    let events_after = events_after.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_after.len(), 2);
    assert_eq!(events_after[1].position(), &Position::new(2));

    Ok(())
}

#[test]
fn iterate_query_reuses_optimized_query() -> Result<(), Error> {
    let mut stream = create_test_stream()?;
    stream.append(create_diverse_events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?;

    let (events1, query_optimized) = stream.iterate_query(query, None);
    let events1 = events1.collect::<Result<Vec<_>, _>>()?;

    let (events2, _) = stream.iterate_query(query_optimized, None);
    let events2 = events2.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events1.len(), events2.len());
    assert_eq!(events1.len(), 3);

    for (e1, e2) in events1.iter().zip(events2.iter()) {
        assert_eq!(e1.position(), e2.position());
    }

    Ok(())
}
