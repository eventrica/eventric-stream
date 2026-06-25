use eventric_domain::action::Action;

#[derive(Action)]
#[action(projections: { p: some_fn() })]
struct Bar;

fn main() {}
