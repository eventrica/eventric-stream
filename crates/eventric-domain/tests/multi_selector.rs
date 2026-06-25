//! Integration tests for projection **named selections**: a single
//! `#[derive(Projection)]` carrying more than one named selection, and the
//! complementary "separate projections" idiom where two projections range over
//! the same event type with different filters.
//!
//! Three shapes over a tiny "wallet" domain (deposits and withdrawals, tagged
//! by account; deposits additionally by channel):
//!
//! 1. ONE selection naming SEVERAL event types. The selection's method matches
//!    an enum (a variant per type), folding both into one state — `Balance`.
//!
//! 2. TWO named selections on the SAME type with DIFFERENT tag filters. Each
//!    has its own method, so the *filter* (not a payload re-check) is what
//!    distinguishes them — `ChannelTotals` (wire vs card). An event matching no
//!    selection is simply not folded.
//!
//! 3. The SEPARATE-PROJECTIONS idiom: two projections over the same event type
//!    with different filters, driven by one action. They fold INDEPENDENTLY
//!    (each gets its own mask bit), and a single event matching BOTH folds into
//!    BOTH (multi-match). Discrimination that is NOT expressible as a tag
//!    filter (`LargeDeposits`' amount threshold) stays a payload check in the
//!    method.
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
// that test 2 can build two same-type selections with different tag filters.

#[revisioned(revision = 1)]
#[derive(new, Event, Debug, PartialEq)]
#[event(identifier: deposit, tags: { account: account, channel: channel })]
struct Deposit {
    #[new(into)]
    account: String,
    #[new(into)]
    channel: String,
    amount: u64,
}

#[revisioned(revision = 1)]
#[derive(new, Event, Debug, PartialEq)]
#[event(identifier: withdrawal, tags: { account: account })]
struct Withdrawal {
    #[new(into)]
    account: String,
    amount: u64,
}

// =================================================================================================
// 1. ONE selection naming SEVERAL event types.
// =================================================================================================

/// Folds the running balance for an account from BOTH event types. The one
/// `balance` selection names both types (filtered to the account); its method
/// matches the enum, so deposits add and withdrawals subtract. Proving the fold
/// is correct means a value derived from each *distinct* type's payload lands
/// in `net`.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    balance: { events: [Deposit, Withdrawal], filter: { account: account } },
})]
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

impl Project<balance::Balance<'_>> for Balance {
    fn project(&mut self, event: projection::Event<balance::Balance<'_>>) {
        match event.event() {
            balance::Balance::Deposit(event) => {
                self.net += i64::try_from(event.amount).expect("deposit fits i64");
                self.deposits += 1;
            }
            balance::Balance::Withdrawal(event) => {
                self.net -= i64::try_from(event.amount).expect("withdrawal fits i64");
                self.withdrawals += 1;
            }
        }
    }
}

// =================================================================================================
// 2. TWO named selections on the SAME type, DIFFERENT tag filters.
// =================================================================================================

/// Two named selections, BOTH on `Deposit`, filtered by different channels
/// ("wire" and "card"). Each has its own method, so the filter — not a payload
/// re-check — routes the amount to the right bucket. A deposit on some other
/// channel matches NEITHER selection and is not folded at all.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    wire: { events: [Deposit], filter: { account: account, channel: "wire" } },
    card: { events: [Deposit], filter: { account: account, channel: "card" } },
})]
struct ChannelTotals {
    #[new(default)]
    wire: u64,
    #[new(default)]
    card: u64,
    #[new(into)]
    account: String,
}

impl Project<channel_totals::Wire<'_>> for ChannelTotals {
    fn project(&mut self, event: projection::Event<channel_totals::Wire<'_>>) {
        let channel_totals::Wire::Deposit(event) = event.event();
        self.wire += event.amount;
    }
}

impl Project<channel_totals::Card<'_>> for ChannelTotals {
    fn project(&mut self, event: projection::Event<channel_totals::Card<'_>>) {
        let channel_totals::Card::Deposit(event) = event.event();
        self.card += event.amount;
    }
}

