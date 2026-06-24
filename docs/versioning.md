# Event versioning — a design exploration

**Status: EXPLORATION / decision-pending.** This is a research-grounded survey of
the design space plus a worked analysis of how versioning could fit `eventric`. It
does not describe anything implemented, and it does not commit to an approach. It
exists so the eventual decision is made with the theory, the prior art, and the
trade-offs in view. The parked work items live in [`FUTURE.md`](./FUTURE.md) §1;
this is the depth behind them.

---

## 0. The problem, scoped

Event sourcing distinguishes two fundamentally different kinds of schema change,
and the literature treats them as separate problems:

- **Non-breaking change** — adding an optional/defaulted field, removing an unused
  one. An old reader can still read new data and vice versa. **`eventric` already
  solves this**: the `revision` crate gives evolvable binary (de)serialisation —
  you bump the revision, add/default/remove fields, and old bytes still decode
  into the current struct. This is the "weak schema" / "tolerant reader" approach
  (Young, Fowler/Verraes), and it has a hard boundary: **renaming a field or
  changing the *semantic meaning* of a field is NOT absorbable** — those are
  breaking changes. [Young/InfoQ]
- **Breaking change** — a rename, a semantic shift, a structural reshape that the
  tolerant reader cannot absorb. **This is the open problem.** Everything below is
  about breaking changes.

The maintainer's working hypothesis (the starting point for this exploration):
when a change is breaking, you should *not* mint an unrelated new event; the event
is *logically the same*, so it should keep the **same identifier** but carry a
distinct **`Version`**, and you should be able to **select by version (or version
range)** when reading the stream.

---

## 1. The research landscape

### 1.1 The canonical taxonomy

The one peer-reviewed treatment, Overeem, Spoor & Jansen, *"The Dark Side of Event
Sourcing: Managing Data Conversion"* (SANER 2017) [SANER2017], names **five**
breaking-change techniques and splits them into **run-time** and **batch**:

| Technique | Kind | What it does |
|---|---|---|
| **Multiple versions** | run-time | Keep every version's bytes as-is; tag with a version; every consumer handles all versions. |
| **Upcasting** | run-time | On read, transform each old event up to the latest version before any consumer sees it. |
| **Lazy transformation** | run-time | Like upcasting, but the transformed result is written back to the store. |
| **In-place transformation** | batch | Rewrite stored events to the new schema (mutates history). |
| **Copy-and-transform** | batch | Write a new stream/store transformed from the old; switch over. |

Their quality comparison (Table II) ranks **upcasting** the overall preferred
technique (performance `+`, reliability `+`, functional suitability `±`,
maintainability `±`). Crucially for our hypothesis, **"multiple versions" is rated
the *least* maintainable** — the only flat `−` in the table — with the verbatim
reason: *"the support of multiple versions is spread throughout the application
code."* Upcasting wins on maintainability precisely because it **centralises**
version knowledge: a consumer only ever handles the latest version. [SANER2017]

One honest caveat from the same paper: **no run-time technique is "functionally
complete"** — operations spanning multiple aggregate streams force you to read
several streams, *"violat[ing] the independence of the streams."* (See §2 for why
this particular objection does **not** apply to DCB.) [SANER2017]

### 1.2 What the industry actually does: read-time upcasting

The production frameworks converge, unanimously, on **read-time upcasting** and
deliberately **do not** surface multi-version reads to consumers:

- **Greg Young** endorses it: *"when old versions are read from the store, they
  can first be converted, or upcasted, to the latest version"* so *"an event
  handler only needs to know how to deal with the latest version."* [Young/InfoQ]
- **Axon** — `@Revision` annotation on the payload; chained `x → x+1` upcasters
  over an `IntermediateEventRepresentation`, applied **at read time**, leaving
  stored history intact. No multi-version-read feature exists; the recommended
  path is always the upcaster chain. [Axon]
- **Marten** — lazy upcasting: *"transforming the old JSON schema into the new
  one … performed on the fly each time the event is read,"* with an explicit N+1
  read-cost caveat. [Marten]
