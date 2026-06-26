# eventric — Vision

**Status: v0.1, exploration.** A guiding document for what eventric *is*, the
convictions behind it, and how it is meant to evolve — the basis against which
technical decisions are weighed and future work is prioritised. Built up
Socratically; it deliberately carries motivations, ideas, and open questions, not
just settled facts. Companion to the technical docs ([`versioning.md`](./versioning.md),
[`derives.md`](./derives.md), [`FUTURE.md`](./FUTURE.md)) and the architecture in
[`CLAUDE.md`](../CLAUDE.md).

---

## 1. The thesis

eventric is a bet that **event-centricity should be the default way to build
systems** — not a specialist tool reached for only "when you need CQRS." It takes
the ideas behind CQRS and Event Sourcing, and — the step-change —
**Dynamic Consistency Boundaries (DCB)**, and turns them into the **fundamental
building blocks of a system**. The name carries the thesis: *event-centric* →
*eventric*. The event is the primitive; everything else derives from it.

Two convictions drive this, set against the prevailing "don't use CQRS for
everything" advice:

1. **Consistency/uniformity has compounding value.** A system that works one way
   throughout is cheaper over its life than one that mixes paradigms (event-sourced
   here, CRUD there). The cost of heterogeneity is paid continuously — in cognitive
   load, tooling, and the seams between models — not once.
2. **The perceived complexity of CQRS/ES is a maturity gap, not an essential one.**
   It *feels* harder only because it isn't the default thing people learn, and
   because it lacks the ecosystem maturity that CRUD-on-a-relational-database has
   accreted over decades. Close that gap — make it as frictionless and well-tooled
   as CRUD — and the "only when you need it" caveat dissolves.

So eventric's job is to make writing event-centric systems *straightforward* — the
path of least resistance — and to supply the surrounding platform (tooling,
observability, communication) that CRUD stacks have long had and event sourcing has
lacked.

## 2. The building blocks

Four core concepts. Three exist in basic form today; one is the key missing piece.

- **Events** — the primitive: an append-only, strongly-consistent ledger within a
  boundary.
- **Projections** — read-models folded from events.
- **Actions** — respond to a *command*: read projections, decide, append events.
  (The write side; exists today.)
- **Reactions** *(not yet built)* — respond to an *event*. The symmetric counterpart
  to actions, and the piece that **closes the loop**.

The action/reaction symmetry is deliberate: an action is triggered by a *command*, a
reaction by an *event*. A reaction can maintain a view, emit a message, or **issue
another command** — and a reaction that also reads projections becomes a de-facto
**process manager / long-running coordinator**: itself just event-driven state with
responsibilities. Reactions are what turn "an event store" into "a system" — they
make behaviour autonomous and composable, not merely request-driven.

## 3. The context model

- A **bounded context** (in the DDD sense) maps, under DCB, to **one stream** — the
  consistency *outer boundary*. Everything in a stream can be strongly consistent
  together; that is the stream's whole purpose.
- The substrate is **content-opaque**: the stream stores bytes + tags + a type name
  and enforces *no* content rules (no schema, no versioning). Only *clients* of the
  stream interpret content. This is load-bearing (see §6), and is exactly why
  eventric is two crates — an opaque `eventric-stream` substrate and a content-aware
  `eventric-domain` client.
- Practically (leaning, not fixed): **one context = one process**, single-writer,
  with resilience from **near-zero-downtime restart** rather than in-context
  replication. Everything for a context happens in its process.

## 4. Two kinds of event, and the contract

A hard distinction, drawn deliberately:

- **Internal events** — the context's private DCB stream. Opaque, strongly
  consistent, and free to churn.
- **Published / integration events** — the **communication media** between contexts:
  a *designed contract*, not raw stream events. Published events are **derived,
  never leaked** — a reaction translates internal events into published ones, and the
  mapping need not be 1-to-1 (a published event may bundle several internal changes).
  Inbound is symmetric: a foreign command or event is translated *in*, never appended
  raw to your stream.

A context therefore **owns its contract**: the **commands it accepts** and the
**events it emits**, as a single, carefully-managed unit of change. This is
eventric's published API. The internal stream evolves freely behind it; the contract
is the thing held stable and versioned deliberately. (What that versioning looks like
concretely is a separate, open decision — §7.)

