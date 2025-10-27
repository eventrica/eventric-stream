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
    ) -> Result<Position, AppendError>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        if let Some(condition) = condition {
            self.append_check(condition)?;
        }

        self.append_put(events)?;

        Ok(self.position)
    }

    #[rustfmt::skip]
    fn append_check(&self, condition: &AppendCondition<'_>) -> Result<(), AppendError> {
        if let Some(position) = condition.after && position >= self.position {
            return Ok(());
        }

        let query = condition.fail_if_matches.into();
        let position = condition.after.map(|position| position + 1);

        if self.data.indices.contains(&query, position) {
            return Err(AppendError::Concurrency);
        }

        Ok(())
    }

    #[rustfmt::skip]
    fn append_put<'a, E>(&mut self, events: E) -> Result<(), Error>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        let mut batch = self.database.batch();

        for event in events {
            let event = event.into();
            let timestamp = Timestamp::now()?;

            self.data.events.put(&mut batch, &event, timestamp, self.position);
            self.data.indices.put(&mut batch, &event, timestamp, self.position);
            self.data.references.put(&mut batch, &event);

            self.position += 1;
        }

        batch.commit().map_err(Error::from)?;

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

// Append Condition

#[derive(new, Debug)]
#[new(name(inner), vis())]
pub struct AppendCondition<'a> {
    #[new(default)]
    pub(crate) after: Option<Position>,
    pub(crate) fail_if_matches: &'a Query,
}

impl<'a> AppendCondition<'a> {
    #[must_use]
    pub fn new(fail_if_matches: &'a Query) -> Self {
        Self::inner(fail_if_matches)
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
