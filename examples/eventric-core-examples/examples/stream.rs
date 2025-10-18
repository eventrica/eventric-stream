use std::error::Error;

use eventric_core::{
    event::{
        Data,
        Descriptor,
        Event,
        Identifier,
        Tag,
        Version,
    },
    query::{
        Query,
        QueryItem,
        Specifier,
    },
    stream::Stream,
};

static PATH: &str = "./temp";

pub fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut stream = Stream::new(PATH, true)?;

        stream.append(Vec::from_iter([
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
        ]))?;

        let student_or_course_query = Query::new(Vec::from_iter([QueryItem::SpecifiersAndTags(
            Vec::from_iter([
                Specifier::new(Identifier::new("StudentSubscribedToCourse".into()), None),
                Specifier::new(Identifier::new("CourseCapacityChanged".into()), None),
            ]),
            Vec::from_iter([Tag::new("course:523".into())]),
        )]));

        for event in stream.query(None, &student_or_course_query) {
            println!("student or course id: {event:#?}");
        }
    }

    Ok(())
}
