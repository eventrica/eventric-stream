# Event versioning — a design exploration

**Status: EXPLORATION, partially landed.** This is a research-grounded survey of
the design space plus the conclusions of an adversarial review of it. One piece
(§5, "version follows revision") is **implemented**; the rest is deliberately
*not* built, and some of it is argued *out*. The parked items live in
[`FUTURE.md`](./FUTURE.md) §1.

This document was rewritten after review. An earlier draft made two concrete
errors (a broken index encoding, an overstated exhaustiveness guarantee) and
oversold a "support everything" reconciliation; those are corrected **explicitly**
below (§7) rather than quietly dropped.

---

## 0. The framing that actually matters

Event sourcing splits schema change into *non-breaking* (an old reader can read
new data) and *breaking* (it can't). But that split is **not** the useful design
axis here, for one verified reason: **the `revision` crate is itself a
field-level upcaster.** With `convert_fn`/`default_fn` it runs explicit transform
code across a struct's revisions — including renames and type changes, which are
*breaking* in the tolerant-reader sense yet trivially transformable. So
"breaking/non-breaking" cuts across "transformable/not."

The axis that actually decides the design is:

> **Does an honest, total function from the old data to the new exist?**
>
> - **Yes** → transform it. Today that means a `revision` `convert_fn` (and, by
>   §5, the schema revision *is* the event's `Version`). No new event.
> - **No** → the information is genuinely absent, lossy, non-deterministic, or
>   needs external context. **This is the only case that forces a new event**
>   (a new identifier — a genuinely different fact).

Everything else (breaking vs non-breaking, type vs version, upcast vs not) is
downstream of that one question.

---

## 1. Research landscape (cited)

The one peer-reviewed treatment, Overeem, Spoor & Jansen, *"The Dark Side of
Event Sourcing"* (SANER 2017) [SANER2017], names five breaking-change techniques
— *multiple versions, upcasting, lazy transformation, in-place transformation,
copy-and-transform* — and rates **upcasting** the preferred run-time technique,
while rating **"multiple versions"** (keep every version's bytes, discriminate by
a version number — i.e. a queryable `Version` field) the **least maintainable**,
because *"the support of multiple versions is spread throughout the application
code."*

Production frameworks converge, unanimously, on **read-time upcasting** and
deliberately do **not** surface multi-version reads: Greg Young [Young/InfoQ],
Axon (`@Revision` + chained upcasters) [Axon], Marten (lazy JSON upcast on read)
[Marten], Commanded (upcast old type → new type) [Commanded]. Confluent's Schema
Registry [Confluent] formalises the compatibility ordering (BACKWARD/FORWARD/FULL)
that bounds the *non-breaking* envelope — i.e. what `revision` absorbs without an
explicit `convert_fn`.

The two takeaways that bear on us: (a) the industry treats "transform to one
current version" as the default and "expose multiple versions" as the
anti-pattern; (b) but they upcast because they're aggregate-replay stores — none
is a *selection-first* store, which is where our model differs (§3, §7).

---

## 2. DCB is silent — `Version` is our own extension

Verified against the spec, the reference implementation, and Savić's canonical
article: **DCB has no notion of event version.** An event is *only* type + opaque
data + tags; type is *purely* a selection key; the sole ordinal is sequence
position. [DCBspec, DCBimpl, Savic] So `eventric`'s `Version` is a genuine
extension with no precedent — freedom and risk. (One bonus: SANER's "upcasting
isn't cross-stream-complete" objection doesn't apply, because DCB has no
per-aggregate streams to keep independent.)

---

## 3. What the review concluded

**`revision` already *is* the upcaster, so a separate upcaster/registry is
largely redundant.** Walking the cases:

- **Transformable** → `revision`'s `convert_fn` already turns old bytes into the
  current struct inside `from_slice`; the projection's `Project<Current>` already
  sees the latest. A *separate* cross-version upcaster would only be needed if you
  modelled versions as *distinct* structs — which is the very "multiple versions"
  path SANER ranks worst.
- **Transform needs external context** → an anti-pattern (non-deterministic
  upcasts break replay reproducibility); it's a signal to remodel, not to build a
  context-aware upcaster.
- **Untransformable** → a new event (new identifier). No upcaster.

And the unknown-version *guard* is half-built already: `revision::from_slice`
**already fails** on a future revision it can't decode. The only gap is that the
error is opaque; making it informative ("saw revision N, this build knows ≤ M") is
a small fix, not a registry.

**Most "versioning needs" are modelling smells.** A large fraction are avoidable
up front:

| Change | Honest transform? | Tool | Avoidable by modelling? |
|---|---|---|---|
| Add optional/defaulted field | yes | `revision` | partly (fat, intentful events) |
| Rename / change type of a field | yes (`convert_fn`) | `revision` | mostly (names are opaque IDs) |
| Change unit (°F→°C) | yes (`convert_fn`) | `revision` | **yes** — capture the unit explicitly |
| Split a conflated event | no | new event; cope with history | **yes** — one fact per event |
| Add genuinely-new info (no honest default) | no | new event | **yes if foreseeable**, **no** if a real domain change |
| Reinterpret the concept | no | new event | **yes** — it was mis-modelled |

The residue that modelling **cannot** remove: foreseeable evolution (→ `revision`)
and *genuine, unforeseeable* domain change (→ new event). You can't model away the
future, but you can shrink the problem to those two.

**The distribution framing is the real crux.** Every genuinely hard part of
versioning is downstream of *"a reader can encounter an event written by a newer
schema than it knows."* Remove that and the difficulty evaporates: one source of
truth for types ⇒ no unknown versions ⇒ `revision` decodes everything and
compile-time checks over your declared types suffice. The registry / runtime
guard / version-index machinery are **"reader-lags-writer" machinery, not
versioning machinery.** Note the dangerous invariant is narrower than
"distributed": it's *"reader schema ≥ every writer schema, always,"* which is
broken by **rollbacks, blue-green deploys, and mixed-version replicas** even in a
single service. The first real decision is whether `eventric` must tolerate that.

---

## 4. The cost `revision` still carries

`revision` being the upcaster means it **inherits upcasting's costs**: `convert_fn`
chains grow without bound (you carry every historical conversion forever), each is
read-path work and a testing obligation — the exact maintainability tax SANER pins
on run-time techniques. So "revision handles it" is clean today and gets heavier
over a long-lived stream. The cheapest long-term lever is therefore the modelling
discipline in §3: every breaking change you model away is a `convert_fn` you never
carry.

---

## 5. LANDED: the event `Version` follows the `revision` number

`Events::append` no longer hardcodes `Version::default()`; it sources the version
from `E::revision()` (the `revision` schema number, reachable via the
`SerializeRevisioned` supertrait), capped into the `u8` `Version` (erroring rather
than truncating past 255). Consequences:

- **The divergence risk is gone** — there's no separate version to declare or
  forget to bump; `Version` is a faithful function of the schema.
- **The "two orthogonal axes" problem dissolves** — schema revision and stream
  `Version` are now *one* notion. `Version` means exactly "the schema revision
  this event was written at," and bumps on every revision change (breaking or
  not). You lose the ability to set a "logical version" independent of the schema
  — and that loss is the point.
- The dead `Version::default()` and the unused `Version` arithmetic
  (`Add`/`Sub`/…) were removed in the same change. (`MIN`/`MAX` stay; the
  `PartialEq`/`PartialOrd<Range>` impls stay — their fate is the deferred
  *selection* decision in §7.)

This is the one piece worth having built ahead of the rest: small, divergence-
proof, and it makes `Version` mean something real for free.

---

## 6. The serialisation seam, and a deferred strategic axis

`revision` is currently named directly in the model (`to_vec`/`from_slice`, the
trait bounds). The recommended boundary is a thin **eventric trait over
serialisation** (serialise + deserialise + the revision number), with `revision`
as the implementation behind it. That's the right altitude regardless (the model
shouldn't know `revision`'s exact API), and it makes a future format swap a
contained change rather than a rewrite.

