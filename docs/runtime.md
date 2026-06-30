# eventric — Runtime

**Status: v0.2, skeleton (adversarially reviewed).** The design of the **runtime**
— the mechanism that drives a *context* deterministically: how appended events are
processed, in what order, under what consistency, and how it invokes actions,
reactions, and effects. A skeleton to refine, not a settled design; it carries leans
and open forks, and a v0.1 review pass already corrected several over-claims (noted
inline). Companion to [`vision.md`](./vision.md) (the *why* — node/runtime/context,
§3), [`boundary.md`](./boundary.md) (the edge: contracts, effects-as-messages,
verbs), and [`reactions-plan.md`](./reactions-plan.md) (the slice that proved the
*pieces*). The runtime lives in the `eventric-runtime` crate.

---

## 1. What the runtime is

The runtime is the **deterministic driver** of a single context. A context is one
stream, one writer (vision §3), one total order. Within it, the stream is the only
source of **truth** *and* the only source of **order**.

The determinism is precise — and narrower than it first looks:

- **Read-state is a function of the stream prefix.** Projections and (idempotent)
  views fold the prefix in position order, so re-processing the same stream
  reproduces the same read-state. This is the valuable, true replay guarantee.
- **The written stream is *not* reproducible from an earlier prefix.** What gets
  appended depends on *exogenous inputs* (external/cross-context commands — §9) and
  on actions reading **as-of-head** at dispatch time (§4). So the runtime does **not**
  guarantee "re-running would make the same decisions and write the same stream" —
  only that, given the stream as written, all read-state and all reaction *proposals*
  replay identically. *(v0.1 over-claimed decision-reproducibility; corrected.)*

## 2. The spine — a single ordered frontier

The runtime advances a **single frontier** through the stream in **position order**.
At each event, in order, it:

1. **folds** the event into the projections/views that select it; *then*
2. **fires** every reaction registered for that event type.

(Step order is **fold-then-fire** — a reaction sees read-state *including* its
triggering event. This is part of the spine, not an open fork.) Reactions **propose
effects** (§7); the *command* effects among them run actions (via the `Enactor`),
which append new events at the **head**. The frontier reaches those in turn.

Because the stream itself is the FIFO queue, there is no separate work queue, and a
reaction's consequences are just *later events* processed when the frontier arrives.
This makes the cascade **breadth-first** by construction; depth-first (drain a
reaction's whole consequence-cascade before its sibling event) would require
processing tail positions before lower ones — out of position order — so the spine
**excludes** it. One logical thread, FIFO by position.

**Liveness — not "quiescence".** A *closed* internal cascade must terminate (§10.9).
But a *live* context's head never stops (external input keeps arriving), so the
property is not "chase the head to quiescence" but **bounded frontier lag** — the
frontier keeps pace with the head. Sustained input that outruns a single thread, and
non-terminating cascades, are real failure modes the spine must address (§10.9), not
assume away.

## 3. The event lifecycle — replayable vs once-only

The thing to **nail down and prove** — and the line is **effect character, not
fold-vs-fire mechanism** *(v0.1 mis-cut this; corrected)*:

- **Replayable** (pure, no external/durable side effect): an in-memory **projection**
  fold, and a **materialised view** maintained by an *idempotent* `MaintainView`
  delta. These can be rebuilt by re-processing the prefix.
- **Once-only + checkpointed** (impure/external): a **command** issued, an event
  **published**, an external **call**. Re-running these on a rebuild re-issues
  real-world effects — they must fire roughly once and **not** be replayed blindly.

The line cuts **through** reactions: a view-maintaining reaction is replayable (its
effect is a pure delta); a command-issuing reaction is once-only. A naïvely
non-idempotent view delta (a bare `+= 1`) is *not* freely replayable — it double-counts
— so it needs a keyed/idempotent application or its own per-view checkpoint.

**Crash-consistency for the once-only effects.** Fire a command, die before advancing
the **checkpoint**, restart re-fires → a duplicate. The honest guarantee is
**at-least-once dispatch + idempotent effect**, not magic exactly-once. And — *(v0.1
unsound; corrected)* — **the DCB append guard does not absorb this.** The guard is a
compare-and-swap against the action's *own* read at head; it rejects concurrent
conflict, it does **not** deduplicate a replayed command (on re-fire the action
re-reads the new head, which already holds its first effect, sees no conflict, and
appends a second). Duplicate-absorption must live in **the action's decision** (its
selection covers its own prior output, so a redo no-ops) or in an **explicit
idempotency key**. The envelope's **causation id** (boundary §5) is the natural key:
"has position *p* already fired?" is answerable from the stream itself — does an event
*caused-by p* exist? — which doubles as a stream-derived progress signal.

