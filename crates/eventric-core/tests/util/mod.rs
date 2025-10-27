use eventric_core::{
    Data,
    Event,
    Identifier,
    Tag,
    Version,
};

// =================================================================================================
// Properties
// =================================================================================================

#[must_use]
pub fn event() -> Event {
    Event::new(
        Data::new("test_data").unwrap(),
        Identifier::new("test_identifier").unwrap(),
        Vec::from_iter([Tag::new("test_tag_1".into()), Tag::new("test_tag_2".into())]),
        Version::new(0),
    )
}
