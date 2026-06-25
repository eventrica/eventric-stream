use derive_more::Debug;
use eventric_domain::projection::{
    Projection,
    ProjectionEvent,
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

impl course_exists::Project for CourseExists {
    fn defined(&mut self, _: ProjectionEvent<course_exists::Defined<'_>>) {
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

impl course_capacity::Project for CourseCapacity {
    fn capacity(&mut self, event: ProjectionEvent<course_capacity::Capacity<'_>>) {
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

impl student_already_subscribed::Project for StudentAlreadySubscribed {
    fn subscribed(&mut self, _: ProjectionEvent<student_already_subscribed::Subscribed<'_>>) {
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

impl number_of_course_subscriptions::Project for NumberOfCourseSubscriptions {
    fn subscribed(&mut self, _: ProjectionEvent<number_of_course_subscriptions::Subscribed<'_>>) {
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

impl number_of_student_subscriptions::Project for NumberOfStudentSubscriptions {
    fn subscribed(&mut self, _: ProjectionEvent<number_of_student_subscriptions::Subscribed<'_>>) {
        self.count += 1;
    }
}
