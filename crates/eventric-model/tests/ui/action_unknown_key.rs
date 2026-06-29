use eventric_model::action::Action;

#[derive(Action)]
#[action(oops: { p: X::new() })]
struct Bar;

fn main() {}
