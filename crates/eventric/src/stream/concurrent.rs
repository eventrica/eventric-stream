//! Concurrent access to a single-threaded [`Stream`](crate::stream::Stream): an
//! [`owner::Owner`] spawns a dedicated writer thread and hands out
//! [`proxy::Proxy`] clones that funnel writes over a bounded channel (the
//! global write lock) and read through cloned `Reader`s.

pub mod owner;
pub mod proxy;

mod processor;
