# Eventric Core

<!-- ANCHOR: overview -->

## Overview

Some overview here...

<!-- ANCHOR_END: overview -->

## Streams

<!-- ANCHOR: stream -->

The [`Stream`] type is the central element of Eventric Core. All interactions happen
relative to a `Stream` instance, whether appending new events or querying existing
events, and any higher-level libraries are built on this underlying abstraction.

<!-- ANCHOR_END: stream -->

### Opening a Stream

To open a new [`Stream`] instance use a [`StreamBuilder`], which can be obtained
using the [`Stream::builder`] function.

```rust
// let path = ...

let mut stream = Stream::builder(path).temporary(true).open()?;

assert!(stream.is_empty()?);
```

Once a new [`Stream`] instance has been opened, see [`Stream::append`] and
[`Stream::query`] for information on how to work with the stream and related
events.