// =================================================================================================
// 3. SEPARATE-PROJECTIONS idiom: two projections, same type, different filters.
// =================================================================================================

/// Counts deposits arriving over the "wire" channel for an account.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    deposited: { events: [Deposit], filter: { account: account, channel: "wire" } },
})]
struct WireDeposits {
    #[new(default)]
    count: u32,
    #[new(default)]
    total: u64,
    #[new(into)]
    account: String,
}

impl Project<wire_deposits::Deposited<'_>> for WireDeposits {
    fn project(&mut self, event: projection::Event<wire_deposits::Deposited<'_>>) {
        let wire_deposits::Deposited::Deposit(event) = event.event();
        self.count += 1;
        self.total += event.amount;
    }
}

/// Counts LARGE deposits (amount >= threshold) for an account, regardless of
/// channel. A single deposit can match BOTH this filter and `WireDeposits`'.
#[derive(new, Projection, Debug)]
#[projection(selections: {
    deposited: { events: [Deposit], filter: { account: account } },
})]
struct LargeDeposits {
    #[new(default)]
    count: u32,
    #[new(default)]
    total: u64,
    #[new(into)]
    account: String,
}

impl Project<large_deposits::Deposited<'_>> for LargeDeposits {
    fn project(&mut self, event: projection::Event<large_deposits::Deposited<'_>>) {
        let large_deposits::Deposited::Deposit(event) = event.event();

        // The amount threshold is not a tag, so it stays a payload check.
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
#[action(projections: {
    balance: Balance::new(&self.account),
})]
struct MakeDeposit {
    #[new(into)]
    account: String,
    #[new(into)]
    channel: String,
    amount: u64,
}

impl Act<make_deposit::Projections> for MakeDeposit {
    type Err = Report<Error>;

    fn act(
        &self,
        events: &mut Events,
        _projections: &make_deposit::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        events.append(&Deposit::new(&self.account, &self.channel, self.amount))?;

        Ok(())
    }
}

/// Append a withdrawal, but only if the folded `Balance` (deposits MINUS
/// withdrawals, across both event types) can cover it. This is what proves the
/// selection folded correctly: the rule reads `net`, which only holds the right
/// value if both `Deposit` and `Withdrawal` events replayed.
#[derive(new, Action, Debug)]
#[action(projections: {
    balance: Balance::new(&self.account),
})]
struct MakeWithdrawal {
    #[new(into)]
    account: String,
    amount: u64,
}

impl Act<make_withdrawal::Projections> for MakeWithdrawal {
    type Err = Report<Error>;

    fn act(
        &self,
        events: &mut Events,
        projections: &make_withdrawal::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        if projections.balance.net < i64::try_from(self.amount).expect("amount fits i64") {
            return Err(Report::new(Error).attach("Insufficient Funds"));
        }

        events.append(&Withdrawal::new(&self.account, self.amount))?;

        Ok(())
    }
}

/// Read-only: returns the folded `Balance` projection for an account.
#[derive(new, Action, Debug)]
#[action(projections: {
    balance: Balance::new(&self.account),
})]
struct ReadBalance {
    #[new(into)]
    account: String,
}

impl Act<read_balance::Projections> for ReadBalance {
    type Err = Report<Error>;
    type Ok = Balance;

    fn act(
        &self,
        _events: &mut Events,
        projections: &read_balance::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        // Clone the folded projection out for inspection.
        Ok(Balance {
            net: projections.balance.net,
            deposits: projections.balance.deposits,
            withdrawals: projections.balance.withdrawals,
            account: projections.balance.account.clone(),
        })
    }
}

/// Read-only: returns the folded `ChannelTotals` (test 2 — same type, two named
/// selections distinguished by filter). Returns `(wire, card)`.
#[derive(new, Action, Debug)]
#[action(projections: {
    channel_totals: ChannelTotals::new(&self.account),
})]
struct ReadChannelTotals {
    #[new(into)]
    account: String,
}

