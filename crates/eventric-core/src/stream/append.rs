use fancy_constructor::new;
use thiserror::Error;

use crate::{
    error::Error,
    model::{
        event::{
            Event,
            timestamp::Timestamp,
        },
        query::Query,
        stream::position::Position,
    },
    stream::Stream,
};

// =================================================================================================
// Append
// =================================================================================================

impl Stream {
    pub fn append<'a, E>(
        &mut self,
        events: E,
        condition: Option<&AppendCondition<'_>>,
    ) -> Result<Position, Error>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        if let Some(condition) = condition {
            self.append_check(condition)?;
        }

        let position = self.append_put(events)?;

        Ok(position)
    }

    #[rustfmt::skip]
    fn append_check(&self, condition: &AppendCondition<'_>) -> Result<(), Error> {
        if let Some(after) = condition.after && after >= self.next {
            return Ok(());
        }

        let query = condition.fail_if_matches.into();
        let from = condition.after.map(|after| after + 1);

        if self.data.indices.contains(&query, from) {
            return Err(Error::Concurrency);
        }

        Ok(())
    }

    #[rustfmt::skip]
    fn append_put<'a, E>(&mut self, events: E) -> Result<Position, Error>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        let mut batch = self.database.batch();

        for event in events {
            let event = event.into();
            let timestamp = Timestamp::now()?;

            self.data.events.put(&mut batch, self.next, &event, timestamp);
            self.data.indices.put(&mut batch, self.next, &event, timestamp);
            self.data.references.put(&mut batch, &event);

            self.next += 1;
        }

        batch.commit().map_err(Error::from)?;

        Ok(self.next - 1)
    }
}

// -------------------------------------------------------------------------------------------------

// Append Condition

#[derive(new, Debug)]
#[new(name(new_inner), vis())]
pub struct AppendCondition<'a> {
    #[new(default)]
    pub(crate) after: Option<Position>,
    pub(crate) fail_if_matches: &'a Query,
}

impl<'a> AppendCondition<'a> {
    #[must_use]
    pub fn new(fail_if_matches: &'a Query) -> Self {
        Self::new_inner(fail_if_matches)
    }
}

impl AppendCondition<'_> {
    #[must_use]
    pub fn after(mut self, after: Position) -> Self {
        self.after = Some(after);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Append Error

#[derive(Debug, Error)]
pub enum AppendError {
    #[error("Concurrency Error")]
    Concurrency,
    #[error("Internal Error: {0}")]
    Internal(#[from] Error),
}
