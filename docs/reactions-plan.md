# Reactions — minimal-slice plan

**Status: v0.1, draft plan.** An implementation roadmap for the first reaction slice
— the gating next build (vision §9). This is a *plan*, distinct from the design it
realises: the boundary model is in [`boundary.md`](./boundary.md), the vision in
[`vision.md`](./vision.md). Deliberately incremental and open to reshaping; the
conceptual breakdown below is a first cut.

---

## Goal & scope

Put boundary.md's *foundations* into code — the `React` trait, the reactor runtime,
and **effects-as-messages** — on the **simplest internal cases**, to learn whether
the design holds before building the elaborate parts.

**Explicitly deferred** (out of this slice): the channel / Iroh, public/private
contracts, cross-context, the message envelope (beyond a trivial in-process one), the
full effect algebra (publish / external / schedule), delivery semantics, the
`#[derive(Reaction)]` macro, and an event-sourced reactor checkpoint. All wait until
the slice validates the core.

**Where it lives:** `eventric-domain` — a new `reaction.rs` alongside
`action` / `event` / `projection` / `enactor`, plus a `Reactor` runtime. Hand-written
reaction impls first; the derive comes later, once the shape is proven.

## The trait shape (the key proposal)

A reaction reacts to **one event type** and is *built from it*, mirroring how an action
is built from a command — `From<Input>` for construction, an invocation method for
execution:

```rust
impl From<TheEvent> for MyReaction { /* capture what the trigger gives you */ }

impl React for MyReaction {
    fn react(&self, effects: &mut Effects) { /* decide + stage effects */ }
}
```

- **Single-event, not multi-event.** A reaction has *one* triggering event type, so
  `From<Event>` is coherent. Reacting to *many* types is what **projections** do (the
  multi-type fold); a reaction is a single *trigger*. The decision *state* it needs
  comes from **reading projections**, not from bundling triggers.
- **`From` builds it, `react` runs it.** Construction is `From` (infallible — it just
  captures the event); the fallible decision lives in `react`, as `Act` already returns
  a `Result`. This mirrors `From<Command> for Action` + `Act`, and realises the
  event→reaction (and command→action) "mapping" as a type-driven `From`, not a registry.
- **Ephemeral and stateless** — `&self`, not `&mut self`. The event is captured by the
  `From`; any persistent state lives *outside* the reaction (a view, or a projection it
  reads).
