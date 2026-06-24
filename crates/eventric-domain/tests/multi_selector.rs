//! Integration tests for *multi-selector* projection behaviour: a single
//! `#[derive(Projection)]` carrying more than one `select(...)` clause, and the
//! complementary "separate projections" idiom where two projections range over
//! the same event type with different filters.
//!
//! These tests exercise three distinct shapes, all built on the same tiny
//! "wallet" domain (deposits and withdrawals, each tagged by account):
//!
//! 1. ONE projection, multiple `select(...)` clauses over DISTINCT event types.
//!    The clauses OR-union into a single `Selection`; each matched event is
//!    routed to the `Project` impl for its own type. (Same mechanism as
//!    `enact.rs`'s `ItemPresent`, but here we additionally fold a numeric value
//!    from each type's payload to prove the union folds correctly.)
//!
//! 2. ONE projection, two `select(...)` clauses on the SAME type but DIFFERENT
//!    tag filters. The two clauses OR-union, so the projection replays events
//!    matching EITHER filter — but because both clauses target the same event
//!    type (one `Project` impl), the projection cannot tell which clause
//!    matched from the type alone. Discrimination is therefore done from the
//!    PAYLOAD inside `project`.
//!
//! 3. The SEPARATE-PROJECTIONS idiom: two projections over the same event type
//!    with different filters, driven by one action. They fold INDEPENDENTLY
//!    (each gets its own mask bit), and a single event matching BOTH filters is
//!    folded into BOTH projections (multi-match).
//!
//! Storage is always a fresh temporary stream, isolated per test.

use derive_more::Debug;
use error_stack::Report;
use eventric_domain::{
    action::{
        Act,
        Action,
    },
    enactor::Enactor as _,
    error::Error,
    event::Event,
    projection::{
        Project,
        Projection,
        ProjectionEvent,
    },
};
use eventric_stream::stream::Stream;
use fancy_constructor::new;
use revision::revisioned;

// =================================================================================================
// Fixture
// =================================================================================================

// Events
//
// Two distinct event types over a "wallet" domain. Both are tagged by account,
// and `Deposit` is *additionally* tagged by `channel` (e.g. "wire" / "card") so
// that test 2 can build two same-type selectors with different tag filters.

#[revisioned(revision = 1)]
#[derive(new, Event, Debug, PartialEq)]
#[event(
    identifier(deposit),
    tags(
        account(&this.account),
        channel(&this.channel)
    )
)]
struct Deposit {
    #[new(into)]
    account: String,
    #[new(into)]
    channel: String,
    amount: u64,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug, PartialEq)]
#[event(
    identifier(withdrawal),
    tags(account(&this.account))
)]
struct Withdrawal {
    #[new(into)]
    account: String,
    amount: u64,
}

// =================================================================================================
// 1. ONE projection, multiple `select` clauses over DISTINCT event types.
// =================================================================================================

/// Folds the running balance for an account from BOTH event types. The two
/// `select(...)` clauses (one per type) OR-union into a single `Selection`;
/// each matched event is dispatched to the `Project` impl for its own type, so
/// deposits add and withdrawals subtract. Proving the union folds correctly
/// means a value derived from each *distinct* type's payload lands in
/// `balance`.
#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(Deposit),
        filter(account(&this.account))
    )
)]
#[projection(
    select(
        events(Withdrawal),
        filter(account(&this.account))
    )
)]
struct Balance {
    #[new(default)]
    net: i64,
    #[new(default)]
    deposits: u32,
    #[new(default)]
    withdrawals: u32,
    #[new(into)]
    account: String,
}

impl Project<Deposit> for Balance {
    fn project(&mut self, event: ProjectionEvent<'_, Deposit>) {
        self.net += i64::try_from(event.amount).expect("deposit fits i64");
        self.deposits += 1;
    }
}

impl Project<Withdrawal> for Balance {
    fn project(&mut self, event: ProjectionEvent<'_, Withdrawal>) {
        self.net -= i64::try_from(event.amount).expect("withdrawal fits i64");
        self.withdrawals += 1;
    }
}

// =================================================================================================
// 2. ONE projection, two `select` clauses on the SAME type, DIFFERENT tag
//    filters.
// =================================================================================================

