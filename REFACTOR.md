# Stream Core Refactor — status & map

> Reconstructed from a deep pass over the code and git history (analysis date:
> 2026-06-22). This documents the in-progress rewrite of the stream core: what's
> original, what's new, *why*, and exactly where the work stopped. For the live
> API and overall workspace, see [`CLAUDE.md`](./CLAUDE.md).

## Progress

The analysis/map sections below describe the **starting state** as of the
analysis date (e.g. they describe `references` as a still-present, write-only
keyspace). Completed phases since then — consult this list (and git) for the
current state:

- **Phase 0 — ✅ done (2026-06-22).** Persisted hashing switched to the stable
  seeded rapidhash (`utils::hashing::hash`); on-disk hash values pinned by a test.
- **Phase 1 — ✅ done (2026-06-22).** `references` keyspace **deleted outright**
  (`store/references.rs` removed; no field/open/insert/re-export remain) and the
  `event_new` representation lattice collapsed to a single `String → u64` hop
  (the `(u64, String)` rung is gone from `event_new.rs` and `operate/select.rs`).
  A `pub(crate)` smoke test in `store.rs` now exercises the new insert→iterate
  round-trip. **So `references` no longer exists anywhere in the new tree** —
  wherever the map below calls it "write-only/retained", read "removed".
- **Phase 2 — ✅ done (2026-06-22).** The query surface. Unified masked
  `select(Condition)` (chosen over a separate `select_multiple`): a `Condition`
  holds an optional `from` position plus a list of `Selection`s (empty = full
  scan), and results are `EventAndMask { event, mask }` where `mask[i]` records
  whether selection `i` matched. The index does the coarse union of all
  selectors; the per-selection mask is re-checked in memory on the `u64` hashes.
  Public builders added (`Condition`/`Selection`/`Selector::types`/
  `types_and_tags`/`TypeSelector::new`/`with_versions`, wiring in the previously
  dead `VersionSelector`), `Stream::builder(path)` is now public, and `size_hint`
  was restored on `AndIter`/`OrIter`. **Known limitation (flagged, not changed):**
  version ranges are half-open `Range<Version>`, so `Version::MAX` (255) is not
  selectable — a pre-existing design point shared with the old tree; revisit if
  255 must be a usable version (would need inclusive ranges).
- **Phase 3 — ✅ done (2026-06-22).** Conditional (DCB) append. `append` now
  takes a `Condition` (reusing the Phase 2 type): empty selections = unconditional
  append; non-empty = reject iff a matching event exists at or after
  `condition.position` (from-inclusive; `None` = whole stream), with a head
  shortcut. Rejection returns an `Error` report carrying a downcastable `Conflict`
  marker (the new `Error` is an opaque ZST — there is no `Concurrency` variant, so
  concurrency is signalled via `report.downcast_ref::<Conflict>()`). Existence is
  checked over the index positions only (`Store::matches`), no event materialized.
  **Note:** the old tree's no-selection *positional* concurrency check ("fail if
  the stream grew at all") was intentionally dropped — DCB conditions are
  selection-scoped; empty selections now mean unconditional.
- **Phase 4 — ✅ done (2026-06-22).** Public surface. `Stream::builder(path)`
  landed in Phase 2; this phase added the **Reader/Writer split** the multi-thread
  wrapper is built on: `Stream::split() -> (Reader, Writer)`, `Reader` (cloneable
  read-only handle, impls `Select`), `Writer` (unique write handle, impls
  `Append`), and `From<Writer> for Stream` to recombine. `Store` + its sub-stores
  are now `Clone` (cheap fjall keyspace handles). The read surface is just
  `Select` (full scan = `select(Condition::new())`, so no separate iterate trait);
  the write surface is `Append`. Thread bounds (`Reader: Send + Sync + Clone`,
  `Writer: Send`) are pinned by a compile-time test. Verified that the split
  provides everything the old multi-thread crate needs, so the Phase 6 cutover is
  a mechanical re-point.