impl Act<read_channel_totals::Projections> for ReadChannelTotals {
    type Err = Report<Error>;
    type Ok = (u64, u64);

    fn act(
        &self,
        _events: &mut Events,
        projections: &read_channel_totals::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        let totals = &projections.channel_totals;

        Ok((totals.wire, totals.card))
    }
}

/// Read-only: drives BOTH `WireDeposits` and `LargeDeposits` in a single action
/// (test 3 — separate projections, same type, different filters, independent
/// folds). Returns `(wire_count, wire_total, large_count, large_total)`.
#[derive(new, Action, Debug)]
#[action(projections: {
    wire_deposits: WireDeposits::new(&self.account),
    large_deposits: LargeDeposits::new(&self.account),
})]
struct ReadDepositStats {
    #[new(into)]
    account: String,
}

impl Act<read_deposit_stats::Projections> for ReadDepositStats {
    type Err = Report<Error>;
    type Ok = (u32, u64, u32, u64);

    fn act(
        &self,
        _events: &mut Events,
        projections: &read_deposit_stats::Projections,
    ) -> Result<Self::Ok, Self::Err> {
        Ok((
            projections.wire_deposits.count,
            projections.wire_deposits.total,
            projections.large_deposits.count,
            projections.large_deposits.total,
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

// 1. ONE selection naming both event types: the enum-matching method folds
//    both, deposits adding and withdrawals subtracting.
#[test]
fn one_selection_folds_several_event_types() {
    let mut stream = stream();

    // Two deposits and one withdrawal for the same account.
    assert!(stream.enact(MakeDeposit::new("alice", "wire", 100)).is_ok());
    assert!(stream.enact(MakeDeposit::new("alice", "card", 50)).is_ok());
    assert!(stream.enact(MakeWithdrawal::new("alice", 30)).is_ok());

    assert_eq!(total_events(&stream), 3);

    // The `Balance` selection folded BOTH deposits AND the withdrawal:
    // 100 + 50 - 30 = 120, from 2 deposits and 1 withdrawal.
    let balance = stream
        .enact(ReadBalance::new("alice"))
        .expect("read balance");

    assert_eq!(balance.deposits, 2);
    assert_eq!(balance.withdrawals, 1);
    assert_eq!(balance.net, 120);

    // The business rule depends on the fold: a withdrawal of 120 is exactly
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

// 1b. Account isolation: the account tag filter scopes the selection. A
// different     account's events must not fold into `alice`'s balance.
#[test]
fn selection_respects_tag_filter() {
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

// 2. TWO named selections on the SAME type with different filters: each event
//    is routed to its selection's method by the filter, and an event matching
//    no selection is not folded.
#[test]
fn named_selections_route_same_type_by_filter() {
    let mut stream = stream();

    // Deposits across three channels for the same account. Only "wire" and
    // "card" have selections; "cash" matches neither, so it must never fold.
    assert!(stream.enact(MakeDeposit::new("carol", "wire", 10)).is_ok());
    assert!(stream.enact(MakeDeposit::new("carol", "wire", 5)).is_ok());
    assert!(stream.enact(MakeDeposit::new("carol", "card", 20)).is_ok());
    assert!(stream.enact(MakeDeposit::new("carol", "cash", 99)).is_ok());

    assert_eq!(total_events(&stream), 4);

    let (wire, card) = stream
        .enact(ReadChannelTotals::new("carol"))
        .expect("read channel totals");

    // Wire: 10 + 5 = 15, routed by the `wire` selection. Card: 20, by `card`.
    assert_eq!(wire, 15);
    assert_eq!(card, 20);

    // The "cash" deposit (99) matched neither selection, so it folded into
    // nothing — wire + card account for 35, not 134.
}

// 3. SEPARATE-PROJECTIONS idiom: two projections over the same event type with
//    different filters fold INDEPENDENTLY, and one event matching BOTH folds
//    into BOTH (multi-match).
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
