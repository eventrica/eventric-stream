# Boundary design — the edge layer

**Status: v0.2, exploration (substantially revised).** The boundary is eventric's
edge — where the hermetic DCB core meets the outside world. Reactions are its engine,
but the boundary is broader: the **public/private contract membrane**, the three
verb-ports, the message envelope, and the inbound/outbound flows. It realises
[`vision.md`](./vision.md) §2 (reactions), §4 (the surface, internal-vs-published),
§5 (the channel, observability), and §7 (views, the consistency split) — and it turns
out to supply the *mechanism* for the versioning story (§4/§9). Still exploratory; the
open forks are in §10.

---

## 1. Core and boundary

The model splits in two:

- **Core (internal).** Stream, events, projections, actions — **pure, synchronous,
  strongly-consistent, replayable.** The DCB world.
- **Boundary (edge).** Reactions, adapters, the envelope, the runtime interpreter —
  **impure, asynchronous, effectful, stateful.** The world-facing skin.

This is **ports-and-adapters (hexagonal)**, derived not imposed: content-opacity and
fail-closed already put meaning and effect *outside* the substrate.

**Three verbs cross the boundary, in both directions:**

| Verb | What it is | Handler | Shape | Cardinality |
|---|---|---|---|---|
| **Command** | a request to *change* state | an **action** | request-response (returns a `Result`) | 1:1 (one responder) |
| **Query** | a request to *read* state | a **view / projection** | request-response (returns data) | 1:1 |
| **Event** | a *fact* that happened | a **reaction** | asynchronous, fire-and-observe | 1:many (pub-sub) |

**Symmetry:** *inbound* turns the world into the core's verbs (a stimulus becomes a
command/query, or feeds a view); *outbound* turns core events into world-effects (a
reaction emits). Commands and queries are request-response (one responder, 1:1); events
are publish (many reactors, 1:many).

## 2. The membrane: public and private

The load-bearing idea. Every verb exists in **two forms**:

- **Public** — the *stable contract*: versioned, shareable, accepted-from / emitted-to
  the outside.
- **Private** — the *unstable internal*: the current internal model, free to change
  with the code.

And the boundary **translates between them, in the direction of flow**:

- **Inbound:** stable **public** command/query → translate → unstable **private**
  command/query → action / view.
- **Outbound:** unstable **private** event → translate (a reaction) → stable **public**
  event → published.

**This is why the split is essential, not cosmetic.** It is the only way to keep a
stable contract while the internal model changes — internal churn is absorbed *in the
translation*; the public form doesn't move. Remove the split and every internal change
breaks the surface.

**It is the versioning mechanism.** The vision makes the contract the versioned unit
with internals evolving freely (§4/§9; the research is in
[`versioning.md`](./versioning.md)); *this is how.* You version the **public** form;
the inbound translation maps `public-vN → current private`; contract and internal model
version on independent clocks, reconciled in the translation. (So a chunk of the
versioning question is answered here, not separately.)

**Public and private are distinct *kinds*, not one type with a flag.** Distinct kinds
make the surface **un-leakable**: a private command simply *isn't* in the accepted set —
enforced by the type system, not by convention — and a private form can't be
accidentally depended on as public. They also have genuinely different lives (one
stable / shared / versioned, one free-to-change), which a flag can't carry honestly. So
internally the vocabulary is **private throughout** (private commands → actions, private
events in the stream); **public is purely the edge skin.**

## 3. Results are returns; the stream records facts

The consistency character of the three verbs:

- **Command and Query are synchronous request-response** — within *and* across contexts.
  Both return a `Result`. (Across contexts the channel carries request and response.)
- **Event is the asynchronous fact-stream** — published, fire-and-observe.

And the principle that keeps the stream clean: **results are returns; only facts are
events.** A command returns accepted/rejected (plus any output, like a new id); a
rejection is written *nowhere*. A *fact* — even a negative one (`PaymentDeclined`) or a
later one (`PaymentConfirmed`) — is an event. Control-flow ("did the command take?") is
a return; a thing-that-happened is a fact. Never pollute the fact-log with non-facts.

So **C + Q are the synchronous request-response interface; E is the asynchronous
notification layer** — exactly the vision's three-faceted surface (§4), split by shape.

## 4. Effects are messages, not handlers

