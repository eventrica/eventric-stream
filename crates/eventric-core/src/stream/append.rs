use std::error::Error;

use crate::{
    model::event::{
        Event,
        timestamp::Timestamp,
    },
    stream::{
        Stream,
        condition::Condition,
    },
};

// =================================================================================================
// Append
// =================================================================================================

impl Stream {
    #[rustfmt::skip]
    pub fn append<'a, E>(&mut self, events: E, _condition: Condition<'_>) -> Result<(), Box<dyn Error>>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        // TODO: Condition checking!

        let mut batch = self.database.batch();

        for event in events {
            let event = event.into();
            let timestamp = Timestamp::now();

            self.data.events.put(&mut batch, &event, timestamp, self.position);
            self.data.indices.put(&mut batch, &event, timestamp, self.position);
            self.data.references.put(&mut batch, &event);
            self.position.increment();
        }

        batch.commit()?;

        Ok(())
    }
}
