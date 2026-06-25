use eventric_domain::projection::Projection;

#[derive(Projection)]
#[projection(selections: { thing: { events: [] } })]
struct Bar;

fn main() {}
