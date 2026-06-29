use eventric_model::projection::Projection;

#[derive(Projection)]
#[projection(selections: {
    dup: { events: [A] },
    dup: { events: [B] },
})]
struct Bar;

fn main() {}
