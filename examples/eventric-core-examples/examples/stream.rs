use std::error::Error;

use eventric_core::{
    event::{
        // Tag,
        insertion::Event,
    },
    query::{
        Query,
        QueryItem,
        Specifier,
    },
    stream::Stream,
};

static PATH: &str = "./temp/data/experiments";

pub fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut stream = Stream::new(PATH)?;

        stream.append(vec![
            Event::new("hello world!", ("StudentSubscribedToCourse", 0), vec![
                "student:3242".into(),
                "course:523".into(),
            ]),
            Event::new("oh, no!", ("CourseCapacityChanged", 0), vec![
                "course:523".into(),
            ]),
            Event::new("goodbye world...", ("StudentSubscribedToCourse", 1), vec![
                "student:7642".into(),
                "course:63".into(),
            ]),
        ])?;

        let student_or_course_query = Query::new([QueryItem::Specifiers(
            [
                Specifier::new("StudentSubscribedToCourse", None),
                Specifier::new("CourseCapacityChanged", None),
            ]
            .into(),
        )]);

        for id in stream.query(None, student_or_course_query) {
            println!("student or course id: {id}");
        }
    }

    // let context = Context::new(PATH)?;
    // let keyspaces = Keyspaces::new(
    //     data::configuration::keyspace(&context)?,
    //     index::configuration::keyspace(&context)?,
    //     reference::configuration::keyspace(&context)?,
    // );
    // let read = Read::new(&keyspaces);

    // let student_specifier = Specifier::new("StudentSubscribedToCourse",
    // None).into(); let student_stream =
    //     index::operation::descriptor::forward::iterate(&read, None,
    // &student_specifier);

    // let course_specifier = Specifier::new("CourseCapacityChanged", None).into();
    // let course_stream =
    //     index::operation::descriptor::forward::iterate(&read, None,
    // &course_specifier);

    // let student_or_course_stream =
    //     eventric_core_util::iter::or::sequential_or([student_stream,
    // course_stream]);

    // for id in student_or_course_stream {
    //     println!("event: {id}");
    // }

    // let course_tag = Tag::new("course:523").into();
    // let course_tag_stream = index::operation::tags::forward::iterate(&read, None,
    // &course_tag);

    // let student_tag = Tag::new("student:3242").into();
    // let student_tag_stream = index::operation::tags::forward::iterate(&read,
    // None, &student_tag);

    // let course_tag_and_student_tag_stream =
    //     eventric_core_util::iter::or::sequential_or([course_tag_stream,
    // student_tag_stream]);

    // for id in course_tag_and_student_tag_stream {
    //     println!("course or student tagged: {id}");
    // }

    Ok(())
}
