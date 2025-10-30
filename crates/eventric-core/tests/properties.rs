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
fn empty_stream_properties_are_correct() -> Result<(), Error> {
    let stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    assert_eq!(0, stream.len());
    assert!(stream.is_empty());

    Ok(())
}

#[test]
fn stream_properties_before_and_after_ok_append_are_correct() -> Result<(), Error> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    assert_eq!(0, stream.len());
    assert!(stream.is_empty());

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

    assert_eq!(3, *position);
    assert_eq!(4, stream.len());
    assert!(!stream.is_empty());

    Ok(())
}

#[test]
fn stream_properties_before_and_after_err_append_are_correct() -> Result<(), Error> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    assert_eq!(0, stream.len());
    assert!(stream.is_empty());

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

    assert_eq!(3, *position);
    assert_eq!(4, stream.len());
    assert!(!stream.is_empty());

    let specifier = Specifier::new(Identifier::new("id")?, None);
    let specifiers = Specifiers::new([specifier])?;
    let selector = Selector::Specifiers(specifiers);
    let query = Query::new([selector])?;
    let condition = Condition::new(&query);

    let result = stream.append(&events, Some(&condition));

    assert_eq!(Err(Error::Concurrency), result);
    assert_eq!(4, stream.len());
    assert!(!stream.is_empty());

    Ok(())
}
