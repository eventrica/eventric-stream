use bytes::BufMut as _;

use crate::{
    model::stream::Position,
    persistence::{
        model::event::Event,
        operation::Write,
    },
};

// =================================================================================================
// Event
// =================================================================================================

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &Event) {
    let key = position.value().to_be_bytes();

    let mut value = Vec::new();

    write_value(&mut value, event);

    write.batch.insert(&write.keyspaces.data, key, value);
}

// -------------------------------------------------------------------------------------------------

// Values

fn write_value(value: &mut Vec<u8>, event: &Event) {
    let descriptor_identifier = event.descriptor.identifer().hash();
    let descriptor_version = event.descriptor.version().value();
    let tags_len = u8::try_from(event.tags.len()).expect("max tag count exceeded");

    value.put_u64(descriptor_identifier);
    value.put_u8(descriptor_version);
    value.put_u8(tags_len);

    for tag in &event.tags {
        let tag = tag.hash();

        value.put_u64(tag);
    }

    let data = &event.data;

    value.put_slice(data);
}
