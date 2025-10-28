use std::error::Error;

use eventric_core::{
    Data,
    Event,
    Identifier,
    Position,
    Query,
    QueryCache,
    QueryCondition,
    QueryItem,
    QueryOptions,
    Specifier,
    Specifiers,
    Stream,
    Tag,
    Tags,
    Version,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = Stream::builder(eventric_core::temp_path())
        .temporary(true)
        .open()?;

    stream.append(
        [
            &Event::new(
                Data::new("hello world!")?,
                Identifier::new("StudentSubscribedToCourse")?,
                Vec::from_iter([Tag::new("student:3242")?, Tag::new("course:523")?]),
                Version::new(0),
            ),
            &Event::new(
                Data::new("oh, no!")?,
                Identifier::new("CourseCapacityChanged")?,
                Vec::from_iter([Tag::new("course:523")?]),
                Version::new(0),
            ),
            &Event::new(
                Data::new("goodbye world...")?,
                Identifier::new("StudentSubscribedToCourse")?,
                Vec::from_iter([Tag::new("student:7642")?, Tag::new("course:63")?]),
                Version::new(1),
            ),
        ],
        None,
    )?;

    let query = Query::new([QueryItem::SpecifiersAndTags(
        Specifiers::new([
            Specifier::new(Identifier::new("StudentSubscribedToCourse")?, None),
            Specifier::new(Identifier::new("CourseCapacityChanged")?, None),
        ])?,
        Tags::new([Tag::new("course:523")?])?,
    )])?;

    let condition = QueryCondition::default()
        .matches(&query)
        .from(Position::MIN);

    let cache = QueryCache::default();
    let options = QueryOptions::default().retrieve_tags(false);

    for event in stream.query(&condition, &cache, Some(options)) {
        println!("event: {event:#?}");
    }

    println!("cache: {cache:#?}");

    Ok(())
}
