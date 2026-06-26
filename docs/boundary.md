# Boundary design — the edge layer

**Status: v0.1, exploration.** The first articulation of eventric's *boundary* — the
edge where the hermetic DCB core meets the outside world. Reactions are its
centrepiece, but the boundary is broader: inbound translators, outbound publishers,
the effect model, and the command/query/event flows that cross it. This realises
[`vision.md`](./vision.md) §2 (reactions), §4 (the surface and the internal-vs-
published split), §5 (the channel), and §7 (views, the consistency split). It is
deliberately a *first cut* with worked examples to react to, not a settled spec —
the open forks are in §7.

---

## 1. Core and boundary

The model splits cleanly in two.

- **The core (internal).** Stream, events, projections, actions. **Pure,
  synchronous, strongly-consistent, replayable** — re-run any of it over the same
  events and you get the same answer, with no external consequence. This is the
  DCB world.
- **The boundary (the edge).** Reactions, listeners, publishers, the effect
  interpreter. **Impure, asynchronous, eventually-consistent, effectful, stateful** —
  it touches stores, the channel, and the outside world.

This is **ports-and-adapters (hexagonal)**, derived rather than imposed: the
content-opacity and fail-closed principles already insist that meaning and effect
live outside the substrate. The core knows nothing of the edge.

**The core speaks three verbs, and the boundary translates the world to and from
them:**

| Verb | Direction | Core handler | Consistency |
|---|---|---|---|
| **Command** | *in* — a request to change state | an **action** (`act`) | strong (the DCB decision) |
| **Event** | *out* — a fact that happened | the **stream** (append) | strong within, eventual across |
| **Query** | *out* — a question about state | a **projection / view** | eventually consistent |

The boundary is **symmetric**:

- **Inbound** turns the world *into the core's verbs*: an external stimulus — a
  foreign published event off the channel, an external signal — becomes a
  **command** (which an action handles) or feeds a **view**. The translator here is
  an anti-corruption layer (a *listener*).
- **Outbound** turns *core events into world-effects*: a **reaction** observes
  internal events and decides on effects, which executors (*publishers*, view
  writers, command dispatch, external clients) carry out.

So the deep pairing the vision already has — *action ↔ command, reaction ↔ event* —
extends to the whole edge: **inbound makes commands from the world; outbound
(reactions) makes effects from events.**

---

## 2. Effects as data

A reaction does **not** perform effects inline. It mirrors how an **action** already
works: an action doesn't append — it *stages events into a buffer*
(`act(&self, events: &mut Events, projections)`) and the Enactor performs the write.
Symmetrically, a reaction stages **effects into a buffer** and the runtime
*interprets* them:

```
react(&self, effects: &mut Effects, projections)   // the symmetric shape
```

**Effects-as-data, not effects-performed.** The reaction's decision is a **pure fold**
— *(triggering event + projections) → a list of effect descriptions* — and the
runtime owns the impure interpretation. This is the load-bearing choice of the whole
boundary, and everything good follows from it:

- the decision stays **pure, replayable, testable** — even an *effectful* reaction is
  pure at the decision layer; you test it by asserting on the effects it stages;
- the genuinely hard machinery — **delivery, idempotency, retry, fail-closed,
  ordering** — lives in the runtime interpreter, in *one* place, not smeared across
  reaction code;
- **observability (§5) sees every intended effect** — the platform can inventory what
  each reaction publishes, commands, and touches, because effects are declared data;
- the **impure resources** you flagged (a view-store handle, an HTTP client, the
  channel) are **capabilities the interpreter holds**, supplied to it — not ambient
  state the reaction reaches for. "What does this reaction need?" becomes a *declared*
  question.

**The effect algebra (first cut).** A small, closed-ish set:

| Effect | Meaning | Executor |
|---|---|---|
| `Publish(event)` | emit a *published* event to the contract | publisher → channel |
| `IssueCommand(target, command)` | request an action — own context or another | command dispatch / channel |
| `MaintainView(view, delta)` | update a materialised read-model | view-store capability |
| `CallExternal(request)` | a generic outside effect (HTTP, email, …) | external-client capability |
| `Schedule(at, command)` *(?)* | do something later (timers) | scheduler *(open — §7)* |