## 4. Consistency — propose vs dispose

Reactions and actions read at **different points**, and the asymmetry is load-bearing:

- **Reactions read as-of-trigger** — a reaction fired at position *p* sees read-state
  folded up to *p*: the world *as of the event it reacts to*. This is **forced** by
  the §1 read-state replay invariant, not a free choice. It is **free** for bounded,
  global projections (just the state while the frontier sits on *p*), but **not** free
  for *per-instance*, tag-keyed projections (one state per live order/student/…, an
  unbounded set the frontier cannot hold resident) — those are folded **on demand** at
  trigger time, a real cost bounded by the result (§6, §10.7).
- **Actions read as-of-head** — an action is a **write**; it must be consistent with
  the real head it appends against (read head, append under the **DCB guard**).

Hence **reactions propose, actions dispose.** A reaction says "I want this" from its
local, deterministic view; the action enforces consistency at the write and **may
reject**. This asymmetry is **correctness-coherent** (the action is always consistent
with what it appends against) but is **not a determinism guarantee**: because the
action reads as-of-head, what it writes depends on the dispatch-time head and the
firing order of co-triggered reactions (§5).

A reaction is **structurally blind to the disposition** — it staged effect *data* and
returned before the runtime interpreted it, so it cannot branch on a rejection. What
*happens* to a rejected/failed reaction-command, and what recovers it, is open
(§10.10).

## 5. Ordering — settled and open

The spine settles the big ones: **position-order FIFO**, hence **breadth-first**
cascade (§2), and **fold-then-fire** (§2). The genuine remaining forks:

- **Multiple reactions on one event.** Their *reads* are order-free (all see the same
  as-of-trigger state), but their *commands* are not: dispatched actions read
  as-of-head in sequence, so the second action sees the first's append. Order is
  therefore **semantically significant through as-of-head dispatch** (not merely via a
  shared projection) — a **stable, declared firing order** is required for
  reproducibility. "Independent reactions are order-free" is false once they command.
- **Multiple effects within one reaction** (e.g. `[Command A, Command B]`): their
  interpretation order, whether they are an atomic group, and partial-failure policy
  are unspecified — and a single per-event checkpoint **cannot express partial
  progress** through a multi-effect fire (§10.3).
- **Processing transaction boundary.** An action appends *N* events atomically. *Lean:
  processing them (folds + fires) is independent per event in position order* —
  atomicity is a *write* property (the append batch), not a *processing* one.

## 6. Reactions gain projections