- **Commanded** — the `Upcaster` protocol transforms events at runtime *before any
  consumer*; its documented example upcasts a `HistoricalEvent` into a **wholly
  different `NewEvent` struct** — i.e. it sidesteps the type-vs-version debate by
  collapsing to one current version at read time. [Commanded]

The convergent pattern: **encode a per-event revision; apply read-time upcaster
chains to the latest; consumers handle only the latest; multi-version reads are
not a thing.** [Axon, Marten, Commanded, Young]

### 1.3 The type-vs-version debate, as the field sees it

The debate the maintainer is weighing — *new event TYPE* vs *same identifier, new
VERSION* — is **largely sidestepped in practice**. Because everyone upcasts to a
single current version, the question of exposing multiple live versions rarely
arises; and when it does (Commanded), the favoured move is to upcast the old type
into a new type, not to expose a version discriminator. The "multiple versions"
approach (the one closest to a queryable `Version`) is the one the taxonomy rates
worst. So: **the field's answer is "neither — upcast."** [SANER2017, Commanded]

### 1.4 Schema-registry prior art

Confluent's Schema Registry formalises the compatibility ordering that bounds the
*non-breaking* envelope: **BACKWARD** (new-schema consumers read old data; upgrade
consumers first), **FORWARD** (old-schema consumers read new data; upgrade
producers first), **FULL** (both; add/remove optional fields only). [Confluent]
The useful mapping: **`revision`-absorbable changes are the FULL-compatible
subset; breaking changes are exactly what falls outside these modes.** Registries
enforce compatibility at *registration time, at runtime* — not in a type system
(this matters for §5.4).

---

## 2. DCB says nothing — so `Version` is our own extension

Verified against the DCB specification, its reference implementation, and the
canonical Savić article: **DCB has no notion of event version.** A DCB `Event` is
*only* `Type` + `Data` (opaque) + `Tags` (+ optional metadata); `Type` is *purely*
a selection/filter key with no format or versioning guidance; the sole ordinal is
the sequence position (for optimistic concurrency). A full-text search for
version/schema/revision/discriminator across the spec, ref impl, and Savić's piece
returns **zero hits**. [DCBspec, DCBimpl, Savic]

Two consequences:

1. **`eventric`'s `u8 Version` is a genuine extension of DCB with no native
   precedent.** Whatever we do, we are charting — not following. Freedom and risk.
2. **The SANER "upcasting isn't functionally complete (cross-stream)" objection
   does not bite us.** That objection is about per-aggregate stream stores where
   replays are per-stream. DCB has *no* per-aggregate streams — reads are global,
   by type+tag selection — so there are no stream-independence boundaries to
   violate. Upcasting is *more* complete in a DCB model than in a classical one.

DCB selection today: query items combine with **OR**; within an item, the type
must match one of the listed types **AND** the tags must all be present. Version
would be a new dimension *within* the type match. [DCBspec]

---

## 3. The tension, and the reconciliation

**The tension.** The maintainer's instinct — same identifier, distinct version,
select by version range — *is* the "multiple versions" technique, which the one
rigorous study ranks least maintainable, and which zero production frameworks
expose. Taken at face value, the research pushes back on the hypothesis.

**Three reasons not to discard the instinct:**

1. **Upcasting already honours the real goal.** The deeper intuition was *"a
   breaking change is still logically the same event — don't make it an unrelated
   new event."* Upcasting satisfies exactly that (it preserves one logical
   identity by transforming old→new) *and* avoids the version-spread. The
   maintainer and Young agree on the **principle**; they differ only on the
   mechanism.
2. **The capability upcasting discards is the one this library is built for.**
   Upcasting collapses everything to "latest," destroying the ability to *select*
   by version. But `eventric`'s entire grain is **DCB selection** — querying the
   stream by type+tag. Extending that to type+tag+**version-range** is the natural
   shape of *this* library. No framework offers it because none are
   selection-first stores; they are aggregate-replay stores. Version-range
   selection is not a worse upcasting — it is a *different capability*.
