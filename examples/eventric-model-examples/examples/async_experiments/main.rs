use std::error;

use eventric_stream::{
    error::Error,
    event::{
        CandidateEvent,
        Data,
        Identifier,
        Position,
        Version,
    },
    stream::{
        Stream,
        Writer,
        append::Append as _,
    },
};
use fancy_constructor::new;
use kameo::{
    Actor,
    actor::{
        ActorRef,
        Spawn,
        WeakActorRef,
    },
    error::{
        ActorStopReason,
        Infallible,
    },
    messages,
};

#[derive(new)]
pub struct StreamWriter {
    writer: Writer,
}

impl Actor for StreamWriter {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("stream writer starting");

        Ok(args)
    }

    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        reason: ActorStopReason,
    ) -> Result<(), Self::Error> {
        println!("stream writer stopping ({reason})");

        Ok(())
    }
}

#[messages]
impl StreamWriter {
    #[message]
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = CandidateEvent> + Send + 'static,
        E::IntoIter: Send,
    {
        self.writer.append(events, after)
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn error::Error>> {
    let stream = Stream::builder("./temp").temporary(true).open()?;
    let stream_components = stream.split();

    let stream_writer = StreamWriter::new(stream_components.1);
    let stream_writer_ref = StreamWriter::spawn_in_thread(stream_writer);

    let position = stream_writer_ref
        .ask(Append {
            events: [
                CandidateEvent::new(
                    Data::new([0x0])?,
                    Identifier::new("test")?,
                    [],
                    Version::new(0),
                ),
                CandidateEvent::new(
                    Data::new([0x1])?,
                    Identifier::new("test")?,
                    [],
                    Version::new(0),
                ),
            ],
            after: None,
        })
        .await?;

    println!("new position: {position:?}");

    stream_writer_ref.stop_gracefully().await?;
    stream_writer_ref.wait_for_shutdown().await;

    Ok(())
}
