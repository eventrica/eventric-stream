use eventric_core::event::{
    Data,
    NewEvent,
    Identifier,
    Tag,
    Version,
};

// =================================================================================================
// Properties
// =================================================================================================

#[must_use]
pub fn event() -> NewEvent {
    NewEvent::new(
        Data::new("test_data").unwrap(),
        Identifier::new("test_identifier").unwrap(),
        Vec::from_iter([
            Tag::new("test_tag_1").unwrap(),
            Tag::new("test_tag_2").unwrap(),
        ]),
        Version::new(0),
    )
}
