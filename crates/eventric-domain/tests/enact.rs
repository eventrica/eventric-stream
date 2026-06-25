//! End-to-end integration tests for the model layer: defining `Event`s,
//! `Projection`s, and `Action`s, then running them through `Enactor::enact`
//! against a real (temporary) `eventric_stream::stream::Stream`.
//!
//! The fixture domain is a tiny item registry: an item can be registered
//! (with a quantity) and removed, each tagged by its SKU. Two distinct event
//! types (`item_registered` / `item_removed`) let us verify recognise-by-hash:
//! a projection must fold only the event types it selects, never one that
//! merely shares a tag.

use derive_more::Debug;
use error_stack::Report;
use eventric_domain::{
    action::{
        Act,
        Action,
    },
    enactor::Enactor as _,
    error::Error,
    event::{
        Event,
        Events,
        Identifier as _,
    },
    projection::{
        self,
        Project,
        Projection,
    },
};
use eventric_stream::{
    event::{
        Name,
        Tag,
    },
    stream::{
        Stream,
        operate::{
            Condition,
            Selection,
            select::{
                Select as _,
                Selector,
                TypeSelector,
            },
        },
    },
};
use fancy_constructor::new;
use revision::revisioned;

// =================================================================================================
// Fixture
// =================================================================================================

// Events

#[revisioned(revision = 1)]
#[derive(new, Event, Debug, PartialEq)]
#[event(identifier: item_registered, tags: { item: sku })]
struct ItemRegistered {
    #[new(into)]
    sku: String,
    qty: u8,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug, PartialEq)]
#[event(identifier: item_removed, tags: { item: sku })]
struct ItemRemoved {
    #[new(into)]
    sku: String,
}

// Projections

/// Folds both event types: whether the SKU is currently present in the
/// registry. Registering sets it present, removing clears it.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    present: { events: [ItemRegistered, ItemRemoved], filter: { item: sku } },
})]
struct ItemPresent {
    #[new(default)]
    present: bool,
    #[new(into)]
    sku: String,
}

impl Project<item_present::Present<'_>> for ItemPresent {
    fn project(&mut self, event: projection::Event<item_present::Present<'_>>) {
        self.present = match event.event() {
            item_present::Present::ItemRegistered(_) => true,
            item_present::Present::ItemRemoved(_) => false,
        };
    }
}

/// Folds ONLY `ItemRegistered` (filtered by SKU). `ItemRemoved` shares the same
/// `item(sku)` tag, so this projection is what proves recognise-by-hash: a
/// removal must never be counted here even though it matches the tag filter.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    registered: { events: [ItemRegistered], filter: { item: sku } },
})]
struct RegistrationCount {
    #[new(default)]
    count: u8,
    #[new(into)]
    sku: String,
}

impl Project<registration_count::Registered<'_>> for RegistrationCount {
    fn project(&mut self, _: projection::Event<registration_count::Registered<'_>>) {
        self.count += 1;
    }
}

// Actions

/// Register an item. Rejected if the SKU is already present.
#[derive(new, Action, Debug)]
#[action(projections: {
    item_present: ItemPresent::new(&self.sku),
})]
struct RegisterItem {
    #[new(into)]
    sku: String,
    qty: u8,
}

impl Act<register_item::Projections> for RegisterItem {
    fn act(
        &self,
        events: &mut Events,
        projections: &register_item::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if projections.item_present.present {
            return Err(Report::new(Error).attach("Item Already Registered"));
        }

        events.append(&ItemRegistered::new(&self.sku, self.qty))?;

        Ok(())
    }
}

/// Remove an item. Rejected if the SKU is not currently present.
#[derive(new, Action, Debug)]
#[action(projections: {
    item_present: ItemPresent::new(&self.sku),
})]
struct RemoveItem {
    #[new(into)]
    sku: String,
}

impl Act<remove_item::Projections> for RemoveItem {
    fn act(
        &self,
        events: &mut Events,
        projections: &remove_item::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if !projections.item_present.present {
            return Err(Report::new(Error).attach("Item Not Registered"));
        }

        events.append(&ItemRemoved::new(&self.sku))?;

        Ok(())
    }
}

/// A read-only action: appends nothing, returns the folded registration count
/// for a SKU. Used to assert projected state directly (`type Ok = u8`).
#[derive(new, Action, Debug)]
#[action(projections: {
    registration_count: RegistrationCount::new(&self.sku),
})]
struct CountRegistrations {
    #[new(into)]
    sku: String,
}

