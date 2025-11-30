mod fixtures;

use eventric_stream::{
    error::Error,
    event::Position,
    stream::append::Append,
};

// =================================================================================================
// Append
// =================================================================================================

#[test]
fn append() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // Appending a single event returns the expected position

    let event = fixtures::event("one", "id_one", &[], 0)?;
    let position = stream.append([event], None)?;

    assert_eq!(position, Position::new(0));

    // Appending multiple events returns the expected position

    let events = fixtures::events()?;
    let position = stream.append(events, None)?;

    assert_eq!(position, Position::new(7));

    // Append passes with concurrency check greater than or equal to head

    let events = fixtures::events()?;
    let result = stream.append(events, Some(Position::new(7)));

    assert_eq!(result, Ok(Position::new(14)));

    // Append fails with concurrency check less than head

    let events = fixtures::events()?;
    let result = stream.append(events, Some(Position::new(7)));

    assert_eq!(result, Err(Error::Concurrency));

    Ok(())
}