/// Two `select(...)` clauses, BOTH on `Deposit`, but filtered by different
/// channels ("wire" and "card"). The clauses OR-union, so the projection
/// replays deposits matching EITHER channel. Both clauses dispatch to the
/// single `Project<Deposit>` impl, which therefore cannot know *which* clause
/// matched from the type alone — it must discriminate from the PAYLOAD
/// (`event.channel`) to bucket the amount.
#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(Deposit),
        filter(
            account(&this.account),
            channel(|_this| "wire")
        )
    )
)]
#[projection(
    select(
        events(Deposit),
        filter(
            account(&this.account),
            channel(|_this| "card")
        )
    )
)]
struct ChannelTotals {
    #[new(default)]
    wire: u64,
    #[new(default)]
    card: u64,
    #[new(default)]
    other: u64,
    #[new(into)]
    account: String,
}

impl Project<Deposit> for ChannelTotals {
    fn project(&mut self, event: ProjectionEvent<'_, Deposit>) {
        // Discrimination is from the PAYLOAD, not the selector: both clauses
        // dispatch here, so the channel is read off the decoded event.
        match event.channel.as_str() {
            "wire" => self.wire += event.amount,
            "card" => self.card += event.amount,
            _ => self.other += event.amount,
        }
    }
}

// =================================================================================================
// 3. SEPARATE-PROJECTIONS idiom: two projections, same type, different filters.
// =================================================================================================

/// Counts deposits arriving over the "wire" channel for an account.
#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(Deposit),
        filter(
            account(&this.account),
            channel(|_this| "wire")
        )
    )
)]
struct WireDeposits {
    #[new(default)]
    count: u32,
    #[new(default)]
    total: u64,
    #[new(into)]
    account: String,
}

impl Project<Deposit> for WireDeposits {
    fn project(&mut self, event: ProjectionEvent<'_, Deposit>) {
        self.count += 1;
        self.total += event.amount;
    }
}

/// Counts LARGE deposits (amount >= threshold) for an account, regardless of
/// channel. A single deposit can match BOTH this filter and `WireDeposits`'.
#[derive(new, Projection, Debug)]
#[projection(
    select(
        events(Deposit),
        filter(account(&this.account))
    )
)]
struct LargeDeposits {
    #[new(default)]
    count: u32,
    #[new(default)]
    total: u64,
    #[new(into)]
    account: String,
}

impl Project<Deposit> for LargeDeposits {
    fn project(&mut self, event: ProjectionEvent<'_, Deposit>) {
        // Only large deposits are counted; discrimination is from the payload.
        if event.amount >= 100 {
            self.count += 1;
            self.total += event.amount;
        }
    }
}

// =================================================================================================
// Actions
// =================================================================================================

/// Append a deposit (unconditionally — no business rule). Tags by account and
/// channel.
#[derive(new, Action, Debug)]
#[action(
    projection(Balance: Balance::new(&this.account))
)]
struct MakeDeposit {
    #[new(into)]
    account: String,
    #[new(into)]
    channel: String,
    amount: u64,
}

impl Act for MakeDeposit {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        context.append(&Deposit::new(&self.account, &self.channel, self.amount))?;

        Ok(())
    }
}

/// Append a withdrawal, but only if the folded `Balance` (deposits MINUS
/// withdrawals, across both event types) can cover it. This is what proves the
/// multi-selector union folded correctly: the rule reads `balance`, which only
/// holds the right value if both `Deposit` and `Withdrawal` clauses replayed.
#[derive(new, Action, Debug)]
#[action(
    projection(Balance: Balance::new(&this.account))
)]
struct MakeWithdrawal {
    #[new(into)]
    account: String,
    amount: u64,
}

impl Act for MakeWithdrawal {
    type Err = Report<Error>;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        if context.balance.net < i64::try_from(self.amount).expect("amount fits i64") {
            return Err(Report::new(Error).attach("Insufficient Funds"));
        }

        context.append(&Withdrawal::new(&self.account, self.amount))?;

        Ok(())
    }
}

/// Read-only: returns the folded `Balance` projection for an account.
#[derive(new, Action, Debug)]
#[action(
    projection(Balance: Balance::new(&this.account))
)]
struct ReadBalance {
    #[new(into)]
    account: String,
}

impl Act for ReadBalance {
    type Err = Report<Error>;
    type Ok = Balance;

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        // Clone the folded projection out of the context for inspection.
        Ok(Balance {
            net: context.balance.net,
            deposits: context.balance.deposits,
            withdrawals: context.balance.withdrawals,
            account: context.balance.account.clone(),
        })
    }
}

