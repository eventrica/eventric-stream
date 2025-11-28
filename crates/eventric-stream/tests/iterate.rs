mod fixtures;

use std::collections::BTreeSet;

use assertables::{
    assert_gt,
    assert_lt,
    assert_none,
};
use eventric_stream::{
    error::Error,
    event::{
        Data,
        Identifier,
        Position,
        Specifier,
        Tag,
        Timestamp,
        Version,
    },
    stream::{
        append::Append,
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
// Iterate
// =================================================================================================

// Iterate

#[rustfmt::skip]
#[test]
fn iterate() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // Iterate on empty stream should return no events

    assert_eq!(stream.iterate(None).next(), None);

    // Iterate on stream after single append return a single event with
    // correct properties

    stream.append(fixtures::event("one", "id_one", &["tag:a"], 1), None)?;

    let events = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);

    assert_eq!(events[0].data(), &Data::new("one")?);
    assert_eq!(events[0].identifier(), &Identifier::new("id_one")?);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[0].tags(), &BTreeSet::from_iter([Tag::new("tag:a")?]));
    assert_gt!(events[0].timestamp(), &Timestamp::new(0));
    assert_lt!(events[0].timestamp(), &Timestamp::now()?);
    assert_eq!(events[0].version(), &Version::new(1));

    // Iterate on stream after batch append (batch size 7) should return 8 events
    // with correct position properties (other properties assumed correct)

    stream.append(fixtures::events()?, None)?;

    let events = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 8);

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.position(), &Position::new(i as u64));
    }

    // Iterate on stream from a specified position (4) should return 4 events with
    // correct position properties

    let events = stream.iterate(Some(Position::new(4))).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 4);

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.position(), &Position::new(4 + i as u64));
    }

    // Iterate on stream from the head position should return a single event with
    // corretc Position property

    let events = stream.iterate(Some(Position::new(7))).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(7));

    // Iterate on stream from a position after the head position should return no
    // events

    assert_eq!(stream.iterate(Some(Position::new(8))).next(), None);

    // Iterate on a stream maintains the order of appended events

    stream.append(fixtures::event("a", "id_a", &[], 0), None)?;
    stream.append(fixtures::event("b", "id_b", &[], 0), None)?;
    stream.append(fixtures::event("c", "id_c", &[], 0), None)?;

    let events = stream.iterate(Some(Position::new(8))).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].identifier(), &Identifier::new("id_a")?);
    assert_eq!(events[1].identifier(), &Identifier::new("id_b")?);
    assert_eq!(events[2].identifier(), &Identifier::new("id_c")?);

    // Iterate on an unchanged stream returns the same events if called multiple
    // times

    let events_a = stream.iterate(None).collect::<Result<Vec<_>, _>>();
    let events_b = stream.iterate(None).collect::<Result<Vec<_>, _>>();

    assert_eq!(events_a, events_b);

    // Iterate on a reversed stream, including from a position, returns events in
    // reverse order

    let events = stream.iterate(Some(Position::new(8))).rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].identifier(), &Identifier::new("id_c")?);
    assert_eq!(events[1].identifier(), &Identifier::new("id_b")?);
    assert_eq!(events[2].identifier(), &Identifier::new("id_a")?);

    Ok(())
}

// Iterate Query

