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

// The core `revision` evolution capability — otherwise untested in the repo:
// bytes written under an earlier revision decode into the current struct, with
// a later-added field filled by its `default_fn`.
#[revisioned(revision = 1)]
#[derive(new, Debug)]
struct AccountOpenedV1 {
    balance: u64,
}

#[revisioned(revision = 2)]
#[derive(new, Debug)]
struct AccountOpened {
    balance: u64,
    #[revision(start = 2, default_fn = "default_currency")]
    currency: String,
}

impl AccountOpened {
    // The `Result` is required by `revision`'s `default_fn` contract (the macro
    // `?`s it); it is not an unnecessary wrap.
    #[allow(clippy::unnecessary_wraps)]
    fn default_currency(_revision: u16) -> Result<String, revision::Error> {
        Ok("USD".to_owned())
    }
}

#[test]
fn revision_evolution_decodes_old_bytes_with_a_default() {
    let old = revision::to_vec(&AccountOpenedV1::new(100)).expect("serialise v1");

    let evolved: AccountOpened = revision::from_slice(&old).expect("decode v1 bytes as v2");

    assert_eq!(evolved.balance, 100);
    assert_eq!(evolved.currency, "USD"); // filled by default_fn for old bytes
}

// A revisioned event always serialises to a non-empty payload (at minimum the
// revision prefix), so even a field-light event clears `Data`'s non-empty check
// and appends — i.e. the "empty payload" edge does not arise in practice.
#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(identifier(ping))]
struct Ping {
    #[new(into)]
    source: String,
}

#[test]
fn minimal_event_serialises_non_empty_and_appends() {
    let mut events = Events::new();
    events
        .append(&Ping::new("node-a"))
        .expect("append minimal event");

    let appended = events.take();

    assert_eq!(appended.len(), 1);
    assert!(!appended[0].data().as_ref().is_empty());
}
