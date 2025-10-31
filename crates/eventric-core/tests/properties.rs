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
fn new() -> Result<(), Error> {
    let stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    // Initial state of a new stream should be empty/zero-length

    {
        assert_eq!(0, stream.len());
        assert!(stream.is_empty());
    }

    Ok(())
}

#[test]
fn after_append() -> Result<(), Error> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    // Initial state of a new stream should be empty/zero-length

    {
        assert_eq!(0, stream.len());
        assert!(stream.is_empty());
    }

    let events = (0..4)
        .map(|_| {
            let data = Data::new("data")?;
            let identifier = Identifier::new("id")?;
            let tags = [Tag::new("tag_a")?, Tag::new("tag_b")?];
            let version = Version::new(0);

            Ok(EphemeralEvent::new(data, identifier, tags, version))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let position = stream.append(&events, None)?;

    // Position after appending 4 events should be 3, with a stream length of 4, and
    // a non-empty stream

    {
        assert_eq!(3, *position);
        assert_eq!(4, stream.len());
        assert!(!stream.is_empty());
    }

    let specifier = Specifier::new(Identifier::new("id")?, None);
    let specifiers = Specifiers::new([specifier])?;
    let selector = Selector::Specifiers(specifiers);
    let query = Query::new([selector])?;
    let condition = Condition::new(&query);

    let result = stream.append(&events, Some(&condition));

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
fn after_drop_and_open() -> Result<(), Error> {
    let path = eventric_core::temp_path();

    {
        let mut stream = Stream::builder(path.clone()).temporary(false).open()?;

        // Initial state of a new stream should be empty/zero-length

        {
            assert_eq!(0, stream.len());
            assert!(stream.is_empty());
        }

        let events = (0..4)
            .map(|_| {
                let data = Data::new("data")?;
                let identifier = Identifier::new("id")?;
                let tags = [Tag::new("tag_a")?, Tag::new("tag_b")?];
                let version = Version::new(0);

                Ok(EphemeralEvent::new(data, identifier, tags, version))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let position = stream.append(&events, None)?;

        // Position after appending 4 events should be 3, with a stream length of 4, and
        // a non-empty stream

        {
            assert_eq!(3, *position);
            assert_eq!(4, stream.len());
            assert!(!stream.is_empty());
        }

        drop(stream);
    }

    let stream = Stream::builder(path).temporary(true).open()?;

    // Length should still be 3 when the stream is re-opened, and stream should
    // still be non-empty

    {
        assert_eq!(4, stream.len());
        assert!(!stream.is_empty());
    }

    Ok(())
}
