mod fixtures;

use eventric_stream::{
    error::Error,
    stream::{
        Stream,
        append::Append as _,
    },
};

// =================================================================================================
// Properties
// =================================================================================================

// Stream Properties

#[test]
fn is_empty() -> Result<(), Error> {
    let path = eventric_stream::temp_path();

    // Property after multiple length changing-operations

    {
        let mut stream = Stream::builder(&path).open()?;

        // For new stream, empty

        assert!(stream.is_empty());

        // After single append, not empty

        stream.append(fixtures::event("one", "id_one", &[], 0), None)?;

        assert!(!stream.is_empty());
    }

    // Property after re-open (persistence) and length-changing operation

    {
        let mut stream = Stream::builder(&path).temporary(true).open()?;

        // For re-opened stream, not empty

        assert!(!stream.is_empty());

        // For re-opened stream after batch append, still not empty

        stream.append(fixtures::events()?, None)?;

        assert!(!stream.is_empty());
    }

    Ok(())
}

#[test]
fn len() -> Result<(), Error> {
    let path = eventric_stream::temp_path();

    // Property after multiple length changing-operations

    {
        let mut stream = Stream::builder(&path).open()?;

        // For new stream, length of 0

        assert_eq!(stream.len(), 0);

        // After single append, length of 1

        stream.append(fixtures::event("one", "id_one", &[], 0), None)?;

        assert_eq!(stream.len(), 1);

        // After multiple appends, length of 4

        stream.append(fixtures::event("two", "id_two", &[], 0), None)?;
        stream.append(fixtures::event("three", "id_three", &[], 0), None)?;
        stream.append(fixtures::event("four", "id_four", &[], 0), None)?;

        assert_eq!(stream.len(), 4);

        // After batch append (batch size 7), length of 11

        stream.append(fixtures::events()?, None)?;

        assert_eq!(stream.len(), 11);
    }

    // Property after re-open (persistence) and length-changing operation

    {
        let mut stream = Stream::builder(&path).temporary(true).open()?;

        // For re-opened stream, length of 11

        assert_eq!(stream.len(), 11);

        // After batch append (batch size 7), length of 18

        stream.append(fixtures::events()?, None)?;

        assert_eq!(stream.len(), 18);
    }

    Ok(())
}
