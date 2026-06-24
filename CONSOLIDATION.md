# CONSOLIDATION.md

Plan + progress tracker for collapsing the workspace from **7 runtime/facade
crates → 2** (`eventric` library + `eventric-macros` proc-macro). Purely
structural: the macros emit the *same* code, repointed to the new crate name.
No codegen-approach change (that is a separate, deferred design piece — see
[`docs/keyed-selectors.md`](docs/keyed-selectors.md) and the groundwork notes).

## Target

- **`eventric`** (lib) — all runtime: stream + model + multi-thread.
- **`eventric-macros`** (proc-macro) — `tag!` + `Event`/`Action`/`Projection`
  derives, re-exported from the lib so users never name it.
- `benches/*`, `examples/*` stay as consumers (crate-name + path-prefix swap).

The 7 that collapse: `stream`, `stream/core`, `stream/macros`, `model`,
`model/core`, `model/macros`, `eventric-stream-multi-thread`.

## Layout

```
eventric/src/
  lib.rs              root: both #![feature(..)]; reconciled lints; macro re-exports
  error.rs            ← stream/core::error
  event.rs            ← stream/core::event   (+ pub use eventric_macros::tag)
  combine.rs          ← stream/core::combine (private)
  stream.rs  stream/  ← stream/core::stream
    concurrent.rs concurrent/{owner,processor,proxy}.rs  ← multi-thread
  utils.rs  utils/    ← stream/core::utils
  model.rs  model/    ← model/core
    action.rs event.rs projection.rs enactor.rs (← core.rs)
eventric-macros/src/
  lib.rs              entries: tag!, Event, Action, Projection
  tag.rs              ← stream/macros::event/tag
  event.rs action.rs projection.rs  ← model/macros
```

Two-tier namespacing is **forced** (Event struct vs model Event trait, two
`Events`, three `Select` traits): stream stays top-level (`eventric::{error,
event, stream, utils}`), model nests (`eventric::model::{action, event,
projection, Enactor}`). No flat prelude.

## Locked decisions

1. Names: `eventric` + `eventric-macros`.
2. Namespacing: stream top-level, model under `eventric::model::`.
3. Multi-thread: `eventric::stream::concurrent`, **unconditional** (no feature).
4. Lints: root `#![deny(missing_docs, unsafe_code)]`; clippy doc-trio allowed
   crate-wide; `#[allow(missing_docs)]` on `model` + `stream::concurrent` to
   preserve their current posture (no new doc-writing this pass). Both nightly
   features at root.
5. Macro paths: `::eventric_stream::X → ::eventric::X`,
   `::eventric_model::X → ::eventric::model::X` (~39 sites).

## Steps — DONE

- [x] 1. Scaffold `eventric` (from `stream/core`) + `eventric-macros` (from `stream/macros`).
- [x] 2. Fold multi-thread → `eventric::stream::concurrent` (`Owner`/`Proxy` re-exported at `stream::`).
- [x] 3. Fold `model/core` → `eventric::model` (`core.rs` → `enactor.rs`).
- [x] 4. Merge `model/macros` into `eventric-macros`; relocate `tag`; one `lib.rs`.
- [x] 5. Rewrite macro-emitted paths (#5 above) — incl. bare (no leading `::`) ones.
- [x] 6. Write `eventric/src/lib.rs` root (features, lints, mods); macros re-exported per-module.
- [x] 7. Rewrite intra-crate paths; model siblings → `crate::model::`; concurrent siblings → `super::`.
- [x] 8. Update workspace `members` + dep table; delete old crate dirs.
- [x] 9. Fix consumers (examples/benches) + move the 4 test files into `eventric/tests/`.
- [x] 10. Gate: fmt, build, clippy `-D warnings`, **130 tests**, doc, examples all green.

**Cycle broken:** the `Event` derive used to call `eventric_stream::event::Name::new` at
expansion to validate the identifier — a cycle once `eventric` re-exports the derive. Since
`parse_identifier` only accepts a single `TokenTree::Ident`, the identifier is already a
valid Rust ident and the check was dead, so it was removed outright (no duplicated
validation logic). The generated `Identifier::type_name` still calls `Name::new` at runtime
as the (effectively unreachable) backstop. `eventric-macros` does **not** depend on `eventric`.

**No within-crate re-export lifts whatsoever** (per the maintainer's preference — keep the
public surface strictly structural for now, judge later whether it needs curating): items
live at their defining module — `eventric::utils::temp_path`,
`eventric::model::enactor::Enactor`, `eventric::stream::concurrent::{owner::Owner,
proxy::Proxy}`, and the query vocabulary split by its real submodules:
`eventric::stream::operate::{Condition, Selection}`, `operate::append::Append`, and
`operate::select::{Select, SelectIter, Selector, TypeSelector, VersionSelector, EventAndMask,
Mask}`. Only the cross-crate macro re-exports (the single-crate UX) and the `pub(crate)`
internal re-export of `operate::Appender` remain.

## Verification

`cargo fmt --all --check && cargo build --workspace --all-targets && cargo clippy
--workspace --all-targets -- -D warnings && cargo test --workspace` plus
`cargo doc --workspace --no-deps` and running each example. Done on the main tree
(revertable); not branched.
