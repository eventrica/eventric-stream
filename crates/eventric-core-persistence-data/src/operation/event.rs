use bytes::BufMut as _;
use eventric_core_model::Position;
use eventric_core_persistence::{
    EventRef,
    Write,
};

// =================================================================================================
// Event
// =================================================================================================

// Insert

pub fn insert<'a>(write: &mut Write<'_>, position: Position, event: &'a EventRef<'a>) {
    let key = position.value().to_be_bytes();

    let mut value = Vec::new();

    write_value(&mut value, event);

    write.batch.insert(&write.keyspaces.data, key, value);
}

// -------------------------------------------------------------------------------------------------

// Values

fn write_value<'a>(value: &mut Vec<u8>, event: &'a EventRef<'a>) {
    let identifier = event.descriptor.identifer().hash();
    let version = event.descriptor.version().value();
    let tags_len = u8::try_from(event.tags.len()).expect("max tag count exceeded");

    value.put_u64(identifier);
    value.put_u8(version);
    value.put_u8(tags_len);

    for tag in &event.tags {
        let tag = tag.hash();

        value.put_u64(tag);
    }

    let data = &event.data;

    value.put_slice(data);
}
