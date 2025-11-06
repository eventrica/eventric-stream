use std::path::Path;

use assertables::{
    assert_none,
    assert_some,
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
        query::{
            Condition,
            Query,
            Selector,
        },
    },
};

// =================================================================================================
// Properties
// =================================================================================================

#[test]
fn default_empty() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), false)?;

    let condition = Condition::default();

    let mut events = stream.query(&condition, None);

    // Query on an empty stream should always return an empty iterator

    {
        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn default() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let condition = Condition::default();

    let mut events = stream.query(&condition, None);

    // A query with default (empty) conditions should return all of the events in
    // the stream

    #[allow(unused_must_use)]
    {
        assert_some!(events.next());
        assert_some!(events.next());
        assert_some!(events.next());
        assert_some!(events.next());
        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn specifier() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_2 = Identifier::new("id_2")?;
    let spec_0 = Specifier::new(id_2);
    let sel_0 = Selector::specifiers([spec_0])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    // A query with a single specifier with no version range should return any
    // events with a specific identifier, regardless of version, in this case 2
    // events have the required identifier but different versions

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_2")?, event.data());
        assert_eq!(&Position::new(2), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn specifier_with_range() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_2 = Identifier::new("id_2")?;
    let range_0 = Version::new(1)..Version::MAX;
    let spec_0 = Specifier::new(id_2).range(range_0);
    let sel_0 = Selector::specifiers([spec_0])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    // A query with a single specifier with a version range should only return
    // events matching both the specifier and the range, in this case the single
    // event with a matching version (1..)

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_specifiers() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let id_1 = Identifier::new("id_1")?;
    let spec_1 = Specifier::new(id_1);
    let sel_0 = Selector::specifiers([spec_0, spec_1])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    // A query with multiple specifiers (without version ranges in this case) should
    // return events which match any of the specifiers, in this case the two events
    // matching the identifiers supplied

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_1")?, event.data());
        assert_eq!(&Position::new(1), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_specifiers_with_range() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let id_2 = Identifier::new("id_2")?;
    let range_0 = Version::new(1)..Version::MAX;
    let spec_1 = Specifier::new(id_2).range(range_0);
    let sel_0 = Selector::specifiers([spec_0, spec_1])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    // A query with multiple specifiers (some of which have version ranges) should
    // again return events matching any of the specifiers

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn specifiers_and_tags() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_2 = Identifier::new("id_2")?;
    let spec_0 = Specifier::new(id_2);
    let tag_4 = Tag::new("tag_4")?;
    let tag_5 = Tag::new("tag_5")?;
    let sel_0 = Selector::specifiers_and_tags([spec_0], [tag_4, tag_5])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    // Should return events matching the specifier AND all tags

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_2")?, event.data());
        assert_eq!(&Position::new(2), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn specifiers_and_tags_with_version_range() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_2 = Identifier::new("id_2")?;
    let range_0 = Version::new(1)..Version::MAX;
    let spec_0 = Specifier::new(id_2).range(range_0);
    let tag_4 = Tag::new("tag_4")?;
    let tag_5 = Tag::new("tag_5")?;
    let sel_0 = Selector::specifiers_and_tags([spec_0], [tag_4, tag_5])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn specifiers_and_tags_no_match() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let tag_4 = Tag::new("tag_4")?;
    let tag_5 = Tag::new("tag_5")?;
    let sel_0 = Selector::specifiers_and_tags([spec_0], [tag_4, tag_5])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_specifiers_and_tags() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let id_1 = Identifier::new("id_1")?;
    let spec_1 = Specifier::new(id_1);
    let tag_2 = Tag::new("tag_2")?;
    let tag_3 = Tag::new("tag_3")?;
    let sel_0 = Selector::specifiers_and_tags([spec_0, spec_1], [tag_2, tag_3])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_1")?, event.data());
        assert_eq!(&Position::new(1), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_selectors_same_type() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Query with two Specifiers selectors (OR between selectors)
    // Selector 1: id_a
    // Selector 2: id_b
    // Should match EVENT_0 and EVENT_1
    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let sel_0 = Selector::specifiers([spec_0])?;

    let id_1 = Identifier::new("id_1")?;
    let spec_0 = Specifier::new(id_1);
    let sel_1 = Selector::specifiers([spec_0])?;

    let query = Query::new([sel_0, sel_1])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_1")?, event.data());
        assert_eq!(&Position::new(1), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

// #[test]
// fn multiple_selectors_different_types() -> Result<(), Error> {
//     let stream = stream(eventric_stream::temp_path(), true)?;

//     // Query with different selector types (OR between selectors)
//     // Selector 1: Specifiers(id_a)
//     // Selector 2: Tags(tag_6)
//     // Should match EVENT_0 (id_a) and EVENT_3 (has tag_6)
//     let id_0 = Identifier::new("id_0")?;
//     let spec_0 = Specifier::new(id_0);
//     let sel_0 = Selector::specifiers([spec_0])?;

//     let tag_6 = Tag::new("tag_6")?;
//     let sel_1 = Selector::tags([tag_6])?;

//     let query = Query::new([sel_0, sel_1])?;
//     let condition = Condition::default().matches(&query);

//     let mut events = stream.query(&condition, None);

//     {
//         let event = assert_some_as_result!(events.next()).unwrap()?;
//         assert_eq!(&Data::new("data_0")?, event.data());
//         assert_eq!(0, **event.position());

//         let event = assert_some_as_result!(events.next()).unwrap()?;
//         assert_eq!(&Data::new("data_3")?, event.data());
//         assert_eq!(3, **event.position());

//         assert_none!(events.next());
//     }

//     Ok(())
// }

#[test]
fn multiple_selectors_mixed_types() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Complex query with three different selector types
    // Selector 1: Specifiers(id_a)
    // Selector 2: Tags(tag_4 AND tag_6)
    // Selector 3: SpecifiersAndTags(id_b, tag_2)
    // Should match EVENT_0 (id_a), EVENT_1 (id_b AND tag_2), EVENT_3 (tag_4 AND
    // tag_6)
    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let sel_0 = Selector::specifiers([spec_0])?;

    let id_1 = Identifier::new("id_1")?;
    let spec_0 = Specifier::new(id_1);
    let tag_2 = Tag::new("tag_2")?;
    let sel_1 = Selector::specifiers_and_tags([spec_0], [tag_2])?;

    let query = Query::new([sel_0, sel_1])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_1")?, event.data());
        assert_eq!(&Position::new(1), event.position());

        // let event = assert_some_as_result!(events.next()).unwrap()?;
        // assert_eq!(&Data::new("data_3")?, event.data());
        // assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

// #[test]
// fn multiple_selectors_overlapping_results() -> Result<(), Error> {
//     let stream = stream(eventric_stream::temp_path(), true)?;

//     // Query with selectors that match overlapping events
//     // Selector 1: id_c (matches EVENT_2 and EVENT_3)
//     // Selector 2: Tags(tag_4 AND tag_5) (also matches EVENT_2 and EVENT_3)
//     // Should return each event only once despite multiple matches
//     let id_2 = Identifier::new("id_2")?;
//     let spec_0 = Specifier::new(id_2);
//     let sel_0 = Selector::specifiers([spec_0])?;

//     let tag_4 = Tag::new("tag_4")?;
//     let tag_5 = Tag::new("tag_5")?;
//     let sel_1 = Selector::tags([tag_4, tag_5])?;

//     let query = Query::new([sel_0, sel_1])?;
//     let condition = Condition::default().matches(&query);

//     let mut events = stream.query(&condition, None);

//     {
//         let event = assert_some_as_result!(events.next()).unwrap()?;
//         assert_eq!(&Data::new("data_2")?, event.data());
//         assert_eq!(&Position::new(2), event.position());

//         let event = assert_some_as_result!(events.next()).unwrap()?;
//         assert_eq!(&Data::new("data_3")?, event.data());
//         assert_eq!(&Position::new(3), event.position());

//         assert_none!(events.next());
//     }

//     Ok(())
// }

#[test]
fn complex_version_ranges() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Query with multiple specifiers with different version ranges
    // Specifier 1: id_a version 0 only
    // Specifier 2: id_c version 1+
    // Should match EVENT_0 (id_a v0) and EVENT_3 (id_c v1)

    let id_0 = Identifier::new("id_0")?;
    let range_0 = Version::new(0)..Version::new(1);
    let spec_0 = Specifier::new(id_0).range(range_0);
    let id_2 = Identifier::new("id_2")?;
    let range_1 = Version::new(1)..Version::MAX;
    let spec_1 = Specifier::new(id_2).range(range_1);
    let sel_0 = Selector::specifiers([spec_0, spec_1])?;

    let query = Query::new([sel_0])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_specifiers_and_tags_selectors() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Query with two SpecifiersAndTags selectors
    // Selector 1: id_a AND (tag_1 AND tag_2)
    // Selector 2: id_c AND (tag_5 AND tag_6)
    // Should match EVENT_0 and EVENT_3

    let id_0 = Identifier::new("id_0")?;
    let spec_0 = Specifier::new(id_0);
    let tag_1 = Tag::new("tag_1")?;
    let tag_2 = Tag::new("tag_2")?;
    let sel_0 = Selector::specifiers_and_tags([spec_0], [tag_1, tag_2])?;

    let id_2 = Identifier::new("id_2")?;
    let spec_0 = Specifier::new(id_2);
    let tag_5 = Tag::new("tag_5")?;
    let tag_6 = Tag::new("tag_6")?;
    let sel_1 = Selector::specifiers_and_tags([spec_0], [tag_5, tag_6])?;

    let query = Query::new([sel_0, sel_1])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_0")?, event.data());
        assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_3")?, event.data());
        assert_eq!(&Position::new(3), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn complex_multi_selector_with_ranges() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Complex query combining all selector types with version ranges
    // Selector 1: SpecifiersAndTags(id_c v0 only, tag_3)
    // Selector 2: Specifiers(id_b v1+)
    // Selector 3: Tags(tag_1 AND tag_2)
    // Should match EVENT_0 (tag_1 AND tag_2), EVENT_1 (id_b v1), EVENT_2 (id_c v0
    // AND tag_3)

    let id_2 = Identifier::new("id_2")?;
    let range_0 = Version::new(0)..Version::new(1);
    let spec_0 = Specifier::new(id_2).range(range_0);
    let tag_3 = Tag::new("tag_3")?;
    let sel_0 = Selector::specifiers_and_tags([spec_0], [tag_3])?;

    let id_1 = Identifier::new("id_1")?;
    let range_0 = Version::new(1)..Version::MAX;
    let spec_1 = Specifier::new(id_1).range(range_0);
    let sel_1 = Selector::specifiers([spec_1])?;

    let query = Query::new([sel_0, sel_1])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, None);

    {
        // let event = assert_some_as_result!(events.next()).unwrap()?;
        // assert_eq!(&Data::new("data_0")?, event.data());
        // assert_eq!(&Position::new(0), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_1")?, event.data());
        assert_eq!(&Position::new(1), event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(&Data::new("data_2")?, event.data());
        assert_eq!(&Position::new(2), event.position());

        assert_none!(events.next());
    }

    Ok(())
}

// -------------------------------------------------------------------------------------------------

// Test Data

fn stream<P>(path: P, populate: bool) -> Result<Stream, Error>
where
    P: AsRef<Path>,
{
    let mut stream = Stream::builder(path).temporary(true).open()?;

    if populate {
        stream.append(
            [
                &EphemeralEvent::new(
                    Data::new("data_0")?,
                    Identifier::new("id_0")?,
                    [Tag::new("tag_1")?, Tag::new("tag_2")?, Tag::new("tag_3")?],
                    Version::new(0),
                ),
                &EphemeralEvent::new(
                    Data::new("data_1")?,
                    Identifier::new("id_1")?,
                    [Tag::new("tag_2")?, Tag::new("tag_3")?, Tag::new("tag_4")?],
                    Version::new(1),
                ),
                &EphemeralEvent::new(
                    Data::new("data_2")?,
                    Identifier::new("id_2")?,
                    [Tag::new("tag_3")?, Tag::new("tag_4")?, Tag::new("tag_5")?],
                    Version::new(0),
                ),
                &EphemeralEvent::new(
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
