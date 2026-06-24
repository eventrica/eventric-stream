# Deferred design: keyed (named) projection selectors

> **SUPERSEDED (2026-06-24) by [`derives.md`](../derives.md).** The named-selector
> design here is folded into that broader declarative derive-grammar redesign and
> agreed for building — the "coherent context" the deferral below was waiting for.
> Kept as the historical record of the original analysis, especially the
> "Coherence: why it is deferred, not built" section, which still reads true and
> explains *why* this only became worth building as part of a wider redesign.

**Status: DEFERRED — a deliberate non-goal until a real need appears, and even
then worth building only if it becomes a *coherent* instance of the existing
"generated companion type" codegen pattern (see "Coherence" below).** This note
records the design at enough fidelity to pick it up later; it describes nothing
that exists in the code today.

## Background: the two-tools rule (what exists today)

A `#[derive(Projection)]` shapes its inputs with two tools, and they already
cover almost everything (see the projection module docs and `CLAUDE.md`):

1. **Many `select(..)` clauses in one projection** OR into a single `Selection`
   (one mask bit): "all of these events feed *one* piece of derived state." The
   projection discriminates finer than type from the decoded **payload**, inside
   its `Project<E>` impls.
2. **One projection per filter**: "*distinct* read-models over overlapping
   events", each with its own mask bit and its own `Project` impls.

Routing is by event type (`Project<E>`, enforced by Rust coherence — you cannot
write two `impl Project<Transfer>` for one projection). Any distinction finer
than type is the projection's own business and is read from the payload, which
is the canonical, type-checked, more-expressive source (the tags were *derived
from* the payload fields in the first place). Multi-match is set-valued and
free, and `Action::update` decodes the payload once and shares the boxed
`DispatchEvent` across every same-type slot.

The one genuine ergonomic gap these two tools leave open: keeping *coupled*
state in **one** projection while still distinguishing, in a fully typed way,
*which named selector* a given event matched — including when a selector names
several event types. That gap is what the design below would close.

## The proposed feature

**Name each selector** in the `#[projection(..)]` attribute, e.g.

```text
#[projection(
    select(outgoing, events(Transfer), filter(from(&this.me))),
    select(incoming, events(Transfer), filter(to(&this.me))),
)]
struct Balance { /* ... */ }
```

**Generate, per projection, a module** named after the projection (snake_case),
holding **one payload enum per selector** with a variant per selectable event
type. Short variant names, namespaced inside the module, so there are no
free-floating-name collisions (the verbose-vs-collide tension that naked,
top-level generated names would create):

```text
mod balance {
    pub enum Outgoing { Transfer(Transfer) }
    pub enum Incoming { Transfer(Transfer) }
}
```

**Generate a trait with one method per selector**, each taking that selector's
enum:

```text
fn outgoing(&mut self, e: balance::Outgoing);
fn incoming(&mut self, e: balance::Incoming);
```

Implementing the trait requires **every** method, so a mis-named selector is a
**compile error** — unlike a stringly-typed label, which would fail (or silently
no-op) only at runtime.

**Dispatch** by re-testing each selector's tags against the matched event's
facets (a **model-layer-only** step — **no** change to the core `Mask`),
wrapping the decoded payload in the matching selector's enum and calling that
selector's method.

**Cost:** the codegen itself, the per-event enum construction, and a small
per-event clone of the event facets to run the tag re-test.

**Benefit:** keeps coupled state in one projection (the one real ergonomic gap
versus separate projections) while remaining fully typed even for multi-type
selectors.

## Coherence: why it is deferred, not built

This feature is justified at best by *loose analogy* to the one place a
generated-companion-type pattern genuinely exists today, and on inspection the
analogy is weak. The narrower, accurate invariant in the codegen is **"generate
a companion only where business logic must hold per-declaration state"** — not
"generate companion types" in general:

- **`#[derive(Action)]`** is the *only* real precedent: it emits a `{Name}Context`
  — a single flat `struct` at module scope with one named, typed field per
  declared projection, reached from user code via the `Context` associated type.
  It keys off whole **projections**, which already have a name and identity, and
  the runtime dispatches by hashed type name + positional mask bit.
- **`#[derive(Projection)]`** emits only a bare marker trait (`{Name}Dispatch`)
  — no data, no methods.
- **`#[derive(Event)]`** emits **no** companion type at all: an event's identity
  is just a hashed type name plus a flat tag list, with no per-event sub-structure
  for a companion to key off, so a companion would be gratuitous. Event is the
  leanest, most consistent point on the spectrum.

Measured against the `Action`/`Context` precedent, the keyed-selector proposal
**diverges** on three axes rather than consolidating a pattern:

- **Structure** — `Context` is a single flat struct at module scope; the proposal
  is a *nested module* of multiple per-selector enums plus a method-bearing trait.
- **Keying** — `Context` keys off whole projections that *already* have identity;
  the proposal keys off named *sub-selectors* that have no name or identity today
  (a `Selector` carries only `events` and `filter` and folds positionally).
- **Dispatch** — the system dispatches by hashed type name and positional mask
  bit, never by re-testing selector tags against event facets; the proposed
  dispatch is brand new. Only the suffix-naming convention actually transfers.

**Do the cheaper, more coherent groundwork first**, after which the keyed enums
would become a real instance of a consolidated pattern rather than a one-off
luxury:

1. Make `{Name}Dispatch` carry the methods it implies, instead of being a bare
   bound.
2. Unify companion-name generation into shared, collision-safe codegen, so future
   companions are cheap.
3. Give selectors a name (or a key) and **de-positionalize the mask**, fixing the
   fragile positional-mask coupling.

**Verdict: defer.** As proposed it is a downstream luxury justified by analogy to
the one precedent that is keyed off whole projections and dispatched
positionally, whereas the proposal adds module nesting, a selector-naming hook,
and runtime tag re-testing — increasing divergence. Not worth building now; do
the groundwork above first, and then reassess.