impl Act<count_registrations::Projections> for CountRegistrations {
    type Ok = u8;

    fn act(
        &self,
        _events: &mut Events,
        projections: &count_registrations::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        Ok(projections.registration_count.count)
    }
}

// A custom `Act::Err`, to exercise the indirection every other action leaves at
// its `Report<Error>` default. The `From<Report<Error>>` impl is what lets the
// `?` on `events.append(..)` propagate a replay/append failure through the
// custom type.
#[derive(Debug)]
struct Rejected(&'static str);

impl From<Report<Error>> for Rejected {
    fn from(_: Report<Error>) -> Self {
        Rejected("domain error")
    }
}

/// Like `RegisterItem`, but rejects with the custom `Rejected` error rather
/// than the default `Report<Error>`.
#[derive(new, Action, Debug)]
#[action(projections: {
    item_present: ItemPresent::new(&self.sku),
})]
struct RegisterItemStrict {
    #[new(into)]
    sku: String,
    qty: u8,
}

impl Act<register_item_strict::Projections> for RegisterItemStrict {
    type Err = Rejected;

    fn act(
        &self,
        events: &mut Events,
        projections: &register_item_strict::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if projections.item_present.present {
            return Err(Rejected("already registered"));
        }

        events.append(&ItemRegistered::new(&self.sku, self.qty))?;

        Ok(())
    }
}

// Helpers

fn stream() -> Stream {
    Stream::builder(eventric_stream::utils::temp_path())
        .temporary(true)
        .open()
        .expect("open temporary stream")
}

/// Count every event in the stream (a full scan: empty condition), asserting
/// each yielded result is `Ok`.
fn total_events(stream: &Stream) -> usize {
    stream
        .select(Condition::new())
        .map(|event| event.expect("scan event"))
        .collect::<Vec<_>>()
        .len()
}

/// Select persisted events of a single type, optionally filtered by an
/// `item(sku)` tag.
fn select_type(name: &str, sku: Option<&str>) -> Condition {
    let selector = match sku {
        Some(sku) => Selector::types_and_tags([TypeSelector::new(name).expect("type selector")], [
            Tag::new(format!("item:{sku}")).expect("tag"),
        ]),
        None => Selector::types([TypeSelector::new(name).expect("type selector")]),
    };

    Condition::new().selections([Selection::new([selector])])
}

// =================================================================================================
// Tests
// =================================================================================================

// 1. Happy path: enacting an action appends the expected event, observable via
//    a direct stream query (right position, type, tags, and decoded payload).
#[test]
fn enact_appends_expected_event() {
    let mut stream = stream();

    assert!(stream.is_empty());
    assert!(stream.enact(RegisterItem::new("widget", 7)).is_ok());

    // Exactly one event landed.
    assert_eq!(total_events(&stream), 1);

    let events = stream
        .select(select_type("item_registered", Some("widget")))
        .map(|event| event.expect("select event"))
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1);

    let event = &events[0].event;

    // Position: first append lands at the head of the stream.
    assert_eq!(
        event.meta().position(),
        eventric_stream::stream::Position::MIN
    );

    // Type name: stored as the hash of `item_registered`, matching the derive's
    // `type_name()`.
    assert_eq!(
        event.facets().ty().name(),
        &ItemRegistered::type_name().expect("type name")
    );

    // Tag: hash of `item:widget` is present.
    let expected_tag: Tag<u64> = Tag::new("item:widget").expect("tag").into();

    assert!(event.facets().tags().contains(&expected_tag));

    // Payload: round-trips back to the exact struct we registered.
    let decoded =
        revision::from_slice::<ItemRegistered>(event.data().as_ref()).expect("decode payload");

    assert_eq!(decoded, ItemRegistered::new("widget", 7));
}

// 2. Projection folding / replay: a later action's projection folds the events
//    appended by earlier actions, and its business rule depends on that folded
//    state. RemoveItem only succeeds because ItemPresent replayed the prior
//    registration as `present = true`.
#[test]
fn projection_folds_prior_events() {
    let mut stream = stream();

    // Removing before registering: ItemPresent folds nothing, present = false,
    // so the rule rejects and nothing is appended.
    assert!(stream.enact(RemoveItem::new("gadget")).is_err());
    assert_eq!(total_events(&stream), 0);

    // Register, then remove: the removal observes the replayed registration.
    assert!(stream.enact(RegisterItem::new("gadget", 3)).is_ok());
    assert!(stream.enact(RemoveItem::new("gadget")).is_ok());

    assert_eq!(total_events(&stream), 2);

    // The removal event landed at position 1, after the registration at 0.
    let removed = stream
        .select(select_type("item_removed", Some("gadget")))
        .map(|event| event.expect("select event"))
        .collect::<Vec<_>>();

    assert_eq!(removed.len(), 1);
    assert_eq!(
        removed[0].event.meta().position(),
        eventric_stream::stream::Position::MIN + 1
    );

    // Now that it's removed, ItemPresent folds register + remove to
    // present = false, so a fresh registration is allowed again.
    assert!(stream.enact(RegisterItem::new("gadget", 9)).is_ok());
    assert_eq!(total_events(&stream), 3);
}

