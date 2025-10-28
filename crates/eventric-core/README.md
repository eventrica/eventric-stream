# Eventric Core

<!-- ANCHOR: overview -->

Eventric Core provides the lowest-level Event Stream abstraction as part of the
Eventrica ecosystem. It consists of a Stream with append/query functionality which
is consistent with the intent behind [Dynamic Consistency Boundaries][dcb] (DCB).

The implementation is not an exact match to the [specification][spec] (it has been
extended in some ways, primarily to provide slightly more capability around
type identification and potentially to add a little more ergonomics in the
formulation of queries in future).

In general, it is not expected that people will interact with this library directly
on a regular basis, but more commonly with a higher-level abstraction/framework
developed on top of it.

[dcb]: https://dcb.events/
[spec]: https://dcb.events/specification/

<!-- ANCHOR_END: overview -->

