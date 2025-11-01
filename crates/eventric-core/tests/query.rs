use std::{
    path::Path,
    sync::LazyLock,
};

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
fn default_empty() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), false)?;
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
fn default() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
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

    Ok(())
}

#[test]
fn specifier() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
    let cache = Cache::default();

    let specifier = Specifier::new(EVENT_2.identifier().clone());
    let selector = Selector::specifiers([specifier])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with a single specifier with no version range should return any
    // events with a specific identifier, regardless of version, in this case 2
    // events have the required identifier but different versions

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_2.data(), event.data());
        assert_eq!(2, **event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_3.data(), event.data());
        assert_eq!(3, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn specifier_with_range() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
    let cache = Cache::default();

    let range = Version::new(1)..Version::MAX;
    let specifier = Specifier::new(EVENT_2.identifier().clone()).range(range);
    let selector = Selector::specifiers([specifier])?;

    println!("selector: {selector:?}");

    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with a single specifier with a version range should only return
    // events matching both the specifier and the range, in this case the single
    // event with a matching version (1..)

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_3.data(), event.data());
        assert_eq!(3, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_specifiers() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
    let cache = Cache::default();

    let specifier_0 = Specifier::new(EVENT_0.identifier().clone());
    let specifier_1 = Specifier::new(EVENT_1.identifier().clone());
    let selector = Selector::specifiers([specifier_0, specifier_1])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with multiple specifiers (without version ranges in this case) should
    // return events which match any of the specifiers, in this case the two events
    // matching the identifiers supplied

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_0.data(), event.data());
        assert_eq!(0, **event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_1.data(), event.data());
        assert_eq!(1, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_specifiers_with_range() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
    let cache = Cache::default();

    let specifier_0 = Specifier::new(EVENT_0.identifier().clone());
    let range = Version::new(1)..Version::MAX;
    let specifier_1 = Specifier::new(EVENT_2.identifier().clone()).range(range);
    let selector = Selector::specifiers([specifier_0, specifier_1])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with multiple specifiers (some of which have version ranges) should
    // again return events matching any of the specifiers

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_0.data(), event.data());
        assert_eq!(0, **event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_3.data(), event.data());
        assert_eq!(3, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn tag() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
    let cache = Cache::default();

    let tag_1 = EVENT_0.tags()[0].clone();
    let selector = Selector::tags([tag_1])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with a single tag should return only events matching that tag, in
    // this case the first event

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_0.data(), event.data());
        assert_eq!(0, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}

#[test]
fn multiple_tags() -> Result<(), Error> {
    let stream = stream(eventric_core::temp_path(), true)?;
    let cache = Cache::default();

    let tag_4 = EVENT_3.tags()[0].clone();
    let tag_5 = EVENT_3.tags()[1].clone();
    let selector = Selector::tags([tag_4, tag_5])?;
    let query = Query::new([selector])?;
    let condition = Condition::default().matches(&query);

    let mut events = stream.query(&condition, &cache, None);

    // A query with multiple tags should return only events matching all tags, in
    // this case the last 2 events

    {
        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_2.data(), event.data());
        assert_eq!(2, **event.position());

        let event = assert_some_as_result!(events.next()).unwrap()?;
        assert_eq!(EVENT_3.data(), event.data());
        assert_eq!(3, **event.position());

        assert_none!(events.next());
    }

    Ok(())
}

// -------------------------------------------------------------------------------------------------

// Test Events

static EVENT_0: LazyLock<EphemeralEvent> = LazyLock::new(|| {
    let event = || {
        Ok::<EphemeralEvent, Error>(EphemeralEvent::new(
            Data::new("data_a")?,
            Identifier::new("id_a")?,
            [Tag::new("tag_1")?, Tag::new("tag_2")?, Tag::new("tag_3")?],
            Version::new(0),
        ))
    };

    event().unwrap()
});

static EVENT_1: LazyLock<EphemeralEvent> = LazyLock::new(|| {
    let event = || {
        Ok::<EphemeralEvent, Error>(EphemeralEvent::new(
            Data::new("data_b")?,
            Identifier::new("id_b")?,
            [Tag::new("tag_2")?, Tag::new("tag_3")?, Tag::new("tag_4")?],
            Version::new(1),
        ))
    };

    event().unwrap()
});

static EVENT_2: LazyLock<EphemeralEvent> = LazyLock::new(|| {
    let event = || {
        Ok::<EphemeralEvent, Error>(EphemeralEvent::new(
            Data::new("data_c")?,
            Identifier::new("id_c")?,
            [Tag::new("tag_3")?, Tag::new("tag_4")?, Tag::new("tag_5")?],
            Version::new(0),
        ))
    };

    event().unwrap()
});

static EVENT_3: LazyLock<EphemeralEvent> = LazyLock::new(|| {
    let event = || {
        Ok::<EphemeralEvent, Error>(EphemeralEvent::new(
            Data::new("data_d")?,
            Identifier::new("id_c")?,
            [Tag::new("tag_4")?, Tag::new("tag_5")?, Tag::new("tag_6")?],
            Version::new(1),
        ))
    };

    event().unwrap()
});

// -------------------------------------------------------------------------------------------------

// Test Stream

fn stream<P>(path: P, populate: bool) -> Result<Stream, Error>
where
    P: AsRef<Path>,
{
    let mut stream = Stream::builder(path).temporary(true).open()?;

    if populate {
        stream.append([&*EVENT_0, &*EVENT_1, &*EVENT_2, &*EVENT_3], None)?;
    }

    Ok(stream)
}
