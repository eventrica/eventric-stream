# eventric — Runtime

**Status: v0.3 — processing model resolved (pending two confirmations).** The
design of the **runtime**: the mechanism that drives a *context* — how appended
events are processed, in what order, under what consistency, and how it invokes
actions, reactions, and effects. Since v0.2 a **deep dive** (four candidate
processing models, each adversarially reviewed, then synthesised) plus the dialogue
decisions below have settled the runtime onto a **single-threaded** design ("the
Drain", §3). Companion to [`vision.md`](./vision.md) (the *why* — node/runtime/
context §3), [`boundary.md`](./boundary.md) (the edge: contracts, effects, verbs),
and [`reactions-plan.md`](./reactions-plan.md) (the slice that proved the *pieces*).
Lives in `eventric-runtime`.

### Where this stands (read this first)

- **Decided:** fold-on-demand, no held state (§2); the Drain — one thread, one FIFO
  work-deque, lag-gated intake, no run-to-completion (§3); the **reads-vs-effects**
  lifecycle (§4); propose/dispose with strict **per-event as-of-trigger** visibility
  (§5); emergent position-ordered firing (§7); resume-not-restart crash model (§11).
- **⚑ Proposed, pending your confirmation (§6):** reaction commands are
  **asynchronous and blind to disposition**, *not* synchronous — this **reverses**
  the earlier "reaction command is sync" assumption; the justification is strong but
  it is yours to ratify.
- **⚑ Open fork, yours to call (§7):** the **queue discipline** — *FIFO-merge*
  (lean) vs *cascade-priority*.
- **Build gaps + residual risks:** §12, §13, §14.

---

## 1. What the runtime is

The runtime is the **deterministic driver** of a single context: one stream, one
writer (vision §3), one total order. The stream is the only source of **truth** and
**order**. The determinism is precise — and narrower than it first looks:

- **Read-state is a function of the stream prefix.** Projections/views are folds of
  the prefix in position order, so re-processing the same stream reproduces the same
  read-state and the same reaction *proposals*. This is the real replay guarantee.
- **The written stream is *not* reproducible from an earlier prefix.** What gets
  appended depends on *exogenous inputs* (external/cross-context commands — §3, §12)
  and on actions reading **as-of-head** (§5). So the runtime does **not** promise
  "re-running writes the same stream" — only that, given the stream as written, all
  read-state and reaction proposals replay identically.

## 2. State — fold-on-demand, nothing held *(decided)*

**The runtime holds no derived state.** Every projection and every view is a **fold
over the stream prefix, computed on demand** — to `[..=p]` for a reaction (§5), to
head for an action (§5). Held/incremental projection state is a **later mechanical
cache** (a performance lever, §3 throughput), never part of the logical model.

Consequence: there is no "projection vs materialised view" distinction in the model —
**a view is just a fold you run when you read it.** This demotes `MaintainView` out of
the baseline effect set (§9): maintaining a held view is exactly the deferred cache,
not a primitive. A reaction therefore never *maintains reads*; **views derive
(folds); reactions cause (effects, §6).** Phase A's view-maintaining reaction is
reclassified as an optimisation demo, not the model.

The only state the runtime keeps is its **progress** (the fire-checkpoint) plus the
durability needed for crash-safety (§11) — and the checkpoint is *not* stream-derivable
(§11, parked idea).

## 3. The processing model — "the Drain" *(decided; one fork open, §7)*

One **context thread** — the **sole appender and sole firer** — drains a single
heterogeneous **FIFO work-deque** holding: external/cross-context commands,
reaction-issued commands, and per-position reaction-fires. **No run-to-completion.**
Each iteration: dequeue one item; if it is a *fire*, fold the position's reactions'
as-of-trigger projections on demand and interpret their staged effects (push local
commands to the deque tail, hand publishes/external/cross-context messages to the
durable outbox, §11); if it is a *command*, enact it in-thread (fold as-of-head,
decide, append under the DCB guard as one atomic fjall batch) and advance the
fire-checkpoint.

- **Loop boundary.** The cut **is the stream**: *touches-only-local-stream* ⇒ inside
  the one thread; *touches-the-world* ⇒ outside (inbound adapters, the bounded ingress
  channel, the outbox drainer, query-serving on cloned `Reader`s), conversing with the
  loop solely through durable **ingress** (in) and durable **outbox** (out). The
  Writer lives *only* on the loop thread, so single-writer-per-context is structural.
