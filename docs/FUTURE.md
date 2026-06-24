# FUTURE.md

Known-but-unplanned items: things we are *aware* of but have not committed to a
plan for. A living list — add to it as things surface, prune as they land.

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
  so it is hard to justify either way.)
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

## 2. Projection codegen ergonomics (design-pending)

- **Keyed / named selectors** — a designed-but-deferred extension recorded in
  [`docs/keyed-selectors.md`]: name each `select(..)` clause and generate a
  per-projection module of per-selector payload enums + a per-selector-method
  trait. It is a deliberate non-goal until there is a real need.
- **The groundwork that would make it coherent** (rather than a one-off) is the
  real prerequisite, and is independently valuable:
  - make `{Name}Dispatch` carry the methods it implies instead of being a bare
    marker trait;
  - unify companion-name generation into shared, collision-safe codegen (the
    Action `{Name}Context` currently does naive string concatenation with no
    collision handling);
  - give selectors a name/key and **de-positionalise the mask** — the Action
    `update()` routes by `event.mask[i]` keyed purely to declaration order
    (internally consistent today, but fragile to extend).
- No `trybuild`/UI tests exist for any derive: a misuse only surfaces as a
  downstream compile error, not a targeted macro test.
- **The `Act::Err` indirection is untested.** An action may declare a custom
  error type (`Act::Err: From<Report<Error>>`) that `Enactor::enact` returns
  verbatim, but every test/example uses the default `Report<Error>` — the
  custom-error path is plumbed but never exercised.

[`docs/keyed-selectors.md`]: ./keyed-selectors.md

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