- **Phase 5 — ✅ done (2026-06-22).** Tests & docs. The new tree's behavioral
  coverage was built incrementally across Phases 1–4 (combinators, insert/iterate,
  masked query, conditional append, split — each adversarially verified), so
  Phase 5 closed the remaining gaps: a **persistence/re-open** test (data + the
  `next` cursor recovered after drop), a **full-scan with `from`** test, and it
  caught & fixed a latent **empty-append underflow** (`append([])` on an empty
  stream underflowed `*next - 1`; now a clean error — the old tree carried this as
  an open TODO). 367 tests pass. **Comprehensive rustdoc + restoring
  `#![deny(missing_docs)]` is folded into Phase 6**, where the modules take their
  final names: writing docs now would partly churn at the rename, and `core`
  currently relaxes that lint, so it isn't gating.

- **Phase 6a — ✅ done (2026-06-22).** Public read surface. Added `#[must_use]`
  accessors so consumers read queried events without naming internal types:
  `Event::{data,facets,meta}`, `Facets::{ty,tags}`, `Type::{name,version}`, and
  `stream_new::Facets::{position,timestamp}`. This let the model match event types
  by comparing `Name<u64>` values (same stable hash) with no raw-hash exposure.

- **Phase 6b — ✅ done (2026-06-22).** Ported `eventric-stream-multi-thread` to
  `split`/`Reader`/`Writer`/`Append(Condition)`/`Select(Condition)`; error rewrite
  to `Report<Error>` channel payloads. **Found & fixed a pre-existing concurrency
  bug**: the proxy used a non-blocking `oneshot::try_recv` that races the writer —
  and blocking `recv` wasn't even compiled in (oneshot 0.2.1 has no default `std`
  feature). Enabled oneshot's `std` feature + switched to blocking `recv`; the
  `stream` example now round-trips correctly.

- **Phase 6c — ✅ done (2026-06-22). The big one — coordinated cutover (approach A).**
  The facade chokepoint couples 6c and the 6d facade re-point (the model macros emit
  `::eventric_stream::…` paths), so they moved together: re-pointed the facade to the
  new surface AND ported every facade consumer in one green step. Scope: model core
  traits + `Enactor` (error_stack end-to-end; `Recognize` by `Name<u64>` hash; one
  `Condition`, select-then-conditional-append), model macros (codegen for the new
  selectors/recognize/update), model example, stream example, profiling examples,
  benches. Made the new event types `Clone` (benchmarks clone a prebuilt array).
  Both examples run correctly end-to-end through the facade; build + clippy
  `-D warnings` + tests all green. Adversarially reviewed (no correctness bugs;
  DCB conflict logic, hash consistency, and mask/dispatch reuse all confirmed sound).
  - **⚠ Coverage gap:** the old facade integration tests (`crates/stream/tests/` —
    `append`, `append_query`, `iterate`, `properties`, ~1600 lines) tested the
    removed old API and were **deleted** (6e's deletion, pulled forward by the
    facade flip). The new tree has core unit tests, but the **facade now has no
    end-to-end integration tests**. Re-creating integration/round-trip tests
    against the new facade is a recommended near-term follow-up.
  - **Note (error model consequence):** business-rule errors in actions are now
    `Report::new(Error).attach("…")` — attachments on the stream `Error` ZST. Works,
    but a business violation riding a stream-error context is a touch muddy (the
    accepted trade-off of error_stack-end-to-end / option B).
  - **6d is effectively absorbed** into 6c (the facade is re-pointed). Remaining:
    **6e** (rename `stream_new`→`stream`, `event_new`→`event`; delete the now-dead
    old tree) and **6f** (docs + `#![deny(missing_docs)]`).

### Deferred extension — stream-level "fail if grown" concurrency

