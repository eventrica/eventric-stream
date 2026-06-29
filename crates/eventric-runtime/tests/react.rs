//! End-to-end test of the reaction slice (Phase A): a view-maintaining reaction
//! folded over a real stream by the `Reactor`. A `Subscribed` event per
//! enrolment; the reaction keeps a per-course count. Exercises the
//! `From<Event>`/`react` shape, the `MaintainView` effect, the reactor's
//! tail-and-fold, the checkpoint (resume), and replay-from-zero idempotence.

use std::collections::BTreeMap;

use eventric_model::{
    event::{
        Event,
        Events,
    },
    reaction::{
        Effects,
        React,
        View,
    },
};
use eventric_runtime::reactor::Reactor;
use eventric_stream::{
    stream::{
        Stream,
        operate::{
            Condition,
            append::Append,
        },
    },
    utils::temp_path,
};
use revision::revisioned;

// The fixture event: a student subscribes to a course.
#[revisioned(revision = 1)]
#[derive(Event)]
#[event(identifier: subscribed, tags: { course: course })]
struct Subscribed {
    course: String,
    student: String,
}

// The view: a per-course enrolment count, maintained by "+1 for this course"
// deltas.
#[derive(Default)]
struct CourseCounts(BTreeMap<String, u64>);

impl View for CourseCounts {
    type Delta = String;

    fn apply(&mut self, course: String) {
        *self.0.entry(course).or_default() += 1;
    }
}

// The reaction: built from a `Subscribed`, stages a count-this-course delta.
struct CountSubscription {
    course: String,
}

impl From<Subscribed> for CountSubscription {
    fn from(event: Subscribed) -> Self {
        Self {
            course: event.course,
        }
    }
}

impl React for CountSubscription {
    type Event = Subscribed;
    type View = CourseCounts;

    fn react(&self, effects: &mut Effects<Self::View>) {
        effects.maintain_view(self.course.clone());
    }
}

fn append(writer: &mut impl Append, events: &[Subscribed]) {
    let mut buffer = Events::new();
    for event in events {
        buffer.append(event).unwrap();
    }
    writer.append(buffer.take(), Condition::new()).unwrap();
}

fn subscribed(course: &str, student: &str) -> Subscribed {
    Subscribed {
        course: course.to_owned(),
        student: student.to_owned(),
    }
}

#[test]
fn reactor_folds_events_into_the_view() {
    let stream = Stream::builder(temp_path()).temporary(true).open().unwrap();
    let (reader, mut writer) = stream.split();

    append(&mut writer, &[
        subscribed("rust", "ana"),
        subscribed("rust", "ben"),
        subscribed("go", "cleo"),
    ]);

    let mut reactor = Reactor::<CountSubscription>::new();
    reactor.run(&reader).unwrap();

    assert_eq!(reactor.view().0.get("rust"), Some(&2));
    assert_eq!(reactor.view().0.get("go"), Some(&1));

    // The checkpoint resumes: appending more and re-running folds in only the
    // new event, not the ones already seen.
    append(&mut writer, &[subscribed("rust", "dan")]);
    reactor.run(&reader).unwrap();

    assert_eq!(reactor.view().0.get("rust"), Some(&3));
    assert_eq!(reactor.view().0.get("go"), Some(&1));

    // Replay from zero rebuilds the same view (the fold is idempotent under
    // replay).
    let mut fresh = Reactor::<CountSubscription>::new();
    fresh.run(&reader).unwrap();

    assert_eq!(fresh.view().0.get("rust"), Some(&3));
    assert_eq!(fresh.view().0.get("go"), Some(&1));
}
