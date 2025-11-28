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

/// Creates a new temporary test stream that will be automatically cleaned up
pub(crate) fn stream() -> Result<Stream, Error> {
    Stream::builder(eventric_stream::temp_path())
        .temporary(true)
        .open()
}

/// Creates a sample `EphemeralEvent` for testing
#[rustfmt::skip]
pub(crate) fn event(data: &str, identifier: &str, tags: &[&str], version: u8) -> Result<EphemeralEvent, Error> {
    let data = Data::new(data)?;
    let identifier = Identifier::new(identifier)?;
    let tags = tags.iter().map(|tag| Tag::new(*tag)).collect::<Result<Vec<_>, _>>()?;
    let version = Version::new(version);
    
    Ok(EphemeralEvent::new(data, identifier, tags, version))
}

/// Creates a diverse set of events for comprehensive testing scenarios.
///
/// This set includes:
/// - Multiple event types (student_enrolled, course_created, course_updated,
///   student_dropped)
/// - Various tag combinations (students, courses)
/// - Different versions
///
/// Total: 7 events
#[rustfmt::skip]
pub(crate) fn events() -> Result<Vec<EphemeralEvent>, Error> {
    Ok(Vec::from_iter([
        event("student:100-enrolled-course:200",    "student_enrolled",     &["student:100", "course:200"],     0)?,
        event("course:200-created",                 "course_created",       &["course:200"],                    0)?,
        event("student:101-enrolled-course:200",    "student_enrolled",     &["student:101", "course:200"],     0)?,
        event("course:200-updated",                 "course_updated",       &["course:200"],                    0)?,
        event("student:102-enrolled-course:201",    "student_enrolled",     &["student:102", "course:201"],     0)?,
        event("course:201-created",                 "course_created",       &["course:201"],                    0)?,
        event("student:100-dropped-course:200",     "student_dropped",      &["student:100", "course:200"],     0)?,
    ]))
}

/// Creates a smaller set of domain-specific events for append testing.
///
/// This set includes:
/// - StudentSubscribedToCourse events with different versions
/// - CourseCapacityChanged event
/// - Multiple student and course tags
///
/// Total: 3 events
pub(crate) fn create_domain_events() -> Result<[EphemeralEvent; 3], Error> {
    Ok([
        event(
            "student subscribed",
            "StudentSubscribedToCourse",
            &["student:100", "course:200"],
            0,
        )?,
        event(
            "capacity changed",
            "CourseCapacityChanged",
            &["course:200"],
            0,
        )?,
        event(
            "another student subscribed",
            "StudentSubscribedToCourse",
            &["student:101", "course:201"],
            1,
        )?,
    ])
}
