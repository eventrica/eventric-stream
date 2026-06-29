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

Three convictions drive this, set against the prevailing wariness of event-centric
architecture:

1. **Uniformity has compounding value.** A system that works one way throughout is
   cheaper over its life than one that mixes paradigms (event-sourced here, CRUD
   there). The cost of heterogeneity is paid continuously — in cognitive load,
   tooling, and the seams between models — not once.
2. **The perceived complexity of CQRS/ES is a maturity gap, not an essential one.**
   It *feels* harder only because it isn't the default thing people learn, and
   because it lacks the ecosystem maturity that CRUD-on-a-relational-database has
   accreted over decades. Close that gap — make it as frictionless and well-tooled
   as CRUD — and the "only when you need it" caveat dissolves.
3. **Consistency is real, and DCB lets you meet it precisely.** The other common
   objection — that event-centric systems force *eventual consistency* and the hard
   async reasoning it demands — gets the cost calculation wrong twice. First, the
   consistency requirements are **inherent to the domain, not introduced by the
   architecture**: real systems have invariants and causal relationships that hold (or
   should) whether or not you model them, and the industry's habit of *ignoring* them —
   leaving them implicit, racy, or "eventually fine" — is a normalised source of error
   we should be moving away from. Event-centricity makes those requirements explicit
   rather than hidden. Second — the **DCB step-change** — DCB gives a *stronger*
   consistency model than traditional ES *within a context*, by making the consistency
   boundary **dynamic**: it escapes the classic dilemma of one *huge aggregate* (a
   boundary big enough to be safe, at the cost of contention and scale) versus *many
   small aggregates plus eventual-consistency coordination between them*. With DCB you
   are consistent over **exactly what a given operation needs** — no more, no less. That
   removes the trade-off the objection assumes and materially changes the cost
   calculation of event-centric architecture: strong consistency is precise and cheap
   where it is needed, and the only *eventual* boundary left is the deliberate one
   between contexts (§4).

So eventric's job is to make writing event-centric systems *straightforward* — the
path of least resistance — and to supply the surrounding platform (tooling,
observability, communication) that CRUD stacks have long had and event sourcing has
lacked.

**The realistic aim.** The ambition is broad, but eventric is honestly a great fit for
systems of a *particular* kind: the important, valuable business systems — internal or
external — that must be **stable, correct, and effective**, are worth building *quickly
and well*, yet never face day-one hyperscale. It is deliberately *not* for the
millions-of-users-from-launch, social-media-giant case — which ends up bespoke
regardless, so it is no real loss. The single-writer-per-context model (§3) draws a
real scaling ceiling, but a *high* one, and these systems live comfortably beneath it.
The bet is **correctness, clarity, and speed of building** for the vast middle of
genuinely valuable software — not raw scale.

A speculative tailwind (a hunch, not a goal): as system-building leans more on AI
assistance, a stable, clear substrate whose primitives are simply *decisions, events,
and processes* — expressed consistently and logically — may prove markedly more
amenable to AI than ad-hoc, paradigm-mixing alternatives. Not a design goal, but a
tailwind worth recording.

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
another command** — and a *set* of single-event reactions sharing a **projection**
(their coordination state) composes into a **process-manager pattern**, conjectured to
be *emergent* (reactions + projections, plus scheduling for time) rather than a
primitive of its own. Reactions are what turn "an event store" into "a system" — they
make behaviour autonomous and composable, not merely request-driven. The **boundary**
they live on — the edge layer of contracts, effects, and the public/private membrane —
is designed in [`boundary.md`](./boundary.md).

## 3. Context, node, and runtime

- A **bounded context** (in the DDD sense) maps, under DCB, to **one stream** — the
  consistency *outer boundary*. Everything in a stream can be strongly consistent
  together; that is the stream's whole purpose.
- The substrate is **content-opaque**: the stream stores bytes + tags + a type name
  and enforces *no* content rules (no schema, no versioning). Only *clients* of the
  stream interpret content. This is load-bearing (see §6), and is exactly why
  eventric is two crates — an opaque `eventric-stream` substrate and a content-aware
  `eventric-domain` client.
- A **node** is the runnable, deployable unit — a **process** that hosts **one
  Runtime and one-or-more contexts**, and talks to other nodes if configured.
- The **Runtime** is the node-provided **mechanism substrate** — invocation,
  communication, observation — that *runs* user code but is not user code. It is the
  thing that exposes a handler or runs a reaction on an event; the action `Enactor`,
  the reaction reactor, the channel, and the observability layer are its components.
