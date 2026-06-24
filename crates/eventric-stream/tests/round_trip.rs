//! Integration tests exercising the public `eventric-stream` facade end to end:
//! appending candidate events through the [`Append`] trait and reading them
//! back through [`Select`], the masked multi-selection query path,
//! version-range selection, the DCB (position-based) append concurrency check,
//! and the threaded [`Owner`]/[`Proxy`] round-trip.

use std::collections::BTreeSet;

use eventric_stream::{
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
        Position,
        Stream,
        concurrent::owner::Owner,
        operate::{
            Condition,
            Selection,
            append::Append as _,
            select::{
                Select as _,
                Selector,
                TypeSelector,
            },
        },
    },
    utils::temp_path,
};

// A small helper mirroring the `stream` example: build a candidate event from
// string parts.
fn event(identifier: &str, data: &str, tags: &[&str], version: u8) -> Event<(), String> {
    let ty = Type::new(Name::new(identifier).unwrap(), Version::new(version));
    let tags = tags
        .iter()
        .map(|tag| Tag::new(*tag).unwrap())
        .collect::<BTreeSet<_>>();

    Event::new(Data::new(data).unwrap(), Facets::new(ty, tags), ())
}

fn open() -> Stream {
    Stream::builder(temp_path()).temporary(true).open().unwrap()
}

// 1. Append a batch, full-scan it back, and assert the exact count and that
//    positions are a contiguous 0..N ascending run.
#[test]
fn append_then_full_scan_yields_contiguous_positions() {
    let mut stream = open();

    let last = stream
        .append(
            vec![
                event("StudentSubscribedToCourse", "a", &["student:1"], 0),
                event("CourseCapacityChanged", "b", &["course:1"], 0),
                event("StudentSubscribedToCourse", "c", &["student:2"], 0),
            ],
            Condition::new(),
        )
        .unwrap();

    // The append returns the position of the *last* appended event.
    assert_eq!(last, Position::new(2));

    let events = stream
        .select(Condition::new())
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 3);

    // Positions are exactly 0, 1, 2 in iteration order.
    let positions = events
        .iter()
        .map(|em| em.event.meta().position())
        .collect::<Vec<_>>();

    assert_eq!(positions, vec![
        Position::new(0),
        Position::new(1),
        Position::new(2)
    ]);

    // The payloads come back in append order too.
    let datas = events
        .iter()
        .map(|em| em.event.data().clone())
        .collect::<Vec<_>>();

    assert_eq!(datas, vec![
        Data::new("a").unwrap(),
        Data::new("b").unwrap(),
        Data::new("c").unwrap(),
    ]);
}

// 2. A two-selection masked query: each returned event's mask records, in
//    order, which selections it satisfied, and only events matching at least
//    one selection are returned.
#[test]
fn masked_multi_selection_query_sets_correct_bits() {
    let mut stream = open();

    stream
        .append(
            vec![
                // pos 0: subscribed + course:523  -> matches selection 0 only
                event(
                    "StudentSubscribedToCourse",
                    "sub-523",
                    &["student:1", "course:523"],
                    0,
                ),
                // pos 1: capacity + course:523     -> matches selection 1 only
                event("CourseCapacityChanged", "cap-523", &["course:523"], 0),
                // pos 2: subscribed, no course:523 -> matches NEITHER (dropped)
                event(
                    "StudentSubscribedToCourse",
                    "sub-999",
                    &["student:2", "course:999"],
                    0,
                ),
                // pos 3: capacity, no course:523   -> matches NEITHER (dropped)
                event("CourseCapacityChanged", "cap-999", &["course:999"], 0),
            ],
            Condition::new(),
        )
        .unwrap();

    // Selection 0: StudentSubscribedToCourse AND course:523.
    // Selection 1: CourseCapacityChanged    AND course:523.
    let condition = Condition::new().selections([
        Selection::new([Selector::types_and_tags(
            [TypeSelector::new("StudentSubscribedToCourse").unwrap()],
            [Tag::new("course:523").unwrap()],
        )]),
        Selection::new([Selector::types_and_tags(
            [TypeSelector::new("CourseCapacityChanged").unwrap()],
            [Tag::new("course:523").unwrap()],
        )]),
    ]);

    let events = stream
        .select(condition)
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    // Only the two course:523 events come back; the course:999 ones are dropped.
    assert_eq!(events.len(), 2);

    // pos 0 matches selection 0 only.
    assert_eq!(events[0].event.meta().position(), Position::new(0));
    assert_eq!(events[0].mask.as_ref(), &[true, false]);

    // pos 1 matches selection 1 only.
    assert_eq!(events[1].event.meta().position(), Position::new(1));
    assert_eq!(events[1].mask.as_ref(), &[false, true]);
}

