# FUTURE.md

Known-but-unplanned items: things we are *aware* of but have not committed to a
plan for. A living list — add to it as things surface, prune as they land.

Priorities here are weighed against the guiding vision in
[`vision.md`](./vision.md); the forward *roadmap* (what's planned next —
**reactions** (designed in [`boundary.md`](./boundary.md)) as the gating building
block, then multi-context composition) lives there (§9). This file is the *unplanned*
residue.

The structural work is **done**: the stream-core rewrite ([`REFACTOR.md`]), the
crate consolidation ([`CONSOLIDATION.md`]), and the content-seam split
([`SPLIT.md`]) are complete, and the model/runtime split has since landed — **four
library crates**: `eventric-stream` (content-agnostic substrate, no `revision`) +
`eventric-model` (user-facing event-sourcing model) + `eventric-runtime` (the
mechanism that runs it — the `Enactor`) + `eventric-macros`, plus
`eventric-examples` / `eventric-benches`.
`error-stack` throughout (one `Error` per runtime crate, `change_context`'d at the
boundary), and the public surface carries no re-export lifts. Nothing below blocks
that; these are the deferred design decisions and the smaller debt.

[`REFACTOR.md`]: ./archived/REFACTOR.md
[`CONSOLIDATION.md`]: ./archived/CONSOLIDATION.md
[`SPLIT.md`]: ./archived/SPLIT.md

---

## 1. Versioning — the biggest open area (design-pending)

A full research-grounded exploration — the theory, the production-framework prior
art, the DCB-specific picture, the conclusions of an adversarial review, and a
suggested order of work — is in [`versioning.md`](./versioning.md).

**Landed (`versioning.md` §5):** the event `Version` is now sourced from the
`revision` schema number, so the two cannot diverge and there is no separate
version to declare. That retired the old "model hardcodes `Version` 0" and "two
orthogonal axes" items — schema revision and stream `Version` are now one notion.
The remaining open question is the *breaking-change* story, which the maintainer
wants to think through before building. (The distinct question of *cross-context
contract* versioning has its mechanism in [`boundary.md`](./boundary.md) §2 — the
public/private membrane — independent of this internal-event story.)

- **Done — orphaned `Version`/`Range` comparison traits dropped.** `impl
  PartialEq<Range<Self>>` / `impl PartialOrd<Range<Self>>` for `Version` had no
  caller (the version filter uses stdlib `Range::contains`), and the
  informational-`Version` lean ([`vision.md`](./vision.md) §8) removed their only
  prospective use — a version-keyed range-scan — so both impls (and the now-unused
  `Ordering`/`Range` imports in `event.rs`) were removed.
- **A `revision`-mismatch decode failure stays the opaque `Error` type.** It now
  carries an informative attachment (the stored version + the revision this
  consumer handles), so it is *diagnosable*; a distinct error *type/variant* is
  intentionally out of scope (the opaque-`Error` design adds detail via
  attachments, not variants).

### Stream-layer `Version` debt (cheaper, independent of the above)

- **The `MAX` (255) sentinel is unqueryable:** the half-open default range and
  all `VersionSelector` lowerings cap the upper bound at the exclusive
  `Version::MAX`, so version-255 events can be appended but never matched. Now
  **pinned by a test** (`select.rs`) as a known limitation; a real fix is tied to
  the version-as-selection question, which the vision leans toward dropping (§7.1
  of [`versioning.md`](./versioning.md)).
- **Tested** *(was untested)*: the `a..` / `..b` / `..` range lowerings and the 255
  boundary (`select.rs`), and multi-version selection (`store.rs`).

## 2. Derive codegen ergonomics (done — all three derives migrated)

The full redesign of the three derives — a **declarative attribute grammar**,
**named projection selectors** (per-selection event enums + a parameterised
`Project<Enum>` fold surface), and the supporting groundwork — is implemented and
specced in [`derives.md`](./derives.md). All three derives are hand-parsed (no
`darling`): the `#[event(..)]`, `#[projection(selections: { .. })]`, and
`#[action(projections: { .. })]` grammars. Projection generates per-selection borrowed
enums (the user folds each via the standard `Project<Enum>`) and de-positionalises the
mask; Action generates a `Projections` struct in a snake_case submodule (built in
`projections(&self)`) and the user implements the parameterised `Act<Projections>` —
retiring the `identity::<fn(&Self)>` coercion and the deref-fused context. The two
derives now share one rule: **implement a standard library trait parameterised by a
generated type** (`Project<Enum>` / `Act<Projections>`); the macros only generate types
+ machinery. `self` is the receiver name throughout. The redesign realised the deferred
keyed-selectors and the codegen groundwork:

- named selectors + a **de-positionalised mask** (model-layer routing by selection,
  not by declaration-order index) — **done**;
- a typed per-selection fold surface — the standard `Project<Enum>`, one impl per
  selection — rather than a bare marker trait — **done**;
- collision-safe companion-name generation (a per-projection module of enums) — **done**.