## 5. The platform

eventric is not just a context engine; it aspires to be a **platform**. Two pillars:

- **eventric owns the inter-context channel.** Contexts never read one another's
  streams — they communicate over an eventric-provided channel (candidate:
  [Iroh](https://www.iroh.computer/) — a p2p/QUIC transport — with a discovery
  mechanism). Owning the channel is a deliberate choice: it is what makes
  platform-wide visibility possible. The channels are *part of the platform*, not an
  external concern bolted on.
- **Observability/introspection is baked in at the substrate.** Standing up an
  eventric system should give you, for free: what event types exist, which are most
  common, where the hot/busy parts are, what is in flight — and rich visualisation
  and tooling over all of it. Plausibly **self-hosted**: the platform's own state
  modelled as eventric events and projections.

The self-hosting idea is the strongest internal consistency check on the whole
thesis: if eventric can build *itself* — its tooling, its observability —
event-centrically, the "use it for everything" claim is *demonstrated* rather than
asserted.

## 6. Principles

- **Correctness over convenience — fail closed.** A silent *incorrect success* is the
  worst possible outcome. When a client cannot fully and correctly interpret what it
  is processing (e.g. an event written at a revision it does not know), it must
  **reject the operation**, not best-effort it. This is precisely why versioning is a
  *client* concern, enforced in `eventric-domain`, and never by the opaque substrate.
- **Uniformity.** One way to build, throughout — the compounding-value argument of §1.
- **Opacity at the substrate.** The stream is content-agnostic; meaning lives in the
  client. Compile-enforced (`eventric-stream` cannot `use revision`).
- **Low barrier.** Event-centric as the easy default, with the tooling to match.
- **A sealed boundary.** Contexts are sealed; every crossing goes through the owned
  contract and channel, never leaked internals.

## 7. Open questions / where this is still forming

Deliberately unresolved — the vision exists partly so we can resolve these
*consistently* rather than ad hoc.

- **Distribution depth.** "One context = one process" is the leaning. Whether a
  context ever needs **multiple instances of the same code** (in-context scale or
  resilience beyond fast restart) is undecided — and reactions are the likeliest
  thing to want to scale out. ([`FUTURE.md`](./FUTURE.md) §4: the
  parallelism/concurrency load test will inform this.)
- **The reader-lags-writer guard, concretely.** The model narrows in-stream lag to a
  deploy-handover edge (an old reader briefly overlapping a new writer); the
  cross-version surface that actually matters is the **inter-context contract**. The
  fail-closed rule (§6) applies at both, but the concrete mechanism is TBD.
  ([`versioning.md`](./versioning.md).)
- **Contract versioning.** Given the contract is the versioned unit, what does
  versioning it look like — compatibility rules, negotiation, rejection on mismatch?
  A separate set of decisions.
- **The scope boundary of uniformity.** The conviction is that event-centric should be
  the default *everywhere*; where (if anywhere) it stops being the right tool — and
  whether projections/read-models are themselves eventric state or may live in
  external stores — is not yet pinned.
- **The channel.** Iroh + discovery is a candidate, not a commitment; its requirements
  (addressing, discovery, delivery guarantees, security) need their own pass.
- **Articulating the DCB step-change.** *Why* DCB over classic per-aggregate streams —
  what it unlocks for this vision — deserves its own section once we dig into it.
- **Owning the on-disk / wire format.** Flagged in [`versioning.md`](./versioning.md)
  §6 — a big, deliberate decision deferred until format control becomes a requirement.

## 8. How this guides the work

Decisions and priorities are weighed against this vision:

- The **content-opacity + meaning-in-the-client** principle already shapes the crate
  split, and says the versioning guard belongs in `eventric-domain`.
- **Reactions** are the highest-value missing building block: they unlock the full
  loop (process managers, inter-context emission) and are a prerequisite for the
  platform/channel.
- The **fail-closed** principle is the lens for the versioning / reader-lags-writer
  design.
- The **platform/observability** ambition argues for eventric owning the channel and
  for a self-hosting introspection layer.