Whether the set is genuinely *closed* (and `CallExternal` the escape hatch) or
*extensible* is an open question (§7) — but a small, named, interpreter-owned algebra
is the shape.

---

## 3. Where the translation logic sits

The recurring question, answered once: **meaning lives at the edges; mechanism lives
in the middle.**

- **Outbound translation** — *internal event(s) → what to publish / command /
  maintain* — lives in the **reaction**. The internal→published mapping (§4: derived,
  not leaked, possibly aggregating several internal changes into one published event)
  is a reaction's whole job.
- **Inbound translation** — *foreign stimulus → a core command (or view update)* —
  lives in the **listener / inbound adapter** (the context's anti-corruption layer).
  This is where §4's "foreign things are translated *in*, never appended raw"
  actually happens.
- **Execution / mechanism** — channel sends, view writes, command dispatch, external
  calls, and the cross-cutting delivery/idempotency/fail-closed/observability — lives
  in the **runtime interpreter**. It carries *no domain meaning*; it only executes the
  algebra.

So a context's domain logic is concentrated in three pure-ish places — **actions**
(command → events), **reactions** (events → effects), and **listeners** (stimulus →
command) — and the runtime is domain-blind plumbing.

---

## 4. The sync/async character — the key asymmetry

Not all boundary crossings have the same shape, and the difference is forced by the
consistency split (§7):

| Crossing | Sync/async | Outcome / return |
|---|---|---|
| **Command** (in or out) | **async** | the outcome is an **event** (success or `…Rejected`) the issuer observes — never a synchronous return |
| **Event published** (out) | **async** | fire-and-(eventually)-observed |
| **Event foreign** (in) | **async** | ingested → a command or a view update |
| **Query** (in or out) | **synchronous** | a **request → response**; reads don't mutate, so no loop, no outcome-event |
| **View maintenance** | **async** | eventually consistent with the stream |

The headline: **commands and events are asynchronous; queries are synchronous.**
A query is a *read* of an eventually-consistent view — it needs none of the
decision/loop/strong-consistency machinery, so it returns a value directly. A command
is a *request to change state*; its outcome is fallible (an action can reject:
validation, a DCB conflict, fail-closed on an event it can't read) and is delivered as
an **event**, because (a) cross-context it goes over the channel and *cannot* be
synchronous, (b) a synchronous intra-context command would break intra/inter-context
**uniformity**, and (c) an outcome-event is the fail-closed, event-sourced default.
*(A synchronous intra-context fast-path is possible as an optimisation; the open fork
in §7 is whether to expose it.)*

---

## 5. Worked examples

Each: the scenario, the flow, the effects staged, where the translation logic sits,
and the sync/async character. `→` is a step; `⤳` is an asynchronous hop (channel or
deferred).

### 5.1 Internal event — *within · core, no boundary* (baseline)

An action handles a command and appends an internal event. This is **the core** — no
boundary involved; shown only as the contrast everything else is measured against.

```
command → action.act(events, projections) → Enactor appends Event to the stream
```

*Logic:* in the action. *Effects:* none (it's a core write). *Character:* synchronous,
strongly consistent (the DCB decision).

### 5.2 Command — *within context · async*

A reaction reacts to an internal event, reads projections to decide, and wants to
trigger an action in its **own** context.

```
Event committed → reaction.react(effects, projections)
                    stages  IssueCommand(self, Cmd)
  runtime dispatch: Cmd → command-handler → action.act(...) → appends Event'
  ⤳ Event' (the outcome) is observed by the reaction (as a process manager) to advance
```

*Effects:* `IssueCommand(self, Cmd)`. *Logic:* the *decision* (which command) in the
reaction; the *command → action* mapping in the runtime's dispatch. *Character:*
async — the outcome (`Event'`, or a `CmdRejected` event) comes back through the
stream, not as a return value. This is why a reaction that depends on the outcome is
a **process manager** (§6).

### 5.3 Command — *cross-context · async*

A reaction in context **B** decides to ask context **A** to do something.

```
Event committed in B → reaction.react(...) stages IssueCommand(A, Cmd)
  runtime ⤳ channel ⤳ A's inbound listener → A's command-handler → A.action → A appends Event_A
  A's reaction stages Publish(OutcomeEvent) ⤳ channel ⤳ B's listener → B view/command
```

*Effects:* `IssueCommand(A, Cmd)` in B; `Publish(Outcome)` in A. *Logic:* B's reaction
decides the command; **A's listener** translates the inbound command into A's own
vocabulary (anti-corruption); A's reaction decides what to publish back. *Character:*
async throughout; the outcome is a **published event** B observes — there is no
synchronous cross-context call.

### 5.4 Published event — *cross-context, outbound · async*

B has changed internally and must tell the world, per its **contract** (§4).

```
Event(s) committed in B → reaction.react(...) stages Publish(PublishedEvent)
  runtime publisher ⤳ channel ⤳ every subscribed context's listener
```

*Effects:* `Publish(PublishedEvent)`. *Logic:* **in the reaction** — the
internal→published translation, which need not be 1-to-1 (it may aggregate several
internal events into one published fact, §4). *Character:* async, fire-and-observed.
Note the published event is a *designed contract type*, not a leaked internal event.

### 5.5 Foreign event — *cross-context, inbound · async*

A's published event arrives at B. B never reads A's stream; it receives the published
fact over the channel.

```
A's PublishedEvent ⤳ channel ⤳ B's listener.translate(PublishedEvent)
   → either:  IssueCommand(self, Cmd)   (B reacts by changing its own state)
       or:    MaintainView(view, delta) (B updates a local read-model of A — see 5.9)
```

*Effects:* `IssueCommand` and/or `MaintainView`. *Logic:* **in B's listener** — the
anti-corruption translation of A's contract into B's vocabulary. A's event is *never*
appended raw to B's stream; if it becomes part of B's history, it does so as B's *own*
internal event, via a command. *Character:* async ingestion.

### 5.6 Query — *within context · synchronous*

Something asks B a question the contract's **Query** surface (§7) answers.

```
Query → query-service → (ephemeral projection fold  |  read of a materialised view) → Response
```

*Effects:* none (read-only). *Logic:* the query-service routes to the projection or
view; the *projection logic* (event → state) is the same fold used everywhere.
*Character:* **synchronous** request→response, against eventually-consistent data — and
so, per §7, **never load-bearing for a decision**, only for whether to *request* one.

### 5.7 Query — *cross-context · two flavours*

B needs an answer about A's data. Two designs, and the vision leans to the second:

- **(a) Remote query** — B asks A's query surface synchronously over the channel.
  Simple, but couples B's read availability to A being up.
  ```
  B → ⤳ channel ⤳ A's query-service → Response ⤳ B
  ```
- **(b) Local anti-corruption view (preferred default)** — B maintains its *own* view
  fed by A's *published events* (5.5 → 5.9), and queries it **locally** (5.6). No
  cross-context call at query time; B stays available even if A is down — the decoupled,
  sealed-context posture the vision favours.
  ```
  (continuously) A's PublishedEvents ⤳ B's listener → MaintainView(localViewOfA, …)
  (at query time) Query → B's query-service → read localViewOfA → Response   [local, sync]
  ```

*Logic:* (a) lives in A's query-service; (b)'s translation lives in **B's listener**
(A's published events → B's local view). *Character:* (a) synchronous-remote; (b)
synchronous-local with async upkeep.

### 5.8 Maintaining a view — *of own events · async*

A read-model B serves queries from.

```
Event committed in B → reaction.react(...) stages MaintainView(view, delta)
  runtime → view-store capability writes the delta
```

*Effects:* `MaintainView(view, delta)`. *Logic:* the projection fold (event → delta)
in the reaction; the *write* is the runtime's, against the view-store capability.
*Character:* async, eventually consistent. This is the **pure** kind of reaction —
idempotent and replayable (re-folding rebuilds the view); the §7 "rules a
view-maintaining reaction plays by" (rebuild, staleness) govern it.

### 5.9 Maintaining a view — *of another context's events · async*

B's local read-model of A's data (the engine behind 5.7b).

```
A's PublishedEvent ⤳ channel ⤳ B's listener.translate → MaintainView(localViewOfA, delta)
```

*Effects:* `MaintainView`. *Logic:* **in B's listener** (anti-corruption: A's contract
→ B's local model). *Character:* async. This is how reuse/composition (vision §9)
stays decoupled — B depends on A's *published contract*, never A's internals or
availability.

---

## 6. The loop, and why reactions are stateful

Pulling 5.2/5.3 together: because a command's outcome is an **event**, a reaction that
needs to *act on the outcome* cannot block on a return — it must **remember it issued
the command** and react again when the outcome event arrives. That is precisely a
**process manager / long-running coordinator** (vision §2): event-triggered state with
responsibilities.

```
Event → reaction decides → IssueCommand → … ⤳ OutcomeEvent → reaction advances → next effect → …
```

And — per uniformity (§1) and self-hosting (§5) — that **state is itself
event-sourced**: a process manager's progress, including its checkpoint/cursor, is
**events in a stream**, not a CRUD side-table. So the usual "where does the reaction's
position live?" question already has a vision answer: *in the model*. The boundary is
effectful, but its *memory* is still event-centric.

---

## 7. Open questions / the forks

- **Command-outcome model.** Async-outcome-as-event is the lean (§4). The fork: do we
  also expose a **synchronous intra-context fast-path** (a `Result` you can `?`), or
  keep the model uniformly async and treat sync as a pure optimisation under the hood?
  This sets the ergonomic character of the whole boundary.
- **Delivery semantics.** Fail-closed rules out silent at-most-once. The fork:
  **at-least-once + idempotency** (effects carry an idempotency key the interpreter
  dedupes) vs. **exactly-once** (heavier, needs transactional outbox-style machinery).
- **Is the effect algebra closed?** A small fixed set (`Publish`/`IssueCommand`/
  `MaintainView`/`CallExternal`) with `CallExternal` as the escape hatch, or an
  extensible/registrable set? Closed is more observable and analysable.
- **Where reactions run, and scaling.** In the context's process (the §3 lean) or as
  separate deployables — reactions are flagged as the likeliest thing to scale out
  (§10). Effects-as-data helps: a pure decision can run anywhere; only the interpreter
  needs the capabilities.
- **Ordering & concurrency.** In stream order? Per-tag? May reactions process
  concurrently, and how is order preserved where it matters?
- **Retry / dead-letter / poison.** What the interpreter does when an effect fails
  repeatedly — and how that interacts with fail-closed (halt the reaction? quarantine
  the event? alert?).
- **Scheduling / timers.** Is `Schedule` a first-class effect (process managers often
  need "if no response in T, do X"), and is a timer just a scheduled command?
- **Listener formalisation.** Inbound translation is named here but not modelled —
  does a *listener* get a derive/trait like actions and reactions, closing the
  inbound/outbound symmetry?
- **Cross-context query default.** 5.7(b) local-view as the default, with 5.7(a)
  remote-query as an opt-in — confirm, and decide whether remote query is even in
  scope for v1.

---

## 8. Summary

The boundary is the **asynchronous, effectful, hexagonal edge** around the synchronous
DCB core. Its one structural idea is **effects-as-data**: reactions (and listeners)
stage typed effect descriptions; a domain-blind runtime interprets them, owning all
the hard delivery machinery. Meaning sits at the edges (reactions outbound, listeners
inbound); mechanism sits in the middle. Commands and events cross asynchronously
(outcomes are events); queries cross synchronously (reads of eventually-consistent
views). And a reaction that awaits an outcome is a process manager whose own state is,
like everything else, event-sourced.
