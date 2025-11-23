use std::{
    error::Error,
    sync::Arc,
};

use eventric_stream::{
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Specifier,
        Tag,
        Version,
    },
    stream::{
        Stream,
        append::Append as _,
        iterate::{
            Cache,
            IterateQuery as _,
            Options,
        },
        query::{
            Query,
            Selector,
            Specifiers,
            Tags,
        },
    },
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = Stream::builder(eventric_stream::temp_path())
        .temporary(true)
        .open()?;

    stream.append(
        [
            EphemeralEvent::new(
                Data::new("hello world!")?,
                Identifier::new("StudentSubscribedToCourse")?,
                [Tag::new("student:3242")?, Tag::new("course:523")?],
                Version::new(0),
            ),
            EphemeralEvent::new(
                Data::new("oh, no!")?,
                Identifier::new("CourseCapacityChanged")?,
                [Tag::new("course:523")?],
                Version::new(0),
            ),
            EphemeralEvent::new(
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

    let cache = Arc::new(Cache::default());
    let options = Options::default()
        .retrieve_tags(false)
        .with_shared_cache(cache.clone());

    let (events, _) = stream.iterate_query_with_options(query, None, options);

    for event in events {
        println!("event: {event:#?}");
    }

    println!("cache: {cache:#?}");

    Ok(())
}
