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
//   - expression  -> evaluated with the event bound as `this` (`&Self`)
//   - closure     -> the same, but naming the receiver; here it also computes
//     the value from a non-string field (`u32`), which the bare/expr forms
//     can't express
//
// All three desugar to the same `let <name> = self; <body>` block and are
// formatted by `tag!` as `prefix:value`.

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: widget_made,
    tags: [
        sku: sku,
        owner: &this.owner,
        count: |e| e.count.to_string()
    ]
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
