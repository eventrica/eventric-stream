use eventric_model::projection::Projection;

#[derive(Projection)]
#[projection(oops: { thing: { events: [Foo] } })]
struct Bar;

fn main() {}