- **`effects`** is the staged-effects buffer (mirroring `Act`'s `Events` buffer) — a
  `MaintainView` delta for view-maintenance, `IssueCommand` for the loop. It is **typed
  by its output set** (`Emits`): a handler can only stage what it declares, so outputs
  are explicit and drift-proof — which feeds the static topology graph and the derived
  emitted-events contract (vision §5, boundary §4). The topology *tooling* is later; the
  buffer is **designed to carry `Emits` from the start** so we don't design it shut.

(Actions converge on the same shape later — `From<Command> for Action` + `Act`,
separating the command message from the action handler per the boundary decision. The
slice keeps actions on the existing `Enactor`; only reactions adopt From+invoke now.)

## The process manager is a pattern, not a primitive (conjecture)

We deliberately do **not** build a process-manager construct. The conjecture: a process
manager *emerges* from primitives we already have — a set of **single-event reactions**
sharing a **projection** as their coordination state. The projection folds the flow's
events into "where we are" (per-instance via tags); each reaction reads it and advances
the flow by issuing the next command. The state is inherently event-sourced (a
projection over events already in the stream — no separate checkpoint). The one part
that doesn't decompose without help is **time** ("if X hasn't happened in T"), which
needs a `Schedule` effect (deferred, boundary §10). Stance: **build the simple pieces;
add a dedicated construct only if the pattern fails to emerge.**

## Phase A — pure view-maintaining reaction

*Validates: the `React` trait, the reactor (tail + drive), effects-as-messages on its
simplest real effect (`MaintainView`), and a basic read.*

1. **`React` trait + `Effects` buffer** (`reaction.rs`), with its first effect,
   `MaintainView`.
2. **`Reactor` runtime** — given a reaction + its selection (`Condition`) + a
   checkpoint: `select` events from the checkpoint → build the reaction via
   `From<Event>` → `react(&self, &mut effects)` (it stages a `MaintainView` delta) →
   interpret the effects (apply the delta to the view store) → advance the checkpoint.
   In-memory checkpoint for the slice. *(Open: the exact `MaintainView` shape — see the
   delta question under Key decisions.)*
3. **State persistence** — a small `ViewStore` (in-memory behind a trait, swappable
   later — matches [`vision.md`](./vision.md) §7's "a view in whatever store fits").
4. **A read path** — a plain read method for the slice; the first-class `Query`
   construct is deferred.
5. **Worked example** (`eventric-examples`) — a view-maintaining reaction over an
   existing example's events (e.g. a per-course enrolment count fed by the
   course-subscription events), end-to-end and dogfooded.
6. **Tests** — incremental fold correctness; replay-from-zero rebuilds the same view
   (idempotence); checkpoint advance + resume.

**Milestone A:** a reaction tails the stream and keeps a queryable view current — the
reactor and trait proven.

## Phase B — the in-memory command-issuing loop

*Validates: effects-as-messages (a real effect), command→action routing, and the loop
— the thing that makes eventric* eventric.

1. **`IssueCommand` effect** added to `Effects` — staged as a **private command
   message** (a proper type, not the action struct, per the boundary decision).
2. **Command → action routing** — a minimal dispatch mapping a command to the action
   that handles it, run via the existing **`Enactor::enact`** (reuse the whole action
   cycle). A simple registry/closure for the slice; the declarative surface-derive is
   deferred.
3. **Reactor interprets `IssueCommand`** → dispatch → `enact` → appends events → which
   the still-tailing reactor reacts to: the loop closes, in-memory, no network hop.
4. **Worked example** — on event X, issue a command → action → event Y (one autonomous
   step, e.g. "on `EnrolmentRequested`, if capacity allows, issue `ConfirmEnrolment`").
5. **Tests** — the loop runs end-to-end; it terminates (bounded example); a guard
   against runaway loops.

**Milestone B:** event → reaction → command → action → event runs autonomously in one
process — the core loop is real.

## Key decisions (with leans)

- **View-update model — the one open item.** Because the reaction is *event-only* (no
  current-view input), view-maintenance leans toward **deltas**: `react` stages a
  `MaintainView` delta the runtime applies, rather than read-modify-write (which needs
  the current view *as an input* — that arrives with projections, later). Open: the
  delta's exact form (a small typed-op vocabulary, or something the runtime applies
  generically), and whether event-only-delta suffices before projection-read RMW.
- **A1 vs A2 — resolved (A2).** View-maintenance uses the real `React` + effects shape
  (a `MaintainView` effect), not a bare persisted `Projection` — validating the design
  is the slice's purpose, and effects-as-messages is exercised from Phase A.
- **Reactor ↔ writer.** The loop (B) writes via the Writer / Enactor, so the reactor
  needs append access. *Lean:* run it in-process over the `Stream` split (or `Owner` /
  `Proxy`); keep concurrency simple for the slice.
- **Checkpoint.** *Lean:* in-memory for the slice; persisting the reactor's progress
  checkpoint (boundary.md §9) is a deferred follow-up.
- **Derive vs hand-written.** *Lean:* hand-write the reaction (recognise / decode /
  react) in the example first; add `#[derive(Reaction)]` once the trait + machinery
  are proven.

## What it buys

A working reaction on a tidy substrate, and — the real point — **empirical feedback on
boundary.md**: does the `React` shape feel right, does effects-as-messages earn its
keep, where does the reactor design strain? Whatever the slice teaches flows back into
`boundary.md` before we commit to the channel, contracts, and the full effect algebra.

## Rough sequence

Scaffolding + decisions → **Phase A** (trait, reactor, view, example, tests) → *pause,
reflect, update boundary.md* → **Phase B** (command effect, routing, loop, example,
tests) → *pause, reflect, update boundary.md* → decide the next increment (the derive
macro? the event-sourced checkpoint? the first contract?).
