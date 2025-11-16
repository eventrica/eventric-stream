use std::{
    path::Path,
    sync::LazyLock,
};

use eventric_stream_core::{
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
        append::Condition,
        query::{
            Query,
            Selector,
            Specifiers,
        },
    },
};

// =================================================================================================
// Properties
// =================================================================================================

#[test]
fn empty() -> Result<(), Error> {
    let stream = stream(eventric_stream::temp_path(), true)?;

    // Initial state of a new stream should be empty/zero-length

    {
        assert_eq!(0, stream.len());
        assert!(stream.is_empty());
    }

    Ok(())
}

#[test]
fn post_append() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    let position = stream.append(&*EVENTS, None)?;

    // Position after appending 4 events should be 3, with a stream length of 4, and
    // a non-empty stream

    {
        assert_eq!(3, *position);
        assert_eq!(4, stream.len());
        assert!(!stream.is_empty());
    }

    Ok(())
}

#[test]
fn post_append_error() -> Result<(), Error> {
    let mut stream = stream(eventric_stream::temp_path(), true)?;

    stream.append(&*EVENTS, None)?;

    let specifier = Specifier::new(Identifier::new("id")?);
    let specifiers = Specifiers::new([specifier])?;
    let selector = Selector::Specifiers(specifiers);
    let query = Query::new([selector])?;
    let condition = Condition::new(&query);

    let result = stream.append(&*EVENTS, Some(&condition));

    // Result should be a concurrency error after attempting an append with a
    // matching fail_if_matches query, and the length should not have changed.

    {
        assert_eq!(Err(Error::Concurrency), result);
        assert_eq!(4, stream.len());
        assert!(!stream.is_empty());
    }

    Ok(())
}

#[test]
fn post_reopen() -> Result<(), Error> {
    let path = eventric_stream::temp_path();

    {
        let mut stream = stream(path.clone(), false)?;

        // Initial state of a new stream should be empty/zero-length

        {
            assert_eq!(0, stream.len());
            assert!(stream.is_empty());
        }

        let position = stream.append(&*EVENTS, None)?;

        // Position after appending 4 events should be 3, with a stream length of 4, and
        // a non-empty stream

        {
            assert_eq!(3, *position);
            assert_eq!(4, stream.len());
            assert!(!stream.is_empty());
        }

        drop(stream);
    }

    let stream = stream(path, true)?;

    // Length should still be 3 when the stream is re-opened, and stream should
    // still be non-empty

    {
        assert_eq!(4, stream.len());
        assert!(!stream.is_empty());
    }

    Ok(())
}

// -------------------------------------------------------------------------------------------------

// Test Events

static EVENTS: LazyLock<Vec<EphemeralEvent>> = LazyLock::new(|| {
    let events = || {
        (0..4)
            .map(|_| {
                let data = Data::new("data")?;
                let identifier = Identifier::new("id")?;
                let tags = [Tag::new("tag_a")?, Tag::new("tag_b")?];
                let version = Version::new(0);

                Ok(EphemeralEvent::new(data, identifier, tags, version))
            })
            .collect::<Result<Vec<_>, Error>>()
    };

    events().unwrap()
});

// -------------------------------------------------------------------------------------------------

// Test Stream

fn stream<P>(path: P, temporary: bool) -> Result<Stream, Error>
where
    P: AsRef<Path>,
{
    Stream::builder(path).temporary(temporary).open()
}
