use eventric_domain::event::{
    Event,
    Tags as _,
};
use eventric_stream::event::Tag;
use fancy_constructor::new;
use revision::revisioned;

// =================================================================================================
// Tag value forms
// =================================================================================================

// One event exercising all three `#[event(tags: ..)]` value forms end-to-end:
//
//   - bare ident  -> the field, `&self.sku`
//   - expression  -> a plain expression with the event in scope as `self` (the
//     `&self` tag method); here `&self.owner`
//   - closure     -> names the receiver (`|e| ..`) and desugars to `{ let e =
//     self; <body> }`; here it computes the value from a non-string field
//     (`u32`), which the bare/expr forms can't express
//
// The value is formatted inline by `tag!` as `prefix:value`; only the closure
// form introduces a `let`-binding (no closure is generated, so no
// higher-ranked-lifetime coercion).

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: widget_made,
    tags: {
        sku,
        owner: &self.owner,
        count: |e| e.count.to_string()
    }
)]
struct WidgetMade {
    #[new(into)]
    sku: String,
    #[new(into)]
    owner: String,
    count: u32,
}

#[test]
fn tag_value_forms_produce_expected_tags() {
    let event = WidgetMade::new("widget", "alice", 3);

    let tags = event.tags().expect("tags build");

    // Built in declaration order, each formatted `prefix:value`.
    assert_eq!(tags, vec![
        Tag::new("sku:widget").unwrap(),
        Tag::new("owner:alice").unwrap(),
        Tag::new("count:3").unwrap(),
    ]);
}

// =================================================================================================
// List-valued tags
// =================================================================================================

// A `[..]` value declares several tags under one prefix — the canonical DCB
// case of an event relating to two entities of the same kind: a transfer
// touches two accounts, so it carries two `account:` tags and surfaces in both
// accounts' queries.
#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(identifier: transferred, tags: { account: [from, to], reference })]
struct Transferred {
    #[new(into)]
    from: String,
    #[new(into)]
    to: String,
    #[new(into)]
    reference: String,
    amount: u64,
}

#[test]
fn list_valued_tag_produces_one_tag_per_value() {
    let event = Transferred::new("alice", "bob", "ref-1", 100);

    let tags = event.tags().expect("tags build");

    // `account: [from, to]` expands to two `account:` tags in place; then the
    // single `reference` tag.
    assert_eq!(tags, vec![
        Tag::new("account:alice").unwrap(),
        Tag::new("account:bob").unwrap(),
        Tag::new("reference:ref-1").unwrap(),
    ]);
}
