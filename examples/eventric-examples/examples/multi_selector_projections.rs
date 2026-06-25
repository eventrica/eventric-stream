use derive_more::Debug;
use error_stack::{
    Report,
    ResultExt as _,
};
use eventric_domain::{
    action::{
        Act,
        Action,
    },
    enactor::Enactor as _,
    error::Error,
    event::{
        Event,
        Events,
    },
    projection::{
        self,
        Project,
        Projection,
    },
};
use eventric_stream::stream::{
    Stream,
    operate::{
        Condition,
        select::Select as _,
    },
};
use fancy_constructor::new;
use revision::revisioned;

// =================================================================================================
// Multi-Selector Projections
// =================================================================================================

// This example demonstrates how named selections shape what a projection folds.
//
// (a) ONE named selection naming SEVERAL event types: every matched event folds
// into a SINGLE derived state, and the selection's method discriminates by enum
// variant. `AccountBalance` folds both deposits and withdrawals into one
// number.
//
// (b) TWO SEPARATE projections over the same (overlapping) events: each folds
// the events it cares about into its OWN state, kept apart on purpose.
// `DepositTotal` and `WithdrawalTotal` each sum just their own side. (The same
// split could live in one projection as two named selections — see
// `multi_selector` in the tests.)

// Events

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: money_deposited,
    tags: { account }
)]
pub struct MoneyDeposited {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: money_withdrawn,
    tags: { account }
)]
pub struct MoneyWithdrawn {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

// (a) One named selection over two event types, folded into one derived state.
//
// `balance` names both `MoneyDeposited` and `MoneyWithdrawn` (filtered to the
// account); both fold into the SAME `AccountBalance`, where the one `balance`
// method moves the single field in opposite directions per variant.

#[derive(new, Projection, Debug)]
#[projection(selections: {
    balance: { events: [MoneyDeposited, MoneyWithdrawn], filter: { account } },
})]
pub struct AccountBalance {
    #[new(into)]
    pub account: String,
    #[new(default)]
    pub balance: i64,
}

impl Project<account_balance::Balance<'_>> for AccountBalance {
    fn project(&mut self, event: projection::Event<account_balance::Balance<'_>>) {
        match event.event() {
            account_balance::Balance::MoneyDeposited(event) => {
                self.balance += i64::try_from(event.amount).unwrap_or(i64::MAX);
            }
            account_balance::Balance::MoneyWithdrawn(event) => {
                self.balance -= i64::try_from(event.amount).unwrap_or(i64::MAX);
            }
        }
    }
}

// (b) Two separate projections over the same overlapping events, folded apart.
//
// Both look at the same account, but each selects only its own event type and
// keeps its own running total — overlapping events folded SEPARATELY (versus
// the union in (a)).

#[derive(new, Projection, Debug)]
#[projection(selections: {
    deposited: { events: [MoneyDeposited], filter: { account } },
})]
pub struct DepositTotal {
    #[new(into)]
    pub account: String,
    #[new(default)]
    pub total: u64,
}

impl Project<deposit_total::Deposited<'_>> for DepositTotal {
    fn project(&mut self, event: projection::Event<deposit_total::Deposited<'_>>) {
        let deposit_total::Deposited::MoneyDeposited(event) = event.event();
        self.total += event.amount;
    }
}

#[derive(new, Projection, Debug)]
#[projection(selections: {
    withdrawn: { events: [MoneyWithdrawn], filter: { account } },
})]
pub struct WithdrawalTotal {
    #[new(into)]
    pub account: String,
    #[new(default)]
    pub total: u64,
}

impl Project<withdrawal_total::Withdrawn<'_>> for WithdrawalTotal {
    fn project(&mut self, event: projection::Event<withdrawal_total::Withdrawn<'_>>) {
        let withdrawal_total::Withdrawn::MoneyWithdrawn(event) = event.event();
        self.total += event.amount;
    }
}

// Actions (used only to seed the stream the normal way)

#[derive(new, Action, Debug)]
#[action(projections: {
    account_balance: AccountBalance::new(&self.account),
})]
pub struct Deposit {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

impl Act<deposit::Projections> for Deposit {
    fn act(
        &self,
        events: &mut Events,
        _projections: &deposit::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        events.append(&MoneyDeposited::new(&self.account, self.amount))?;

        Ok(())
    }
}

#[derive(new, Action, Debug)]
#[action(projections: {
    account_balance: AccountBalance::new(&self.account),
})]
pub struct Withdraw {
    #[new(into)]
    pub account: String,
    pub amount: u64,
}

impl Act<withdraw::Projections> for Withdraw {
    fn act(
        &self,
        events: &mut Events,
        projections: &withdraw::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        let balance = projections.account_balance.balance;
        let amount = i64::try_from(self.amount).unwrap_or(i64::MAX);

        if balance < amount {
            return Err(Report::new(Error).attach("Insufficient Funds"));
        }

        events.append(&MoneyWithdrawn::new(&self.account, self.amount))?;

        Ok(())
    }
}

// Fold a projection directly from the stream and return its derived state.
//
// This is exactly what the `Enactor` does internally for an action's
// projections: query the stream with the projection's own selections, then for
// each event the projection `recognize`s, `dispatch` it into the projection's
// fold (handing it the event's mask, which — standalone — is the projection's
// own slice).

fn project<P>(stream: &Stream, mut projection: P) -> Result<P, Report<Error>>
where
    P: Projection,
{
    let condition = Condition::new().selections(projection.select()?);

    for event in stream.select(condition) {
        let event = event.change_context(Error)?;

        if let Some(dispatch) = projection.recognize(&event)? {
            projection.dispatch(event.mask.as_ref(), &dispatch);
        }
    }

    Ok(projection)
}

// Example

pub fn main() -> Result<(), Report<Error>> {
    let mut stream = Stream::builder(eventric_stream::utils::temp_path())
        .temporary(true)
        .open()
        .change_context(Error)?;

    // Seed the stream: a sequence of deposits and withdrawals on one account.
    stream.enact(Deposit::new("alice", 100))?;
    stream.enact(Deposit::new("alice", 50))?;
    stream.enact(Withdraw::new("alice", 30))?;
    stream.enact(Deposit::new("alice", 25))?;
    stream.enact(Withdraw::new("alice", 40))?;

    println!("Seeded 5 events (3 deposits, 2 withdrawals) for account \"alice\".");
    println!();

    // (a) One projection, one selection over two event types -> a single state.
    let balance = project(&stream, AccountBalance::new("alice"))?;

    println!("(a) ONE projection, ONE selection over two event types -> ONE derived state:");
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
