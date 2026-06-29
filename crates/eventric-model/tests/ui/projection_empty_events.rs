use eventric_model::projection::Projection;

#[derive(Projection)]
#[projection(selections: { thing: { events: [] } })]
struct Bar;

fn main() {}