3. **There is no precedent constraining either choice** (§2).

**The reconciliation — they are layers, not alternatives.** They answer different
questions, and a low-level mechanism library should provide both:

- **Stream layer = mechanism.** Make `Version` a first-class **selection**
  dimension: select `type A, versions 2..4, tagged X`. The library *offers* the
  capability; it does not impose any maintainability cost by doing so.
- **Model layer = policy.** Provide a **read-time upcasting** hook so the *default*
  consumption path collapses to the latest version (the mainstream maintainability
  win). Most projections register an upcaster chain and write one `Project<…V_n>`.

A consumer who just wants the read-model uses upcasting and never sees a version.
A consumer who genuinely needs per-version behaviour (audit, analytics, migration
tooling, "the world as it looked at v1") selects a version range and handles
versions explicitly. **The SANER critique only bites if you opt out of upcasting**
— and you only do that when version-awareness is the actual point.

This also yields a clean two-tier evolution story:

> **`revision` absorbs the non-breaking deltas *within* a version; upcasters
> bridge *across* versions.** (`revision`-handled changes are Confluent's
> FULL-compatible subset; upcasters handle what falls outside it.)

---

## 4. The four hard parts (concrete, with trade-offs)

### 4.1 Encoding the version for efficient selection

Today the `types` inverted index is `[2][name_hash][pos] → version` — the version
sits in the **value**, so it can only be checked *after* materialising a candidate
(which is what the in-memory `mask()` re-check does). To make a version range a
real **index scan**, version moves into the **key**:

```
[2][name_hash][version][pos] → ∅
```

A version range is then a bounded prefix scan over `[2][name_hash]`, and
big-endian key ordering keeps it numeric (as the rest of the index relies on).

- **Pro:** efficient version-range selection; reuses the existing BE-key /
  prefix-scan / sorted-combinator machinery; gives the orphaned
  `PartialOrd<Range<Version>>` trait its purpose (the three-way *below / inside /
  above* is exactly the scan-advance primitive — see [`FUTURE.md`](./FUTURE.md)
  §1).
- **Con:** a storage-format change to the `types` index; and it is **genuinely
  novel** — no event store does range-queryable versions, so there is no prior art
  to validate the design against.

### 4.2 Disjoint version ranges

Disjoint ranges of the *same* type are a **union (OR)**, not an AND — an event is
v2 *or* v5, never both; AND would match nothing. (This corrects an initial
framing.) And it is **already expressible** in the model: a `Selector`
OR-combines its `TypeSelector`s, so `type A versions 1..3` ⊕ `type A versions 5..7`
is two `TypeSelector`s in one `Selector`. The AND axis (type+tag) is untouched and
orthogonal. So disjoint ranges need **no new combinator** — only the index change
in §4.1 to turn each range from a post-materialisation filter into a scan.

### 4.3 Multi-version consumption (the "extension")

The frameworks' validated answer is an **upcaster chain**: register
`(identifier, version) → (struct, fn(prev) -> next)`, and run it inside dispatch —
right where `DispatchEvent::from_event` currently does `revision::from_slice` — so
that `Project<E>` only ever sees the latest `E`.

- The two-tier split is clean: **`revision` decodes the stored bytes into that
  version's struct; the upcaster chain lifts that struct to the latest.**
- For the rare consumer that genuinely wants per-version behaviour, the existing
  type-keyed dispatch already allows `Project<AV1>` + `Project<AV2>` directly (the
  "multiple versions" path) — available, but not the default.

This is the smallest, most precedented, highest-value piece of the whole story.

### 4.4 Exhaustiveness — the genuinely open problem

The research is blunt: **the literature does not solve compile-time detection of
an unhandled new version.** Frameworks dodge it (upcasting ⇒ only one version to
handle); schema registries enforce compatibility at *registration/runtime*, not in
a type system. [Confluent] For `eventric`:

