# Crates

The `eventric-core-*` crates support `eventric-core`. Consumers of `eventric-core` should only
ever take a dependency on the single `eventric-core` crate -- all functionality to use `eventric-core`
is exported/re-exported from that crate (note that some optional functionality may be gated
behind features - check the `eventric-core` documentation for information).