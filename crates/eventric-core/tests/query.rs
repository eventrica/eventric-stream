use assertables::{
    assert_none,
    assert_some,
    assert_some_as_result,
};
use eventric_core::{
    error::Error,
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Specifier,
        Tag,
        Version,
    },
    stream::{
        Stream,
        query::{
            Cache,
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
fn new() -> Result<(), Error> {
    let stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    let cache = Cache::default();
    let condition = Condition::default();

    let mut events = stream.query(&condition, &cache, None);

    // Query on an empty stream should always return an empty iterator

    {
        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn condition() -> Result<(), Error> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    // New stream should always have a length of zero and be empty

    {
        assert_eq!(0, stream.len());
        assert!(stream.is_empty());
    }

    let event_0 = EphemeralEvent::new(
        Data::new("data_a")?,
        Identifier::new("id_a")?,
        [Tag::new("tag_1")?, Tag::new("tag_2")?, Tag::new("tag_3")?],
        Version::new(0),
    );

    let event_1 = EphemeralEvent::new(
        Data::new("data_b")?,
        Identifier::new("id_b")?,
        [Tag::new("tag_2")?, Tag::new("tag_3")?, Tag::new("tag_4")?],
        Version::new(1),
    );

    let event_2 = EphemeralEvent::new(
        Data::new("data_c")?,
        Identifier::new("id_c")?,
        [Tag::new("tag_3")?, Tag::new("tag_4")?, Tag::new("tag_5")?],
        Version::new(0),
    );

    let event_3 = EphemeralEvent::new(
        Data::new("data_d")?,
        Identifier::new("id_c")?,
        [Tag::new("tag_4")?, Tag::new("tag_5")?, Tag::new("tag_6")?],
        Version::new(1),
    );

    let position = stream.append([&event_0, &event_1, &event_2, &event_3], None)?;

    // Position should be 3 after appending 4 test events

    {
        assert_eq!(3, *position);
    }

    let cache = Cache::default();
    let condition = Condition::default();

    let mut events = stream.query(&condition, &cache, None);

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

    let specifier = Specifier::new(event_2.identifier().clone(), None);
    let selector = Selector::specifiers([specifier])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with a single specifier with no version range should return any
    // events with a specific identifier, regardless of version, in this case 2
    // events have the required identifier but different versions

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(event_2.data(), event.data());
        assert_eq!(2, **event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(event_3.data(), event.data());
        assert_eq!(3, **event.position());

        assert_none!(events.next());
    }

    let range = Version::new(1)..Version::MAX;
    let specifier = Specifier::new(event_2.identifier().clone(), Some(range));
    let selector = Selector::specifiers([specifier])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with a single specifier with a version range should only return
    // events matching both the specifier and the range, in this case the single
    // event with

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(event_3.data(), event.data());
        assert_eq!(3, **event.position());

        assert_none!(events.next());
    }

    // Query Multiple Specifiers

    let specifier_0 = Specifier::new(event_0.identifier().clone(), None);
    let specifier_1 = Specifier::new(event_1.identifier().clone(), None);
    let selector = Selector::specifiers([specifier_0, specifier_1])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(event_0.data(), event.data());
        assert_eq!(0, **event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(event_1.data(), event.data());
        assert_eq!(1, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}