A reaction does not perform effects or invoke handlers — it **stages messages** the
runtime interprets, mirroring how an action stages events (`act(&self, events: &mut
Events, …)`) rather than appending. A reaction stages **verb-messages**:

- a **Command** value → routed to an action,
- a **Query** value → routed to a view (awaits a result),
- an **Event** value → published.

There is **no "action effect"** — a handler isn't a message you can stage. You stage the
**Command**; the runtime routes it to the action. So **Command (message) and Action
(handler) are distinct types**, joined by an explicit **command → action mapping** (and
query → view, event → reactions). The general rule: *the boundary conveys the three
verb-messages; handlers are bound at the routing, never conveyed.*

This also keeps observability honest — every command is a first-class, traceable thing,
with no bypass that would put a blind spot at the internal hops.

Shape note: request-response effects (command, query, external call) are **awaited calls
with returns**, through mediated capabilities (so still testable/observable);
fire-and-forget effects (publish a public event) are pure staged data. That is the
pure-vs-effectful seam — a view-maintaining reaction is a pure, replayable fold; an
emitting/commanding one is imperative-with-returns.

## 5. The envelope

Every message — C, Q, or E, in any direction — travels wrapped in an **envelope**
carrying cross-cutting metadata:

- **correlation id** (the originating flow), **causation id** (what directly caused this
  message),
- **origin** (local / which remote context / external), **timestamp**, **trace context**.

The envelope does two jobs: it lets the runtime **distinguish local vs. remote without
branching the path** — origin is just a field — and it is the **substrate observability
runs on** (§5). The whole causal graph — command → events → reaction → command → … — is
reconstructable by following correlation/causation. Nothing bypasses the envelope; if a
message did, the introspection would gain a blind spot.

## 6. One path, locality-driven adapters

There is **one command path**: whatever the origin — a local reaction, another context,
an external caller — a command is routed and invoked the same way. No local fast-path,
no second mechanism; origin lives in the envelope.

But **transport is the adapter's job**, chosen by locality (ports-vs-adapters):

- **local / private** → **in-memory delivery** — no translation, no serialisation, no
  network hop;
- **remote / public** → the **channel** (or, for non-eventric targets, an external
  protocol — HTTP, SQL — under the maximally-uniform stance: external systems are just
  more C/Q/E through adapters).

So the uniform model costs nothing locally: a reaction commanding its own context uses
the **private** command in-memory; only genuine edge crossings pay translation +
transport. **"One path everywhere" does not mean "network-everywhere."**

## 7. Where the logic sits

**Meaning at the edges; mechanism in the middle.**

- **Outbound translation** (private event → public event; deciding what to
  publish/command/query) — in the **reaction**.
- **Inbound translation** (public command/query → private; a foreign event → a private
  command or view update) — in the **inbound adapter** (the anti-corruption layer).
- **Execution + cross-cutting** (routing, transport, the envelope, delivery, fail-closed,
  observability) — in the **runtime**, which carries no domain meaning.

A context's domain logic concentrates in **actions** (command → events), **reactions**
(events → effects, private→public out), and **inbound adapters** (public→private in); the
runtime is domain-blind plumbing.

## 8. Worked examples

`→` a step; `⤳` an asynchronous hop (channel/deferred); `[pub]`/`[priv]` the form.

### 8.1 Command — local (a reaction commands its own context)
```
Event committed → reaction stages Command[priv]  (enveloped, origin=local)
  → command port (in-memory) → command→action mapping → action.act(…) → appends Event[priv]
  ← Result (accepted + output | rejected) returned to the reaction
```
*Logic:* reaction decides, mapping routes. *Transport:* in-memory — no translation, no
serialisation. *Character:* synchronous return; only the appended events are facts.

### 8.2 Command — cross-context (B commands A)
```
B reaction stages Command[pub for A]  (enveloped, target=A)
  ⤳ channel ⤳ A inbound adapter: translate Command[pub]→Command[priv]
            → A command→action mapping → A.action → Event_A[priv]
  ← Result ⤳ channel ⤳ B
```
*Logic:* B decides; **A's adapter translates public→private** (anti-corruption +
version-absorption). *Character:* synchronous request-response over the channel (B awaits
A — accepted coupling; model with events to decouple). Any long-running completion is a
separate **public event** A publishes (8.4), not this return.