- **Ingress — lag-gated single pull.** One bounded channel is the single entry for
  *all* command origins (origin is an envelope field, not a second mechanism —
  boundary §6, one command path). The loop pulls **exactly one** external command per
  iteration, **gated on frontier lag** (`head − checkpoint`), so the bounded channel
  genuinely **fails closed** and propagates backpressure — it does *not* drain into an
  unbounded deque. Reaction-issued local commands skip the channel and push to the
  deque tail. Reads bypass everything (cloned `Reader`s); writes and fires never bypass.
- **No quiescence; bounded lag.** A deep cascade cannot starve ingress (new cascade
  fires queue behind the one admitted external command per iteration), so the live
  property is **bounded frontier lag**, not a quiescent point — which is precisely why
  the model is *not* run-to-completion (contrast §13's strict-RTC candidate).
- **Throughput.** One core's worth of fold+fire+enact per context — the deliberate
  single-writer ceiling (vision §3); scale is **across contexts**, never within one.
  Reads scale out on cloned `Reader`s off-thread.

**Why single-threaded (justification).** The two *decoupled* (multi-threaded) models
scored worse on crash-safety and simplicity; their only real win — pipelined
throughput — is **deferrable with no semantic change**: because as-of-trigger folds
touch only the immutable prefix, the fold/fire side can later be split from the
enact/append side across the "committed-stream membrane" (a lagging fold thread
changes *when* a reaction fires, never *what* it sees). So pipelining is a documented
**escape hatch** (§12), not the base model. The *strict run-to-completion* model buys
whole-stream reproducibility — which §1 explicitly disclaims we need — at the price of
catastrophic liveness (a non-terminating cascade wedges the *whole* context) and a
return to quiescence. The Drain gets fairness and bounded lag while staying single-
threaded and simple.

## 4. The event lifecycle — reads vs effects *(decided; reframed)*

The lifecycle line is **effect character, not fold-vs-fire mechanism** (v0.2 mis-cut
this):

- **Reads** (pure folds, no external/durable side effect): projections and views.
  **Free, replayable, recomputable anytime, no checkpoint.** (§2: there is nothing
  held to get out of sync.)
- **Effects** (impure/external): a **command** issued, an event **published**, an
  external **call**. These must fire ~once and **not** be replayed blindly on a
  rebuild.

The line cuts **through** reactions: a reaction's *reads* are free folds; its
*effects* are once-only. The honest crash guarantee for effects is **at-least-once +
idempotent = effectively-once** (§11), never magic exactly-once.

## 5. Consistency — propose vs dispose *(decided)*

Reactions and actions read at **different points**, and the asymmetry is load-bearing:

- **Reactions read as-of-trigger** — a reaction fired at position *p* sees read-state
  folded over **`prefix[0..=p]`**, the world *as of its own event in the total order*.
  This is **forced** by the §1 replay invariant. Two refinements that matter:
  - **Per-event, not per-batch.** Even when an action committed `[E1,E2,E3]`
    atomically, E1's reaction sees `[..=p]` and **not** E2/E3 (later positions). Each
    reaction sees the prefix up to *itself*. (The coherent alternative — *batch
    visibility*, where every reaction in an append sees the whole batch — is
    **rejected**: it would make the unit of consistency the append, so a reaction's
    view would depend on what *else* its triggering action happened to commit.)
  - **Implementation discipline.** This forces **fold-and-fire strictly
    position-by-position, interleaved** (fold E1 → fire E1 → fold E2 → …), *never*
    batch-fold-then-batch-fire (which would leak the rest of the batch into the
    earlier event's reaction). With fold-on-demand (§2) it is also explicit: the fold
    is bounded to `[..=p]`, leak-proof by construction.
- **Actions read as-of-head** — an action is a *write*; it must be consistent with the
  real head it appends against (read head, append under the **DCB guard**).

Hence **reactions propose, actions dispose.** A reaction says "I want this" from its
local as-of-trigger view; the action enforces consistency at the write and **may
reject**. This asymmetry is **correctness-coherent** (the action is always consistent
with what it appends against) but is **not a determinism guarantee** — the written
stream depends on the dispatch-time head and the firing order of co-triggered
reactions (§7). (Illustration: E1's reaction's *command* runs an action reading
as-of-head — already `p+2` — so the **action** sees E2/E3 even though the reaction
that proposed it did not. The split is doing real work.)

## 6. Reaction commands — async + blind ⚑ *(proposed, pending confirmation)*

**Recommendation: reaction commands are asynchronous, enqueued, and blind to
disposition** — the Enactor's `Result` returns to the **runtime** (bookkeeping /
dead-letter, §11), never into `react()`. This **reverses** the earlier "reaction
command is sync" assumption. The reasoning:

- **"Synchronous" (timing) and "blind to disposition" (data-flow) are orthogonal.**
  `react()` stages a Command *message* and returns; its stack frame is gone before the
  runtime disposes the command (later, when the work item is dequeued — genuinely
  distinct propose/dispose moments). So even an eagerly-run command cannot hand a
  result *back into* the reaction.
- **A sync result buys nothing usable.** Under a *single writer* a local command
  **cannot be DCB-rejected** — nothing writes between the action's read and its append
  — so the disposition a sync call would return is *always* "accepted." The one place
  rejection is real (**cross-context**) is exactly where you *cannot* block (a remote
  await on the single thread is head-of-line blocking + distributed deadlock — §10).
  So sync is empty locally and unavailable remotely.
- **Meaningful outcomes are facts.** Because rejections are written *nowhere* (boundary
  §3: only facts are events), any flow-relevant negative outcome of a
  reaction-commanded action **must be emitted as a fact** the reaction re-observes by
  being re-triggered (a `PaymentDeclined`). Only *non-fact* dispositions (a
  cross-context DCB conflict, an infra failure) stay runtime-only, for the dead-letter
  channel (§11). This is exactly what makes the **process-manager-as-pattern** work
  (issue command → react to the resulting fact → carry coordination state in a
  projection), and it is consistent with the vision.

This corrects boundary §8.1's "← Result returned to the reaction" to "returned to the
**runtime** that staged on the reaction's behalf." **Cross-context** reaction-commands
are necessarily async-out (to the outbox; any reply returns as a later inbound
fact/command). *Local-inline is what single-threading makes safe; remote-async is what
it makes necessary.*

## 7. Ordering — emergent, and the one open fork ⚑

- **Serialization point = the single dequeue.** The dequeue+commit sequence *is* the
  one total order.
- **Ascending fire-order is emergent**, not a priority queue: positions are assigned at
  append by the same thread that pushes each `Fire{p}` before its next dequeue, so a
  plain FIFO deque preserves position order (single-appender + same-thread
  append-then-enqueue).
- **Co-triggered reactions need a stable, declared firing order.** Their *reads* are
  order-free (all see the same as-of-trigger state), but their *commands* are not — the
  second action reads the first's append as-of-head, so order is semantically
  significant *through as-of-head dispatch*. "Independent reactions are order-free" is
  false once they command.
- **Fold-then-fire and breadth-first** are preserved (the stream *is* the FIFO
  work-queue; `enact` only appends at head, the frontier still advances `p, p+1, …`).
  These are consequences of the spine, not open forks.
- **Outbound order** must be pinned by a strict-FIFO outbox drainer (so a downstream
  context's ingress is not reordered).

**⚑ The one genuine decision — queue discipline (yours to call):**
- **FIFO-merge** *(lean — mine and the synthesis's)*: one merged queue; an external
  command *may* interpose between a trigger at *p* and the command it caused. **Fair,
  bounded acceptance latency.** Its named cost — interposed external state "polluting"
  the co-triggered command's as-of-head read, so the written stream is non-reproducible
  — is, I argue, **already paid**: §1 disclaims whole-stream reproducibility, and the
  "drift" is just propose/dispose working as designed (the reaction proposed as-of-
  trigger; the action disposes as-of-head, including whatever is at head).
- **Cascade-priority:** drain a cascade's reaction-commands before admitting new
  external work. Cascade stays contiguous, stream closer to reproducible — but it
  reintroduces **head-of-line blocking / quiescence**, which §3 rejected, and grows
  external acceptance latency without bound.

Through the lens of decisions already made it tilts to FIFO-merge (its downside is
already accepted; cascade-priority's is one we rejected) — but it materially changes
latency and determinism character, so it must be a **conscious, documented** choice.

## 8. Reactions gain projections *(decided)*

A reaction reads projections (it is not event-only), making it **symmetric** with an
action:

- **action** = (command + projections) → events
- **reaction** = (event + projections) → effects

Same fold machinery, read **as-of-trigger** (§5). A *set* of single-event reactions
sharing a **tag-keyed, per-instance** projection is the **process-manager pattern**
(vision §2) — and those per-instance projections are the *unbounded* ones folded on
demand (§2), never held resident.

## 9. Effects — pluggable and introspectable

Effects must be **pluggable**: we cannot enumerate every effect future contexts need.
The lean (shared with boundary §10, reactions-plan — **still a lean, not committed**)
is a **trait-based `Effect`** rather than a closed enum, each carrying static metadata
(kind, target). The runtime holds an **interpreter per effect kind**; staging an effect
is data, interpreting it is the runtime's job. The static **topology graph** comes
primarily from the typed **`Emits`** buffer (boundary §4 — declared emissions that
cannot drift); per-effect metadata supplements it.

The baseline set (note: **`MaintainView` is *not* here** — a view is a fold, §2/§4):
- **Command** — route to its action (`From<Command>`) and enact (settled;
  `reactions-plan.md`).
- **Event (publish)** — emit a public event (the third verb; **deferred**, named so the
  Command/Query/Event trio stays intact).
- **Query** — *emit* a synchronous read request (**provisional / at-risk**, §10).

*Open:* the `Effect` trait's shape; interpreter registration/dispatch; whether built-ins
share the trait or sit beside it; trait-vs-enum (still a lean). (We say **fire** for
invoking a reaction, reserving *dispatch* for the existing mask-routing / command-
routing senses.)

## 10. Query as an effect (provisional)

A **query effect** is the *emit* side: a reaction/action asks (e.g. another context),
awaits a reply, uses it. **Provisional** — the one effect that strains the model:
- **Liveness.** Blocking the single thread on an external reply **head-of-line-blocks
  the whole context**; a slow/down service freezes it (fail-open, against vision §6);
  and two contexts querying each other synchronously is a **distributed deadlock**. A
  blocking call in the loop is therefore likely wrong; an **async exchange** (issue →
  suspend the causal flow → resume on the reply) is the candidate shape.
- **Determinism.** A reply is not in the stream, so a reaction branching on it is not a
  function of the prefix. Recording the raw reply *as an event* would violate boundary
  §3 (results are returns, only facts are events) and freeze a stale read into the log.
  If a reply must be captured for replay, it is runtime bookkeeping or a context-
  *asserted* fact (`ObservedXAtP`), never the raw reply.

Distinct from the **inbound** side (a context *serving* a query, vision §2), which is
**deferred**.

## 11. Crash & recovery *(decided)*

`fjall` is the only durable truth (atomic batch appends, no position gaps, no torn
reads). **Resume, don't restart:** re-fold all replayable read-state from the prefix
(free, §2/§4), then re-fire once-only from `checkpoint+1` to head.

Durable state the model requires (none of it exists on today's substrate — §12):
- a **per-position fire-checkpoint** — **must be persisted**; it is *not* reliably
  stream-derivable (a no-op fire and a view-only fire leave no caused-by event — this
  is why the "stream-derived / fully-stateless checkpoint" idea is **parked**, see
  below);
- a **durable ingress log** for accepted-but-not-yet-appended external commands (else a
  crash is fail-*open*, against vision §6);
- a **durable outbox** for publish/external/cross-context effects.

**Effectively-once = at-least-once dispatch + idempotent effect.** Dedup is **scoped by
effect kind**: a **command** effect dedups from the stream via a **causation id**
`(p, k)` — "does an event caused-by `(p,k)` exist?"; **publish/external-call** effects
leave no local event, so they dedup at the **outbox**, keyed by `(p,k)` at the
consumer. The **DCB guard does *not* dedup a replay** (it is a concurrency guard, not a
dedup) — absorption lives in the action's idempotent decision or the explicit causation
key. A multi-effect fire is the checkpoint grain, so a mid-fire crash re-runs the whole
fire → **each effect must be individually idempotent**.

> **Parked idea — stream-derived (zero-state) checkpoint.** Explored: derive "has *p*
> fired?" purely from causation tags, making the runtime fully stateless. Parked
> because it covers only local-command effects: **no-op and view-only fires leave no
> trace** (so it cannot be a resume pointer, only a dedup guard), and **external
> effects leave no local event** (so they need the outbox anyway). Recorded so we do
> not re-derive it.

## 12. Where the code stands + build gaps

- **`Reactor<R>` is a stepping-stone, retired by the Drain.** Its per-reaction `drive`
  loop (and `MAX_PASSES`) reorders relative to the stream; the single position-ordered
  frontier replaces it (optionally woken by a commit watermark rather than polling).
- **The `Enactor` stays** — the action cycle (fold as-of-head, decide, append under
  DCB). The runtime invokes it both to interpret a command effect **and** to serve
  **inbound** commands (boundary §6's one command path — external/cross-context
  commands are *exogenous*, not prefix-derived).
- **Build gaps the model implies (do not exist today):**
  1. an **upper-bounded fold** — firing at *p* must fold `[..=p]` (`take_while pos<=p`);
     today's `Condition` carries only a lower `from` bound. **Load-bearing.**
  2. a **causation id** on events (they carry only position + timestamp now) — for
     command-effect dedup; naturally modelled as a **tag** so the existing tag index
     answers it (and it doubles as cause→effect traceability).
  3. **durable ingress log + outbox + per-position checkpoint** (the §11 crash story).
  4. an **external-command idempotency key** has no natural source (no triggering
     position) → caller-supplied.

## 13. The candidate models considered (justification record)

The deep dive designed four models, reviewed each adversarially (scores: ordering /
determinism / throughput / simplicity / sync-support / crash-safety), and synthesised
the Drain from the field:

- **Strict run-to-completion** (single thread, inline-sync local commands) —
  4/4/3/4/4/3. Buys whole-stream reproducibility; **rejected** — catastrophic liveness
  (a runaway cascade wedges the entire context, undefendable statically) and a return
  to quiescence §3 rejected, for reproducibility §1 says we don't need. *Grafted:* the
  timing-vs-data-flow sync reframe (§6); causation-dedup scoped by effect kind (§11);
  resume-not-restart (§11).
- **Single-thread unified FIFO ("the Drain")** — 4/4/3/4/4/4. **Recommended base.**
  Best-balanced; honours bounded-lag; clean sync resolution; its raw flaws
  (backpressure, cross-context, vestigial-DCB) were all graftable fixes. *Grafted:*
  emergent fire-order; lag-gated single-pull intake; immutable-prefix safety; per-
  external reply oneshot + frontier-lag as a backpressure/introspection signal.
- **Decoupled writer + tailing reactor (async)** — 3/4/3/3/2/2. Buys pipelined
  write-latency; **deferred not adopted** — reactor starvation under load, an
  at-least-once claim that is at-*most*-once in a crash window (in-memory queue +
  decoupled checkpoint). *Grafted:* the committed-stream-membrane as the **deferred
  pipelining escape hatch** (§3); retiring `Reactor::drive`/`MAX_PASSES` (§12).
- **Decoupled + synchronous (bicameral)** — 3/3/3/2/3/2. Engaged sync hardest, but its
  marquee value **collapses**: the position-based DCB guard is **inert under a single
  serial writer**, so the local conflict-rejection that sync exists to deliver can
  never fire, while sync is barred cross-context where conflicts are real. *Grafted:*
  non-fact dispositions are invisible to the fact path → the runtime **dead-letter**
  channel + the "meaningful negatives are facts" discipline (§6); the acyclic-wait
  invariant.

## 14. Status & open items

**Decided:** §2 fold-on-demand / no held state · §3 the Drain (single thread, FIFO
deque, lag-gated intake, no RTC) · §4 reads-vs-effects · §5 propose/dispose +
per-event as-of-trigger · §7 emergent ordering · §11 resume-not-restart + effectively-
once.

**⚑ Awaiting you:**
1. **Confirm §6** — reaction commands async + blind (reverses the sync assumption).
2. **Decide §7** — queue discipline: FIFO-merge (lean) vs cascade-priority.

**Residual open / risk (post-decision):**
- Cascade non-termination has **no sound static defense** (halting problem); best is
  build-time topology cycle-detection (best-effort) + a runtime **divergence budget**
  that dead-letters — *containment, not prevention*. Under FIFO-merge a runaway appends
  junk but does not wedge the context.
- **Per-fire on-demand fold is on the critical path** — for a long-lived per-instance
  projection folded on every triggering event this is ~`O(fires × history)` over the
  instance's life (each fold bounded-by-result, but the *repetition* is the cost). The
  only fix — the held/incremental cache — is explicitly **deferred** (§2/§3).
- The **query effect** shape (§10) — blocking-vs-async, determinism, reply capture.
- The **`Effect` trait** shape and registration (§9); trait-vs-enum still a lean.
- **Startup/recovery** precise sequencing (where fold-only rebuild ends and live
  fold+fire begins) and **reaction onboarding** (a new reaction: catch-up vs
  start-at-head) — fold rebuilds free; once-only effects cannot.
- **Time-triggered firing** (a timeout) has no position, so it sits *outside* the
  position frontier — forward-ref the deferred `Schedule` effect (boundary §10).
