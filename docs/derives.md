# Derive grammar + codegen — design

**Status: complete — `Event`, `Projection`, and `Action` all implemented.**
This is the agreed target for a redesign of the three model-layer derives
(`Event`, `Projection`, `Action`) and the `tag!` macro: a single *declarative*
attribute grammar across all three, and — for projections — **named selections**
that generate a per-selection event enum, which the user folds via the standard
`Project<Enum>` trait. It
supersedes [`keyed-selectors.md`](./archived/keyed-selectors.md) (the deferred
keyed-selectors note, now folded in) and subsumes the codegen-groundwork items in
[`FUTURE.md`](./FUTURE.md) §2.

All three derives are built and on the hand-rolled grammar. `event.rs` parses
`#[event(identifier: X, tags: { prefix: value, .. })]`; `projection.rs` parses
`#[projection(selections: { name: { events: [..], filter: { .. } }, .. })]` and generates the
per-projection module (a borrowed enum per selection) plus the de-positionalised-mask
`Select`/`Recognize`/`Dispatch` impls (the user folds each selection via the standard
`Project<Enum>` trait, one impl per selection);
`action.rs` parses `#[action(projections: { field_name: Ctor::new(..), .. })]`,
emitting a `Projections` struct in a snake_case submodule (built inside
`projections(&self)`) and a two-arg `Act::act(&self, events, projections)` — no
more `identity::<fn(&Self)>` coercion or deref-fused context. `darling` is gone from
all three. All call sites are migrated, and the receiver name is `self` throughout.

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
- **Values** keep three forms, orthogonal to the container grammar — formatted by
  `tag!` as `prefix:value` (the value need only be `Display`). Each expands inside
  a generated `&self` method, so the event is in scope as `self`:
  - **bare ident** — `course: id` ⇒ `&self.id` (the terse common case).
  - **expression** — `course: &self.id` — any expression; `self` is the event (the
    escape hatch).
  - **closure** — `course: |e| …` ⇒ `{ let e = self; … }` — desugars to a `let`
    block binding the event to the closure's own parameter name, for when you want a
    different name (or `|_| …` to ignore it) or a multi-statement body. No closure is
    actually generated, so there is no higher-ranked-lifetime coercion (and no `Cow`).

> **Receiver name — `self`.** The event (in `tags`/`filter` values) or the action
> (in `projections:` constructors) is named `self`, uniformly across all three
> derives. Each expands inside a generated `&self` method, so call-site hygiene
> binds `self`: tag/filter values inside `tags(&self)`/`select(&self)`, action
> constructors inside `projections(&self)`. Building the projections in a `&self`
> method is what retired the old `identity::<fn(&Self) -> T>` coercion — the
> constructor is now a plain expression in a struct literal — and what let `self`
> resolve there (it could not when the projections were built in an associated
> `Context::new(action)`, where `self` read as the module path). The closure value
> form still lets you bind a different receiver name when you want one.

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
**borrowed enum per selection** (a variant per event type):

```rust
pub mod course_capacity {
    pub enum Capacity<'a> {
        CourseDefined(&'a super::CourseDefined),
        CourseCapacityChanged(&'a super::CourseCapacityChanged),
    }
}
```

**User impl** — fold each selection via the standard `Project<Enum>` trait, one impl
per selection (the enum wrapped in a `projection::Event`, so position/timestamp come
along):

