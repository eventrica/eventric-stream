use eventric_domain::projection::Projection;

#[derive(Projection)]
#[projection(selections: {
    dup: { events: [A] },
    dup: { events: [B] },
})]
struct Bar;

fn main() {}
