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
            IterateSelect as _,
        },
        select::{
            Mask,
            Selection,
            Selections,
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

    assert_eq!(stream.iter(None).next(), None);

    // Iterate on stream after single append return a single event with
    // correct properties

    stream.append(fixtures::event("one", "id_one", &["tag:a"], 1), None)?;

    let events = stream.iter(None).collect::<Result<Vec<_>, _>>()?;

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

    let events = stream.iter(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 8);

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.position(), &Position::new(i as u64));
    }

    // Iterate on stream from a specified position (4) should return 4 events with
    // correct position properties

    let events = stream.iter(Some(Position::new(4))).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 4);

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.position(), &Position::new(4 + i as u64));
    }

    // Iterate on stream from the head position should return a single event with
    // corretc Position property

    let events = stream.iter(Some(Position::new(7))).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(7));

    // Iterate on stream from a position after the head position should return no
    // events

    assert_eq!(stream.iter(Some(Position::new(8))).next(), None);

    // Iterate on a stream maintains the order of appended events

    stream.append(fixtures::event("a", "id_a", &[], 0), None)?;
    stream.append(fixtures::event("b", "id_b", &[], 0), None)?;
    stream.append(fixtures::event("c", "id_c", &[], 0), None)?;

    let events = stream.iter(Some(Position::new(8))).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].identifier(), &Identifier::new("id_a")?);
    assert_eq!(events[1].identifier(), &Identifier::new("id_b")?);
    assert_eq!(events[2].identifier(), &Identifier::new("id_c")?);

    // Iterate on an unchanged stream returns the same events if called multiple
    // times

    let events_a = stream.iter(None).collect::<Result<Vec<_>, _>>();
    let events_b = stream.iter(None).collect::<Result<Vec<_>, _>>();

    assert_eq!(events_a, events_b);

    // Iterate on a reversed stream, including from a position, returns events in
    // reverse order

    let events = stream.iter(Some(Position::new(8))).rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].identifier(), &Identifier::new("id_c")?);
    assert_eq!(events[1].identifier(), &Identifier::new("id_b")?);
    assert_eq!(events[2].identifier(), &Identifier::new("id_a")?);

    Ok(())
}

// Iterate Select: Selection

