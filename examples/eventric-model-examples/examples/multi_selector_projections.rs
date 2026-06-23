use derive_more::Debug;
use error_stack::Report;
use eventric_model::{
    Enactor as _,
    action::{
        Act,
        Action,
    },
    event::Event,
    projection::{
        Project,
        Projection,
        ProjectionEvent,
    },
};
use eventric_stream::{
    error::Error,
    stream::{
        Condition,
        Select as _,
        Stream,
    },
};
use fancy_constructor::new;
use revision::revisioned;

// =================================================================================================
// Multi-Selector Projections
// =================================================================================================

// This example demonstrates the two tools the model layer gives you for folding
// more than one kind of event into read-model state.
//
// (a) ONE projection with MULTIPLE `select(..)` clauses: the selections are
// OR-unioned, and every matched event is folded into a SINGLE derived state.
// `AccountBalance` folds both deposits and withdrawals into one number.
//
// (b) TWO SEPARATE projections over the same (overlapping) events: each folds
// the events it cares about into its OWN state, kept apart on purpose.
// `DepositTotal` and `WithdrawalTotal` each sum just their own side.
//
// `#[derive(Projection)]` allowing repeated `select(..)` is the same "generated
// companion type" mechanism `#[derive(Action)]` uses for its `{Name}Context`:
// the macro reads a declarative attribute and emits the wiring (here:
// `Selection` + `Dispatch`/`Recognize`) so the hand-written code only states
// the fold.

// Events

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier(money_deposited),
    tags(account(&this.account))
)]
pub struct MoneyDeposited {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier(money_withdrawn),
    tags(account(&this.account))
)]
pub struct MoneyWithdrawn {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

// (a) One projection, multiple `select(..)` clauses, one derived state.
//
// The two `select(..)` clauses are OR-unioned into the projection's
// `Selection`; both `MoneyDeposited` and `MoneyWithdrawn` events for the
// account are replayed into the SAME `AccountBalance`, where the two `Project`
// impls move the single `balance` field in opposite directions.

#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(MoneyDeposited),
        filter(account(&this.account))
    ),
    select(
        events(MoneyWithdrawn),
        filter(account(&this.account))
    )
)]
pub struct AccountBalance {
    #[new(into)]
    pub account: String,
    #[new(default)]
    pub balance: i64,
}

impl Project<MoneyDeposited> for AccountBalance {
    fn project(&mut self, event: ProjectionEvent<'_, MoneyDeposited>) {
        self.balance += i64::try_from(event.amount).unwrap_or(i64::MAX);
    }
}

impl Project<MoneyWithdrawn> for AccountBalance {
    fn project(&mut self, event: ProjectionEvent<'_, MoneyWithdrawn>) {
        self.balance -= i64::try_from(event.amount).unwrap_or(i64::MAX);
    }
}

// (b) Two separate projections over the same overlapping events, folded apart.
//
// Both projections look at the same account, but each selects only its own
// event type and keeps its own running total. This is how you fold overlapping
// events SEPARATELY (versus the union in (a)).

#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(MoneyDeposited),
        filter(account(&this.account))
    )
)]
pub struct DepositTotal {
    #[new(into)]
    pub account: String,
    #[new(default)]
    pub total: u64,
}

impl Project<MoneyDeposited> for DepositTotal {
    fn project(&mut self, event: ProjectionEvent<'_, MoneyDeposited>) {
        self.total += event.amount;
    }
}

#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(MoneyWithdrawn),
        filter(account(&this.account))
    )
)]
pub struct WithdrawalTotal {
    #[new(into)]
    pub account: String,
    #[new(default)]
    pub total: u64,
}

impl Project<MoneyWithdrawn> for WithdrawalTotal {
    fn project(&mut self, event: ProjectionEvent<'_, MoneyWithdrawn>) {
        self.total += event.amount;
    }
}

// Actions (used only to seed the stream the normal way)

#[derive(new, Action, Debug)]
#[action(projection(AccountBalance: AccountBalance::new(&this.account)))]
pub struct Deposit {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

impl Act for Deposit {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        context.append(&MoneyDeposited::new(&self.account, self.amount))?;

        Ok(())
    }
}

#[derive(new, Action, Debug)]
#[action(projection(AccountBalance: AccountBalance::new(&this.account)))]
pub struct Withdraw {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

impl Act for Withdraw {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        let balance = context.account_balance.balance;
        let amount = i64::try_from(self.amount).unwrap_or(i64::MAX);

        if balance < amount {
            return Err(Report::new(Error).attach("Insufficient Funds"));
        }

        context.append(&MoneyWithdrawn::new(&self.account, self.amount))?;

        Ok(())
    }
}

// Fold a projection directly from the stream and return its derived state.
//
// This is exactly what the `Enactor` does internally for an action's
// projections: query the stream with the projection's own `Selection`, then for
// each event the projection `recognize`s, `dispatch` it into the projection's
// fold. We do it by hand here so the example can fold and print read models on
// demand.

fn project<P>(stream: &Stream, mut projection: P) -> Result<P, Report<Error>>
where
    P: Projection,
{
    let condition = Condition::new().selections([projection.select()?]);

    for event in stream.select(condition) {
        let event = event?;

        if let Some(dispatch) = projection.recognize(&event)? {
            projection.dispatch(&dispatch);
        }
    }

    Ok(projection)
}

// Example

pub fn main() -> Result<(), Report<Error>> {
    let mut stream = Stream::builder(eventric_stream::temp_path())
        .temporary(true)
        .open()?;

    // Seed the stream: a sequence of deposits and withdrawals on one account.
    stream.enact(Deposit::new("alice", 100))?;
    stream.enact(Deposit::new("alice", 50))?;
    stream.enact(Withdraw::new("alice", 30))?;
    stream.enact(Deposit::new("alice", 25))?;
    stream.enact(Withdraw::new("alice", 40))?;

    println!("Seeded 5 events (3 deposits, 2 withdrawals) for account \"alice\".");
    println!();

    // (a) One projection, multiple selectors -> a single unioned derived state.
    let balance = project(&stream, AccountBalance::new("alice"))?;

    println!("(a) ONE projection, MULTIPLE select(..) clauses -> ONE derived state:");
    println!("    {balance:?}");
    println!(
        "    (deposits + withdrawals folded into a single balance: 100 + 50 - 30 + 25 - 40 = 105)"
    );
    println!();

    // (b) Two separate projections over the same events -> two derived states.
    let deposits = project(&stream, DepositTotal::new("alice"))?;
    let withdrawals = project(&stream, WithdrawalTotal::new("alice"))?;

    println!("(b) TWO SEPARATE projections over the same events -> TWO derived states:");
    println!("    {deposits:?}");
    println!("    {withdrawals:?}");
    println!("    (deposits and withdrawals folded apart: 100 + 50 + 25 = 175, 30 + 40 = 70)");

    Ok(())
}