// 3. Business-rule rejection: an action whose precondition fails on the
//    replayed state returns its `Err`, and no event is appended.
#[test]
fn business_rule_rejection_appends_nothing() {
    let mut stream = stream();

    assert!(stream.enact(RegisterItem::new("sprocket", 2)).is_ok());
    assert_eq!(total_events(&stream), 1);

    // Re-registering the same SKU: ItemPresent folds the prior registration to
    // present = true, so RegisterItem's rule rejects.
    let result = stream.enact(RegisterItem::new("sprocket", 5));

    // The error is the action's `Report<Error>`, carrying its rule message.
    let report = result.expect_err("duplicate registration must be rejected");

    assert!(
        report
            .frames()
            .any(|frame| frame.downcast_ref::<&str>() == Some(&"Item Already Registered"))
    );

    // No event was appended by the rejected action.
    assert_eq!(total_events(&stream), 1);

    // And the surviving event is still the original payload (qty 2, not 5).
    let events = stream
        .select(select_type("item_registered", Some("sprocket")))
        .map(|event| event.expect("select event"))
        .collect::<Vec<_>>();

    assert_eq!(events.len(), 1);

    let decoded =
        revision::from_slice::<ItemRegistered>(events[0].event.data().as_ref()).expect("decode");

    assert_eq!(decoded, ItemRegistered::new("sprocket", 2));
}

// 4. Recognise-by-hash: a projection folds only the event types it selects.
//    RegistrationCount selects only `item_registered`; `item_removed` shares
//    the `item(sku)` tag but a different type name, so it must NOT be counted.
#[test]
fn projection_recognizes_only_selected_types() {
    let mut stream = stream();

    // register, remove, register again — for the same SKU and tag.
    assert!(stream.enact(RegisterItem::new("bolt", 1)).is_ok());
    assert!(stream.enact(RemoveItem::new("bolt")).is_ok());
    assert!(stream.enact(RegisterItem::new("bolt", 1)).is_ok());

    assert_eq!(total_events(&stream), 3);

    // RegistrationCount must observe exactly 2 registrations: the interleaved
    // `item_removed` (same tag) is not one of its selected types, so it is not
    // folded in.
    let count = stream
        .enact(CountRegistrations::new("bolt"))
        .expect("count registrations");

    assert_eq!(count, 2);

    // Cross-check directly against the index: two `item_registered` and one
    // `item_removed` for this tag.
    assert_eq!(
        stream
            .select(select_type("item_registered", Some("bolt")))
            .count(),
        2
    );
    assert_eq!(
        stream
            .select(select_type("item_removed", Some("bolt")))
            .count(),
        1
    );
}

// The two event identifiers must hash to distinct `Name<u64>` values: the
// recognise-by-hash matching relies on no collision between type names.
#[test]
fn distinct_identifiers_do_not_collide() {
    let registered = ItemRegistered::type_name().expect("registered type name");
    let removed = ItemRemoved::type_name().expect("removed type name");

    assert_ne!(registered, removed);

    // And each equals the hash of its own identifier string.
    let from_str: Name<u64> = Name::new(ItemRegistered::identifier())
        .expect("name")
        .into();

    assert_eq!(registered, from_str);
}

// A custom `Act::Err` propagates through `enact` verbatim — the typed-error
// indirection every other action leaves at its `Report<Error>` default.
#[test]
fn enact_returns_custom_error() {
    let mut stream = stream();

    assert!(stream.enact(RegisterItemStrict::new("widget", 1)).is_ok());

    // The second registration is rejected, and `enact` hands back the action's own
    // `Rejected` error type — not `Report<Error>`.
    let err = stream
        .enact(RegisterItemStrict::new("widget", 1))
        .expect_err("second register rejected");

    assert_eq!(err.0, "already registered");
}
