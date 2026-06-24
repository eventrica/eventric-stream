use derive_more::Debug;
use error_stack::Report;
use eventric_domain::{
    action::{
        Act,
        Action,
    },
    error::Error,
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
#[action(
    projection(CourseExists: CourseExists::new(&this.id))
)]
pub struct DefineCourse {
    #[new(into)]
    pub id: String,
    pub capacity: u8,
}

impl Act for DefineCourse {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        if context.course_exists.exists {
            return Err(Report::new(Error).attach("Course Already Exists"));
        }

        context.append(&CourseDefined::new(&self.id, self.capacity))?;

        Ok(())
    }
}

#[derive(new, Action, Debug)]
#[action(
    projection(CourseExists: CourseExists::new(&this.id)),
    projection(CourseCapacity: CourseCapacity::new(&this.id))
)]
pub struct ChangeCourseCapacity {
    #[new(into)]
    id: String,
    new_capacity: u8,
}

impl Act for ChangeCourseCapacity {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        if !context.course_exists.exists {
            return Err(Report::new(Error).attach("Course Does Not Exist"));
        }

        if context.course_capacity.capacity == self.new_capacity {
            return Err(Report::new(Error).attach("Current Course Capacity Equals New Capacity"));
        }

        context.append(&CourseCapacityChanged::new(&self.id, self.new_capacity))?;

        Ok(())
    }
}

#[derive(new, Action, Debug)]
#[action(
    projection(CourseExists: CourseExists::new(&this.course_id)),
    projection(CourseCapacity: CourseCapacity::new(&this.course_id)),
    projection(NumberOfCourseSubscriptions: NumberOfCourseSubscriptions::new(&this.course_id)),
    projection(NumberOfStudentSubscriptions: NumberOfStudentSubscriptions::new(&this.student_id)),
    projection(StudentAlreadySubscribed: StudentAlreadySubscribed::new(&this.course_id, &this.student_id))
)]
pub struct SubscribeStudentToCourse {
    #[new(into)]
    course_id: String,
    #[new(into)]
    student_id: String,
}

impl Act for SubscribeStudentToCourse {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        if !context.course_exists.exists {
            return Err(Report::new(Error).attach("Course Does Not Exist"));
        }

        if context.number_of_course_subscriptions.count >= context.course_capacity.capacity {
            return Err(Report::new(Error).attach("Course Fully Booked"));
        }

        if context.student_already_subscribed.subscribed {
            return Err(Report::new(Error).attach("Student Already Subscribed"));
        }

        if context.number_of_student_subscriptions.count >= 5 {
            return Err(Report::new(Error).attach("Student Reached Course Limit"));
        }

        context.append(&StudentSubscribedToCourse::new(
            &self.course_id,
            &self.student_id,
        ))?;

        Ok(())
    }
}