/// Read-only: returns the folded `ChannelTotals` (test 2 — same-type, two tag
/// filters, payload discrimination).
#[derive(new, Action, Debug)]
#[action(
    projection(ChannelTotals: ChannelTotals::new(&this.account))
)]
struct ReadChannelTotals {
    #[new(into)]
    account: String,
}

impl Act for ReadChannelTotals {
    type Err = Report<Error>;
    type Ok = (u64, u64, u64);

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        let totals = &context.channel_totals;

        Ok((totals.wire, totals.card, totals.other))
    }
}

/// Read-only: drives BOTH `WireDeposits` and `LargeDeposits` in a single action
/// (test 3 — separate projections, same type, different filters, independent
/// folds). Returns `(wire_count, wire_total, large_count, large_total)`.
#[derive(new, Action, Debug)]
#[action(
    projection(WireDeposits: WireDeposits::new(&this.account)),
    projection(LargeDeposits: LargeDeposits::new(&this.account))
)]
struct ReadDepositStats {
    #[new(into)]
    account: String,
}

impl Act for ReadDepositStats {
    type Err = Report<Error>;
    type Ok = (u32, u64, u32, u64);

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err> {
        Ok((
            context.wire_deposits.count,
            context.wire_deposits.total,
            context.large_deposits.count,
            context.large_deposits.total,
        ))
    }
}

// =================================================================================================
// Helpers
// =================================================================================================

fn stream() -> Stream {
    Stream::builder(eventric_stream::utils::temp_path())
        .temporary(true)
        .open()
        .expect("open temporary stream")
}

fn total_events(stream: &Stream) -> usize {
    use eventric_stream::stream::operate::{
        Condition,
        select::Select as _,
    };

    stream
        .select(Condition::new())
        .map(|event| event.expect("scan event"))
        .collect::<Vec<_>>()
        .len()
}

// =================================================================================================
// Tests
// =================================================================================================

// 1. ONE projection, multiple `select` clauses over DISTINCT event types: the
//    OR-union folds correctly, each type via its own `Project` impl.
#[test]
fn single_projection_distinct_type_clauses_union_folds() {
    let mut stream = stream();

    // Two deposits and one withdrawal for the same account.
    assert!(stream.enact(MakeDeposit::new("alice", "wire", 100)).is_ok());
    assert!(stream.enact(MakeDeposit::new("alice", "card", 50)).is_ok());
    assert!(stream.enact(MakeWithdrawal::new("alice", 30)).is_ok());

    assert_eq!(total_events(&stream), 3);

    // The `Balance` projection folded BOTH deposit clauses AND the withdrawal
    // clause: 100 + 50 - 30 = 120, from 2 deposits and 1 withdrawal.
    let balance = stream
        .enact(ReadBalance::new("alice"))
        .expect("read balance");

    assert_eq!(balance.deposits, 2);
    assert_eq!(balance.withdrawals, 1);
    assert_eq!(balance.net, 120);

    // The business rule depends on the union: a withdrawal of 120 is exactly
    // covered (succeeds), but 121 is not (rejected, nothing appended).
    assert!(stream.enact(MakeWithdrawal::new("alice", 121)).is_err());
    assert_eq!(total_events(&stream), 3);

    assert!(stream.enact(MakeWithdrawal::new("alice", 120)).is_ok());
    assert_eq!(total_events(&stream), 4);

    // Balance is now zero: 100 + 50 - 30 - 120.
    let balance = stream
        .enact(ReadBalance::new("alice"))
        .expect("read balance");

    assert_eq!(balance.net, 0);
    assert_eq!(balance.deposits, 2);
    assert_eq!(balance.withdrawals, 2);
}

// 1b. Account isolation: the account tag filter scopes the union. A different
//     account's events must not fold into `alice`'s balance.
#[test]
fn single_projection_union_respects_tag_filter() {
    let mut stream = stream();

    assert!(stream.enact(MakeDeposit::new("alice", "wire", 100)).is_ok());
    assert!(stream.enact(MakeDeposit::new("bob", "wire", 999)).is_ok());
    assert!(stream.enact(MakeWithdrawal::new("alice", 40)).is_ok());

    let alice = stream.enact(ReadBalance::new("alice")).expect("read alice");
    let bob = stream.enact(ReadBalance::new("bob")).expect("read bob");

    // Alice: 100 - 40 = 60, one deposit, one withdrawal.
    assert_eq!(alice.net, 60);
    assert_eq!(alice.deposits, 1);
    assert_eq!(alice.withdrawals, 1);

    // Bob: 999, one deposit, no withdrawals. Bob's deposit never folded into
    // alice's balance, nor vice versa.
    assert_eq!(bob.net, 999);
    assert_eq!(bob.deposits, 1);
    assert_eq!(bob.withdrawals, 0);
}

