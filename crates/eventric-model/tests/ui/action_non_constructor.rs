use eventric_model::action::Action;

#[derive(Action)]
#[action(projections: { p: some_fn() })]
struct Bar;

fn main() {}
