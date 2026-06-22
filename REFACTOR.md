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

### Phase 2 — Complete the query surface *(the frontier — where work stopped)*
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

### Phase 3 — Conditional append (DCB parity)
- Add a selection-based optimistic-concurrency check to append: a `Condition`
  (selection + `after`) fails with a concurrency error iff a matching event exists at
  `after+1..` (an existence-only index scan — the old `Indices::contains`). Today
  append only does position-only `after >= next`.
- Decide whether to keep the old "return a compiled condition for reuse in the next
  append" optimization (`Prepared`) — recommend keeping a compiled-condition form
  since the model reuses the select condition as the append condition.
- **Done when:** conditional append rejects conflicts and passes when clear;
  parity with `append_select{,_multiple}` behaviour the model relies on.

### Phase 4 — Public `Stream` API & the Reader/Writer split
- Add a public `Stream::builder(path)` (currently `Builder::new` is private).
- **Re-add the Reader/Writer split** (or an equivalent cloneable read handle +
  unique write handle). *The multi-thread crate is built entirely on
  `Stream::split() → (Reader, Writer)`* — reads-scale / writes-serialize — so cutover
  needs it. Settle the public trait surface (`Append` + conditional, `Select`,
  iteration).
- **Done when:** `Stream` opens via a public builder and splits into read/write
  handles; the trait surface the model + multi-thread need is present.

### Phase 5 — Tests & docs for the new tree
- Port the suites (`append`, `append_query`, `iterate`, `properties`) to the new API.
  **Note:** existing tests assert string identifier/tag *values*; with hash-only
  results they must assert on hashes (or positions/masks). Update fixtures.
- Add rustdoc (the new modules have none; core relaxes `missing_docs` but the facade
  denies it). Delete the dead commented-out `*HashRef` blocks.
- **Done when:** `cargo test -p eventric-stream-core` covers append/query/iterate/
  concurrency on the new API; docs build clean.

### Phase 6 — Cutover
- **Model** (`eventric-model`): re-point `select_multiple` → new multi-selection
  select, `append_select_multiple` → new conditional append, `EventAndMask`/`Mask` →
  the new mask. **Key change:** `Recognize` matches events by comparing identifier
  *strings* today — switch to hash comparison or pure mask-based dispatch, since
  results are now hash-only. Repoint `Selector`/`Specifier` → `Selector`/`TypeSelector`.
- **Multi-thread**: re-point to the new `Stream` + Reader/Writer (Phase 4); update the
  `Operation` enum's `Prepared`/`PreparedMultiple` payloads to the new condition type.
- **Facade** (`crates/stream/src/lib.rs`): re-export the new modules; fix the `tag!`
  macro's expansion path if it changes.
- **Rename & delete:** `stream_new` → `stream`, `event_new` → `event` (delete originals
  first, then rename); update `lib.rs`. Consolidate the `stream_new/iterate.rs` fork
  back into `utils/iteration` (one copy) and delete the duplicate.
- **Done when:** no `_new` suffixes remain; old `stream/`, `event/` deleted; all
  consumers compile against the single implementation.

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