### 8.3 Query — local & cross-context
```
local:  Query[priv] → query port (in-memory) → view/projection read → Result
remote: Query[pub for A] ⤳ channel ⤳ A: translate→Query[priv] → A view → Response[pub] ⤳ B
```
*Character:* synchronous request→response, against eventually-consistent views — **never
load-bearing for a decision** (§7). Cross-context, you *may* instead keep a local view fed
by A's public events (8.6) and query it locally — an optimisation, not the general
mechanism.

### 8.4 Event — published (outbound)
```
Event[priv](s) committed → reaction translates → Event[pub]  (designed contract type, may aggregate)
  → publisher ⤳ channel ⤳ every subscriber's inbound adapter
```
*Logic:* **in the reaction** — private→public, derived not leaked (§4). *Character:* async,
fire-and-observe, 1:many.

### 8.5 Event — foreign (inbound)
```
A's Event[pub] ⤳ channel ⤳ B inbound adapter:
   translate → Command[priv]  (B changes its own state)   or   → view update (8.6)
```
*Logic:* **in B's adapter** — A's public event never lands raw; it becomes B's *own*
private command/event. *Character:* async ingestion.

### 8.6 Maintaining a view (own / foreign events)
```
own:     Event[priv] → reaction → write the view  (a Command[priv] to the view-store adapter)
foreign: A's Event[pub] ⤳ B adapter translate → update B's local read-model of A
```
*Character:* async, eventually consistent. The **pure** kind of reaction — idempotent,
replayable. Under the maximally-uniform stance, a view-store write is itself just a
private command to a view adapter.

## 9. The loop, and stateful reactions

A command returns its *immediate* result; if the *work* is long-running, the eventual
completion arrives later as a **published fact** (`OrderShipped`) the issuer reacts to — a
real event, not a control-flow outcome. A reaction that awaits such an outcome must
**remember it issued the command** and react again when the fact arrives — a **process
manager / coordinator** (vision §2). Its durable state, **including its checkpoint**, is
itself event-sourced — private events in a stream, not a CRUD side-table (uniformity §1;
self-hosting §5). The boundary is effectful, but its *memory* is event-centric.

## 10. Open questions / forks

- **How a contract is physically shared.** To *send* A a verb you need A's public contract
  (accepted commands, served queries, emitted events). A code dependency, a generated
  schema, something published over discovery? This is where cross-context type-coupling
  lives.
- **Ports↔adapters wiring.** Where "this verb → this transport (in-memory / channel / HTTP
  / SQL)" is declared, kept separate from the verb itself.
- **Public↔private translation surface.** Every public command paired 1:1 with a private
  one, or may translations fan-in/out? (Events already aggregate.)
- **Internal-only commands** — allowed: a Command is always a first-class *private*
  message; the *published* contract is the externally-accepted subset.
- **External fit.** Maximally-uniform (external = C/Q/E through adapters) is the provisional
  stance — revisit if forcing HTTP/SQL/streaming into verbs gets procrustean; a raw-adapter
  escape hatch may remain.
- **Delivery semantics.** Fail-closed rules out silent at-most-once; at-least-once +
  idempotency (the envelope id) vs. exactly-once is open.
- **Ordering / concurrency, retry / dead-letter, scheduling / timers** — runtime concerns
  still to specify.
- **Surface declaration.** Declarative, mirroring the internal derives: per-handler
  bindings + a context-level derive assembling the contract descriptor + routers +
  serialisation. Commands/queries reuse the Event derive's `Identifier` + `revision`
  machinery.

## 11. Summary

The boundary is the **asynchronous, effectful, hexagonal edge** around the synchronous DCB
core. Its spine is the **public/private membrane**: public = stable contract, private =
unstable internal, translation between them absorbing internal change — which is *why* a
contract can stay stable while internals churn, and which supplies the **versioning
mechanism**. Across it move **three verb-messages** (C/Q/E), each public-or-private, each
**enveloped** with correlation / causation / origin for traceability. **Commands and
queries are synchronous request-response (results are returns); events are asynchronous
facts (the stream records only facts).** Reactions **stage messages, never handlers**; a
domain-blind runtime routes and transports them — **in-memory when local, the channel when
remote** — so one uniform path costs nothing locally. Meaning lives at the edges (reactions
out, adapters in); mechanism in the middle.
