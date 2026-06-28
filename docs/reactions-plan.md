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

Mirror `Act` + `Project`:

```rust
fn react(&mut self, event: /* recognised event */, effects: &mut Effects)
```

- `&mut self` is the reaction's **persisted state** — for a view-maintaining reaction,
  the view itself (a fold target, like `Project`); for a process manager, its
  coordination state.
- `effects` is the **staged-effects buffer** (mirroring the `Events` buffer in `Act`)
  — empty for pure view-maintenance, carrying `IssueCommand` for the loop.

One shape covers both phases: pure reactions fold into `self`; effectful ones also
stage effects.

## Phase A — pure view-maintaining reaction

*Validates: the `React` trait, the reactor (tail + drive), the effects mechanism on
the trivial "no effects" case, persisted state, and a basic read.*

1. **`React` trait + `Effects` buffer** (`reaction.rs`). `Effects` can be a no-op in
   Phase A — the point is the signature.
2. **`Reactor` runtime** — given a reaction + its selection (`Condition`) + a
   checkpoint: `select` events from the checkpoint → `react(&mut state, event, &mut
   effects)` per event → persist `state` + interpret `effects` (no-op in A) → advance
   the checkpoint. In-memory checkpoint for the slice.
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

- **View-update model.** *Lean:* `react` reads current `self`, folds the event, the
  reactor persists `self` (read-modify-write of the full state) — simplest, no delta-op
  interpreter. Revisit deltas only if state size makes full rewrites costly.
- **A1 vs A2.** View-maintenance could be just a *persisted `Projection`* (reuse
  `Project`, no `React` / effects) — cheaper, but it tests *less* of boundary.md.
  *Lean:* do **A2** (the real `React` + effects shape) — validating the design is the
  slice's purpose.
- **Reactor ↔ writer.** The loop (B) writes via the Writer / Enactor, so the reactor
  needs append access. *Lean:* run it in-process over the `Stream` split (or `Owner` /
  `Proxy`); keep concurrency simple for the slice.
- **Checkpoint.** *Lean:* in-memory for the slice; the event-sourced checkpoint
  (boundary.md §9) is a deferred follow-up.
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
