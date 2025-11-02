use std::{
    error::Error,
    sync::Arc,
};

use eventric_core::{
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Position,
        Specifier,
        Tag,
        Version,
    },
    stream::{
        Stream,
        query::{
            Cache,
            Condition,
            Options,
            Query,
            Selector,
            Specifiers,
            Tags,
        },
    },
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    stream.append(
        [
            &EphemeralEvent::new(
                Data::new("hello world!")?,
                Identifier::new("StudentSubscribedToCourse")?,
                [Tag::new("student:3242")?, Tag::new("course:523")?],
                Version::new(0),
            ),
            &EphemeralEvent::new(
                Data::new("oh, no!")?,
                Identifier::new("CourseCapacityChanged")?,
                [Tag::new("course:523")?],
                Version::new(0),
            ),
            &EphemeralEvent::new(
                Data::new("goodbye world...")?,
                Identifier::new("StudentSubscribedToCourse")?,
                [Tag::new("student:7642")?, Tag::new("course:63")?],
                Version::new(1),
            ),
        ],
        None,
    )?;

    let query = Query::new([Selector::SpecifiersAndTags(
        Specifiers::new([
            Specifier::new(Identifier::new("StudentSubscribedToCourse")?),
            Specifier::new(Identifier::new("CourseCapacityChanged")?),
        ])?,
        Tags::new([Tag::new("course:523")?])?,
    )])?;

    let condition = Condition::default().matches(&query).from(Position::MIN);

    let cache = Arc::new(Cache::default());
    let options = Options::default()
        .retrieve_tags(false)
        .with_shared_cache(cache.clone());

    for event in stream.query(&condition, Some(options)) {
        println!("event: {event:#?}");
    }

    println!("cache: {cache:#?}");

    Ok(())
}
