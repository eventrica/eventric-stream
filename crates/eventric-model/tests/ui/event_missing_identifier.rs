use eventric_model::event::Event;

#[derive(Event)]
#[event(tags: { item: sku })]
struct Foo {
    sku: String,
}

fn main() {}