```rust
impl Project<course_capacity::Capacity<'_>> for CourseCapacity {
    fn project(&mut self, e: projection::Event<course_capacity::Capacity<'_>>) {
        match e.event() {
            course_capacity::Capacity::CourseDefined(ev)         => self.capacity = ev.capacity,
            course_capacity::Capacity::CourseCapacityChanged(ev) => self.capacity = ev.new_capacity,
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
- Each named selection is its own mask bit + enum + `Project<Enum>` impl,
  **subsuming the two-tools rule**: "coupled state in one projection, distinguished
  by which selection matched" *and* "distinct read-models over overlapping events"
  are both simply *how many selections you declare*. (This `Project<NamedEnum>` form
  — one impl per selection — superseded the old per-event `Project<E>`: parameterising
  by the distinct named enum is what lets two selections over the *same* event type
  coexist as two impls, which `Project<E>` couldn't.) (`NumberOfCourseSubscriptions` /
  `NumberOfStudentSubscriptions` would collapse into one projection with `course:`
  and `student:` selections over the same event type.)

## Action

```rust
#[action(
    projections: {
        course_exists:   CourseExists::new(&self.id),
        course_capacity: CourseCapacity::new(&self.id),
    },
)]
pub struct ChangeCourseCapacity { /* … */ }
```

- `projections:` is a map of **explicit `field_name: constructor`** entries. The
  field name is the key — what you access on the folded projections
  (`projections.course_exists`) — so two slots of the same projection type can
  coexist; the projection type itself is read from the constructor's path
  (`CourseCapacity::new(..)` ⇒ `CourseCapacity`, so it must be a `Type::new(..)`-style
  call). The constructor runs inside `projections(&self)`, so `self` is the action.
- The derive emits a `pub struct Projections { .. }` in a module named after the
  action (snake_case, mirroring the Projection derive), bound as `type Projections`.
- The user implements `Act<Projections>` with that generated struct as the type
  argument (`impl Act<change_course_capacity::Projections> for ChangeCourseCapacity`),
  mirroring how a projection implements `Project<Enum>`. `fn act(&self, events: &mut
  Events, projections: &Projections)` takes **two args** — stage output into the
  `events` buffer, reading the folded `projections` (immutable) to decide (a
  single-sided action `_`-prefixes whichever it doesn't use). `type Err` keeps the
  custom-error hook but **defaults to `Report<Error>`**, so the common case omits it;
  a custom `Err: From<Report<Error>>` is exercised by `tests/enact.rs`.

Naming: `selections:` / `projections:` — **nouns naming what the derive *has***
(the selections a projection declares; the projections an action uses). Chosen
over the shorter verbs `select:` / `project:`: those read less obviously, and
`project:` is inaccurate (an action *uses* projections, it doesn't project). The
extra length buys clarity — explicit over implicit.

**Why two args, not a fused context.** An earlier shape generated a `{Name}Context`
bundling the projections *and* the `Events` buffer, deref-ing to `Events` so the
action could `context.append(..)`. That `Deref`-for-inheritance is a Rust
anti-pattern, and it fused the read-side (folded projections) with the write-side
(output events). Splitting them keeps the projections immutable, the `Events` buffer
concrete and owned by the `Enactor`, and `events.append(..)` a real inherent call.
The cost: one bound-free `type Projections` survives on the (generated, internal)
`Context`, and a single-sided action `_`-prefixes the param it doesn't use. Net: less
machinery, no anti-pattern.

**Uniform surface.** Both derives put the user on the same footing: implement a
*standard* library trait parameterised by a *generated* type — `impl Project<Enum> for
P` (one per selection) and `impl Act<Projections> for A`. No derive makes you implement
a *generated* trait; the macros only ever generate types (the per-selection enums, the
`Projections` struct) plus machinery impls. The action's `type Projections` stays an
internal associated type on `Context` — the `Enactor` supplies the `Act<P>` argument
from it (`A: Act<A::Projections>`), so the user never writes the associated type; they
write `Act<make_deposit::Projections>`, just as a projection writes
`Project<channel_totals::Wire<'_>>`. Dropping that last associated type *entirely* would
mean moving the whole replay/fold/append loop into per-action macro output (opaque, no
single readable Enactor) — verified achievable, but not worth it.

## Dispatch + the mask (model-layer only)

Routing an event to the right selection must **not** re-test each selection's
tags against the event per event (the cost the deferred note flagged, plus a
facets clone). Instead, **de-positionalise the mask at the model layer**: each
named selection maps to a known mask index, so dispatch reads `mask[selection]`
and calls that selection's `Project<Enum>::project` directly.

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
   The value codegen uses `self` directly for the bare-ident and expression forms;
   only the closure form desugars to a `{ let <name> = self; … }` block. No closure
   is generated, so the old `identity::<for<'a> fn(&Self) -> Cow<…>>` coercion is
   gone. `tag!` was tidied to
   a `syn::Parse` and now emits a `Tag::prefixed(prefix, value)` constructor that
   owns the `prefix:value` format (previously inlined in the macro). Unit-tested,
   plus an end-to-end `tests/tags.rs` over all three value forms.