#[rustfmt::skip]
#[test]
fn iterate_selection_selection() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // A selection on an empty stream returns no events

    let student_enrolled = Specifier::new(Identifier::new("student_enrolled")?);
    let selection = Selection::new([Selector::specifiers([student_enrolled.clone()])?])?;

    assert_none!(stream.iter_select(selection, None).0.next());

    // Using the standard set of events

    stream.append(fixtures::events()?, None)?;

    // A selection with a single identifier selector should return the expected
    // events

    let selection = Selection::new([Selector::specifiers([student_enrolled.clone()])?])?;

    let events = stream.iter_select(selection, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(2));
    assert_eq!(events[2].position(), &Position::new(4));

    // A selection with a single identifier selector and a version should return
    // the expected events

    let selection = Selection::new([Selector::specifiers([student_enrolled
        .clone()
        .with_range(Version::new(1)..)])?])?;

    let events = stream.iter_select(selection, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position(), &Position::new(2));
    assert_eq!(events[1].position(), &Position::new(4));

    // A selection with multiple identifier selectors should return the expected
    // events

    let course_created = Specifier::new(Identifier::new("course_created")?);
    let course_updated = Specifier::new(Identifier::new("course_updated")?);
    let selection = Selection::new([Selector::specifiers([
        course_created.clone(),
        course_updated.clone(),
    ])?])?;

    let events = stream.iter_select(selection, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].position(), &Position::new(1));
    assert_eq!(events[1].position(), &Position::new(3));
    assert_eq!(events[2].position(), &Position::new(5));

    // A selection with a Specifier and Tags selectors should return the expected
    // events

    let course_200 = Tag::new("course:200")?;
    let selection = Selection::new([Selector::specifiers_and_tags(
        [student_enrolled.clone()],
        [course_200.clone()],
    )?])?;

    let events = stream.iter_select(selection, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(2));

    // A selection with a Specifier and Tags selectors should return the expected
    // events (tags being used in a logical AND operation)

    let student_100 = Tag::new("student:100")?;
    let selection = Selection::new([Selector::specifiers_and_tags(
        [student_enrolled.clone()],
        [course_200.clone(), student_100.clone()],
    )?])?;

    let events = stream.iter_select(selection, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(0));

    // A selection which doesn't match, returns no events

    let unknown = Specifier::new(Identifier::new("unknown")?);
    let selection = Selection::new([Selector::specifiers([unknown])?])?;

    assert_none!(stream.iter_select(selection, None).0.next());

    // A selection with a from position only returns events matching and at positions
    // greater than or equal to the from position

    let selection = Selection::new([Selector::specifiers([student_enrolled.clone()])?])?;

    let events = stream.iter_select(selection, Some(Position::new(3))).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position(), &Position::new(4));

    // A selection with a from position after the head position should return no
    // events

    let selection = Selection::new([Selector::specifiers([student_enrolled.clone()])?])?;

    assert_none!(stream.iter_select(selection, Some(Position::new(8))).0.next());

    // A selection with multiple selectors should return the expected events

    let selection = Selection::new([
        Selector::specifiers(vec![course_created.clone()])?,
        Selector::specifiers_and_tags(vec![student_enrolled.clone()], vec![student_100.clone()])?,
    ])?;

    let events = stream.iter_select(selection, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3,);
    assert_eq!(events[0].position(), &Position::new(0));
    assert_eq!(events[1].position(), &Position::new(1));
    assert_eq!(events[2].position(), &Position::new(5));

    // Iterate returns the same events when called on an unchanged stream multiple
    // times

    let selection = Selection::new([Selector::specifiers([student_enrolled.clone()])?])?;
    let position = Some(Position::new(3));

    let events_a = stream.iter_select(selection.clone(), position).0.collect::<Result<Vec<_>, _>>()?;
    let events_b = stream.iter_select(selection, position).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_a, events_b);

    // Iterate over a selection reversed returns the expected events in reverse
    // order

    let selection = Selection::new([
        Selector::specifiers(vec![course_created.clone()])?,
        Selector::specifiers_and_tags(vec![student_enrolled.clone()], vec![student_100.clone()])?,
    ])?;

    let events = stream.iter_select(selection, None).0.rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3,);
    assert_eq!(events[0].position(), &Position::new(5));
    assert_eq!(events[1].position(), &Position::new(1));
    assert_eq!(events[2].position(), &Position::new(0));

    // Iterate over a selection using the prepared selection returns the same
    // events as the original selection

    let selection = Selection::new([
        Selector::specifiers(vec![course_created.clone()])?,
        Selector::specifiers_and_tags(vec![student_enrolled.clone()], vec![student_100.clone()])?,
    ])?;

    let (events_a, prepared_a) = stream.iter_select(selection, None);
    let (events_b, _) = stream.iter_select(prepared_a, None);

    let events_a = events_a.collect::<Result<Vec<_>, _>>()?;
    let events_b = events_b.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_a, events_b);

    Ok(())
}

// Iterate Select: Selections

