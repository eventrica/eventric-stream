use eventric_model::event::Event;

#[derive(Event)]
#[event(identifier: foo, identifier: bar)]
struct Foo;

fn main() {}
