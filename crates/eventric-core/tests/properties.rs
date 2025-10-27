use eventric_core::{
    Data,
    Event,
    Identifier,
    Position,
    Stream,
    Tag,
    Version,
};

// =================================================================================================
// Properties
// =================================================================================================

fn test_event() -> Event {
    Event::new(
        Data::new("test_data".bytes().collect()),
        Identifier::new("test_identifier".into()),
        Vec::from_iter([Tag::new("test_tag_1".into()), Tag::new("test_tag_2".into())]),
        Version::new(0),
    )
}

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

    let position = stream.append([&test_event()], None).unwrap();

    assert_eq!(position, Position::new(0));
    assert_eq!(stream.len().unwrap(), 1);
    assert!(!stream.is_empty().unwrap());
}
