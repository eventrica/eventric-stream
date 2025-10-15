use std::error::Error;

use bytes::{
    Buf as _,
    BufMut as _,
};
use eventric_core_model::{
    Data,
    Position,
    Version,
};
use eventric_core_persistence::{
    DescriptorHash,
    EventHash,
    EventHashRef,
    IdentifierHash,
    TagHash,
};
use eventric_core_state::{
    Read,
    Write,
};

// =================================================================================================
// Event
// =================================================================================================

// Get

pub fn get(read: &Read<'_>, position: Position) -> Result<Option<EventHash>, Box<dyn Error>> {
    let key = position.value().to_be_bytes();
    let value = read.keyspaces.data.get(key)?;
    let event = value.map(|slice| read_value(&slice[..]));

    Ok(event)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &EventHashRef<'_>) {
    let key = position.value().to_be_bytes();

    let mut value = Vec::new();

    write_value(&mut value, event);

    write.batch.insert(&write.keyspaces.data, key, value);
}

// -------------------------------------------------------------------------------------------------

// Values

fn read_value(mut value: &[u8]) -> EventHash {
    let identifier = IdentifierHash::new(value.get_u64());
    let version = Version::new(value.get_u8());
    let descriptor = DescriptorHash::new(identifier, version);

    let tags_len = value.get_u8();

    let mut tags = Vec::with_capacity(tags_len as usize);

    for _ in 0..tags_len {
        let tag = TagHash::new(value.get_u64());

        tags.push(tag);
    }

    let data = Data::new(value.iter().map(ToOwned::to_owned).collect::<Vec<_>>());

    EventHash::new(data, descriptor, tags)
}

fn write_value(value: &mut Vec<u8>, event: &EventHashRef<'_>) {
    let descriptor = &event.descriptor;
    let identifier = descriptor.identifer().hash();
    let version = descriptor.version().value();

    value.put_u64(identifier);
    value.put_u8(version);

    let tags_len = u8::try_from(event.tags.len()).expect("max tag count exceeded");

    value.put_u8(tags_len);

    for tag in &event.tags {
        let tag = tag.hash();

        value.put_u64(tag);
    }

    let data = event.data.as_ref();

    value.put_slice(data);
}
