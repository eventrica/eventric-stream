//! Reactions: the [`React`] trait — a single-event handler built from its
//! triggering event (via `From`) that stages effects. The first (and, for now,
//! only) effect is a [`View`]-maintaining delta; the reaction never reads the
//! view (it reacts to its event alone — the *event-only* shape), and the
//! runtime applies the staged deltas. Reactions are driven by the
//! `eventric-runtime` reactor.

use crate::event::Event;

// =================================================================================================
// Reaction
// =================================================================================================

// View

/// A read-model maintained by applying deltas. The reactor owns the view,
/// starts it from [`Default`], and applies each delta a reaction stages — the
/// reaction itself never sees the view.
pub trait View: Default {
    /// The incremental update a reaction stages for this view.
    type Delta;

    /// Apply one staged delta.
    fn apply(&mut self, delta: Self::Delta);
}

// Effects

/// The buffer a reaction stages effects into. For now it carries only
/// `MaintainView` deltas (the sole effect kind); a pluggable, multi-kind effect
/// set is a later step.
pub struct Effects<V>
where
    V: View,
{
    deltas: Vec<V::Delta>,
}

impl<V> Effects<V>
where
    V: View,
{
    /// An empty buffer. (Constructed by the runtime, once per reaction.)
    #[must_use]
    pub fn new() -> Self {
        Self { deltas: Vec::new() }
    }

    /// Stage a `MaintainView` effect: a `delta` for the runtime to apply to the
    /// reaction's [`View`].
    pub fn maintain_view(&mut self, delta: V::Delta) {
        self.deltas.push(delta);
    }

    /// Take the staged deltas, in stage order. (Drained by the runtime after
    /// the reaction has run.)
    #[must_use]
    pub fn into_deltas(self) -> Vec<V::Delta> {
        self.deltas
    }
}

impl<V> Default for Effects<V>
where
    V: View,
{
    fn default() -> Self {
        Self::new()
    }
}

// React

/// A reaction: handles **one** event type, built from it via `From`, staging
/// effects. The event-only shape — it reacts to the triggering event alone,
/// reading no state; the runtime applies the staged deltas to its [`View`].
pub trait React: From<Self::Event> {
    /// The single event type this reaction reacts to.
    type Event: Event;

    /// The view this reaction maintains.
    type View: View;

    /// React to the triggering event (captured by `From`), staging effects.
    fn react(&self, effects: &mut Effects<Self::View>);
}
