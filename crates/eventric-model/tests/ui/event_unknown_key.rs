use eventric_model::event::Event;

#[derive(Event)]
#[event(identifier: foo, oops: { item: sku })]
struct Foo {
    sku: String,
}

fn main() {}
