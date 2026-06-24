use eventric_domain::event::Event;
use fancy_constructor::new;
use revision::revisioned;

// =================================================================================================
// Course Subscriptions: Events
// =================================================================================================

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: course_defined,
    tags: [course: id]
)]
pub struct CourseDefined {
    #[new(into)]
    pub id: String,
    pub capacity: u8,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: course_capacity_changed,
    tags: [course: id]
)]
pub struct CourseCapacityChanged {
    #[new(into)]
    pub id: String,
    pub new_capacity: u8,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: student_subscribed_to_course,
    tags: [
        course: course_id,
        student: student_id
    ]
)]
pub struct StudentSubscribedToCourse {
    #[new(into)]
    pub course_id: String,
    #[new(into)]
    pub student_id: String,
}
