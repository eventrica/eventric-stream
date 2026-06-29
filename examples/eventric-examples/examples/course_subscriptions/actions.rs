use derive_more::Debug;
use error_stack::Report;
use eventric_model::{
    action::{
        Act,
        Action,
    },
    error::Error,
    event::Events,
};
use fancy_constructor::new;

use crate::{
    events::{
        CourseCapacityChanged,
        CourseDefined,
        StudentSubscribedToCourse,
    },
    projections::{
        CourseCapacity,
        CourseExists,
        NumberOfCourseSubscriptions,
        NumberOfStudentSubscriptions,
        StudentAlreadySubscribed,
    },
};

// =================================================================================================
// Course Subscriptions
// =================================================================================================

// Actions

#[derive(new, Action, Debug)]
#[action(projections: {
    course_exists: CourseExists::new(&self.id),
})]
pub struct DefineCourse {
    #[new(into)]
    pub id: String,
    pub capacity: u8,
}

impl Act<define_course::Projections> for DefineCourse {
    fn act(
        &self,
        events: &mut Events,
        projections: &define_course::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if projections.course_exists.exists {
            return Err(Report::new(Error).attach("Course Already Exists"));
        }

        events.append(&CourseDefined::new(&self.id, self.capacity))?;

        Ok(())
    }
}

#[derive(new, Action, Debug)]
#[action(projections: {
    course_exists: CourseExists::new(&self.id),
    course_capacity: CourseCapacity::new(&self.id),
})]
pub struct ChangeCourseCapacity {
    #[new(into)]
    id: String,
    new_capacity: u8,
}

impl Act<change_course_capacity::Projections> for ChangeCourseCapacity {
    fn act(
        &self,
        events: &mut Events,
        projections: &change_course_capacity::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if !projections.course_exists.exists {
            return Err(Report::new(Error).attach("Course Does Not Exist"));
        }

        if projections.course_capacity.capacity == self.new_capacity {
            return Err(Report::new(Error).attach("Current Course Capacity Equals New Capacity"));
        }

        events.append(&CourseCapacityChanged::new(&self.id, self.new_capacity))?;

        Ok(())
    }
}

#[derive(new, Action, Debug)]
#[action(projections: {
    course_exists: CourseExists::new(&self.course_id),
    course_capacity: CourseCapacity::new(&self.course_id),
    course_subscriptions: NumberOfCourseSubscriptions::new(&self.course_id),
    student_subscriptions: NumberOfStudentSubscriptions::new(&self.student_id),
    student_subscribed: StudentAlreadySubscribed::new(&self.course_id, &self.student_id),
})]
pub struct SubscribeStudentToCourse {
    #[new(into)]
    course_id: String,
    #[new(into)]
    student_id: String,
}

impl Act<subscribe_student_to_course::Projections> for SubscribeStudentToCourse {
    fn act(
        &self,
        events: &mut Events,
        projections: &subscribe_student_to_course::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if !projections.course_exists.exists {
            return Err(Report::new(Error).attach("Course Does Not Exist"));
        }

        if projections.course_subscriptions.count >= projections.course_capacity.capacity {
            return Err(Report::new(Error).attach("Course Fully Booked"));
        }

        if projections.student_subscribed.subscribed {
            return Err(Report::new(Error).attach("Student Already Subscribed"));
        }

        if projections.student_subscriptions.count >= 5 {
            return Err(Report::new(Error).attach("Student Reached Course Limit"));
        }

        events.append(&StudentSubscribedToCourse::new(
            &self.course_id,
            &self.student_id,
        ))?;

        Ok(())
    }
}
