mod fixtures;

use eventric_stream::{
    error::Error,
    event::{
        Identifier,
        Position,
        Specifier,
        Tag,
        Version,
    },
    stream::{
        append::Append,
        iterate::IterateQuery,
        query::{
            Query,
            Selector,
        },
    },
};

// =================================================================================================
// Iterate Query
// =================================================================================================

#[test]
fn iterate_query_by_single_identifier() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let query = Query::new([Selector::specifiers(vec![
        Specifier::new(Identifier::new("CourseCreated")?),
        Specifier::new(Identifier::new("CourseUpdated")?),
    ])?])?;

    let (events, _query_opt) = stream.iterate_query(query, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        3,
        "Should match CourseCreated and CourseUpdated events"
    );
    assert_eq!(events[0].position(), &Position::new(1));
    assert_eq!(events[1].position(), &Position::new(3));
    assert_eq!(events[2].position(), &Position::new(5));

    Ok(())
}

#[test]
fn iterate_query_by_identifier_and_tags() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    let event = fixtures::event("test data", "TestEvent", &["tag:a", "tag:b"], 3)?;
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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let stream = fixtures::stream()?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;

    stream.append(
        vec![
            fixtures::event("v0", "Event", &["tag:1"], 0)?,
            fixtures::event("v1", "Event", &["tag:1"], 1)?,
            fixtures::event("v2", "Event", &["tag:1"], 2)?,
            fixtures::event("v0_again", "Event", &["tag:1"], 0)?,
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
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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
    let mut stream = fixtures::stream()?;
    let initial_events = vec![
        fixtures::event("event1", "EventA", &["tag:1"], 0)?,
        fixtures::event("event2", "EventB", &["tag:2"], 0)?,
    ];

    stream.append(initial_events, None)?;

    let query = Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (events_before, _) = stream.iterate_query(query.clone(), None);
    let events_before = events_before.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_before.len(), 1);

    stream.append([fixtures::event("event3", "EventA", &["tag:3"], 0)?], None)?;

    let (events_after, _) = stream.iterate_query(query, None);
    let events_after = events_after.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_after.len(), 2);
    assert_eq!(events_after[1].position(), &Position::new(2));

    Ok(())
}

#[test]
fn iterate_query_reuses_optimized_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

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

// Vec<Query>

#[test]
fn iterate_vec_query_with_single_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("StudentEnrolled")?,
    )])?])?];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3, "Should match 3 StudentEnrolled events");

    for (event, mask) in &events {
        assert_eq!(mask.len(), 1, "Mask should have 1 entry");
        assert!(mask[0], "First query should match");
        let identifier: &str = event.identifier().as_ref();
        assert!(identifier.contains("StudentEnrolled"));
    }

    Ok(())
}

#[test]
fn iterate_vec_query_with_two_non_overlapping_queries() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        5,
        "Should match 3 StudentEnrolled + 2 CourseCreated"
    );

    // First event: StudentEnrolled at position 0
    assert_eq!(events[0].0.position(), &Position::new(0));
    assert_eq!(events[0].1, vec![true, false]);

    // Second event: CourseCreated at position 1
    assert_eq!(events[1].0.position(), &Position::new(1));
    assert_eq!(events[1].1, vec![false, true]);

    // Third event: StudentEnrolled at position 2
    assert_eq!(events[2].0.position(), &Position::new(2));
    assert_eq!(events[2].1, vec![true, false]);

    // Fourth event: StudentEnrolled at position 4
    assert_eq!(events[3].0.position(), &Position::new(4));
    assert_eq!(events[3].1, vec![true, false]);

    // Fifth event: CourseCreated at position 5
    assert_eq!(events[4].0.position(), &Position::new(5));
    assert_eq!(events[4].1, vec![false, true]);

    Ok(())
}

#[test]
fn iterate_vec_query_with_overlapping_queries() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
            vec![Tag::new("course:200")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3, "Should match 3 StudentEnrolled events");

    // First event: position 0, has course:200 tag
    assert_eq!(events[0].0.position(), &Position::new(0));
    assert_eq!(events[0].1, vec![true, true], "Should match both queries");

    // Second event: position 2, has course:200 tag
    assert_eq!(events[1].0.position(), &Position::new(2));
    assert_eq!(events[1].1, vec![true, true], "Should match both queries");

    // Third event: position 4, has course:201 tag
    assert_eq!(events[2].0.position(), &Position::new(4));
    assert_eq!(
        events[2].1,
        vec![true, false],
        "Should match only first query"
    );

    Ok(())
}

#[test]
fn iterate_vec_query_with_three_queries() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("CourseUpdated")?)],
            vec![Tag::new("course:200")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        6,
        "Should match StudentEnrolled, CourseCreated, and CourseUpdated"
    );

    // Check that mask length is correct for all events
    for (_, mask) in &events {
        assert_eq!(mask.len(), 3, "Mask should have 3 entries");
    }

    // CourseUpdated at position 3 should only match third query
    let course_updated = events
        .iter()
        .find(|(e, _)| e.position() == &Position::new(3))
        .unwrap();
    assert_eq!(course_updated.1, vec![false, false, true]);

    Ok(())
}

