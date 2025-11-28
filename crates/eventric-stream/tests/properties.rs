mod fixtures;

use eventric_stream::{
    error::Error,
    stream::{
        Stream,
        append::Append,
    },
};

// =================================================================================================
// Properties
// =================================================================================================

// Stream::len

#[test]
fn stream_len() -> Result<(), Error> {
    let path = eventric_stream::temp_path();

    // Property after multiple length changing-operations

    {
        let mut stream = Stream::builder(&path).open()?;

        assert_eq!(stream.len(), 0);

        stream.append(fixtures::event("one", "id_one", &[], 0), None)?;

        assert_eq!(stream.len(), 1);

        stream.append(fixtures::event("two", "id_two", &[], 0), None)?;
        stream.append(fixtures::event("three", "id_three", &[], 0), None)?;
        stream.append(fixtures::event("four", "id_four", &[], 0), None)?;

        assert_eq!(stream.len(), 4);

        stream.append(fixtures::events()?, None)?;

        assert_eq!(stream.len(), 11);
    }

    // Property after re-open (persistence) and length-changing operation

    {
        let mut stream = Stream::builder(&path).temporary(true).open()?;

        assert_eq!(stream.len(), 11);

        stream.append(fixtures::events()?, None)?;

        assert_eq!(stream.len(), 18);
    }

    Ok(())
}

// Stream::is_empty

#[test]
fn stream_is_empty() -> Result<(), Error> {
    let path = eventric_stream::temp_path();

    // Property after multiple length changing-operations

    {
        let mut stream = Stream::builder(&path).open()?;

        assert!(stream.is_empty());

        stream.append(fixtures::event("one", "id_one", &[], 0), None)?;

        assert!(!stream.is_empty());
    }

    // Property after re-open (persistence) and length-changing operation

    {
        let mut stream = Stream::builder(&path).temporary(true).open()?;

        assert!(!stream.is_empty());

        stream.append(fixtures::events()?, None)?;

        assert!(!stream.is_empty());
    }

    Ok(())
}