// 2. ONE projection, two `select` clauses on the SAME type with DIFFERENT tag
//    filters: the OR-union is replayed, and the projection discriminates the
//    matched clause from the PAYLOAD (not the selector).
#[test]
fn single_projection_same_type_clauses_discriminate_by_payload() {
    let mut stream = stream();

    // Deposits across three channels for the same account. Only "wire" and
    // "card" are selected by the two clauses; "cash" is NOT selected, so it
    // must never reach the projection.
    assert!(stream.enact(MakeDeposit::new("carol", "wire", 10)).is_ok());
    assert!(stream.enact(MakeDeposit::new("carol", "wire", 5)).is_ok());
    assert!(stream.enact(MakeDeposit::new("carol", "card", 20)).is_ok());
    assert!(stream.enact(MakeDeposit::new("carol", "cash", 99)).is_ok());

    assert_eq!(total_events(&stream), 4);

    let (wire, card, other) = stream
        .enact(ReadChannelTotals::new("carol"))
        .expect("read channel totals");

    // Wire: 10 + 5 = 15. Card: 20. The two clauses OR-unioned, both dispatched
    // to the single Project<Deposit>, which bucketed by payload channel.
    assert_eq!(wire, 15);
    assert_eq!(card, 20);

    // The "cash" deposit was outside BOTH clauses' tag filters, so it never
    // reached `project`: `other` stays zero even though it carries the largest
    // amount.
    assert_eq!(other, 0);
}

// 3. SEPARATE-PROJECTIONS idiom: two projections over the same event type with
//    different filters fold INDEPENDENTLY, and one event matching BOTH filters
//    folds into BOTH (multi-match).
#[test]
fn separate_projections_fold_independently_and_multi_match() {
    let mut stream = stream();

    // - wire/150 : matches WireDeposits (wire) AND LargeDeposits (>=100) -> BOTH
    // - wire/40  : matches WireDeposits only (wire, but < 100)
    // - card/200 : matches LargeDeposits only (>=100, but not wire)
    // - card/10  : matches NEITHER (not wire, < 100) but still a deposit.
    assert!(stream.enact(MakeDeposit::new("dave", "wire", 150)).is_ok());
    assert!(stream.enact(MakeDeposit::new("dave", "wire", 40)).is_ok());
    assert!(stream.enact(MakeDeposit::new("dave", "card", 200)).is_ok());
    assert!(stream.enact(MakeDeposit::new("dave", "card", 10)).is_ok());

    assert_eq!(total_events(&stream), 4);

    let (wire_count, wire_total, large_count, large_total) = stream
        .enact(ReadDepositStats::new("dave"))
        .expect("read deposit stats");

    // WireDeposits folds the two wire deposits independently of size:
    // 150 + 40 = 190 over 2 events.
    assert_eq!(wire_count, 2);
    assert_eq!(wire_total, 190);

    // LargeDeposits folds the two large deposits independently of channel:
    // 150 + 200 = 350 over 2 events.
    assert_eq!(large_count, 2);
    assert_eq!(large_total, 350);

    // The multi-match is the wire/150 deposit: it was counted in BOTH
    // projections (it is one of WireDeposits' 2 and one of LargeDeposits' 2),
    // proving a single event folds into every projection whose filter it
    // satisfies, with each projection's mask bit set independently.
}

// 3b. A single event matching BOTH projections' filters is the ONLY event, to
//     isolate the multi-match: both projections fold the same one deposit.
#[test]
fn separate_projections_single_event_multi_match() {
    let mut stream = stream();

    // One deposit: wire (matches WireDeposits) AND amount 500 >= 100 (matches
    // LargeDeposits).
    assert!(stream.enact(MakeDeposit::new("erin", "wire", 500)).is_ok());

    assert_eq!(total_events(&stream), 1);

    let (wire_count, wire_total, large_count, large_total) = stream
        .enact(ReadDepositStats::new("erin"))
        .expect("read deposit stats");

    // The single event folded into BOTH projections.
    assert_eq!(wire_count, 1);
    assert_eq!(wire_total, 500);
    assert_eq!(large_count, 1);
    assert_eq!(large_total, 500);
}
