use fancy_constructor::new;
use thiserror::Error;

use crate::{
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
    #[rustfmt::skip]
    pub fn append<'a, E>(
        &mut self,
        events: E,
        condition: Option<&AppendCondition<'_>>,
    ) -> Result<(), AppendError>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        if let Some(condition) = condition {
            let query_hash = &condition.query.into();
            let position = condition.position;

            if self.data.indices.contains(query_hash, position) {
                return Err(AppendError::Concurrency);
            }
        }

        let mut batch = self.database.batch();

        for event in events {
            let event = event.into();
            let timestamp = Timestamp::now();

            self.data.events.put(&mut batch, &event, timestamp, self.position);
            self.data.indices.put(&mut batch, &event, timestamp, self.position);
            self.data.references.put(&mut batch, &event);

            self.position = self.position.increment();
        }

        batch.commit()?;

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

// Append Condition

#[derive(new, Debug)]
#[new(vis())]
pub struct AppendCondition<'a> {
    pub(crate) query: &'a Query,
    #[new(default)]
    pub(crate) position: Option<Position>,
}

impl<'a> AppendCondition<'a> {
    #[must_use]
    pub fn build(query: &'a Query) -> Self {
        Self::new(query)
    }
}

impl AppendCondition<'_> {
    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Append Error

#[derive(Debug, Error)]
pub enum AppendError {
    #[error("Concurrency Error")]
    Concurrency,
    #[error("Database IO Error")]
    Database(#[from] fjall::Error),
}
