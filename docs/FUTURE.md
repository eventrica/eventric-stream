# FUTURE.md

Known-but-unplanned items: things we are *aware* of but have not committed to a
plan for. A living list — add to it as things surface, prune as they land.

The structural work is **done**: the stream-core rewrite ([`REFACTOR.md`]) and
the crate consolidation ([`CONSOLIDATION.md`]) are complete (one `eventric`
library + one `eventric-macros` proc-macro crate, plus `eventric-examples` /
`eventric-benches`), the error model is unified on `error-stack`, and the public
surface carries no re-export lifts. Nothing below blocks that; these are the
deferred design decisions and the smaller debt.

[`REFACTOR.md`]: ./archived/REFACTOR.md
[`CONSOLIDATION.md`]: ./archived/CONSOLIDATION.md

---

## 1. Versioning — the biggest open area (design-pending)

Two orthogonal versioning axes exist; neither is fully realised or documented.
The maintainer wants to think the overall story through before implementing. A
full research-grounded exploration — the theory, the production-framework prior
art, the DCB-specific picture, and how it could fit `eventric` (with a suggested
order of work) — is in [`versioning.md`](./versioning.md).

- **The model can't set the type `Version`.** `Events::append` hardcodes
  `Version::default()` (= 0) for every event, and `#[derive(Event)]` has no
  version attribute — so type-versioning is plumbed in the stream layer but
  unreachable from the model UX, and every model query spans `MIN..MAX`. Decide
  whether to expose it (e.g. `#[event(version = N)]` threaded through append +
  the read-side `Specifier`/`TypeSelector`).
- **`revision` payload evolution is unexercised.** The in-place, lenient
  schema-evolution capability that is the entire reason `revision` was chosen is
  never tested or demonstrated — every `#[revisioned(...)]` in the repo is
  `revision = 1`. No test decodes `revision = 1` bytes with a `revision = 2`
  type, added/defaulted fields, or a `convert_fn`.
- **The two axes are uncoupled and undocumented.** Type `Version` (indexable,
  queryable via `with_versions`) vs `revision` (opaque payload bytes) are
  orthogonal; the model uses the latter and pins the former to 0. Decide the
  intended relationship and document it (they currently overlap conceptually in
  the docs without the distinction being stated).
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
- **Edge of an empty payload:** a revisioned struct that serialises to zero
  bytes would hit `Data::new`'s non-empty check and fail to append — untested.
- **A `revision`-mismatch decode failure maps to the opaque `Error`** with only
  a string attachment, so it is indistinguishable from any other error and would
  surface mid-projection-replay.

### Stream-layer `Version` debt (cheaper, independent of the above)

- **The `MAX` (255) sentinel is unqueryable:** the half-open default range and
  all `VersionSelector` lowerings cap the upper bound at the exclusive
  `Version::MAX`, so version-255 events can be appended but never matched.
- **Untested:** the `a..` / `..b` / `..` range lowerings, the 255 boundary,
  multi-version OR-ing; and `Version`'s `Add`/`Sub` panic on overflow/underflow.

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
  `eventric::stream::operate::select::TypeSelector`. Left deliberately structural
  so the surface can be judged with full visibility; revisit whether to flatten
  `operate`'s submodules, add a curated prelude, or leave as-is.
- **Close the `missing_docs` gap.** The `model` and `stream::concurrent` modules
  carry a local `#[allow(missing_docs)]` (preserving the old model-core /
  multi-thread posture). Documenting their public items would let
  `#![deny(missing_docs)]` hold uniformly across the crate.

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

## 5. Hygiene

- **`include-utils` is an unused workspace dependency** — it was only used by the
  old `eventric-stream` facade for a README `include_md!`; the consolidated crate
  uses a plain crate doc. Drop it from `[workspace.dependencies]`.

---

## Done this cycle (for context — not outstanding)

Integration tests (model `enact`, multi-thread `concurrency`, facade
`round_trip`); the `NoTrailingWhiteSpace` validator bug; folding `eventric-utils`
into the crate; error unification + dropping `thiserror`; the multi-selector
two-tools rule (documented + tested + exampled); the crate consolidation; the
re-export de-lift; consolidating examples/benches to one crate each.
