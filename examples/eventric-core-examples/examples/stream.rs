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
    Stream,
    Tag,
    Version,
};

static PATH: &str = "./temp";

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = Stream::builder(PATH).temporary(true).open()?;

    stream.append(
        [
            &Event::new(
                Data::new("hello world!".bytes().collect()),
                Identifier::new("StudentSubscribedToCourse".into()),
                Vec::from_iter([
                    Tag::new("student:3242".into()),
                    Tag::new("course:523".into()),
                ]),
                Version::new(0),
            ),
            &Event::new(
                Data::new("oh, no!".bytes().collect()),
                Identifier::new("CourseCapacityChanged".into()),
                Vec::from_iter([Tag::new("course:523".into())]),
                Version::new(0),
            ),
            &Event::new(
                Data::new("goodbye world...".bytes().collect()),
                Identifier::new("StudentSubscribedToCourse".into()),
                Vec::from_iter([
                    Tag::new("student:7642".into()),
                    Tag::new("course:63".into()),
                ]),
                Version::new(1),
            ),
        ],
        None,
    )?;

    let query = Query::new(Vec::from_iter([QueryItem::SpecifiersAndTags(
        Vec::from_iter([
            Specifier::new(Identifier::new("StudentSubscribedToCourse".into()), None),
            Specifier::new(Identifier::new("CourseCapacityChanged".into()), None),
        ]),
        Vec::from_iter([Tag::new("course:523".into())]),
    )]));

    let condition = QueryCondition::default()
        .query(&query)
        .position(Position::new(0));

    let cache = QueryCache::default();
    let options = QueryOptions::default().retrieve_tags(true);

    for event in stream.query(&condition, &cache, Some(options)) {
        println!("event: {event:#?}");
    }

    println!("cache: {cache:#?}");

    Ok(())
}