A coarse "fail if the stream has grown since position P" guard is **not currently
expressible** and is **deliberately deferred** until there's evidence it's needed
(it does not block the cutover — it's cleanly addable later).

Why it isn't expressible today: every `Selector` is anchored on specific
type-name(s) (`TypeSelector` carries a concrete `Name`; the index is scanned
per-name), there is no match-all/wildcard selector, and an empty selector set
matches *nothing* (not everything). Tags are always an AND-refinement of a type
selection, never standalone. So you cannot write an "any event" condition.

If it turns out to be needed, the clean way is **a positional guard on append**
(option a): an optional "expected head" position on `Condition` (or a separate
arg) that rejects when `next` exceeds it — one cursor comparison, no index scan.
This is the honest representation ("fail if grown" is a *positional* question, not
a content query) and is exactly what the old tree did (`after + 1 < next`). A
true match-all selector (option b) was considered the wrong tool — heavier and a
poor fit for a positional question.

## TL;DR

There are **two parallel implementations of the stream core living side by side**:

| | Original (live, shipping) | New (in-flight rewrite) |
|---|---|---|
| Stream | `stream.rs` + `stream/` | `stream_new.rs` + `stream_new/` |
| Events | `event.rs` + `event/` | `event_new.rs` |
| Status | consumed by facade, model, multi-thread, all tests/examples | compiled & `pub` in core, but **re-exported by nothing, consumed by nothing** |

Both are declared in `crates/stream/core/src/lib.rs` (`pub mod event; pub mod
event_new; pub mod stream; pub mod stream_new;`). The crate **compiles cleanly**.
The new code is reachable only via the core crate directly — the `eventric-stream`
facade still re-exports only the *original* modules, so no consumer sees `_new`.

**The driver:** you decided that **queries don't need to return the original
identifier/tag string values** — only their `u64` hashes. This lets the read path
drop an entire dereference round-trip (the `references` reads, the `Cache`, the
per-query `Lookup`, and the `Retrieve` hydration). That decision is fully realised
in the new code; what remains is mostly the query-construction surface.

**Where you stopped:** mid-way through the **select/query surface**.
`stream_new/operate/select.rs` is the most-recently-touched file. The literal next
step is the marker you left in `stream_new/operate.rs:21`:

```rust
pub struct Selection {
    pub(crate) selectors: Vec<Selector<u64>>,
    // ALSO NEED A MASK HERE
}
```

There is also **no public way to build a query or a Stream yet** — `Condition`/
`Selection` fields are `pub(crate)`, no constructors/builders exist, and the new
`Stream` has no public `builder()`. So `Stream::select` can't be called externally
yet.

---

## The driver: the hashing / dereference change

This is the heart of the rework. Both designs store **only `u64` hashes** of
identifiers and tags in the `events` and `indices` keyspaces, and keep a
`references` keyspace mapping `hash → original string`.

**Original read path** *resolves hashes back to strings*, so a query returns a
fully-hydrated `Event` carrying real `Identifier`/`Tag` values. Two mechanisms:
- `iterate` path — a concurrent `Cache` (`DashMap<…Hash, …>`, `stream/iterate/cache.rs`)
  backed by the `references` keyspace; `Retrieve` (`stream/iterate/iter.rs`) hydrates
  each event (≈ N+1 reference reads per event on a cold cache).
- `select` path — an in-memory `Lookup` (`stream/select/lookup.rs`) pre-populated
  **only from the values named in the query itself**; it does *not* read `references`.
  (Consequence: returned events can only resolve identifiers/tags that appeared in
  the query — other tags are silently dropped. The persisted `references` store is
  bypassed entirely on this path.)

**New read path** *never resolves hashes*. `SelectIter`/`StoreIter` yield
`Event<Facets, u64>` — the type-name and tags are bare `u64` hashes.
`store/events.rs::EventReader` reconstructs `Name(u64)`/`Tag(u64)` directly from
stored hashes. **There is no `Cache`, no `Lookup`, no `Retrieve`.** The
`references` keyspace still exists and is still written on every append, but it is
**write-only** — `store/references.rs` exposes only `insert`, no read/dereference.

### Why
- Type-names and tags are only meaningful as *match keys*; the caller supplied the
  query strings and already knows the mapping, so returning hashes is sufficient.
- It deletes an entire class of hot-path work: per-result reference reads, the
  DashMap cache, the per-query lookup, string allocations, and the `Retrieve` code.
- `references` is retained purely as a write-time record (future reverse-resolution,
  debug tooling, or dedup) but is decoupled from reads so it can never slow queries.

### Decisions you parked (open questions)
- **Is reverse hash→value resolution ever coming back?** `references` still stores
  the strings but exposes no reader. Either it's for a future/offline resolve API,
  or — under the no-dereference contract — the keyspace could arguably be dropped.
- **How does a caller map returned `u64`s back to strings?** Is the contract "the
  consumer already holds the mapping", or will `Stream` gain a resolve method?
- **Hash portability hazard.** Persisted hashes are computed via `hashing::get`
  (std `DefaultHasher`/SipHash), *not* the stable seeded `hashing::hash` (rapidhash)
  that also lives in the same module. `DefaultHasher` output is **not guaranteed
  stable across Rust versions/platforms**, yet it's used as on-disk keys. This
  predates the rework but is worth resolving before any data is persisted long-term.

---

## Module map: original → new

All paths under `crates/stream/core/src/`.

| Concern | Original | New |
|---|---|---|
| Top-level Stream | `stream.rs` — `Stream`, `Builder`, **`Reader`/`Writer` split**, `split()` | `stream_new.rs` — single `Stream`, `Builder` (no Reader/Writer split) |
| Storage aggregate | `stream/data.rs` — struct **`Data`** | `stream_new/store.rs` — struct **`Store`** |
| Event log | `stream/data/events.rs` | `stream_new/store/events.rs` |
| Inverted indices | `stream/data/indices.rs` + `indices/{identifiers,tags,timestamps}.rs` | `stream_new/store/indices.rs` (**single file**) |
| References (dedup) | `stream/data/references.rs` + `references/{identifiers,tags}.rs` (read+write) | `stream_new/store/references.rs` (**write-only**) |
| Append | `stream/append.rs` — free fns + `Append`/`AppendSelect` traits | `stream_new/operate/append.rs` — `Append` trait on a tuple |
| Query | `stream/select/` (12 files: selector, filter, mask, lookup, prepared, …) | `stream_new/operate/select.rs` (**single file**) |
| Event iteration | `stream/iterate/` — `Iter`, **`Cache`**, `Retrieve` | folded into `store.rs` (`StoreIter`) + `events.rs` (`EventsIter`) |
| Boolean combinators | `utils/iteration/{and,or}.rs` (shared) | `stream_new/iterate.rs` (**forked copy**) |
| Event model | `event.rs` + `event/` (10 files) | `event_new.rs` (**single file**) |

> ⚠️ **`iterate` means different things in the two worlds.** In the original,
> `stream/iterate/` is *event* iteration (with the dereference `Cache`). In the new
> tree, `stream_new/iterate.rs` is the *boolean `AndIter`/`OrIter` position
> combinators* — i.e. a fork of `utils/iteration`. The new event-iteration role
> lives in `store.rs`'s `StoreIter`.

### Vocabulary convergence (the "tidying" theme)
- `Data` → `Storage` → **`Store`**; `operations` → **`operate`**; `storage` → **`store`**.
- `Identifier` → **`Name`**, folded together with `Version` into **`Type<T>`**.
- `Specifier` → **`TypeSelector`**.
- The `(selection, from)` argument pair → a single **`Condition { position, selection }`**.
- `*Converter` structs → directional **`EventReader`/`EventWriter`/`PositionReader`**.
- `std::sync::Exclusive` → **`std::sync::SyncView`** (incidental — rode the nightly bump).
- Error handling → `error_stack` `Report<Error>` (new) vs the crate-wide `error::Error` enum (old).

### The new event model (`event_new.rs`)
Collapses the original's combinatorial type families (`Identifier` +
`IdentifierHash` + `IdentifierHashAndValue`, ditto `Tag`, plus `Specifier` × 3) into
**one generic `Event<M, T>`** where:
- `T` is the *representation* of names/tags: `String` → `(u64, String)` → `u64`,
  with macro-generated `From` conversions that compute the hash (`hashing::get`)
  eagerly at each step. This directly models the encode pipeline: input strings →
  hash+string (for `references` writes) → hash-only (for `events`/`indices`).
