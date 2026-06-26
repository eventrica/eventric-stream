# FUTURE.md

Known-but-unplanned items: things we are *aware* of but have not committed to a
plan for. A living list — add to it as things surface, prune as they land.

Priorities here are weighed against the guiding vision in
[`vision.md`](./vision.md); the forward *roadmap* (what's planned next —
**reactions** as the gating building block, then multi-context composition) lives
there (§9). This file is the *unplanned* residue.

The structural work is **done**: the stream-core rewrite ([`REFACTOR.md`]), the
crate consolidation ([`CONSOLIDATION.md`]), and the content-seam split
([`SPLIT.md`]) are complete — **three crates**: `eventric-stream` (content-agnostic
substrate, no `revision`) + `eventric-domain` (event-sourcing layer, its own
`Error`) + `eventric-macros`, plus `eventric-examples` / `eventric-benches`.
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
wants to think through before building.

- **Orphaned `Version`/`Range` comparison traits (a design decision, not dead
  code).** `impl PartialEq<Range<Self>>` / `impl PartialOrd<Range<Self>>` for
  `Version` (`event.rs`) were deliberately added (commit `7ce9c043`,
  "implementing comparison traits for version range") as a version-range
  primitive, but the filtering that shipped uses stdlib `Range::contains` (the
  in-memory mask re-check), which bypasses them, so they have no caller today.
  The three-way `PartialOrd<Range>` (below / inside / above the range) is
  plausibly the right primitive *if* versioning moves `Version` into the index
  key (enabling a version-keyed range-scan). Decide its fate as part of the
  versioning design: keep it and **pin the semantics with a test + doc** (it has
  neither), or drop both. (`PartialEq<Range>` merely re-spells `Range::contains`,
  so it is hard to justify either way.) **Update:** [`vision.md`](./vision.md) §8
  leans `Version` toward *informational-only* — not a selection dimension — which
  removes the only justification (the version-keyed range-scan) and points toward
  **drop**.
- **A `revision`-mismatch decode failure stays the opaque `Error` type.** It now
  carries an informative attachment (the stored version + the revision this
  consumer handles), so it is *diagnosable*; a distinct error *type/variant* is
  intentionally out of scope (the opaque-`Error` design adds detail via
  attachments, not variants).

### Stream-layer `Version` debt (cheaper, independent of the above)

- **The `MAX` (255) sentinel is unqueryable:** the half-open default range and
  all `VersionSelector` lowerings cap the upper bound at the exclusive
  `Version::MAX`, so version-255 events can be appended but never matched.
- **Untested:** the `a..` / `..b` / `..` range lowerings, the 255 boundary, and
  multi-version OR-ing.

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
(`crates/eventric-domain/tests/ui/`), and `Act::Err` defaults to `Report<Error>` with
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
- **The timestamp index is write-only** — it is built and maintained on every
  append but no read path consumes it. Either expose timestamp-range queries
  (the index is ready) or drop the index to save the write.
- **Tag count is capped at 255** (the `u8` length prefix in the events keyspace)
  and **panics** if exceeded. Decide: document it as a hard limit, return an
  error instead of panicking, or widen the prefix.
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
(content-agnostic, `revision`-free, compile-enforced) + `eventric-domain` (its own
`Error`, `change_context`'d from the stream) + unified `eventric-macros`; and the
**`missing_docs` closure** — every public item across both crates documented,
`#![deny(missing_docs)]` now holding uniformly (no `allow` escapes).