#[rustfmt::skip]
#[test]
fn iterate_query() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // A query on an empty stream returns no events

    let student_enrolled = Specifier::new(Identifier::new("student_enrolled")?);
    let query = Query::new([Selector::specifiers([student_enrolled.clone()])?])?;

    assert_none!(stream.iterate_query(query, None).0.next());

    // Using the standard set of events

    stream.append(fixtures::events()?, None)?;

    // A query with a single identifier selector should return the expected events

    let query = Query::new([Selector::specifiers([student_enrolled.clone()])?])?;

    let events = stream.iterate_query(query, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(2));
    assert_eq!(events[2].position(), &Position::new(4));

    // A query with a single identifier selector and a version should return the
    // expected events

    let query = Query::new([Selector::specifiers([student_enrolled
        .clone()
        .range(Version::new(1)..)])?])?;

    let events = stream.iterate_query(query, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position(), &Position::new(2));
    assert_eq!(events[1].position(), &Position::new(4));

    // A query with multiple identifier selectors should return the expected events

    let course_created = Specifier::new(Identifier::new("course_created")?);
    let course_updated = Specifier::new(Identifier::new("course_updated")?);
    let query = Query::new([Selector::specifiers([
        course_created.clone(),
        course_updated.clone(),
    ])?])?;

    let events = stream.iterate_query(query, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].position(), &Position::new(1));
    assert_eq!(events[1].position(), &Position::new(3));
    assert_eq!(events[2].position(), &Position::new(5));

    // A query with a Specifier and Tags selector should return the expected events

    let course_200 = Tag::new("course:200")?;
    let query = Query::new([Selector::specifiers_and_tags(
        [student_enrolled.clone()],
        [course_200.clone()],
    )?])?;

    let events = stream.iterate_query(query, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(2));

    // A query with a Specifier and Tags selector should return the expected events
    // (tags being used in a logical AND operation)

    let student_100 = Tag::new("student:100")?;
    let query = Query::new([Selector::specifiers_and_tags(
        [student_enrolled.clone()],
        [course_200.clone(), student_100.clone()],
    )?])?;

    let events = stream.iterate_query(query, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(0));

    // A query which doesn't match, returns no events

    let unknown = Specifier::new(Identifier::new("unknown")?);
    let query = Query::new([Selector::specifiers([unknown])?])?;

    assert_none!(stream.iterate_query(query, None).0.next());

    // A query with a from position only returns events matching and at positions
    // greater than or equal to the from position

    let query = Query::new([Selector::specifiers([student_enrolled.clone()])?])?;

    let events = stream.iterate_query(query, Some(Position::new(3))).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(4));

    // A query with a from position after the head position should return no events

    let query = Query::new([Selector::specifiers([student_enrolled.clone()])?])?;

    assert_none!(stream.iterate_query(query, Some(Position::new(8))).0.next());

    // A query with multiple selectors should return the expected events

    let query = Query::new([
        Selector::specifiers(vec![course_created.clone()])?,
        Selector::specifiers_and_tags(vec![student_enrolled.clone()], vec![student_100.clone()])?,
    ])?;

    let events = stream.iterate_query(query, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3,);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(1));
    assert_eq!(events[2].position(), &Position::new(5));

    // Iterate returns the same events when called on an unchanged stream multiple
    // times

    let query = Query::new([Selector::specifiers([student_enrolled.clone()])?])?;
    let position = Some(Position::new(3));

    let events_a = stream.iterate_query(query.clone(), position).0.collect::<Result<Vec<_>, _>>()?;
    let events_b = stream.iterate_query(query, position).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_a, events_b);

    // Iterate over a query reversed returns the expected events in reverse order

    let query = Query::new([
        Selector::specifiers(vec![course_created.clone()])?,
        Selector::specifiers_and_tags(vec![student_enrolled.clone()], vec![student_100.clone()])?,
    ])?;

    let events = stream.iterate_query(query, None).0.rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3,);
    assert_eq!(events[0].position(), &Position::new(5));
    assert_eq!(events[1].position(), &Position::new(1));
    assert_eq!(events[2].position(), &Position::new(0));

    // Iterate over a query using the prepared query returns the same events as the
    // original query

    let query = Query::new([
        Selector::specifiers(vec![course_created.clone()])?,
        Selector::specifiers_and_tags(vec![student_enrolled.clone()], vec![student_100.clone()])?,
    ])?;

    let (events_a, prepared_a) = stream.iterate_query(query, None);
    let (events_b, _) = stream.iterate_query(prepared_a, None);

    let events_a = events_a.collect::<Result<Vec<_>, _>>()?;
    let events_b = events_b.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_a, events_b);

    Ok(())
}