- `M` is post-persistence metadata: `()` for a candidate, `stream_new::Facets`
  (= `Position` + `Timestamp`) for a persisted event.
- `Type<T> = (Name<T>, Version)`; `Facets<T> = (Type<T>, BTreeSet<Tag<T>>)`.

Dropped/not-yet-ported from the rich original `event/` tree: the `Specifier`
family, the ergonomic `Range` enum (partly succeeded by `VersionSelector` in
select.rs), `Specification`, all rustdoc, and all unit tests. `Position`/`Timestamp`
were *relocated* into `stream_new.rs` (not reimplemented in `event_new`).

> Naming hazard: there are **two `Facets` types** — `event_new::Facets<T>` (type +
> tags) and `stream_new::Facets` (position + timestamp). They occupy different slots
> of `Event<M, T>` but share a name across modules.

---

## Timeline (git)

| Commit | Date | What |
|---|---|---|
| `bb14725` | Feb 25 | "beginning a tidying-up rework of the stream code" — bootstrapped `stream_new` + `event_new` (the latter **untouched since**) |
| `149ef4c` | Feb 26 | "more reworking" (+975 LOC) — fleshed out `operations` + `storage` + the inverted indices |
| `a27caa3` | Feb 27 | "starting iterate" — added the `AndIter`/`OrIter` boolean combinators |
| `b6bb8e2` | Mar 6 | "flake update" — nightly bump + incidental `Exclusive` → `SyncView` sweep (also touched the legacy `stream/`) |
| `f6264bf` | Mar 7 | "updates" — `Cargo.lock` only |
| **uncommitted** | Mar 9 → Jun 22 | the rename frontier: `operations`→`operate`, `storage`→`store`, `iterate` promoted to top-level; `Storage`→`Store`, `Iter`→`SelectIter`. `operate/select.rs` touched **last (today)**. |

