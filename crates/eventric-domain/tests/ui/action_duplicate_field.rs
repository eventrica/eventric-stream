use eventric_domain::action::Action;

#[derive(Action)]
#[action(projections: { p: X::new(), p: Y::new() })]
struct Bar;

fn main() {}
