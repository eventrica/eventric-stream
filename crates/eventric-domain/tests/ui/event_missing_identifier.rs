use eventric_domain::event::Event;

#[derive(Event)]
#[event(tags: { item: sku })]
struct Foo {
    sku: String,
}

fn main() {}