- **Single writer per context** is the consistency invariant; **co-location is
  deployment, nothing more.** A node may host several contexts, but each remains a
  *fully sealed, single-writer* boundary — separate stream, separate writer,
  communicating only via its contract (in-memory when co-located). Resilience comes
  from **near-zero-downtime restart**, not in-context replication. That single ordered
  log per context is the real (logical and physical) scaling ceiling — a *high* one
  (the systems of §1's realistic aim live well beneath it), and the deliberate price
  of the strong, *precise* consistency a single log buys (§8).
- **Location transparency** decouples deployment from domain: user code speaks only
  contracts (commands/queries/events to and from *named* contexts) and can neither
  observe nor assume whether a counterpart is co-located or remote — the Runtime
  resolves the transport (in-memory local, channel remote). It is **transport**
  transparency, not **semantic** transparency: a cross-context call is fallible
  request-response and an event is async *regardless* of location, so user code
  already handles those realities. The payoff: build a whole system in one node (all
  in-memory) and distribute it across nodes by **configuration, with no code change**.

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

A context therefore **owns its surface** — its published API — as a single,
carefully-managed unit of change. The surface is **three-faceted**: the **commands
it accepts**, the **events it emits**, and the **queries it serves** (capital-Q
Queries — see §7). The internal stream evolves freely behind it; the surface is the
thing held stable and versioned deliberately. (What that versioning looks like
concretely is a separate, open decision — §10.)

## 5. The platform

eventric is not just a context engine; it aspires to be a **platform**. Two pillars:

- **eventric owns the inter-context channel.** Contexts never read one another's
  streams — they communicate over an eventric-provided channel (candidate:
  [Iroh](https://www.iroh.computer/) — a p2p/QUIC transport — with a discovery
  mechanism). Owning the channel is a deliberate choice: it is what makes
  platform-wide visibility possible. The channels are *part of the platform*, not an
  external concern bolted on.
- **Observability/introspection is baked in at the substrate — in two layers.**
  *Statically:* because every action and reaction declares both its **trigger**
  (`From<Command>` / `From<Event>`) and, via a **typed effects/events buffer**, the
  commands/events it **emits**, the *whole system topology is derivable at build time*
  — a graph of `command → action → events → reactions → …`, visualisable and checkable
  (orphan events, cycles, contract-vs-emissions) before anything runs. *At runtime:*
  standing up a system gives you, for free, what event types exist, which are most
  common, where the hot/busy parts are, what is in flight — the actual flow overlaid on
  the static map via the message envelope. Plausibly **self-hosted**: the platform's
  own state modelled as eventric events and projections.

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

## 7. Scope: the source of truth, and the read side

The line that defines what eventric *refuses* to be.

**Uniformity is of the *source of truth*, not of implementation.** You cannot
practically make every store the same technology, and that is not the goal — the
event stream is simply the *only* source of truth (no CRUD-as-truth, ever).
Everything else is a *derived view* of that truth: disposable, rebuildable, and free
to live in whatever store fits the query (relational, search, key-value, graph). The
heterogeneity canonical CQRS embraces is permitted, but **only on the read side and
only as derived state**, and it must be **principled, not incidental** — a view is
materialised and maintained *through a reaction* that plays by defined rules
(consistency, rebuild/recreation, staleness), not "whatever gets built." (Those
concrete rules are open — §10.)

**The surface is three-faceted: Commands, Events, and Queries.** Alongside the
commands a context accepts and the events it emits (§4), it serves **Queries**
(capital Q) — a Query is a first-class part of the published surface, answerable by an
*ephemeral projection* (folded on demand from the stream) or a *persistent view* of
any type.

**The consistency split is load-bearing.** Reads divide by purpose:

- **Decision reads** must be strongly consistent, and are folded *fresh from the
  stream* at the moment of decision (what an action does today). They are never served
  from a materialised view.
- **Query / display reads** may lag — a materialised view is eventually consistent with
  the stream and must be treated as such.

The rule that ties this to the consistency conviction (§1.3): **a view is never
load-bearing for a decision — only for whether you might *request* one.** A view can
tell you a command is *worth issuing*; the action then makes the actual decision under
strong consistency. Intent comes from views; correctness comes from the stream.

**Owned vs. external decides "is it eventric state?"** — it splits on *control*, not
data shape:

- Data you **own** — a product catalogue, a rate table, reference data — is a
  **stream**, even if it changes rarely. It is your truth; it is event-centric.
- Data you **do not control** is an **external service**, treated *explicitly* as such
  — reached over the contract/channel like any other foreign context (§4), never
  pretended to be local truth.

**Large binary objects** have no settled story; the likely first (and often
sufficient) approach is the conventional one — the blob lives in object storage and the
event carries a reference, not the bytes.

So eventric is **not**: a store of *polyglot truth* (truth is always the stream); a
place where views are *unmanaged* (every view is reaction-maintained, by rules); nor a
home for *un-owned* reference data (that is external, and explicitly so).

## 8. The DCB consistency model — why dynamic boundaries

This is the technical heart of conviction §1.3, in full.

**The problem: the aggregate as a *static* boundary.** Classic event sourcing makes
the **aggregate** the unit of consistency — and you must design that boundary *up
front*, before you know every decision it will have to serve. Aggregates don't grow
or scale gracefully, and they tend to creep **maximal**: each invariant that spans
the existing boundary pressures it outward. Their concurrency control is **coarse** —
the optimistic check is *"have any new events appeared for this aggregate?"*, which
conflicts even on events that don't affect the operation at hand (false contention).
And invariants that aren't naturally per-aggregate — **uniqueness, global ordering**,
which are *common* — have no aggregate that owns them, forcing hacks or a retreat to
cross-aggregate, eventually-consistent coordination.

**The DCB inversion: the *decision* is the unit of consistency.** Dynamic Consistency
Boundaries make the boundary *dynamic* — defined per operation by a **query** (a
selection over event types and tags), evaluated at decision time. There are no
per-aggregate streams; there is one ordered log, and an operation asks for *exactly
the events relevant to the decision it is making*. The append is optimistic against
*that query*: rejected iff an event **matching the decision's own selection** has
appeared since the position it read at — **fine-grained** conflict detection, not
"anything new." So you are consistent over **precisely what the decision needs** — no
more (no false contention from an over-broad aggregate), no less (no invariant
stranded outside a boundary). You never guess the boundary up front; it *follows from
the decision*. And it maps to how you actually reason: you gather what's relevant to
the call you're making — you don't ask "which aggregate am I in?"

This dissolves the set-validation problem (uniqueness; "this course's capacity vs.
this student's other enrolments") that aggregates hack around — the query simply
selects both sides — and it removes a whole category of *accidental* sagas: process
managers that existed only to bridge boundaries that shouldn't have been boundaries.
With DCB, reactions (§2) are reserved for *genuine* long-running coordination, not as
a workaround for aggregate granularity.

**The cost, honestly.** Precision is bought with a **single ordered log per context**:
every decision serialises through one writer (§3). That is a real bottleneck —
logical and physical — and so a genuine scaling ceiling. A *high* one, and the systems
eventric is for (§1, the realistic aim) live well beneath it; but it is the deliberate
price of strong, precise consistency, and the reason the one-context-one-process
posture is the starting point. The other often-cited DCB costs are milder here: there
is **no aggregate to cache**, but reconstructing state by query/fold is cheap because
the selection leapfrogs (consistency-scoped reads ≈ O(matches·log N)); and the
boundary is **implicit** (a query inside an action, not a named class), which costs
discoverability — recovered, by design, through the platform's introspection (§5):
eventric can *show* what each decision selects, and so what is consistent with what.
The largest honest caveat is **maturity**: DCB is very new, so §1's maturity-gap point
bites hardest here — eventric is early not just to event-sourcing-as-default but to
DCB itself.

**Where eventric sits relative to DCB.** DCB is a refinement *within* event sourcing —
it changes the *consistency model*, not the append-only-log premise. eventric is a
faithful implementation: type + tag selection, position-based optimistic concurrency,
one log per context. DCB already has an event **type/name** as a first-class selection
key — eventric's validated `Name` is directly analogous, **not** an extension. The one
genuine addition is **`Version`**, and even that is shrinking: rather than a
load-bearing *selection* dimension (a version-keyed index, version-range queries —
deferred, and now unlikely), it is becoming **informational** — it follows the
`revision` schema number purely so events can *evolve* (old bytes decode forward) via
the `revision` crate. A small but useful extension on an otherwise-faithful core. The
model is woven through the building blocks already: an **action** reads-by-query →
decides → appends-conditional; a **projection** is the query-fold — so DCB is not a
separable feature but *the* model.

Provenance: Dynamic Consistency Boundaries are the work of Sara Pellegrini and Milan
Savić ([dcb.events](https://dcb.events/)); eventric is an opinionated, typed, Rust
implementation of the idea.

## 9. Trajectory — how this grows

eventric starts **personal**, and becomes more *only if it earns it*:

- **Phase 0 — personal proof.** Build real systems with it, for myself. **Stability
  requirements are low** here — the substrate can churn freely until there is a
  personal-level proof; nothing is frozen or promised. The point of the phase is to
  find out whether the core claim holds.
- **Phase 1 — if it works, a platform.** A hybrid open/commercial platform, and
  possibly the basis for products built on it. Stability is a *function of phase*: it
  ratchets up as eventric moves toward platform — which is when the substrate API and
  the wire-format decision ([`versioning.md`](./versioning.md) §6) get pinned, not
  before.

**What "it works" means — the proof markers:**

1. **Faster and easier to build real systems** than the CRUD-plus-glue default — §1's
   thesis made concrete.
2. **Reusable contexts as building blocks.** Build a context once — *user management*,
   *security/access* — and drop it into a new product seamlessly. This is the deeper
   proof, and the deepest *why-a-platform*: it makes the **contract** (§4) the reuse
   interface and the **channel** (§5) the composition fabric, reframing eventric from
   "a way to build a system" into "a way to **compose products out of sound, sealed
   contexts**." A library of snap-together domain modules is a far stronger pitch than
   another ES framework. Reuse is of the *model*, not the running instance: a reused
   context deploys as a **per-product instance** with its own data — which keeps the
   single-writer ceiling *per product* and sidesteps a shared-service bottleneck.
   Shared data *across* products, where ever needed, is **composition** (talking to
   another context over the contract), not reuse.

**The capability ladder:**

- **Single context, complete** — events + projections + actions + **reactions** in one
  stream: the first *usable* rung, a whole single-context system end to end. Reactions
  (§2) are the gating missing piece — their boundary/effects design is
  [`boundary.md`](./boundary.md).
- **Multi-context, soon after** — the channel + contracts, so contexts compose. *Not* a
  distant phase: composition/reuse is a primary proof marker, so multi-context follows
  close behind, not years later.
- **Observability from almost day 1** — not a late "platform" phase, but woven in early.
  That is also the natural **dogfooding** path: build the observability *on* eventric
  (§5, self-hosted), early, so the substrate proves itself by hosting its own tooling.

The shape is less a strict sequence than a *compressed front* — single → multi-context
— with observability and self-hosting threaded through from the start, not bolted on at
the end.

## 10. Open questions / where this is still forming

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
  [`boundary.md`](./boundary.md) §2 supplies the core *mechanism* — the public/private
  membrane: version the stable **public** form, and the inbound translation absorbs the
  diff to the unstable **private** one. The remaining detail (compatibility rules,
  negotiation) is still open.
- **The rules a view-maintaining reaction plays by.** Views are reaction-maintained
  and eventually consistent (§7); the concrete rules — how a view is rebuilt/recreated,
  how staleness is bounded or signalled, what a Query is guaranteed — are not yet
  defined.
- **`Query` as a built concept.** §7 makes Queries first-class in the surface, but the
  model has commands / events / projections and no `Query` construct yet — its shape,
  and how it picks an ephemeral projection vs. a persistent view, is open.
- **The channel.** Iroh + discovery is a candidate, not a commitment; its requirements
  (addressing, discovery, delivery guarantees, security) need their own pass.
- **Owning the on-disk / wire format.** Flagged in [`versioning.md`](./versioning.md)
  §6 — a big, deliberate decision deferred until format control becomes a requirement.

## 11. How this guides the work

Decisions and priorities are weighed against this vision:

- The **content-opacity + meaning-in-the-client** principle already shapes the crate
  split, and says the versioning guard belongs in `eventric-domain`.
- **Reactions** are the highest-value missing building block: they unlock the full
  loop (process managers, inter-context emission), maintain materialised views (§7),
  and are a prerequisite for the platform/channel.
- The surface is **three-faceted** (commands, events, **queries**), so a `Query`
  construct and **materialised views** are concepts the model must grow — after
  reactions, which maintain them.
- The **fail-closed** principle is the lens for the versioning / reader-lags-writer
  design.
- The **platform/observability** ambition argues for eventric owning the channel and
  for a self-hosting introspection layer.
