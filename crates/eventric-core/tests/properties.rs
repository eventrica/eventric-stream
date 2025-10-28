mod util;

use std::error::Error;

use eventric_core::stream::Stream;

// =================================================================================================
// Properties
// =================================================================================================

#[test]
fn empty_stream_properties_are_correct() -> Result<(), Box<dyn Error>> {
    let stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    assert_eq!(0, stream.len()?);
    assert!(stream.is_empty()?);

    Ok(())
}

#[test]
fn stream_properties_before_and_after_append_are_correct() -> Result<(), Box<dyn Error>> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    assert_eq!(0, stream.len()?);
    assert!(stream.is_empty()?);

    let position = stream.append(
        [
            &util::event(),
            &util::event(),
            &util::event(),
            &util::event(),
        ],
        None,
    )?;

    assert_eq!(3, *position);
    assert_eq!(4, stream.len()?);
    assert!(!stream.is_empty()?);

    Ok(())
}
