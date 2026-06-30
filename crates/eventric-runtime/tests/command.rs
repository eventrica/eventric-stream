//! End-to-end test of the command-issuing reaction (Phase B): a reaction reacts
//! to an `EnrolmentRequested` event and *issues a `ConfirmEnrolment` command*;
//! the reactor's `drive` routes that command to its action (`From<Command>`)
//! and enacts it via the `Enactor`. The action reads a capacity projection and
//! appends an `EnrolmentConfirmed` only while the course is under capacity —
//! closing the event→reaction→command→action→event loop in one process, with a
//! real projection-based decision inside the action.

use eventric_model::{
    action::{
        Act,
        Action,
        Command,
    },
    event::{
        Event,
        Events,
        Specifier as _,
    },
    projection::{
        self,
        Project,
        Projection,
    },
    reaction::{
        Effects,
        React,
    },
};
use eventric_runtime::reactor::Reactor;
use eventric_stream::stream::{
    Stream,
    operate::{
        Condition,
        Selection,
        append::Append as _,
        select::{
            Select as _,
            Selector,
        },
    },
};
use fancy_constructor::new;
use revision::revisioned;

// A course with capacity one — so the second request for the same course is
// turned away by the action's capacity check.
const CAPACITY: u64 = 1;

#[revisioned(revision = 1)]
#[derive(Event)]
#[event(identifier: enrolment_requested, tags: { course: course })]
struct EnrolmentRequested {
    course: String,
    student: String,
}

#[revisioned(revision = 1)]
#[derive(Event)]
#[event(identifier: enrolment_confirmed, tags: { course: course })]
struct EnrolmentConfirmed {
    course: String,
    student: String,
}

// A read-model the action folds: how many confirmations a course already has.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    confirmed: { events: [EnrolmentConfirmed], filter: { course } },
})]
struct CourseEnrolments {
    #[new(into)]
    course: String,
    #[new(default)]
    count: u64,
}

impl Project<course_enrolments::Confirmed<'_>> for CourseEnrolments {
    fn project(&mut self, _event: projection::Event<course_enrolments::Confirmed<'_>>) {
        self.count += 1;
    }
}

// The command — a plain message, distinct from the action that handles it.
struct ConfirmEnrolment {
    course: String,
    student: String,
}

impl Command for ConfirmEnrolment {
    type Action = ConfirmEnrolmentAction;
}

// The action handling the command, built from it via `From` (command ≠ action,
// joined by `From<Command>`). It folds the course's enrolments and confirms
// only while under capacity.
#[derive(Action)]
#[action(projections: {
    enrolments: CourseEnrolments::new(&self.course),
})]
struct ConfirmEnrolmentAction {
    course: String,
    student: String,
}

impl From<ConfirmEnrolment> for ConfirmEnrolmentAction {
    fn from(command: ConfirmEnrolment) -> Self {
        Self {
            course: command.course,
            student: command.student,
        }
    }
}

impl Act<confirm_enrolment_action::Projections> for ConfirmEnrolmentAction {
    fn act(
        &self,
        events: &mut Events,
        projections: &confirm_enrolment_action::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if projections.enrolments.count < CAPACITY {
            events.append(&EnrolmentConfirmed {
                course: self.course.clone(),
                student: self.student.clone(),
            })?;
        }

        Ok(())
    }
}

// The reaction: on a request, issue a confirm command. Maintains no view.
struct ConfirmOnRequest {
    course: String,
    student: String,
}

impl From<EnrolmentRequested> for ConfirmOnRequest {
    fn from(event: EnrolmentRequested) -> Self {
        Self {
            course: event.course,
            student: event.student,
        }
    }
}

impl React for ConfirmOnRequest {
    type Command = ConfirmEnrolment;
    type Event = EnrolmentRequested;

    fn react(&self, effects: &mut Effects<Self::View, Self::Command>) {
        effects.issue_command(ConfirmEnrolment {
            course: self.course.clone(),
            student: self.student.clone(),
        });
    }
}

fn request(course: &str, student: &str) -> EnrolmentRequested {
    EnrolmentRequested {
        course: course.to_owned(),
        student: student.to_owned(),
    }
}

#[test]
fn reactor_drive_routes_commands_through_actions() {
    let mut stream = Stream::builder(eventric_stream::utils::temp_path())
        .temporary(true)
        .open()
        .unwrap();

    // Two requests for the same (capacity-one) course.
    let mut buffer = Events::new();
    buffer.append(&request("rust", "ana")).unwrap();
    buffer.append(&request("rust", "ben")).unwrap();
    stream.append(buffer.take(), Condition::new()).unwrap();

    // Drive: each request issues a ConfirmEnrolment command, routed to
    // ConfirmEnrolmentAction and enacted. The first confirms; the second, folding
    // the now-present confirmation, sees the course at capacity and appends
    // nothing.
    let mut reactor = Reactor::<ConfirmOnRequest>::new();
    reactor.drive(&mut stream).unwrap();

    // The loop closed, and the action's capacity decision held: exactly one
    // confirmation exists.
    let confirmed = Selection::new([Selector::types([EnrolmentConfirmed::specifier().unwrap()])]);
    let count = stream
        .select(Condition::new().selections([confirmed]))
        .count();

    assert_eq!(count, 1);
}