The uncommitted working-tree state (deleted `operations/` + `storage/`, added
`operate/` + `store/` + `iterate.rs`) *is* the in-flight reorg — it's a rename +
reshape, not new functionality.

---

## Per-module completeness (in `stream_new`)

- ✅ **`event_new`** — complete (compiles, fully consumed by the new tree).
- ✅ **`iterate.rs`** (And/Or combinators) — complete, double-ended. *(Lost the
  `size_hint` impls the original `utils` versions had — possibly an accidental
  regression in the move.)*
- ✅ **`store/events.rs`** — complete (open/len/get/insert/iterate + reader/writer).
- ✅ **`operate/append.rs`** — complete (despite being tiny: it's a thin
  concurrency-check + delegation to `Store::insert`). Concurrency is position-only
  (`after >= next`).
- 🟡 **`store/indices.rs`** — tags + types done (incl. version-range filtering at
  read time); **timestamps index is write-only** (inserted, never iterated — no
  `IndicesIter::Timestamps` variant).
- 🟡 **`store/references.rs`** — **write-only** by design (insert only, no reader).
- 🟡 **`operate/select.rs`** — the wiring exists (`Select`, `SelectIter`, `Selector`,
  `TypeSelector`), but **`VersionSelector` is defined and unused** — nothing
  converts it into the `Range<Version>` that `TypeSelector` takes. Staged for the
  not-yet-built query constructor.
