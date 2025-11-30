#![allow(clippy::doc_markdown)]
#![allow(dead_code)]

use eventric_stream::{
    error::Error,
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Tag,
        Version,
    },
    stream::Stream,
};

// =================================================================================================
// Fixtures
// =================================================================================================

/// Creates a new temporary event stream, at a temporary path - the event stream
/// data will be automatically deleted on drop
pub(crate) fn stream() -> Result<Stream, Error> {
    Stream::builder(eventric_stream::temp_path())
        .temporary(true)
        .open()
}

/// Creates an [`EphemeralEvent`] given properties which can be converted to
/// strongly-typed event properties as required
#[rustfmt::skip]
pub(crate) fn event(data: &str, identifier: &str, tags: &[&str], version: u8) -> Result<EphemeralEvent, Error> {
    let data = Data::new(data)?;
    let identifier = Identifier::new(identifier)?;
    let tags = tags.iter().map(|tag| Tag::new(*tag)).collect::<Result<Vec<_>, _>>()?;
    let version = Version::new(version);
    
    Ok(EphemeralEvent::new(data, identifier, tags, version))
}

/// Creates a known collection of [`EphemeralEvent`] instances which can be used
/// to verify the various aspects of append/iterate functions
#[rustfmt::skip]
pub(crate) fn events() -> Result<Vec<EphemeralEvent>, Error> {
    Ok(Vec::from_iter([
        event("student:100-enrolled-course:200",    "student_enrolled",     &["student:100", "course:200"],     0)?,
        event("course:200-created",                 "course_created",       &["course:200"],                    0)?,
        event("student:101-enrolled-course:200",    "student_enrolled",     &["student:101", "course:200"],     1)?,
        event("course:200-updated",                 "course_updated",       &["course:200"],                    0)?,
        event("student:102-enrolled-course:201",    "student_enrolled",     &["student:102", "course:201"],     1)?,
        event("course:201-created",                 "course_created",       &["course:201"],                    1)?,
        event("student:100-dropped-course:200",     "student_dropped",      &["student:100", "course:200"],     0)?,
    ]))
}
