use std::error::Error;

use eventric_core::{
    Data,
    Descriptor,
    Event,
    Identifier,
    Query,
    QueryCache,
    QueryCondition,
    QueryItem,
    Specifier,
    Stream,
    Tag,
    Version,
};

static PATH: &str = "./temp";

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = Stream::configure(PATH).temporary(true).open()?;

    stream.append(
        Vec::from_iter([
            &Event::new(
                Data::new("hello world!".bytes().collect()),
                Descriptor::new(
                    Identifier::new("StudentSubscribedToCourse".into()),
                    Version::new(0),
                ),
                Vec::from_iter([
                    Tag::new("student:3242".into()),
                    Tag::new("course:523".into()),
                ]),
            ),
            &Event::new(
                Data::new("oh, no!".bytes().collect()),
                Descriptor::new(
                    Identifier::new("CourseCapacityChanged".into()),
                    Version::new(0),
                ),
                Vec::from_iter([Tag::new("course:523".into())]),
            ),
            &Event::new(
                Data::new("goodbye world...".bytes().collect()),
                Descriptor::new(
                    Identifier::new("StudentSubscribedToCourse".into()),
                    Version::new(1),
                ),
                Vec::from_iter([
                    Tag::new("student:7642".into()),
                    Tag::new("course:63".into()),
                ]),
            ),
        ]),
        None,
    )?;

    let cache = QueryCache::default();
    let query = Query::new(Vec::from_iter([QueryItem::SpecifiersAndTags(
        Vec::from_iter([
            Specifier::new(Identifier::new("StudentSubscribedToCourse".into()), None),
            Specifier::new(Identifier::new("CourseCapacityChanged".into()), None),
        ]),
        Vec::from_iter([Tag::new("course:523".into())]),
    )]));

    let condition = QueryCondition::builder().query(&query).build();

    for event in stream.query(&cache, condition) {
        println!("student or course id: {event:#?}");
    }

    Ok(())
}
