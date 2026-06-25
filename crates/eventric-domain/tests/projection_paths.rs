//! Regression: event types named by *qualified* paths in a selection. The
//! generated companion enum re-roots a relative path with `super::` but leaves
//! a crate-rooted (absolute) path unchanged — so both forms compile and fold.

use error_stack::{
    Report,
    ResultExt as _,
};
use eventric_domain::{
    error::Error,
    event::{
        Event,
        Events,
    },
    projection::{
        self,
        Dispatch as _,
        Project,
        Projection,
        Recognize as _,
        Select as _,
    },
};
use eventric_stream::stream::{
    Stream,
    operate::{
        Condition,
        append::Append as _,
        select::Select as _,
    },
};
use fancy_constructor::new;
use revision::revisioned;

// An event at the crate root, referenced below as the absolute `crate::Opened`.
#[revisioned(revision = 1)]
#[derive(new, Event, Debug)]
#[event(identifier: opened, tags: { account: account })]
struct Opened {
    #[new(into)]
    account: String,
}

mod events {
    use super::*;

    // An event in a submodule, referenced below as the relative `events::Closed`.
    #[revisioned(revision = 1)]
    #[derive(new, Event, Debug)]
    #[event(identifier: closed, tags: { account: account })]
    pub struct Closed {
        #[new(into)]
        pub account: String,
    }
}

// One selection naming both — a crate-rooted absolute path and a
// submodule-relative path, with distinct last segments (so distinct variants).
#[derive(new, Projection, Debug)]
#[projection(selections: {
    status: { events: [crate::Opened, events::Closed], filter: { account: account } },
})]
struct Status {
    #[new(default)]
    open: bool,
    #[new(into)]
    account: String,
}

impl Project<status::Status<'_>> for Status {
    fn project(&mut self, event: projection::Event<status::Status<'_>>) {
        self.open = match event.event() {
            status::Status::Opened(_) => true,
            status::Status::Closed(_) => false,
        };
    }
}

fn append<E: Event>(stream: &mut Stream, event: &E) -> Result<(), Report<Error>> {
    let mut events = Events::new();
    events.append(event)?;
    stream
        .append(events.take(), Condition::new())
        .change_context(Error)?;
    Ok(())
}

#[test]
fn qualified_event_paths_compile_and_fold() -> Result<(), Report<Error>> {
    let mut stream = Stream::builder(eventric_stream::utils::temp_path())
        .temporary(true)
        .open()
        .change_context(Error)?;

    append(&mut stream, &Opened::new("a1"))?;
    append(&mut stream, &events::Closed::new("a1"))?;

    let mut status = Status::new("a1");
    let condition = Condition::new().selections(status.select()?);

    for event in stream.select(condition) {
        let event = event.change_context(Error)?;

        if let Some(dispatch) = status.recognize(&event)? {
            status.dispatch(event.mask.as_ref(), &dispatch);
        }
    }

    // Opened (pos 0) then Closed (pos 1): the last event folded wins.
    assert!(!status.open);

    Ok(())
}
