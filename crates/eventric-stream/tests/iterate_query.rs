mod fixtures;

use eventric_stream::{
    error::Error,
    event::{
        Identifier,
        Position,
        Specifier,
        Tag,
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

// Vec<Query>

#[test]
fn iterate_vec_query_with_single_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![Query::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("student_enrolled")?,
    )])?])?];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3, "Should match 3 student_enrolled events");

    for (event, mask) in &events {
        assert_eq!(mask.len(), 1, "Mask should have 1 entry");
        assert!(mask[0], "First query should match");
        let identifier: &str = event.identifier().as_ref();
        assert!(identifier.contains("student_enrolled"));
    }

    Ok(())
}

#[test]
fn iterate_vec_query_with_two_non_overlapping_queries() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append(fixtures::events()?, None)?;

    let queries = vec![
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        5,
        "Should match 3 student_enrolled + 2 course_created"
    );

    // First event: student_enrolled at position 0
    assert_eq!(events[0].0.position(), &Position::new(0));
    assert_eq!(events[0].1, vec![true, false]);

    // Second event: course_created at position 1
    assert_eq!(events[1].0.position(), &Position::new(1));
    assert_eq!(events[1].1, vec![false, true]);

    // Third event: student_enrolled at position 2
    assert_eq!(events[2].0.position(), &Position::new(2));
    assert_eq!(events[2].1, vec![true, false]);

    // Fourth event: student_enrolled at position 4
    assert_eq!(events[3].0.position(), &Position::new(4));
    assert_eq!(events[3].1, vec![true, false]);

    // Fifth event: course_created at position 5
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
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("student_enrolled")?)],
            vec![Tag::new("course:200")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3, "Should match 3 student_enrolled events");

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
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
        )])?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("course_updated")?)],
            vec![Tag::new("course:200")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        6,
        "Should match student_enrolled, course_created, and course_updated"
    );

    // Check that mask length is correct for all events
    for (_, mask) in &events {
        assert_eq!(mask.len(), 3, "Mask should have 3 entries");
    }

    // course_updated at position 3 should only match third query
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
            vec![Specifier::new(Identifier::new("student_enrolled")?)],
            vec![Tag::new("student:100")?],
        )?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("student_enrolled")?)],
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
            Identifier::new("course_created")?,
        )])?])?,
        Query::new([Selector::specifiers_and_tags(
            vec![
                Specifier::new(Identifier::new("course_created")?),
                Specifier::new(Identifier::new("course_updated")?),
            ],
            vec![Tag::new("course:200")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    // Should return course_created at 1 and 5, and course_updated at 3
    assert_eq!(events.len(), 3);

    // course_created at position 1 should match both queries
    assert_eq!(events[0].0.position(), &Position::new(1));
    assert_eq!(events[0].1, vec![true, true]);

    // course_updated at position 3 should only match second query
    assert_eq!(events[1].0.position(), &Position::new(3));
    assert_eq!(events[1].1, vec![false, true]);

    // course_created at position 5 should match first query only (no course:200
    // tag)
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
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, Some(Position::new(3)));
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        events.len(),
        2,
        "Should match student_enrolled and course_created from position 3"
    );

    // student_enrolled at position 4
    assert_eq!(events[0].0.position(), &Position::new(4));
    assert_eq!(events[0].1, vec![true, false]);

    // course_created at position 5
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
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 5);

    // Last event should be course_created at position 5
    assert_eq!(events[0].0.position(), &Position::new(5));
    assert_eq!(events[0].1, vec![false, true]);

    // First event should be student_enrolled at position 0
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
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
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
            Selector::specifiers(vec![Specifier::new(Identifier::new("course_created")?)])?,
            Selector::specifiers_and_tags(
                vec![Specifier::new(Identifier::new("student_enrolled")?)],
                vec![Tag::new("student:100")?],
            )?,
        ])?,
        Query::new([Selector::specifiers_and_tags(
            vec![
                Specifier::new(Identifier::new("student_enrolled")?),
                Specifier::new(Identifier::new("student_dropped")?),
            ],
            vec![Tag::new("student:100")?],
        )?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 4);

    // student_enrolled with student:100 at position 0
    assert_eq!(events[0].0.position(), &Position::new(0));
    assert_eq!(events[0].1, vec![true, true], "Should match both queries");

    // course_created at position 1
    assert_eq!(events[1].0.position(), &Position::new(1));
    assert_eq!(events[1].1, vec![true, false], "Should match first query");

    // course_created at position 5
    assert_eq!(events[2].0.position(), &Position::new(5));
    assert_eq!(events[2].1, vec![true, false], "Should match first query");

    // student_dropped with student:100 at position 6
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
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
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
            Identifier::new("course_created")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("student_enrolled")?,
        )])?])?,
        Query::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_updated")?,
        )])?])?,
    ];

    let (events, _prepared) = stream.iterate_query(queries, None);
    let events = events.collect::<Result<Vec<_>, _>>()?;

    // Verify mask order corresponds to query order
    for (event, mask) in &events {
        assert_eq!(mask.len(), 3);

        let identifier: &str = event.identifier().as_ref();
        if identifier.contains("course_created") {
            assert!(mask[0], "First query should match course_created");
        } else if identifier.contains("student_enrolled") {
            assert!(mask[1], "Second query should match student_enrolled");
        } else if identifier.contains("course_updated") {
            assert!(mask[2], "Third query should match course_updated");
        }
    }

    Ok(())
}
