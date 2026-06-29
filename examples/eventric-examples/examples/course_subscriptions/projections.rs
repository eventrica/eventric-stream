use derive_more::Debug;
use eventric_model::projection::{
    Event,
    Project,
    Projection,
};
use fancy_constructor::new;

use crate::events::{
    CourseCapacityChanged,
    CourseDefined,
    StudentSubscribedToCourse,
};

// =================================================================================================
// Course Subscriptions: Projections
// =================================================================================================

// Projections

#[derive(new, Projection, Debug)]
#[projection(selections: {
    defined: {
        events: [CourseDefined],
        filter: { course: id }
    },
})]
pub struct CourseExists {
    #[new(default)]
    pub exists: bool,
    #[new(into)]
    pub id: String,
}

impl Project<course_exists::Defined<'_>> for CourseExists {
    fn project(&mut self, _: Event<course_exists::Defined<'_>>) {
        self.exists = true;
    }
}

#[derive(new, Projection, Debug)]
#[projection(selections: {
    capacity: {
        events: [
            CourseDefined,
            CourseCapacityChanged
        ],
        filter: { course: id }
    },
})]
pub struct CourseCapacity {
    #[new(default)]
    pub capacity: u8,
    #[new(into)]
    pub id: String,
}

impl Project<course_capacity::Capacity<'_>> for CourseCapacity {
    fn project(&mut self, event: Event<course_capacity::Capacity<'_>>) {
        match event.event() {
            course_capacity::Capacity::CourseDefined(event) => self.capacity = event.capacity,
            course_capacity::Capacity::CourseCapacityChanged(event) => {
                self.capacity = event.new_capacity;
            }
        }
    }
}

#[derive(new, Projection, Debug)]
#[projection(selections: {
    subscribed: {
        events: [StudentSubscribedToCourse],
        filter: {
            course: course_id,
            student: student_id
        }
    },
})]
pub struct StudentAlreadySubscribed {
    #[new(default)]
    pub subscribed: bool,
    #[new(into)]
    pub course_id: String,
    #[new(into)]
    pub student_id: String,
}

impl Project<student_already_subscribed::Subscribed<'_>> for StudentAlreadySubscribed {
    fn project(&mut self, _: Event<student_already_subscribed::Subscribed<'_>>) {
        self.subscribed = true;
    }
}

#[derive(new, Projection, Debug)]
#[projection(selections: {
    subscribed: {
        events: [StudentSubscribedToCourse],
        filter: { course: course_id }
    },
})]
pub struct NumberOfCourseSubscriptions {
    #[new(into)]
    pub course_id: String,
    #[new(default)]
    pub count: u8,
}

impl Project<number_of_course_subscriptions::Subscribed<'_>> for NumberOfCourseSubscriptions {
    fn project(&mut self, _: Event<number_of_course_subscriptions::Subscribed<'_>>) {
        self.count += 1;
    }
}

#[derive(new, Projection, Debug)]
#[projection(selections: {
    subscribed: {
        events: [StudentSubscribedToCourse],
        filter: { student: student_id }
    },
})]
pub struct NumberOfStudentSubscriptions {
    #[new(into)]
    pub student_id: String,
    #[new(default)]
    pub count: u8,
}

impl Project<number_of_student_subscriptions::Subscribed<'_>> for NumberOfStudentSubscriptions {
    fn project(&mut self, _: Event<number_of_student_subscriptions::Subscribed<'_>>) {
        self.count += 1;
    }
}
