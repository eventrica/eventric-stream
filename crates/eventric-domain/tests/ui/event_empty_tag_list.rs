use eventric_domain::event::Event;

#[derive(Event)]
#[event(identifier: foo, tags: { item: [] })]
struct Foo {
    sku: String,
}

fn main() {}