- **Closed ranges → compile-time, via a sum type.** Model "the handled versions of
  A" as a generated `enum A { V1(AV1), V2(AV2) }`; consuming is a `match`, so
  adding `V3` breaks every match site — the loud failure you want. A derive could
  generate the enum. This is the Rust-idiomatic answer for a *closed* set.
- **Open ranges (`2..`) → only a runtime registry.** And the deep point: **an open
  range and compile-time exhaustiveness are fundamentally contradictory.** "Handle
  everything from v2 onward" is a claim about versions that *do not yet exist*; no
  type system can force you to handle a `v4` defined in another crate next year.
  The best achievable is a runtime registration guard: *every registered version
  in the requested range has a handler, else refuse to start.* The maintainer's
  intuition that this *"needs an event registration mechanism which doesn't
  currently exist"* is correct — and correct for a **fundamental** reason, not a
  missing-feature one.

Honest design stance: **compile-time safety for closed ranges (sum type), a
runtime registry guard for open ranges, and documentation that open-range
exhaustiveness is a runtime guarantee by nature.** An event registry is also the
natural home for the version→struct→upcaster mappings of §4.3, so it earns its
keep twice.

---

## 5. What the literature does *not* resolve (ours to own)

- **Version-range reads as a first-class capability** — no prior art; first
  principles only.
- **Open-range consumer exhaustiveness** — *provably* not a compile-time problem;
  best-effort runtime only.
- **Whether version *selection* earns its storage-format cost at all** — if
  read-time upcasting already covers the real use cases, the version-in-index
  apparatus may be over-engineering.

---

## 6. Recommendation: prove the need before building the index

The last bullet of §5 is load-bearing. The entire version-in-index apparatus
(§4.1) is justified *only if* something genuinely needs to **select** by version at
the stream layer — not merely *consume the latest*. Upcasting alone is cheaper,
precedented, and maintainable, and may cover the overwhelming majority of real
need.

So the cheapest decisive experiment is **not** to build the index change first.
It is to take **one realistic breaking-change scenario for a target consumer and
ask: does this need version *selection*, or just version *upcasting*?**

- If **upcasting suffices** → build the **model-layer upcaster hook** (§4.3):
  small, precedented, high-value; `Version` can stay a value-side attribute. This
  is almost certainly worth doing regardless.
- If a real **"query v2-only events"** case appears → *that* justifies the
  index change (§4.1), built in the knowledge that it is a deliberate DCB
  extension, not a default.

Suggested order, then: **(1)** the upcaster hook + the event registry it needs
(also unlocks the exhaustiveness guards of §4.4); **(2)** the sum-type derive for
closed-range exhaustiveness; **(3)** the version-in-index change *iff* a selection
use case is found; and **(4)** decide the fate of the orphaned `Version`/`Range`
comparison traits as part of (3).

---

## References

- **[SANER2017]** Overeem, Spoor, Jansen, *"The Dark Side of Event Sourcing:
  Managing Data Conversion,"* SANER 2017.
  <https://www.movereem.nl/files/2017SANER-eventsourcing.pdf>
- **[Young/InfoQ]** *"Versioning in an Event Sourced System"* (Greg Young), InfoQ
  summary. <https://www.infoq.com/news/2017/07/versioning-event-sourcing/>
- **[Axon]** Axon Framework reference — Event Versioning.
  <https://docs.axoniq.io/axon-framework-reference/4.11/events/event-versioning/>
- **[Marten]** Marten — Event Versioning.
  <https://martendb.io/events/versioning.html>
- **[Commanded]** Commanded — Upcasting events.
  <https://github.com/commanded/commanded>
- **[DCBspec]** Dynamic Consistency Boundaries — specification.
  <https://dcb.events/specification/>
- **[DCBimpl]** `bwaidelich/dcb-eventstore` (reference implementation).
  <https://github.com/bwaidelich/dcb-eventstore>
- **[Savic]** Milan Savić, *"Dynamic Consistency Boundaries."*
  <https://milan.event-thinking.io/2025/05/dynamic-consistency-boundaries.html>
- **[Confluent]** Confluent Schema Registry — Schema Evolution and Compatibility.
  <https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html>
