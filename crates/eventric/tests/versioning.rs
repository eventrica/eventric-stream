//! A model event's stream [`Version`] is sourced directly from its `revision`
//! schema number — the two cannot diverge (there is no separate version to
//! declare or forget to bump).

use eventric::{
    event::Version,
    model::event::{
        Event,
        Events,
    },
};
use fancy_constructor::new;
use revision::revisioned;

#[revisioned(revision = 3)]
#[derive(new, Event, Debug)]
#[event(identifier(thing_happened), tags(thing(&this.id)))]
struct ThingHappened {
    #[new(into)]
    id: String,
}

// A revision-3 event must be buffered at stream `Version` 3 (not the old
// hardcoded 0, and not a separately-declared number).
#[test]
fn stream_version_is_sourced_from_revision() {
    let mut events = Events::new();
    events.append(&ThingHappened::new("x")).expect("append");

    let appended = events.take();

    assert_eq!(appended.len(), 1);
    assert_eq!(appended[0].facets().ty().version(), Version::new(3));
}
