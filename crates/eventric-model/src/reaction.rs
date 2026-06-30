//! Reactions: the [`React`] trait — a single-event handler built from its
//! triggering event (via `From`) that stages effects: [`View`]-maintaining
//! deltas and/or commands to issue. The reaction reads no state (the
//! *event-only* shape); the runtime applies the deltas and dispatches the
//! commands. Driven by the `eventric-runtime` reactor. A reaction declares only
//! the effect kinds it uses — [`View`] defaults to [`NoView`] and `Command` to
//! `()`.

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

/// The unit view, for a reaction that maintains none (the [`React::View`]
/// default).
#[derive(Debug, Default)]
pub struct NoView;

impl View for NoView {
    type Delta = ();

    fn apply(&mut self, (): ()) {}
}

// Effects

/// The buffer a reaction stages effects into: `MaintainView` deltas (the
/// runtime applies them to the view) and commands to issue (`C`, the runtime
/// dispatches them). A fully heterogeneous, pluggable effect set is a later
/// step — for now these are the two kinds.
pub struct Effects<V, C = ()>
where
    V: View,
{
    deltas: Vec<V::Delta>,
    commands: Vec<C>,
}

impl<V, C> Effects<V, C>
where
    V: View,
{
    /// An empty buffer. (Constructed by the runtime, once per reaction.)
    #[must_use]
    pub fn new() -> Self {
        Self {
            deltas: Vec::new(),
            commands: Vec::new(),
        }
    }

    /// Stage a `MaintainView` effect: a `delta` for the runtime to apply to the
    /// reaction's [`View`].
    pub fn maintain_view(&mut self, delta: V::Delta) {
        self.deltas.push(delta);
    }

    /// Stage an `IssueCommand` effect: a `command` for the runtime to dispatch.
    pub fn issue_command(&mut self, command: C) {
        self.commands.push(command);
    }

    /// Take the staged effects, in stage order — the view deltas and the
    /// commands. (Drained by the runtime after the reaction has run.)
    #[must_use]
    pub fn into_parts(self) -> (Vec<V::Delta>, Vec<C>) {
        (self.deltas, self.commands)
    }
}

impl<V, C> Default for Effects<V, C>
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
/// reading no state; the runtime applies the staged view deltas and dispatches
/// the staged commands.
pub trait React: From<Self::Event> {
    /// The single event type this reaction reacts to.
    type Event: Event;

    /// The view this reaction maintains (none, by default).
    type View: View = NoView;

    /// The command this reaction issues (none, by default).
    type Command = ();

    /// React to the triggering event (captured by `From`), staging effects.
    fn react(&self, effects: &mut Effects<Self::View, Self::Command>);
}
