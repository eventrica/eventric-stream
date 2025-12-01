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
        append::AppendSelect,
        iterate::Iterate,
        select::{
            Selection,
            Selector,
        },
    },
};
use eventric_stream_core::stream::select::Selections;

// =================================================================================================
// Append Query
// =================================================================================================

#[test]
fn append_query_with_no_conflict() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &["tag:1"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let (position, _query_opt) = stream.append_select(
        [fixtures::event("event2", "EventA", &["tag:2"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    assert_eq!(position, Position::new(1));

    Ok(())
}

#[test]
fn append_query_detects_conflict() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &["tag:1"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event2", "EventB", &["tag:2"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventB")?,
    )])?])?;

    let result = stream.append_select(
        [fixtures::event("event3", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event(
            "first",
            "student_enrolled",
            &["student:100"],
            0,
        )?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("course_created")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("student_enrolled")?,
    )])?])?;

    let result = stream.append_select(
        [fixtures::event("second", "course_created", &[], 0)?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect student_enrolled event exists"
    );

    Ok(())
}

#[test]
fn append_query_with_tag_filter() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("dummy", "EventA", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event1", "EventB", &["course:200"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("EventB")?)],
        vec![Tag::new("course:200")?],
    )?])?;

    let result = stream.append_select(
        [fixtures::event("event2", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("dummy", "EventA", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event(
            "event1",
            "student_enrolled",
            &["student:100", "course:200"],
            0,
        )?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("student_enrolled")?)],
        vec![Tag::new("student:100")?, Tag::new("course:200")?],
    )?])?;

    let result = stream.append_select(
        [fixtures::event("event2", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    let (position1, _) = stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (position2, _) = stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        query,
        Some(position1),
    )?;

    assert_eq!(position2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_returns_optimized_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (_position, query_optimized) =
        stream.append_select([fixtures::event("event1", "EventB", &[], 0)?], query, None)?;

    assert!(
        !format!("{query_optimized:?}").is_empty(),
        "Should return non-empty optimized query"
    );

    Ok(())
}

#[test]
fn append_query_reuses_optimized_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let (position1, query_optimized) =
        stream.append_select([fixtures::event("event1", "EventB", &[], 0)?], query, None)?;

    assert_eq!(position1, Position::new(0));

    let (position2, _) = stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        query_optimized,
        None,
    )?;

    assert_eq!(position2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_with_multiple_identifiers() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![
        Specifier::new(Identifier::new("EventA")?),
        Specifier::new(Identifier::new("EventB")?),
    ])?])?;

    let result = stream.append_select([fixtures::event("event3", "EventC", &[], 0)?], query, None);

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventA or EventB exists"
    );

    Ok(())
}

#[test]
fn append_query_with_multiple_selectors() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &["tag:1"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event2", "EventB", &["tag:2"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([
        Selector::specifiers(vec![Specifier::new(Identifier::new("EventA")?)])?,
        Selector::specifiers_and_tags(vec![Specifier::new(Identifier::new("EventB")?)], vec![
            Tag::new("tag:2")?,
        ])?,
    ])?;

    let result = stream.append_select([fixtures::event("event3", "EventC", &[], 0)?], query, None);

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect EventA OR (EventB with tag:2) exists"
    );

    Ok(())
}

#[test]
fn append_query_appends_successfully() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("ConflictEvent")?,
    )])?])?;

    let (position, _) = stream.append_select(
        [
            fixtures::event("event1", "EventA", &[], 0)?,
            fixtures::event("event2", "EventB", &[], 0)?,
        ],
        query,
        None,
    )?;

    assert_eq!(position, Position::new(1));

    let events: Vec<_> = stream.iter(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(1));

    Ok(())
}

#[test]
fn append_query_preserves_existing_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("existing", "ExistingEvent", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("ExistingEvent")?,
    )])?])?;

    let result = stream.append_select([fixtures::event("new", "NewEvent", &[], 0)?], query, None);

    assert!(matches!(result, Err(Error::Concurrency)));

    let events: Vec<_> = stream.iter(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1, "Should preserve existing event");

    Ok(())
}

#[test]
fn append_query_empty_stream() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("AnyEvent")?,
    )])?])?;

    let (position, _) = stream.append_select(
        [fixtures::event("first", "FirstEvent", &[], 0)?],
        query,
        None,
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_query_with_position_boundary() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let pos1 = stream
        .append_select(
            [fixtures::event("event2", "EventB", &[], 0)?],
            Selection::new([Selector::specifiers(vec![Specifier::new(
                Identifier::new("EventC")?,
            )])?])?,
            None,
        )?
        .0;

    stream.append_select(
        [fixtures::event("event3", "EventA", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("EventA")?,
    )])?])?;

    let result = stream.append_select(
        [fixtures::event("event4", "EventD", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("ConflictEvent")?,
    )])?])?;

    let (pos1, query_opt) =
        stream.append_select([fixtures::event("event1", "EventA", &[], 0)?], query, None)?;

    assert_eq!(pos1, Position::new(0));

    let (pos2, _) = stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        query_opt,
        None,
    )?;

    assert_eq!(pos2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_no_false_positives() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &["tag:1"], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("EventA")?)],
        vec![Tag::new("tag:2")?],
    )?])?;

    let (position, _) =
        stream.append_select([fixtures::event("event2", "EventB", &[], 0)?], query, None)?;

    assert_eq!(position, Position::new(1));

    Ok(())
}