**Deferred strategic axis — owning the on-disk format.** A foundational event
store arguably *should* own its wire format (stability, no external breakage, full
control forever). That is a real but **big, deliberate, format-lock-in decision**,
to be taken on its own merits if/when format control becomes a requirement — **not**
pulled in by the divergence fix (§5, done with the external crate) or by a
dependency worry. For the record, the worry that prompted this — that `revision`
drags in an orphaned `bincode` — is **false**: `revision` 0.28 depends only on
`revision-derive`; `bincode` is absent from the dependency tree entirely. The seam
keeps the build-our-own option open without paying for it now.

---

## 7. The hard parts — corrected

### 7.1 Version-in-index for selection — the earlier encoding was broken

The earlier draft proposed a `types` key of `[name_hash][version][pos]` and a
"prefix scan bounded by the version byte." **That is wrong:** it sorts
version-major, so interleaved versions yield positions **out of order**, violating
the ascending-position invariant the AND/OR combinators depend on. Concretely,
positions written as (v1,5),(v2,6),(v1,7) would scan as 5,7,6.

The salvage is a **union of per-version sub-scans**: each `[name_hash][v][pos…]`
*is* position-ordered for a fixed `v`, so a version range becomes up to 255
single-version scans merged by the existing `Union` combinator (a sorted merge; an event has
exactly one version, so it's a clean interleave with no dedup). It works, but it's
an N-way merge, not a single scan — and it's only worth the cost when the version
filter is **selective**; for "all/most versions" the current value-side filter is
simpler *and already correct*. So version-in-index is **justify-on-demand**, and
the orphaned `PartialOrd<Range<Version>>` trait would only find its purpose
(the three-way scan-advance primitive) *if* this is built.

### 7.2 Disjoint ranges

Disjoint version ranges of one type are a **union (OR)**, not an AND. And they're
already expressible: a `Selector` OR-combines its `TypeSelector`s, so
`A versions 1..3` ⊕ `A versions 5..7` is two `TypeSelector`s in one `Selector`. No
new combinator needed — only the §7.1 index change to make each range a scan.

### 7.3 Exhaustiveness — the earlier guarantee was overstated

The earlier draft said closed ranges give *compile-time* exhaustiveness via a sum
type. That is **only half true**, and the missing half matters: a sum-type `match`
is exhaustive over the variants **you declare**, not over what is **in the
stream**. In any system where a reader's schema can lag a writer's, the stream can
contain a version your binary never compiled against — detectable only at runtime.

The selection model sharpens this into a *worse-than-a-crash* failure: a **closed**
selection (`1..3`) doesn't decode-fail on a `v3` — it simply doesn't *select* it,
so you silently compute a read-model on incomplete data. Detecting "there are
versions outside my range" is about the stream's actual content vs your declared
range — **irreducibly runtime**, and (since the version set is open under
divergence) best-effort. And an **open** range (`2..`) is *fundamentally* at odds
with compile-time exhaustiveness: it is a claim about versions that don't exist
yet; no type system can force you to handle a `v4` defined elsewhere next year.

Honest stance: the compile-time sum type is a useful *local* completeness aid (it
catches *your* omissions); it is **not** a guarantee against the stream. Open-range
or divergence-tolerant exhaustiveness is a **runtime guard**, by nature — and only
needed once you accept reader-lags-writer (§3).

---

## 8. Recommendation / order of work

1. **Done:** version follows revision (§5).
2. **Cheap, high-value, do next when building:** the serialisation seam (§6) and
   making `revision`'s unknown-revision failure informative (§3).
3. **Modelling guidance, not code:** document the §3 "honest transform?" axis and
   the fat-event / explicit-context / one-fact-per-event discipline — it's the
   biggest lever and costs nothing to write down.
4. **Decide consciously, don't drift into:** whether `eventric` must tolerate
   **reader-lags-writer** (§3). This gates the runtime guard and everything in §7.
5. **Justify-on-demand:** version-in-index *selection* (§7.1) — build only if a
   concrete "query version N only" need appears; and only then does the
   `Version`/`Range` trait question (§7's parenthetical, [`FUTURE.md`](./FUTURE.md)
   §1) resolve.
6. **Deferred strategic axis:** owning the serialisation format (§6).

The through-line: with `revision` (transform) + new-events (untransformable) +
modelling discipline, the *current* (no-divergence) reality needs almost no new
versioning machinery. The hard parts are real but are all downstream of a
divergence requirement we have not yet taken on.

---

## References

- **[SANER2017]** Overeem, Spoor, Jansen, *"The Dark Side of Event Sourcing,"*
  SANER 2017. <https://www.movereem.nl/files/2017SANER-eventsourcing.pdf>
- **[Young/InfoQ]** *"Versioning in an Event Sourced System"* (Greg Young), InfoQ.
  <https://www.infoq.com/news/2017/07/versioning-event-sourcing/>
- **[Axon]** Axon Framework reference — Event Versioning.
  <https://docs.axoniq.io/axon-framework-reference/4.11/events/event-versioning/>
- **[Marten]** Marten — Event Versioning. <https://martendb.io/events/versioning.html>
- **[Commanded]** Commanded — Upcasting events. <https://github.com/commanded/commanded>
- **[DCBspec]** Dynamic Consistency Boundaries — specification. <https://dcb.events/specification/>
- **[DCBimpl]** `bwaidelich/dcb-eventstore`. <https://github.com/bwaidelich/dcb-eventstore>
- **[Savic]** Milan Savić, *"Dynamic Consistency Boundaries."*
  <https://milan.event-thinking.io/2025/05/dynamic-consistency-boundaries.html>
- **[Confluent]** Confluent Schema Registry — Schema Evolution and Compatibility.
  <https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html>
