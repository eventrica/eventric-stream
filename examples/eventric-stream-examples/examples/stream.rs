use std::error::Error;

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
        iterate::IterateQuery as _,
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

    let (events, query_optimized) = stream.iterate_query(query, None);

    for event in events {
        println!("event: {event:#?}");
    }

    println!("query: {query_optimized:#?}");

    Ok(())
}