#[test]
fn append_query_complex_scenario() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event(
            "enrollment1",
            "student_enrolled",
            &["student:100", "course:200"],
            0,
        )?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseDeleted")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event(
            "course_created",
            "course_created",
            &["course:200"],
            0,
        )?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("CourseDeleted")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers_and_tags(
        vec![Specifier::new(Identifier::new("student_enrolled")?)],
        vec![Tag::new("course:200")?],
    )?])?;

    let result = stream.append_select(
        [fixtures::event(
            "enrollment2",
            "student_enrolled",
            &["student:101"],
            0,
        )?],
        query,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect existing student_enrolled with course:200"
    );

    Ok(())
}

#[test]
fn append_query_with_version_variants() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("v0", "Event", &[], 0)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("v1", "Event", &[], 1)?],
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        None,
    )?;

    let query = Selection::new([Selector::specifiers(vec![Specifier::new(
        Identifier::new("Event")?,
    )])?])?;

    let result = stream.append_select([fixtures::event("v2", "Event", &[], 2)?], query, None);

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Should detect Event exists regardless of version"
    );

    Ok(())
}

// Vec<Query> Tests

#[test]
fn append_query_with_vec_query_basic_usage() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let (position, _query_multi_opt) = stream.append_select(
        [fixtures::event("event1", "EventC", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_query_with_vec_query_returns_multi_optimized() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let (_position, query_multi_opt) = stream.append_select(
        [fixtures::event("event1", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let (position1, query_multi_opt) = stream.append_select(
        [fixtures::event("event1", "EventC", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position1, Position::new(0));

    let (position2, _) = stream.append_select(
        [fixtures::event("event2", "EventD", &[], 0)?],
        query_multi_opt,
        None,
    )?;

    assert_eq!(position2, Position::new(1));

    Ok(())
}

#[test]
fn append_query_with_vec_query_appends_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictB")?,
        )])?])?,
    ])?;

    stream.append_select(
        [
            fixtures::event("event1", "EventA", &[], 0)?,
            fixtures::event("event2", "EventB", &[], 0)?,
        ],
        queries,
        None,
    )?;

    let events: Vec<_> = stream.iter(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);

    Ok(())
}

#[test]
fn append_query_vec_query_with_single_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let queries = Selections::new([Selection::new([Selector::specifiers(vec![
        Specifier::new(Identifier::new("EventA")?),
    ])?])?])?;

    let (position, _) = stream.append_select(
        [fixtures::event("event1", "EventB", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position, Position::new(0));

    Ok(())
}

#[test]
fn append_query_vec_query_sequential_operations() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("ConflictEvent")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("BlockerEvent")?,
        )])?])?,
    ])?;

    let (pos1, _) = stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        queries.clone(),
        None,
    )?;

    assert_eq!(pos1, Position::new(0));

    let (pos2, _) = stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        queries.clone(),
        None,
    )?;

    assert_eq!(pos2, Position::new(1));

    let (pos3, _) = stream.append_select(
        [fixtures::event("event3", "EventC", &[], 0)?],
        queries,
        Some(pos2),
    )?;

    assert_eq!(pos3, Position::new(2));

    Ok(())
}

#[test]
fn append_query_vec_query_fails_if_first_query_matches() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let result = stream.append_select(
        [fixtures::event("event2", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventB", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let result = stream.append_select(
        [fixtures::event("event2", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event3", "EventC", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
    ])?;

    let result = stream.append_select(
        [fixtures::event("event4", "EventD", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventD")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventE")?,
        )])?])?,
    ])?;

    let (position, _) = stream.append_select(
        [fixtures::event("event3", "EventF", &[], 0)?],
        queries,
        None,
    )?;

    assert_eq!(position, Position::new(2));

    Ok(())
}

#[test]
fn append_query_vec_query_with_tags_any_match() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &["tag:1"], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers_and_tags(
            vec![Specifier::new(Identifier::new("EventA")?)],
            vec![Tag::new("tag:1")?],
        )?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let result = stream.append_select(
        [fixtures::event("event2", "EventC", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("event1", "EventA", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    stream.append_select(
        [fixtures::event("event2", "EventB", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventC")?,
        )])?])?,
    ])?;

    let result = stream.append_select(
        [fixtures::event("event3", "EventD", &[], 0)?],
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
    let mut stream = fixtures::stream()?;
    stream.append_select(
        [fixtures::event("only_a", "EventA", &[], 0)?],
        Selections::new([Selection::new([Selector::specifiers(vec![
            Specifier::new(Identifier::new("ConflictEvent")?),
        ])?])?])?,
        None,
    )?;

    let queries = Selections::new([
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventA")?,
        )])?])?,
        Selection::new([Selector::specifiers(vec![Specifier::new(
            Identifier::new("EventB")?,
        )])?])?,
    ])?;

    let result = stream.append_select(
        [fixtures::event("new_event", "EventC", &[], 0)?],
        queries,
        None,
    );

    assert!(
        matches!(result, Err(Error::Concurrency)),
        "Vec<Query> uses OR semantics: fails if EventA OR EventB exists (only EventA exists, so \
         it fails)"
    );

    Ok(())
}
