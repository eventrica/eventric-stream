//! Integration tests for the multi-thread `Owner`/`Proxy` wrapper: concurrent
//! appends serialise through the single writer thread (no gaps/dupes), reads
//! observe committed writes, and a rejected (DCB) append carries its `Conflict`
//! marker back across the channel.

use std::{
    collections::BTreeSet,
    thread,
};

use error_stack::Report;
use eventric_stream_core::{
    error::Conflict,
    event::{
        Data,
        Event,
        Facets,
        Name,
        Tag,
        Type,
        Version,
    },
    stream::{
        Append as _,
        Condition,
        Position,
        Select as _,
        Selection,
        Selector,
        Stream,
        TypeSelector,
    },
    utils::temp_path,
};
use eventric_stream_multi_thread::owner::Owner;

// =================================================================================================
// Helpers
// =================================================================================================

fn owner() -> Owner {
    Owner::new(Stream::builder(temp_path()).temporary(true).open().unwrap())
}

fn event(name: &str, data: &str, tags: &[&str]) -> Event<(), String> {
    let ty = Type::new(Name::new(name).unwrap(), Version::new(0));
    let tags = tags
        .iter()
        .map(|tag| Tag::new(*tag).unwrap())
        .collect::<BTreeSet<_>>();

    Event::new(Data::new(data).unwrap(), Facets::new(ty, tags), ())
}

// =================================================================================================
// Tests
// =================================================================================================

// 1. Concurrent unconditional appends serialise through the single writer: N
//    threads each append M events; the result must be exactly N*M events at
//    contiguous positions 0..N*M with no gaps and no duplicates.
#[test]
fn concurrent_appends_serialise_to_contiguous_positions() {
    const THREADS: u64 = 8;
    const PER_THREAD: u64 = 25;

    let owner = owner();

    let handles = (0..THREADS)
        .map(|t| {
            let mut proxy = owner.proxy();

            thread::spawn(move || {
                for e in 0..PER_THREAD {
                    let data = format!("t{t}-e{e}");
                    proxy
                        .append([event("Appended", &data, &["worker"])], Condition::new())
                        .unwrap();
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    // Read every event back through one proxy and collect their positions.
    let proxy = owner.proxy();
    let positions = proxy
        .select(Condition::new())
        .map(|result| result.unwrap().event.meta().position())
        .collect::<Vec<_>>();

    let total = THREADS * PER_THREAD;

    // Count: exactly N*M events committed.
    assert_eq!(positions.len() as u64, total, "every append must commit");

    // Contiguity + uniqueness: the position set is exactly {0, 1, ..., N*M-1}.
    let unique = positions.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        unique.len() as u64,
        total,
        "positions must be unique (no duplicate writes)"
    );

    let expected = (0..total)
        .map(|p| Position::MIN + p)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        unique, expected,
        "positions must be a contiguous 0..N*M set (no gaps)"
    );

    // A full scan yields events in ascending position order.
    let mut sorted = positions.clone();
    sorted.sort();
    assert_eq!(positions, sorted, "scan order must be ascending position");
}

// 2. A read on a fresh proxy observes a write committed through another proxy.
#[test]
fn reads_see_committed_writes() {
    let owner = owner();

    let mut writer = owner.proxy();
    let position = writer
        .append(
            [event("StudentSubscribedToCourse", "hi", &["student:1"])],
            Condition::new(),
        )
        .unwrap();

    // The single appended event lands at the head position.
    assert_eq!(position, Position::MIN);

    // A *different* proxy clone observes it.
    let reader = owner.proxy();
    let events = reader
        .select(Condition::new())
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1, "the committed event must be visible");
    assert_eq!(events[0].event.meta().position(), Position::MIN);
    assert_eq!(events[0].event.data().as_ref(), b"hi");
}

// 3. A conditional (DCB) append that conflicts with an already-committed
//    matching event is rejected, and the rejection's `Report<Error>` carries
//    the `Conflict` marker intact across the writer-thread channel.
#[test]
fn conflicting_append_is_rejected_with_conflict_marker() {
    let owner = owner();

    let mut a = owner.proxy();
    let mut b = owner.proxy();

    // Proxy A unconditionally commits a matching event at position 0.
    let position = a
        .append(
            [event("CourseCapacityChanged", "full", &["course:42"])],
            Condition::new(),
        )
        .unwrap();
    assert_eq!(position, Position::MIN);

    // The selection B guards against: any "CourseCapacityChanged" event.
    let matching = || {
        Selection::new([Selector::types([TypeSelector::new(
            "CourseCapacityChanged",
        )
        .unwrap()])])
    };

    // Proxy B appends conditionally from the head (a position before A's
    // event). The DCB check finds A's matching event in [0..) and must reject.
    let rejected = b.append(
        [event("CourseCapacityChanged", "again", &["course:42"])],
        Condition::new()
            .from(Position::MIN)
            .selections([matching()]),
    );

    let report: Report<_> = rejected.expect_err("conflicting append must be rejected");
    assert!(
        report.downcast_ref::<Conflict>().is_some(),
        "rejection must carry the Conflict marker across the channel"
    );

    // The rejected append wrote nothing: only A's single event exists, so the
    // cursor did not advance.
    let count = owner.proxy().select(Condition::new()).count();
    assert_eq!(count, 1, "a rejected append must not commit any events");

    // A non-conflicting conditional append (guarding an unrelated type, from
    // just past A's event) still succeeds, landing at position 1.
    let next = b
        .append(
            [event("CourseCapacityChanged", "ok", &["course:99"])],
            Condition::new()
                .from(position + 1)
                .selections([Selection::new([Selector::types([TypeSelector::new(
                    "SomethingElse",
                )
                .unwrap()])])]),
        )
        .expect("non-conflicting append must succeed");
    assert_eq!(next, Position::MIN + 1);
}
