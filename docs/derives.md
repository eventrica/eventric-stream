# Derive grammar + codegen — design

**Status: in progress — `Event` + `Projection` implemented; `Action` pending.**
This is the agreed target for a redesign of the three model-layer derives
(`Event`, `Projection`, `Action`) and the `tag!` macro: a single *declarative*
attribute grammar across all three, and — for projections — **named selections**
that generate a per-selection event enum and a per-selection method surface. It
supersedes [`keyed-selectors.md`](./archived/keyed-selectors.md) (the deferred
keyed-selectors note, now folded in) and subsumes the codegen-groundwork items in
[`FUTURE.md`](./FUTURE.md) §2.

The **Event** and **Projection** derives are built. `event.rs` hand-parses
`#[event(identifier: X, tags: { prefix: value, .. })]`; `projection.rs` hand-parses
`#[projection(selections: { name: { events: [..], filter: { .. } }, .. })]` and generates the
per-projection module (a borrowed enum per selection + a `Project` trait of
per-selection methods) plus the de-positionalised-mask `Select`/`Recognize`/`Dispatch`
impls. All call sites are migrated. **`Action`** is still on the old `darling`
grammar — its `select`/`update` wiring was updated for the new per-selection mask
layout, but its `#[action(projection(Name: Ctor))]` attribute grammar (and the
`identity::<fn(&Self)>` constructor codegen) are unchanged.

**Today, for contrast:** the attributes use `darling`'s `FromMeta` conventions
(`identifier(..)`, `tags(course(&this.id))`, `select(events(..), filter(..))`,
`projection(Name: Ctor::new(..))`), and projections route by event type via one
`impl Project<E>` per event type. The redesign below changes both the attribute
grammar and the projection impl surface.

## Goals + principles

Two independent axes, per derive: **(1) declaration expressiveness** — what the
attribute can say, declaratively and tersely; **(2) required impl** — what the
user hand-writes beyond the attribute. Agreed principles:

- **Explicit over implicit.** Write field names, identifiers, selection names —
  don't infer. (Simplify with defaults later if one proves safe; easier to relax
  than to tighten.)
- **Declarative, uniform syntax.** `key: value` entries, the same across all three
  derives; **`{ … }` for keyed collections** (maps with field-like keys — `tags`,
  `select`, `project`, the per-selection body) and **`[ … ]` for plain element
  lists** (`events`), so the bracket mirrors Rust's struct-vs-array intuition.
  (Originally `[ … ]` for every multiple; revised once the keyed collections were
  recognised as maps — braces read more honestly.)
- **No single-value sugar for plain lists** — `events: [One]` is a list of one. A
  keyed value, by contrast, is a single value or a `[list]` (`account: [from, to]`).
- **Explicit identifiers are a feature, not boilerplate.** The event identifier
  is the durable, hashed **on-disk identity**; it must *not* default from the Rust
  type name, because then a type rename would silently re-identify the event and
  orphan stored data. Same principle as "names are opaque, stable contracts" from
  the versioning work ([`versioning.md`](./versioning.md)).

## The shared grammar

Attribute bodies are a small declarative DSL:

