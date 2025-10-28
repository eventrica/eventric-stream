pub mod condition;

use eventric_core_error::Error;
use eventric_core_event::{
    Event,
    position::Position,
    timestamp::Timestamp,
};

use crate::{
    Stream,
    append::condition::Condition,
};

// =================================================================================================
// Append
// =================================================================================================

impl Stream {
    pub fn append<'a, E>(
        &mut self,
        events: E,
        condition: Option<&Condition<'_>>,
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
    fn append_check(&self, condition: &Condition<'_>) -> Result<(), Error> {
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