#[rustfmt::skip]
#[test]
fn iterate_selection_selections() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // Iterate with selectors on an empty stream should return no events

    let student_enrolled = Specifier::new(Identifier::new("student_enrolled")?);
    let selections = Selections::new([Selection::new([Selector::specifiers([student_enrolled.clone()])?])?])?;

    assert_none!(stream.iter_select(selections, None).0.next());

    // Using the standard set of events

    stream.append(fixtures::events()?, None)?;

    // Selections containing a single selection should return the expected events
    // (identical to the selection alone), and with a mask value where all events
    // match the single selection

    let selections = Selections::new([Selection::new([Selector::specifiers([student_enrolled.clone()])?])?])?;

    let events = stream.iter_select(selections, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!((events[0].event.position(), &events[0].mask), (&Position::new(0), &Mask::new(vec![true].into())));
    assert_eq!((events[1].event.position(), &events[1].mask), (&Position::new(2), &Mask::new(vec![true].into())));
    assert_eq!((events[2].event.position(), &events[2].mask), (&Position::new(4), &Mask::new(vec![true].into())));

    // A selections containing non-overlapping selections should return events
    // matching any selection with a correct mask value

    let course_created = Specifier::new(Identifier::new("course_created")?);
    let course_200 = Tag::new("course:200")?;
    let student_100 = Tag::new("student:100")?;
    let selections = Selections::new([
        Selection::new([Selector::specifiers_and_tags(
            vec![student_enrolled.clone()],
            vec![student_100.clone()]
        )?])?,
        Selection::new([Selector::specifiers_and_tags(
            vec![course_created.clone()],
            vec![course_200.clone()]
        )?])?
    ])?;

    let events = stream.iter_select(selections, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!((events[0].event.position(), &events[0].mask), (&Position::new(0), &Mask::new(vec![true, false].into())));
    assert_eq!((events[1].event.position(), &events[1].mask), (&Position::new(1), &Mask::new(vec![false, true].into())));

    // A selection containing overlapping selections should return events matching
    // any selection with a correct mask value

    let selections = Selections::new([
        Selection::new([Selector::specifiers(
            vec![student_enrolled.clone()]
        )?])?,
        Selection::new([Selector::specifiers_and_tags(
            vec![student_enrolled.clone()],
            vec![course_200.clone()]
        )?])?
    ])?;

    let events = stream.iter_select(selections, None).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 3);
    assert_eq!((events[0].event.position(), &events[0].mask), (&Position::new(0), &Mask::new(vec![true, true].into())));
    assert_eq!((events[1].event.position(), &events[1].mask), (&Position::new(2), &Mask::new(vec![true, true].into())));
    assert_eq!((events[2].event.position(), &events[2].mask), (&Position::new(4), &Mask::new(vec![true, false].into())));

    // A selection with a from position should only return events greater than or
    // equal to the from position

    let selections = Selections::new([
        Selection::new([Selector::specifiers(
            vec![student_enrolled.clone()]
        )?])?,
        Selection::new([Selector::specifiers_and_tags(
            vec![student_enrolled.clone()],
            vec![course_200.clone()]
        )?])?
    ])?;

    let events = stream.iter_select(selections, Some(Position::new(2))).0.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!((events[0].event.position(), &events[0].mask), (&Position::new(2), &Mask::new(vec![true, true].into())));
    assert_eq!((events[1].event.position(), &events[1].mask), (&Position::new(4), &Mask::new(vec![true, false].into())));

    // A selection which is iterated and reversed should return the same events
    // as the selection but in reverse order

    let selections = Selections::new([
        Selection::new([Selector::specifiers(
            vec![student_enrolled.clone()]
        )?])?,
        Selection::new([Selector::specifiers_and_tags(
            vec![student_enrolled.clone()],
            vec![course_200.clone()]
        )?])?
    ])?;

    let events = stream.iter_select(selections, Some(Position::new(2))).0.rev().collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events.len(), 2);
    assert_eq!((events[0].event.position(), &events[0].mask), (&Position::new(4), &Mask::new(vec![true, false].into())));
    assert_eq!((events[1].event.position(), &events[1].mask), (&Position::new(2), &Mask::new(vec![true, true].into())));

    // Iterate over a selection using the prepared selection returns the same events
    // as the original selection

    let selections = Selections::new([
        Selection::new([Selector::specifiers(
            vec![student_enrolled.clone()]
        )?])?,
        Selection::new([Selector::specifiers_and_tags(
            vec![student_enrolled.clone()],
            vec![course_200.clone()]
        )?])?
    ])?;

    let (events_a, prepared_a) = stream.iter_select(selections, None);
    let (events_b, _) = stream.iter_select(prepared_a, None);

    let events_a = events_a.collect::<Result<Vec<_>, _>>()?;
    let events_b = events_b.collect::<Result<Vec<_>, _>>()?;

    assert_eq!(events_a, events_b);

    Ok(())
}