- `key: value` entries (not `darling`'s `key(..)` / `key = ..`).
- `{ … }` for **keyed** collections — maps with unique, field-like keys (`tags`,
  `select`, `project`, and the per-selection `{ events, filter }` body), reading
  like struct literals.
- `[ … ]` for **plain** element lists (`select`'s `events`).
- A value in a `{ … }` map may itself be a `[list]` for the multi-valued case —
  `tags: { account: [from, to] }` declares two `account:` tags (a multi-entity
  event like a transfer), which replaced the earlier repeated-prefix form.
- **Values** keep three forms, orthogonal to the container grammar — all
  desugaring to one `{ let <recv> = self; <body> }` block (no closure is
  generated, so no higher-ranked-lifetime coercion and no `Cow`), then formatted
  by `tag!` as `prefix:value` (the value need only be `Display`):
  - **bare ident** — `course: id` ⇒ `&self.id` (the terse common case).
  - **expression** — `course: &this.id` ⇒ `{ let this = self; &this.id }` (`this`
    is `&Self`; the escape hatch).
  - **closure** — `course: |this| …` ⇒ `{ let this = self; … }` — the same, but
    you name the receiver (e.g. `|_| …` to ignore it).

> **Receiver name — `this`, not `self` (decided to keep `this`).** The receiver is
> named `this` uniformly across all three derives. `self` *does* resolve in tag and
> filter expressions (they expand inside generated `&self` methods, so call-site
> hygiene binds it), but **not** in action `projections:` constructors — those are
> emitted in an associated `Context::new(action)` with no `self` receiver, so `self`
> reads as the module path (`expected value, found module self`). `this` is the one
> name that works in all three positions, so it stays the documented form. Switching
> the convention to `self` is **deferred to this redesign's Action step**: build the
> context inside a `&self` method (replacing the `identity::<fn(&Self) -> T>`
> coercion with the same `{ let this = self; … }` block the Event derive now uses),
> which removes that last coercion *and* lets `self` resolve there too — at which
> point `self` could become the canonical name across the board.

This DSL is **not** `Meta` syntax, so `darling::FromMeta` cannot parse it: the
implementation hand-rolls `syn::Parse` per derive and **owns its error messages**
— that is the real cost of the declarative reading, traded for consistency and
readability. The grammars are small.

## Event

```rust
#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(
    identifier: course_defined,
    tags: { course: id, student: student_id },   // optional — omit entirely if none
)]
pub struct StudentSubscribedToCourse { /* … */ }
```

- **Impl burden: none** — just the struct.
- `identifier:` mandatory and explicit (see principles).
- `tags:` optional, a map; each entry `prefix: value` (or `prefix: [values]`).
- Still requires `#[revisioned(revision = N)]` (serialisation — a separate crate's
  macro that must wrap) and a constructor (`#[derive(new)]`) as today.

## Projection — named selections

```rust
#[projection(
    selections: {
        capacity: { events: [CourseDefined, CourseCapacityChanged], filter: { course: id } },
        // … more named selections …
    },
)]
pub struct CourseCapacity { /* … */ }
```

- `selections:` is a map of **named selections**: each `name: { events: [ … ], filter: { … } }`,
  `filter` optional.
- The **name is the selection's identity** — it keys the generated impl surface and
  de-positionalises the mask (below).

**Generated, per projection** — a module (projection name, snake_case) holding one
**borrowed enum per selection** (a variant per event type), and a trait with one
**method per selection** taking that enum wrapped in a `ProjectionEvent` (so
position/timestamp come along):

```rust
pub mod course_capacity {
    pub enum Capacity<'a> {
        CourseDefined(&'a super::CourseDefined),
        CourseCapacityChanged(&'a super::CourseCapacityChanged),
    }

    pub trait Project {
        // one method per selection:
        fn capacity(&mut self, e: ProjectionEvent<Capacity<'_>>);
    }
}
```

**User impl** — implement the generated `Project` trait, one method per selection:

```rust
impl course_capacity::Project for CourseCapacity {
    fn capacity(&mut self, e: ProjectionEvent<course_capacity::Capacity<'_>>) {
        match e.event() {
            Capacity::CourseDefined(ev)         => self.capacity = ev.capacity,
            Capacity::CourseCapacityChanged(ev) => self.capacity = ev.new_capacity,
        }
    }
}
```

Decisions:

- **Always an enum, even for a single-event selection.** Adding/removing an event
  in a selection is then a localised match-arm change, not a signature churn, and
  the match is **compile-time exhaustive** — a new event type adds a variant and
  every impl's match breaks until handled (the loud-failure property; it echoes
  the versioning "closed set ⇒ compile-time exhaustive" point — a selection's
  event list *is* the closed set).
- **Borrowed enum** (`&'a EventType` variants) is the target — no per-event clone;
  the price is a lifetime parameter on the generated enum. **Owned-clone is the
  fallback**, taken only if the lifetimes prove too awkward (the one known
  implementation risk — see below).
- This **replaces `Project<E>`** (per-type routing) with per-selection methods, and
  **subsumes the two-tools rule**: each named selection is its own mask bit + enum
  + method, so "coupled state in one projection, distinguished by which selection
  matched" *and* "distinct read-models over overlapping events" are both simply
  *how many selections you declare*. (`NumberOfCourseSubscriptions` /
  `NumberOfStudentSubscriptions` would collapse into one projection with `course:`
  and `student:` selections over the same event type.)

## Action

```rust
#[action(
    projections: {
        course_exists:   CourseExists::new(&this.id),
        course_capacity: CourseCapacity::new(&this.id),
    },
)]
pub struct ChangeCourseCapacity { /* … */ }
```

- `projections:` is a map of **explicit `field_name: constructor`** entries. The field
  name is what you access on the generated context (`context.course_exists`),
  written explicitly — no inference from the type, which lets two slots of the same
  projection type coexist.
- The `Act` impl is unchanged in shape (`type Err`, `fn action`). Defaulting
  `type Err = Report<Error>` is a plausible separate ergonomic win, **out of scope
  here** (noted in open questions).

Naming: `selections:` / `projections:` — **nouns naming what the derive *has***
(the selections a projection declares; the projections an action uses). Chosen
over the shorter verbs `select:` / `project:`: those read less obviously, and
`project:` is inaccurate (an action *uses* projections, it doesn't project). The
extra length buys clarity — explicit over implicit.

## Dispatch + the mask (model-layer only)

Routing an event to the right named method must **not** re-test each selection's
tags against the event per event (the cost the deferred note flagged, plus a
facets clone). Instead, **de-positionalise the mask at the model layer**: each
named selection maps to a known mask index, so dispatch reads `mask[selection]`
and calls that selection's method directly.

**No change to the stream's `Mask`** — it stays a positional `bool`-per-selection;
the model layer owns the name→index mapping. This is exactly the
[`FUTURE.md`](./FUTURE.md) §2 groundwork ("name selectors + de-positionalise the
mask"; "`{Name}Dispatch` carries the methods it implies"), and doing it *through*
this grammar is what makes the keyed-selector feature a **coherent** instance of
the codegen pattern rather than the one-off the archived note rightly deferred.

## Implementation order

1. **Event** — ✅ **done.** Hand-rolled `EventArgs`/`TagEntry` `syn::Parse`; tags
   moved to an ordered `Vec`; the `Tag` value parser's bare-ident branch is
   speculative (so expression values like `this.id()` / `foo::BAR` aren't
   mis-eaten); the missing-`identifier` error is raised against the attribute span.
   The value codegen desugars every form (bare ident / expression / closure) to a
   single `{ let <recv> = self; … }` block — no generated closure, so the old
   `identity::<for<'a> fn(&Self) -> Cow<…>>` coercion is gone. `tag!` was tidied to
   a `syn::Parse` and now emits a `Tag::prefixed(prefix, value)` constructor that
   owns the `prefix:value` format (previously inlined in the macro). Unit-tested,
   plus an end-to-end `tests/tags.rs` over all three value forms.
2. **Projection** — ✅ **done.** Hand-rolled `ProjectionArgs`/`NamedSelection`
   `syn::Parse` (reusing the Event parser's `TagEntry`/value-forms for `filter`);
   generates the per-projection module (a borrowed enum per selection + a `Project`
   trait of per-selection methods) and the `Select` (`Vec<Selection>` + `const
   SELECTIONS`)/`Recognize`/`Dispatch` impls. The mask is de-positionalised: each
   selection is its own `Selection`, the action flattens them and hands each
   projection its `mask[base..base+SELECTIONS]` slice. The **borrowed enum worked**
   (no owned-clone fallback). `ProjectionEvent` now holds the enum by value
   (`event()` accessor); `Project<E>` is gone. The obsolete `tags_map`/`tags_fold`
   (and the dead `util::List`) were deleted rather than relocated — the new filter
   parse reuses `TagEntry`/`TagInitialize` directly. All call sites migrated;
   covered by the existing `enact`/`multi_selector` integration tests + examples.
3. **Action** — the same declarative grammar with an explicit field-name list; the
   context generation is the existing precedent to build on. (Its `select`/`update`
   already produce/consume the per-selection mask layout; this step is the
   `#[action(..)]` *grammar* + replacing the `identity::<fn(&Self)>` constructor
   coercion — at which point `self` could become the receiver name, per the note
   above.)

## Known risks / open at build time

- **Borrowed-enum lifetimes** — **resolved.** The enum wraps `&E` downcast from the
  decoded box and `ProjectionEvent<T>` holds it by value; the borrow is valid for
  the duration of the dispatch call (the `&DispatchEvent` box outlives it), so no
  owned-clone fallback was needed.
- **Error-message quality** — hand-rolled `syn` parsing means owning spans and
  messages that `darling` provided for free; budget for good diagnostics.
- **`type Err` defaulting** on `Act` — a separate, deferred ergonomic question.
- **No `trybuild`/UI tests** exist for any derive today; the new parsers are the
  point at which to add them (a misuse should be a targeted macro error, not a
  downstream compile error).

## References

- [`keyed-selectors.md`](./archived/keyed-selectors.md) — the original deferred
  keyed-selectors note. Its "coherence / why-deferred" analysis still reads true;
  this design is the coherent context it was waiting for.
- [`FUTURE.md`](./FUTURE.md) §2 — the codegen-groundwork items this folds in.
- [`versioning.md`](./versioning.md) — the "names are opaque contracts" and
  "closed-set exhaustiveness" principles this leans on.
