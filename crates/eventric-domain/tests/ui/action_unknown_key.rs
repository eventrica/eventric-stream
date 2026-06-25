use eventric_domain::action::Action;

#[derive(Action)]
#[action(oops: { p: X::new() })]
struct Bar;

fn main() {}
