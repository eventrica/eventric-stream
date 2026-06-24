mod actions;
mod events;
mod projections;

use error_stack::Report;
use eventric::{
    error::Error,
    model::enactor::Enactor as _,
    stream::Stream,
};

use crate::actions::{
    ChangeCourseCapacity,
    DefineCourse,
    SubscribeStudentToCourse,
};

// =================================================================================================
// Course Subscriptions
// =================================================================================================

// This example implements the Course Subscription example for Dynamic
// Consistency Boundaries as illustrated at [https://dcb.events/examples/course-subscriptions/].

// Example

pub fn main() -> Result<(), Report<Error>> {
    let mut stream = Stream::builder("./temp").open()?;

    let action = DefineCourse::new("cs:101", 30);
    let result = stream.enact(action);

    println!("Define Course Result: {result:?}");

    let action = ChangeCourseCapacity::new("eng_lit:200", 20);
    let result = stream.enact(action);

    println!("Change Course Capacity Result: {result:?}");

    let action = SubscribeStudentToCourse::new("cs:101", "andrew");
    let result = stream.enact(action);

    println!("Subscribe Student To Course Result: {result:?}");

    Ok(())
}