- ❌ **The `mask` on `Selection`** (`operate.rs:21`) — not started. In the original,
  `select/mask.rs` + `select_multiple` produce a per-event bitmask of which
  selections matched (used by the model layer's projection dispatch). The new design
  needs an equivalent.
- ❌ **Public query construction** — no builder/constructor for
  `Condition`/`Selection`/`Selector`; fields are `pub(crate)`. Queries can't be
  assembled by an external caller yet.
- ❌ **`Stream::builder()`** — the new `Builder::new` is private; no public entry
  point to open a new `Stream`.
- ❌ **Dropped capabilities** (present in original, absent in new — decide whether
  intentional): `select_multiple`, `append_select`/`append_select_multiple`, and the
  `Reader`/`Writer` split. **The model layer currently depends on `select_multiple`
  + `append_select_multiple`**, so these (or an equivalent) are needed before
  `stream_new` can replace `stream`.
- ❌ Docs + tests for all new code (the original had exhaustive unit tests; the
  commented-out test blocks in `event/*.rs` reference an abandoned by-reference
  hashing design — `*HashRef` — and are dead).

---

## What "done" looks like (suggested cutover checklist)

To make `stream_new`/`event_new` the production path:

1. Finish the query surface: the `Selection` mask, public `Condition`/`Selection`/
   `Selector` constructors (consuming `VersionSelector`), and a `Stream::builder()`.
2. Decide the fate of the dropped capabilities the **model layer needs**
   (`select_multiple` / `append_select_multiple` / mask-based dispatch) — port or
   redesign.
3. Resolve the parked design questions above (reverse resolution? hash stability?
   the two `Facets`?).
4. Re-point the facade (`crates/stream/src/lib.rs`) and `eventric-model` /
   `eventric-stream-multi-thread` from `stream`/`event` to the new modules.
5. Port docs + tests, then delete `stream/`, `event/`, and (if `utils/iteration` is
   no longer used) the original combinators.

---

## Completion plan (decided 2026-06-22)

Three directions are now locked:

- **Masking / multi-selection → ported into `stream_new`** (model keeps its shape).
- **`references` keyspace → dropped entirely.** Consequence: the `u64` hash becomes
  the *only* persisted record of an identifier/tag — strings are never stored. So
  (a) the hash function must be **stable forever** (no re-indexing from strings is
  possible), and (b) the `event_new` representation lattice collapses to
  `String → u64` (the `(u64, String)` stage existed only to feed `references`).
- **Full cutover** — re-point facade + model + multi-thread, delete the old tree,
  rename `stream_new`/`event_new` back to `stream`/`event`.

### Phase 0 — Baseline & hash foundation ✅ done (2026-06-22)
*Small, do first; unblocks everything and de-risks the data format.*
- Commit the current uncommitted reorg as-is (it compiles) so there's a clean
  baseline: `operations→operate`, `storage→store`, top-level `iterate.rs`.
- **Lock the hash function.** Switch persisted hashing from `hashing::get` (std
  `DefaultHasher`/SipHash, *not* portable) to the stable seeded
  `hashing::hash` (rapidhash) — used by `event_new`'s `String → u64` conversion.
  With `references` gone this is load-bearing, not cosmetic. Collapse to one hashing
  path; document the on-disk hash as a stable wire contract.
- **Done when:** baseline committed; all stored hashes derive from rapidhash; a test
  pins a known string→hash value.

### Phase 1 — Drop `references` & simplify the event model ✅ done (2026-06-22)
*Self-contained; shrinks the surface before building on it.*
- Delete `stream_new/store/references.rs`; remove the `References` field from
  `Store`, the `references.insert` call in `Store::insert`, and the
  `TAGS/TYPES_REFERENCE_ID` constants.
- Simplify `event_new.rs`: drop the `(u64, String)` representation and the
  `event_from!`/`facets_from!`/`type_from!`/`string_type!` arms for it. Keep
  `String` (validated input) → `u64` (stored). `Store::insert` now does a single
  `Event<(), String> → Event<(), u64>` hop.
- **Done when:** only `events` + `indices` keyspaces exist; `cargo build -p
  eventric-stream-core` is green; no `(u64,String)` representation remains.

### Phase 2 — Complete the query surface ✅ done (2026-06-22) *(was the frontier — where work stopped)*
- **Mask + multi-selection.** Resolve the `// ALSO NEED A MASK HERE` marker: a query
  is a set of selections; results report which selection(s) matched. Port the old
  `select/mask.rs` + in-memory `Filter` re-check (version-range + tag-subset),
  adapted to hash-only events, on top of the union index scan. Decide the mask shape
  (per-event bitmask over the N selections, as the original `Mask`).
- **Public query construction.** Add constructors/builders for `Condition` /
  `Selection` / `Selector` / `TypeSelector` (currently `pub(crate)`, no builders) and
  wire in the already-defined-but-unused `VersionSelector`. This is what makes
  `Stream::select` callable by a consumer.
- Restore the `size_hint` impls on `AndIter`/`OrIter` lost in the move.
- **Done when:** an external caller can build a multi-selection query, run it, and
  read back events with a correct match-mask; round-trip test passes.

### Phase 3 — Conditional append (DCB parity) ✅ done (2026-06-22)
- Add a selection-based optimistic-concurrency check to append: a `Condition`
  (selection + `after`) fails with a concurrency error iff a matching event exists at
  `after+1..` (an existence-only index scan — the old `Indices::contains`). Today
  append only does position-only `after >= next`.
- Decide whether to keep the old "return a compiled condition for reuse in the next
  append" optimization (`Prepared`) — recommend keeping a compiled-condition form
  since the model reuses the select condition as the append condition.
- **Done when:** conditional append rejects conflicts and passes when clear;
  parity with `append_select{,_multiple}` behaviour the model relies on.

### Phase 4 — Public `Stream` API & the Reader/Writer split ✅ done (2026-06-22)
- Add a public `Stream::builder(path)` (currently `Builder::new` is private).
- **Re-add the Reader/Writer split** (or an equivalent cloneable read handle +
  unique write handle). *The multi-thread crate is built entirely on
  `Stream::split() → (Reader, Writer)`* — reads-scale / writes-serialize — so cutover
  needs it. Settle the public trait surface (`Append` + conditional, `Select`,
  iteration).
- **Done when:** `Stream` opens via a public builder and splits into read/write
  handles; the trait surface the model + multi-thread need is present.

### Phase 5 — Tests & docs for the new tree ✅ done (2026-06-22) — docs deferred to Phase 6
- Port the suites (`append`, `append_query`, `iterate`, `properties`) to the new API.
  **Note:** existing tests assert string identifier/tag *values*; with hash-only
  results they must assert on hashes (or positions/masks). Update fixtures.
- Add rustdoc (the new modules have none; core relaxes `missing_docs` but the facade
  denies it). Delete the dead commented-out `*HashRef` blocks.
- **Done when:** `cargo test -p eventric-stream-core` covers append/query/iterate/
  concurrency on the new API; docs build clean.

### Phase 6 — Cutover (detailed plan, decided 2026-06-22)

**Strategy: phased, migrate-then-rename.** This is the only safe path — the old
`crate::error::Error`, the old `stream::{append,iterate,select}` traits,
`CandidateEvent`, and `Prepared*` are load-bearing for the model, multi-thread,
*and* the integration tests, so they can't be deleted atomically. Keep the old
tree alive; port each consumer onto `stream_new`/`event_new` (still under those
names) and verify it; do the destructive rename + delete **last**.

**Locked decisions:**
- **Errors → error_stack end-to-end.** The model adopts the stream's
  `Report<Error>` throughout: model trait + macro signatures return
  `Result<_, Report<Error>>`; `validation::Error` (Name/Tag/Data ctors) and
  `revision` (de)serialization failures are attached via `.change_context(Error)`;
  a concurrency conflict is detected by `report.downcast_ref::<Conflict>()`. The
  old crate-level `Error` enum is deleted once nothing references it. (`eventric-model`
  + `eventric-model-macros` gain a direct `error-stack` dep.)
- **`Recognize` matches by hash, not string.** Cache each event type's
  `Name<u64>`/`u64` in a `OnceLock` (mirroring today's `OnceLock<Identifier>`),
  hashed with the **stable `utils::hashing::hash` (rapidhash, seed 0x28112017)** —
  *not* `hashing::get` (DefaultHasher), which would compile but never match.
- **Selections → a single `Condition`.** `Action::Select` yields `Vec<Selection>`;
  `Enactor` builds `Condition::new().from(after).selections(...)` and calls
  `select(condition)` then `append(events, condition)`. No `Selections` type,
  no `Prepared` reuse.
- **Facade: hybrid layout** — flat `stream` + an `event` module (+ `tag!`); drop
  the `append`/`iterate`/`select` submodules. **Rename `stream_new::Facets` →
  `Metadata`** to break the clash with `event_new::Facets`.

**Sub-phases (each independently verifiable):**

> **Status (2026-06-22): 6a ✅, 6b ✅, 6c ✅ (see Progress for outcomes).** Two
> deviations from this original plan, both deliberate: (1) **6d is absorbed into
> 6c** — the facade had to be re-pointed as part of the coordinated cutover
> (approach A), since the model macros emit `::eventric_stream::…` paths. (2) The
> **`stream_new::Facets` → `Metadata` rename is NOT needed** — the 6a accessors let
> consumers reach position/timestamp/type without ever naming `Facets`, so neither
> `Facets` is exposed and there's no clash. The old integration tests (6e's
> deletion) were also pulled forward (the facade flip obsoleted them). Remaining:
> **6e** (rename `*_new` → final names; delete the now-dead old tree) and **6f** (docs).

- **6a — ✅ Public read surface (prerequisite).** The queried `Event<Facets,u64>` has
  no public accessors; add them (data, position, timestamp, tags, **type-name
  hash**) and expose a **stable string→hash entry point**. Without this the model
  cannot compile or match events. Required new public API.
- **6b — Port `eventric-stream-multi-thread`** to `split`/`Reader`/`Writer`/
  `Append(Condition)`/`Select(Condition)`. Mechanical except the error rewrite
  (`Error::general(..)` → `Report::new(Error).attach(..)`; channel payloads become
  `Result<R, Report<Error>>`; add `error-stack` dep; pass `Report` verbatim +
  re-export `Conflict`; drop `Iterate`). Verify build + the `stream` example.
- **6c — Port `eventric-model`** (the big one): error_stack end-to-end; `Recognize`
  hash matching; selector codegen (`specifiers`→`types`, `Specifier`→`TypeSelector`,
  drop the `?` on the now-infallible `Selection::new`); `Events` buffer builds
  `Event<(),String>` not `CandidateEvent`; `DispatchEvent.identifier` String→hash;
  rewrite `Enactor::enact` (one `Condition`, `select` then `append`). Verify the
  `course_subscriptions` example end-to-end.
- **6d — Re-point the facade** (`crates/stream/src/lib.rs`): hybrid layout; expose
  the new surface; rename `stream_new::Facets` → `Metadata` in core first.
- **6e — Rename + delete:** `stream_new`→`stream`, `event_new`→`event`; delete old
  `stream.rs`+`stream/`, `event.rs`+`event/`, the now-dead `utils/iteration`, and
  `error.rs`; delete the old integration tests (behavior covered by the in-module
  tests from Phases 2–5). Update `lib.rs` mod decls.
- **6f — Restore docs + `#![deny(missing_docs)]`** against the final names.

**Carried-forward narrowings (not blockers):** `Prepared`-reuse is gone (the append
re-builds its `Condition`); empty selections no longer validate-as-error; the
`Version::MAX` half-open edge.

- **Done when:** no `_new` suffixes remain; old `stream/`, `event/`, `error.rs`,
  `utils/iteration` deleted; model + multi-thread + facade compile against the
  single implementation; the examples run.

### Phase 7 — Verify & land
- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D
  warnings`, `cargo fmt`.
- Run examples end-to-end: `course_subscriptions` (model layer), `stream`
  (Owner/Proxy), the profiling examples; confirm benches run.
- **Done when:** workspace is green, examples run, and the two-trees state is gone.

### Finer decisions folded in (recommended defaults)
- Hashing: one stable rapidhash path (Phase 0).
- Compiled-condition reuse (`Prepared`): keep a compiled form (Phase 3).
- Reader/Writer split: re-add to the new `Stream` (Phase 4) — required by multi-thread.
- Model dispatch: match by hash / mask, not identifier string (Phase 6).
- Combinators: one copy in `utils/iteration`; delete the fork (Phase 6).

### Suggested commit sequence
P0 baseline + hash → P1 drop references → P2 query surface → P3 conditional append →
P4 public API/split → P5 tests/docs → P6 cutover (model, multi-thread, facade, rename)
→ P7 verify. Phases 0–5 keep the old tree intact and shippable; only Phase 6 flips
the consumers, so everything before it is low-risk and independently committable.
