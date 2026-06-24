# SPLIT.md

Plan + tracker for splitting the single `eventric` library into **two runtime
crates** along the one real seam — a **content-agnostic stream substrate** and a
**content-aware domain layer** — plus the existing unified proc-macro crate.

This **refines, not reverses, the consolidation.** The consolidation removed the
*artificial* over-decomposition (three crates for the stream layer alone). This
reinstates the *one* genuinely real boundary and makes content-agnosticism a
**compile-enforced invariant** rather than a convention upheld by review.

## Why

The stream layer must never know about content or serialisation — that is exactly
what keeps `Version` serialisation-neutral and lets the stream store/serve opaque
payloads for any consumer. Today the invariant holds by convention; as separate
crates it becomes a *compile error* for `eventric-stream` to `use revision`. And
the seam is **already real**: `revision` appears only in the domain layer
(`model/event.rs`, `model/projection.rs`); the stream stores opaque `Data`. So the
split freezes an invariant that already holds — it untangles nothing.

## Target — three crates

- **`eventric-stream`** — the substrate. `error`, `event`, `stream` (+ `stream/`),
  `utils` (+ `utils/`), `combine` (private). Stream/storage/concurrency deps
  (fjall, error-stack, hashing, the channel deps) — **NOT `revision`**, no
  serialisation, no event-type knowledge. Root `#![feature(exclusive_wrapper)]`
  (the `SyncView`); `#![deny(missing_docs, unsafe_code)]`.
- **`eventric-domain`** — the domain layer (the artefact currently misnamed
  `model`). `action`, `event`, `projection`, `enactor` **at the crate root** —
  the `model::` nesting is dropped (see below). Deps: `eventric-stream` +
  `revision` + error-stack + fancy_constructor. Root
  `#![feature(associated_type_defaults)]`.
  - The name is **provisional** — the eventual project shape is unsettled, so
    `eventric-domain` is "right enough for now," explicitly *not* a final claim on
    the headline `eventric` name.
- **`eventric-macros`** — unified, unchanged in scope (`tag!` + the three
  derives). Depends on *neither* runtime crate; emits paths into both. Kept single
  on purpose: the seam is about *runtime* deps, which a proc-macro crate doesn't
  create, so splitting `tag!` from the derives would be needless re-decomposition.

## Locked decisions

1. Three crates: `eventric-stream`, `eventric-domain`, `eventric-macros`.
2. The domain layer **un-nests** to its crate root (no `model::`). The nesting
   only ever existed to dodge name clashes (`event::Event` struct vs
   `model::event::Event` trait; two `Events`; three `Select`s) — two crate names
   disambiguate those for free.
3. Macros stay **unified**.
4. **Strict no-lift.** Consumers depend on *both* runtime crates and import stream
   types (`Position`, `Timestamp`, `Selection`, `Append`, `Select`, …) from
   `eventric-stream` directly. No cross-crate re-export facade.
5. **Per-crate error types.** `eventric-domain` gets its own opaque `error::Error`
   (ZST) + `Result` alias. Stream results are `.change_context(domain::Error)`'d
   at the seam — and because `change_context` *infers* the source type from the
   `Report`, the domain never **names** `eventric_stream::error::Error`, so that
   coupling drops to **zero**. This is the same boundary pattern already in use
   (`validation::Error` → stream `Error`) applied one layer up — the correct
   per-layer form of the consolidation's "one error", not a regression of it.

## What still legitimately couples domain → stream

Decoupling the error removes the *most pervasive* reference (it sat in every
signature). What remains is correct and intended: the domain depends on
`eventric-stream` for the **appendable-event vocabulary** (`Event`, `Data`,
`Facets`, `Name`, `Tag`, `Type`, `Version`) and the **stream operations**
(`Append`, `Select`, `Condition`, `Selection`, `Position`, `Timestamp`,
`EventAndMask`) — "the actual stream itself". The surface shrinks; it does not, and
should not, vanish.

## Steps

- [ ] 1. Scaffold `eventric-stream`: move `error`/`event`/`stream`/`utils`/`combine`;
  Cargo.toml with the stream deps (**no `revision`**); root features + lints.
  Gate it building **standalone** — that build is the proof the substrate carries
  no content/serialisation dependency.
- [ ] 2. Scaffold `eventric-domain` from `model/`: promote `action`/`event`/
  `projection`/`enactor` to the crate root; Cargo.toml (dep `eventric-stream` +
  `revision`).
- [ ] 3. Domain error type: add `eventric_domain::error::{Error, Result}`; switch
  domain signatures from the stream `Error` to the domain one; `.change_context(Error)`
  at every stream call (source inferred); confirm the domain no longer names the
  stream error. (Includes the `Act::Err: From<Report<Error>>` bound.)
- [ ] 4. Macros: repoint emitted paths — `tag!` → `::eventric_stream::…`, the
  derives → `::eventric_domain::…`; re-export `tag!` from `eventric-stream`, the
  derives from `eventric-domain`.
- [ ] 5. Split test suites: stream (`round_trip`, `concurrency`) →
  `eventric-stream/tests`; domain (`enact`, `multi_selector`, `versioning`) →
  `eventric-domain/tests`.
- [ ] 6. Examples/benches: repoint deps + paths (stream examples →
  `eventric-stream`; domain examples → both crates).
- [ ] 7. Docs: `CLAUDE.md` (its "consolidated into a single library" framing
  becomes the two-crate substrate/domain story), `versioning.md`, `FUTURE.md`,
  the archived pointers, and the project memory.
- [ ] 8. Gate: fmt, build, clippy `-D warnings`, tests, doc, examples — all green.

*(Exact dependency partition between the two Cargo.tomls is an execution detail —
settled against the compiler in steps 1–2.)*

## Verification

`cargo fmt --all --check && cargo build --workspace --all-targets && cargo clippy
--workspace --all-targets -- -D warnings && cargo test --workspace` + `cargo doc
--workspace --no-deps` + each example. The decisive *new* check: **`eventric-stream`
has no `revision` (and no serialisation crate) anywhere in its dependency tree.**
