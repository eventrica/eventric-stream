use eventric_domain::event::Event;

#[derive(Event)]
#[event(identifier: foo, identifier: bar)]
struct Foo;

fn main() {}