A reaction today sees only its triggering event (the slice's event-only shape). That
is insufficient — a reaction usually decides from accumulated state. Giving reactions
projections makes them **symmetric with actions**:

- **action** = (command + projections) → events
- **reaction** = (event + projections) → effects

Same projection machinery, read **as-of-trigger** (§4). A *set* of single-event
reactions sharing a (tag-keyed, per-instance) projection is the **process-manager
pattern** (vision §2) — and it is exactly those per-instance projections that are
unbounded and folded on demand rather than held resident (§4).

## 7. Effects — pluggable and introspectable

Effects must be **pluggable**: we cannot enumerate every effect future contexts need.
The lean (shared with boundary §10, reactions-plan — **still a lean, not committed**)
is a **trait-based `Effect`** rather than a closed enum, each carrying static metadata
(kind, target type). The runtime holds an **interpreter per effect kind**; staging an
effect is data, interpreting it is the runtime's job. The static **topology graph**
(boundary §4–5) comes primarily from the typed **`Emits`** buffer (a handler declares
what it can emit, so emissions can't drift) — the per-effect metadata supplements it.

The verb-trio from the boundary (Command / Query / Event) plus the view effect:

- **Command** — route to its action and enact (settled; `reactions-plan.md`).
- **Event (publish)** — emit a public event (the third verb; **deferred**, but named
  here so the trio stays intact).
- **Query** — *emit* a synchronous read request (**provisional / at-risk** — see §8).
- **MaintainView** — apply a (idempotent) delta to a read-model.

*Open:* the `Effect` trait's exact shape; interpreter registration/dispatch; whether
built-ins share the trait or sit beside it; trait-vs-enum (still a lean). (We use
**fire** for invoking a reaction, reserving *dispatch* for the existing mask-routing
and command-routing senses.)

## 8. Query as an effect (provisional)

A **query effect** is the **emit** side: a reaction/action asks (e.g. another
context), awaits a reply, uses it. It is **provisional** — the one effect that strains
the spine in two ways:

- **Liveness.** Blocking the single frontier on an external reply **head-of-line-blocks
  the entire context** (no folds, fires, or appends until it returns); a slow/down
  service freezes the context (a fail-open hole, against vision §6); and two contexts
  querying each other synchronously is a **distributed deadlock** (boundary §3/§8.2
  make cross-context calls synchronous between single-threaded frontiers). A blocking
  call inside the spine is therefore likely wrong; an **async exchange** (issue →
  suspend that causal flow → resume on the reply) is a candidate shape.
- **Determinism.** A reply is not in the stream, so a reaction that branches on it is
  not a function of the prefix (§1). Recording the raw reply **as an event** would
  violate boundary §3 (*results are returns; only facts are events*) and freeze a
  stale read into the permanent log. If a reply must be captured for replay, it would
  be runtime/envelope bookkeeping or a context-*asserted* fact (`ObservedXAtP`), never
  the raw reply.

Both are open (§10.6). Distinct from the **inbound** side — a context *serving* a
query against its own read-models (the surface's Q, vision §2) — which is **deferred**.

## 9. Where the code stands

- **`Reactor<R>` is a stepping-stone, not the driver.** It drives one reaction over
  its own matching events (`drive`, looping until drained). That proved the mechanism
  (fire → stage effects → route a command to its action → enact) but its per-reaction
  iteration **reorders relative to the stream**; the real driver replaces it with the
  single-frontier loop (§2). Its `MAX_PASSES` runaway guard goes with it — and a count
  guard cannot undo a divergent cascade already committed to an append-only log (§10.9).
- **The `Enactor` stays** — the action-cycle mechanism (fold projections, decide,
  append under the DCB guard). The runtime *invokes* it both to interpret a command
  effect **and** to serve **inbound** commands (boundary §6's single command path —
  external/cross-context commands are exogenous, not prefix-derived; §10.12).
- **Next code increment:** the real driver — one frontier, position order,
  fold-then-fire, incremental projections, the replayable/once-only checkpoint split
  (§3) — once the forks below are settled enough.

## 10. Open questions

1. *(Settled — recorded for clarity)* Cascade is **breadth-first** and step order is
   **fold-then-fire**; both are forced by the spine (§2), not live forks.
2. **Within-reaction effects:** ordering, atomicity, and partial-failure of a
   reaction's multiple staged effects — and that one per-event checkpoint cannot
   express partial fire progress (§5).
3. **Multiple reactions on one event:** the stable, declared firing order (significant
   through as-of-head dispatch, §5).
4. **Effect model:** the `Effect` trait shape; interpreter registration/dispatch;
   metadata surfacing vs the `Emits` topology graph; built-ins-share-trait-or-beside;
   trait-vs-closed-enum (still a lean) (§7).
5. **Query effect:** blocking-in-spine vs async exchange; determinism; capturing a
   reply for replay without polluting the fact-log (§8).
6. **Projection maintenance (split):** *reaction-read* projections are continuous
   incremental for bounded/global (forced/free, §4) but on-demand for per-instance;
   *action-read* is a fresh fold at head. Confirm and cost the incremental side.
7. **Checkpoint:** durability; granularity (single frontier position vs per-reaction
   fire-progress); **reaction onboarding** — a newly deployed reaction catches up over
   history (re-firing → stale commands) or starts at head (skips history)? The
   replayable kind can rebuild from zero; the once-only kind cannot.
8. **Liveness:** closed-cascade termination (likely **static cycle detection** via the
   topology graph as the primary defense, since a runtime count-guard cannot un-commit
   events); **backpressure** when the head outpaces the frontier; **frontier lag**
   (head − frontier) as first-class introspection (the runtime overlay of vision §5).
9. **Rejected/failed reaction-command:** dropped, retried, dead-lettered? What
   re-triggers a decision, given the reaction is blind to the disposition (§4)?
   Reconcile with boundary §8.1's synchronous-return depiction of the local case.
10. **Startup/recovery:** where fold-only rebuild ends and live fold+fire begins (the
    fire-checkpoint), distinct from the steady-state maintenance question (#6).
11. **Inbound seam:** how external and cross-context commands enter the single ordered
    frontier and serialise against reaction-issued commands (their *resulting events*
    keep replay sound; the inputs themselves are exogenous) (§1, §9).
12. **Time-triggered firing** (a timeout: "if X hasn't happened within T") has no event
    and no position, so it does **not** fit the position-ordered frontier — forward-ref
    the deferred `Schedule` effect (boundary §10).