// A single event can satisfy more than one selection at once: both bits set.
#[test]
fn masked_query_event_satisfies_multiple_selections() {
    let mut stream = open();

    stream
        .append(
            vec![event(
                "StudentSubscribedToCourse",
                "x",
                &["student:1", "course:523"],
                0,
            )],
            Condition::new(),
        )
        .unwrap();

    // Selection 0 keys off the type, selection 1 keys off the tag; the single
    // event satisfies both.
    let condition = Condition::new().selections([
        Selection::new([Selector::types([TypeSelector::new(
            "StudentSubscribedToCourse",
        )
        .unwrap()])]),
        Selection::new([Selector::types_and_tags(
            [TypeSelector::new("StudentSubscribedToCourse").unwrap()],
            [Tag::new("course:523").unwrap()],
        )]),
    ]);

    let events = stream
        .select(condition)
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].mask.as_ref(), &[true, true]);
}

// 3. A half-open version range `0..2` returns versions 0 and 1 but excludes 2.
#[test]
fn version_range_selection_is_half_open() {
    let mut stream = open();

    stream
        .append(
            vec![
                event("Versioned", "v0", &["t:1"], 0),
                event("Versioned", "v1", &["t:1"], 1),
                event("Versioned", "v2", &["t:1"], 2),
            ],
            Condition::new(),
        )
        .unwrap();

    let condition = Condition::new().selections([Selection::new([Selector::types([
        TypeSelector::with_versions("Versioned", Version::new(0)..Version::new(2)).unwrap(),
    ])])]);

    let versions = stream
        .select(condition)
        .map(Result::unwrap)
        .map(|em| em.event.facets().ty().version())
        .collect::<Vec<_>>();

    // 0 and 1 included, 2 excluded (half-open upper bound).
    assert_eq!(versions, vec![Version::new(0), Version::new(1)]);
    assert!(!versions.contains(&Version::new(2)));
}

// 4. The DCB concurrency guard: an append conditioned on a window that already
//    holds a matching event is rejected with a `Conflict`; an append whose
//    `from` is past the head succeeds.
#[test]
fn dcb_conflict_on_matching_window_and_success_past_head() {
    let mut stream = open();

    // Seed three "Counter"/"k:1" events at positions 0, 1, 2.
    let last = stream
        .append(
            vec![
                event("Counter", "0", &["k:1"], 0),
                event("Counter", "1", &["k:1"], 0),
                event("Counter", "2", &["k:1"], 0),
            ],
            Condition::new(),
        )
        .unwrap();

    assert_eq!(last, Position::new(2));

    let selection = || {
        Selection::new([Selector::types_and_tags(
            [TypeSelector::new("Counter").unwrap()],
            [Tag::new("k:1").unwrap()],
        )])
    };

    // Conditioned from position 0: a matching event exists at >= 0 -> conflict.
    let report = stream
        .append(
            vec![event("Counter", "3", &["k:1"], 0)],
            Condition::new()
                .from(Position::new(0))
                .selections([selection()]),
        )
        .expect_err("a matching event in the window must reject the append");

    assert!(
        report.downcast_ref::<Conflict>().is_some(),
        "rejection must carry the Conflict marker"
    );

    // The rejected append must not have advanced the stream: still 3 events.
    assert_eq!(
        stream.select(Condition::new()).count(),
        3,
        "a rejected append must not advance the position cursor"
    );

    // Conditioned from the head (position 3 == next): the window is empty, so
    // there can be no conflict and the append succeeds.
    let appended = stream
        .append(
            vec![event("Counter", "3", &["k:1"], 0)],
            Condition::new()
                .from(Position::new(3))
                .selections([selection()]),
        )
        .expect("an append whose window starts past the head cannot conflict");

    assert_eq!(appended, Position::new(3));
    assert_eq!(stream.select(Condition::new()).count(), 4);

    // A non-matching selection never conflicts even within a populated window.
    let appended = stream
        .append(
            vec![event("Counter", "4", &["k:1"], 0)],
            Condition::new()
                .from(Position::new(0))
                .selections([Selection::new([Selector::types([TypeSelector::new(
                    "NoSuchType",
                )
                .unwrap()])])]),
        )
        .expect("a selection that matches nothing must not conflict");

    assert_eq!(appended, Position::new(4));
}

// 5. The threaded Owner/Proxy path round-trips appended events back through a
//    select.
#[test]
fn owner_proxy_round_trip() {
    let owner = Owner::new(open());
    let mut proxy = owner.proxy();

    let last = proxy
        .append(
            vec![
                event("StudentSubscribedToCourse", "hello", &["student:3242"], 0),
                event("CourseCapacityChanged", "world", &["course:523"], 0),
            ],
            Condition::new(),
        )
        .unwrap();

    assert_eq!(last, Position::new(1));

    let events = proxy
        .select(Condition::new())
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 2);

    let positions = events
        .iter()
        .map(|em| em.event.meta().position())
        .collect::<Vec<_>>();

    assert_eq!(positions, vec![Position::new(0), Position::new(1)]);

    let datas = events
        .iter()
        .map(|em| em.event.data().clone())
        .collect::<Vec<_>>();

    assert_eq!(datas, vec![
        Data::new("hello").unwrap(),
        Data::new("world").unwrap()
    ]);
}