2. **Projection** — ✅ **done.** Hand-rolled `ProjectionArgs`/`NamedSelection`
   `syn::Parse` (reusing the Event parser's `TagEntry`/value-forms for `filter`);
   generates the per-projection module (a borrowed enum per selection) and the
   `Select` (`Vec<Selection>` + `const SELECTIONS`)/`Recognize`/`Dispatch` impls. The
   mask is de-positionalised: each selection is its own `Selection`, the action
   flattens them and hands each projection its `mask[base..base+SELECTIONS]` slice. The
   **borrowed enum worked** (no owned-clone fallback). `projection::Event` holds the
   enum by value (`event()` accessor); the user folds each selection via the standard
   `Project<Enum>` trait (one impl per selection), and `Dispatch` routes `<Self as
   Project<Enum>>::project` by mask. (`Project<NamedEnum>` superseded both the old
   per-event `Project<E>` and the brief generated-per-projection `Project` trait.) The obsolete `tags_map`/`tags_fold`
   (and the dead `util::List`) were deleted rather than relocated — the new filter
   parse reuses `TagEntry`/`TagInitialize` directly. All call sites migrated;
   covered by the existing `enact`/`multi_selector` integration tests + examples.
3. **Action** — ✅ **done.** Hand-rolled `ActionArgs`/`ProjectionEntry`
   `syn::Parse`: `projections: { field_name: Ctor::new(..), .. }`, the field type
   read from the constructor's path. The derive emits a `pub struct Projections` in
   a snake_case submodule (mirroring the Projection derive) and builds it inside
   `projections(&self)` as a plain struct literal (`field: ctor`). The user
   implements the parameterised `Act<Projections>` (e.g. `impl
   Act<change_course_capacity::Projections> for ChangeCourseCapacity`), mirroring a
   projection's `Project<Enum>`; the `Enactor` supplies the `Act<P>` argument from the
   internal `Context::Projections` (`Action: Context + Act<Self::Projections>`). This
   retired the `identity::<fn(&Self)>` coercion, the `Context::new(action)` assoc fn,
   and the `Deref<Target=Events>+DerefMut+Into<Events>` fused-context bounds — and
   with that, `self` became the receiver name across all three derives (the `&this.`
   sites were migrated). `select`/`update` were already on the per-selection mask layout.

## Known risks / open at build time

- **Borrowed-enum lifetimes** — **resolved.** The enum wraps `&E` downcast from the
  decoded box and `projection::Event<T>` holds it by value; the borrow is valid for
  the duration of the dispatch call (the `&Recognized` box outlives it), so no
  owned-clone fallback was needed.
- **Error-message quality** — hand-rolled `syn` parsing means owning spans and
  messages that `darling` provided for free; budget for good diagnostics.
- **`type Err` defaulting** on `Act` — ✅ done: defaults to `Report<Error>`; a custom
  `Err` is exercised by `tests/enact.rs`.
- **`trybuild`/UI tests** — ✅ done: `tests/ui/` pins each derive's targeted parser
  diagnostic (missing/duplicate/unknown keys, empty lists, non-`Type::new`
  constructor). Run `TRYBUILD=overwrite cargo test -p eventric-domain --test ui` to
  regenerate the `.stderr` after an *intentional* diagnostic change.

## References

- [`keyed-selectors.md`](./archived/keyed-selectors.md) — the original deferred
  keyed-selectors note. Its "coherence / why-deferred" analysis still reads true;
  this design is the coherent context it was waiting for.
- [`FUTURE.md`](./FUTURE.md) §2 — the codegen-groundwork items this folds in.
- [`versioning.md`](./versioning.md) — the "names are opaque contracts" and
  "closed-set exhaustiveness" principles this leans on.