#[test]
fn iterate_vec_query_with_tags_filter() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
            vec![Tag::new("student:100")?],
        )?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
            vec![Tag::new("student:101")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2, "Should match 2 specific students");

    // First event: student:100 at position 0
    assert_eq!(events[0].0.position(), &Position::new(0));
    assert_eq!(events[0].1, vec![true, false]);

    // Second event: student:101 at position 2
    assert_eq!(events[1].0.position(), &Position::new(2));
    assert_eq!(events[1].1, vec![false, true]);

    Ok(())
}

#[test]
fn iterate_vec_query_all_queries_match_same_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![
                Specifier::new(Identifier::new("CourseCreated")?),
                Specifier::new(Identifier::new("CourseUpdated")?),
            ],
            vec![Tag::new("course:200")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    // Should return CourseCreated at 1 and 5, and CourseUpdated at 3
    assert_eq!(events.len(), 3);

    // CourseCreated at position 1 should match both queries
    assert_eq!(events[0].0.position(), &Position::new(1));
    assert_eq!(events[0].1, vec![true, true]);

    // CourseUpdated at position 3 should only match second query
    assert_eq!(events[1].0.position(), &Position::new(3));
    assert_eq!(events[1].1, vec![false, true]);

    // CourseCreated at position 5 should match first query only (no course:200 tag)
    assert_eq!(events[2].0.position(), &Position::new(5));
    assert_eq!(events[2].1, vec![true, false]);

    Ok(())
}

#[test]
fn iterate_vec_query_from_position() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, Some(Position::new(3)));
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        2,
        "Should match StudentEnrolled and CourseCreated from position 3"
    );

    // StudentEnrolled at position 4
    assert_eq!(events[0].0.position(), &Position::new(4));
    assert_eq!(events[0].1, vec![true, false]);

    // CourseCreated at position 5
    assert_eq!(events[1].0.position(), &Position::new(5));
    assert_eq!(events[1].1, vec![false, true]);

    Ok(())
}

#[test]
fn iterate_vec_query_empty_queries() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries: Vec<Query> = vec![];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 0, "Empty queries should yield no events");

    Ok(())
}

#[test]
fn iterate_vec_query_backward() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 5);

    // Last event should be CourseCreated at position 5
    assert_eq!(events[0].0.position(), &Position::new(5));
    assert_eq!(events[0].1, vec![false, true]);

    // First event should be StudentEnrolled at position 0
    assert_eq!(events[4].0.position(), &Position::new(0));
    assert_eq!(events[4].1, vec![true, false]);

    Ok(())
}

#[test]
fn iterate_vec_query_maintains_order() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    for i in 0..events.len() - 1 {
        assert!(
            events[i].0.position() < events[i + 1].0.position(),
            "Events should be in position order"
        );
    }

    Ok(())
}

#[test]
fn iterate_vec_query_with_complex_selectors() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([
            Selector::specifiers(vec![Specifier::new(Identifier::new("CourseCreated")?)])?,
            Selector::specifiers_and_tags(
                vec![Specifier::new(Identifier::new("StudentEnrolled")?)],
                vec![Tag::new("student:100")?],
            )?,
        ])?,
        Query::new([Selector::specifiers_and_tags(
            vec![
                Specifier::new(Identifier::new("StudentEnrolled")?),
                Specifier::new(Identifier::new("StudentDropped")?),
            ],
            vec![Tag::new("student:100")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 4);

    // StudentEnrolled with student:100 at position 0
    assert_eq!(events[0].0.position(), &Position::new(0));
    assert_eq!(events[0].1, vec![true, true], "Should match both queries");

    // CourseCreated at position 1
    assert_eq!(events[1].0.position(), &Position::new(1));
    assert_eq!(events[1].1, vec![true, false], "Should match first query");

    // CourseCreated at position 5
    assert_eq!(events[2].0.position(), &Position::new(5));
    assert_eq!(events[2].1, vec![true, false], "Should match first query");

    // StudentDropped with student:100 at position 6
    assert_eq!(events[3].0.position(), &Position::new(6));
    assert_eq!(events[3].1, vec![false, true], "Should match second query");

    Ok(())
}

#[test]
fn iterate_vec_query_reuses_prepared() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
    ];

    let (events1, prepared) = stream.iterate_query(queries, None);
    let events1 = events1.collect::<Result<Vec<_>, _>>()?;

    let (events2, _) = stream.iterate_query(prepared, None);
    let events2 = events2.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events1.len(), events2.len());
    assert_eq!(events1.len(), 5);

    for ((e1, m1), (e2, m2)) in events1.iter().zip(events2.iter()) {
        assert_eq!(e1.position(), e2.position());
        assert_eq!(m1, m2);
    }

    Ok(())
}

#[test]
fn iterate_vec_query_preserves_mask_order() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseCreated")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("StudentEnrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseUpdated")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    // Verify mask order corresponds to query order
    for (event, mask) in &events {
        assert_eq!(mask.len(), 3);

        let identifier: &str = event.identifier().as_ref();
        if identifier.contains("CourseCreated") {
            assert!(mask[0], "First query should match CourseCreated");
        } else if identifier.contains("StudentEnrolled") {
            assert!(mask[1], "Second query should match StudentEnrolled");
        } else if identifier.contains("CourseUpdated") {
            assert!(mask[2], "Third query should match CourseUpdated");
        }
    }

    Ok(())
}
