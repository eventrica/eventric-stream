mod util;

use eventric_core::{
    Position,
    Stream,
};

// =================================================================================================
// Properties
// =================================================================================================

#[test]
fn empty_stream_properties_are_correct() {
    let stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()
        .unwrap();

    assert_eq!(stream.len().unwrap(), 0);
    assert!(stream.is_empty().unwrap());
}

#[test]
fn stream_properties_before_and_after_append_are_correct() {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()
        .unwrap();

    assert_eq!(stream.len().unwrap(), 0);
    assert!(stream.is_empty().unwrap());

    let position = stream.append([&util::event()], None).unwrap();

    assert_eq!(position, Position::new(0));
    assert_eq!(stream.len().unwrap(), 1);
    assert!(!stream.is_empty().unwrap());
}
