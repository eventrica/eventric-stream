# Eventric Core

## Overview

## Streams

<!-- ANCHOR: stream -->
The [`Stream`] type is the central element of Eventric Core. All interactions happen
relative to a `Stream` instance, whether appending new events or querying existing
events, and any higher-level libraries are built on this underlying abstraction.
<!-- ANCHOR_END: stream -->

<!-- ANCHOR: open_stream -->
To open a new [`Stream`] instance use a [`StreamBuilder`], which can be obtained
using the [`Stream::builder`] function.

```rust
# use eventric_core::{
#     Error,
#     Stream,
# };
#
# let path = eventric_core::temp_path();
#
// let path = ...

let mut stream = Stream::builder(path).temporary(true).open()?;

assert!(stream.is_empty()?);
#
# Ok::<(), Error>(())
```

Once a new [`Stream`] instance has been obtained, see [`Stream::append`] and
[`Stream::query`] for information on how to work with the stream and related
events.
<!-- ANCHOR_END: open_stream -->