Done since: `trybuild`/UI tests now pin each derive's parser diagnostics
(`crates/eventric-model/tests/ui/`), and `Act::Err` defaults to `Report<Error>` with
a custom-`Err` action exercised in `tests/enact.rs`. Still open, independent of the
redesign:

- **Generated child-module re-rooting is head-only — sufficient because the bug it
  would have is unreachable.** Both derives put a type in a `mod <snake>` and
  `super::`-prefix relative paths only at the head, not inside angle brackets — so a
  *relative* generic argument would emit `super::Foo<LocalType>` with `LocalType`
  unresolved in the child. But no such argument can arise: events are `#[revisioned]`
  concrete types and the derives don't support generic projections, so a relative path
  in a child module never carries a generic argument. (A `use super::*` in the child
  modules — to drop the `enum_field`/`projection_field` helpers — was tried and
  reverted: the glob pulls the parent's `use derive_more::Debug` in, making the
  generated `#[derive(Debug)]` ambiguous.) Revisit (re-root recursively into
  `PathArguments`) only if generic events/projections are ever supported.
- **Action child-module vs Projection companion-module name clash.** Both derive a
  `mod <snake_case_ident>` in the same namespace; an action and a projection (or two
  actions) whose idents share a snake_case form collide with an opaque rustc error.
  Very unlikely (an action is a command, a projection a read-model — rarely the same
  name), and the same class as two same-named projections. Fix if it ever bites:
  suffix the action's internal module (it's never named by user code).
- **`#[action(projections: {})]` (no projections) doesn't compile.** A
  precondition-free action — one that just appends, reading nothing — is legitimate
  (and the natural target of a reaction's `IssueCommand`), but the generated `update`
  emits an `Option` the empty projection set never pins (`E0282`), and `select`
  binds an unused-`mut` empty `Vec`. Fix: emit empty-friendly `select`/`update`
  bodies when there are no projections. Until then a command's action needs at least
  one projection (the reactions Phase-B test gives its action a real capacity
  projection, which is the more realistic shape anyway).

## 3. Public surface & lints

- **Whether to curate the public surface.** The strict de-lift means the public
  paths mirror the module tree exactly, including deep ones like
  `eventric_stream::stream::operate::select::TypeSelector`. Left deliberately
  structural (now across two crates) so the surface can be judged with full
  visibility; revisit whether to flatten `operate`'s submodules, add a curated
  prelude/facade, or leave as-is. Also a cross-crate question: model-layer
  consumers currently depend on **both** crates (strict, no re-export facade) —
  decide whether that stays.

## 4. Storage / engine

- **Parallelism / concurrency load testing.** The single-threaded retrieval cost
  is now benchmarked (`benches/select.rs`) and the dense∩sparse intersection
  leapfrogs (≈O(matches·log N)). What is *not* yet measured is behaviour under
  concurrency: many `Proxy` readers + the single writer thread (`stream/concurrent/`)
  under contention — read throughput while appends commit, the bounded write
  channel's backpressure, and how the optimistic DCB append-conflict retry loop
  behaves under write contention. A load/soak test harness (not a microbenchmark)
  would surface lock/channel/IO contention the per-op benches cannot. Wanted, not
  yet scheduled.
- **The timestamp index is write-only** — built and maintained on every append, no
  read path consumes it *yet*. **Decided: keep it** — a timestamp-range query read
  path is anticipated; the index is ready for it. (Not dropped.)
- **Tag count is capped at 255** (the `u8` length prefix in the events keyspace).
  **Resolved:** an append carrying more than 255 tags is now **rejected with an
  error** at `Store::insert` (tested), rather than panicking in the serializer —
  whose cast is now an upstream-enforced invariant.
- **Position-bounded index scans use an exclusive `Position::MAX` upper bound**
  (the same half-open/sentinel pattern as the version-`MAX` quirk), so an event
  at `Position(u64::MAX)` is unreachable via a `from(..)` scan. Marginal —
  `u64::MAX` positions are not practically reachable — but the same class of
  issue.

---

## Done this cycle (for context — not outstanding)

Integration tests (model `enact`, multi-thread `concurrency`, facade
`round_trip`); the `NoTrailingWhiteSpace` validator bug; folding `eventric-utils`
into the crate; error unification + dropping `thiserror`; the multi-selector
two-tools rule (documented + tested + exampled); the crate consolidation; the
re-export de-lift; consolidating examples/benches to one crate each; **the event
`Version` now follows the `revision` schema number** (`versioning.md` §5, tested),
retiring the hardcoded-0 / two-axes items and removing the dead `Version`
`Default` + arithmetic; and a small-wins pass — dropping the unused `include-utils`
dep, a friendlier (version-bearing) `revision`-decode error, and tests pinning
`revision` evolution (old bytes → defaulted field) and the non-empty minimal
payload; and **the content-seam split** (`SPLIT.md`) — `eventric` → `eventric-stream`
(content-agnostic, `revision`-free, compile-enforced) + `eventric-model` (its own
`Error`, `change_context`'d from the stream) + unified `eventric-macros`; and the
**`missing_docs` closure** — every public item across both crates documented,
`#![deny(missing_docs)]` now holding uniformly (no `allow` escapes).
