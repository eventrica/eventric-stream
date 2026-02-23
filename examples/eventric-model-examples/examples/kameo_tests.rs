use std::ops::ControlFlow;

use fancy_constructor::new;
use kameo::{
    Actor,
    actor::{
        ActorId,
        ActorRef,
        Spawn,
        WeakActorRef,
    },
    error::{
        ActorStopReason,
        Infallible,
        PanicError,
    },
    messages,
    prelude::Context,
    reply::ForwardedReply,
};
use tokio::signal;

// =================================================================================================
// Kameo Supervisor Experiment
// =================================================================================================

// Supervisor

#[derive(new)]
struct Supervisor {
    #[new(into)]
    message: String,
    #[new(default)]
    primary: Option<(ActorId, ActorRef<Primary>)>,
}

impl Supervisor {
    async fn start_primary(&mut self, actor_ref: &ActorRef<Self>) {
        let primary_ref = Primary::spawn_link(actor_ref, self.message.clone()).await;
        let primary_id = primary_ref.id();

        println!("registering primary in supervisor");

        primary_ref
            .register("primary")
            .expect("registration failed");

        println!("setting primary in supervisor");

        self.primary = Some((primary_id, primary_ref));
    }
}

#[messages]
impl Supervisor {
    #[message(ctx)]
    async fn start(&mut self, ctx: &mut Context<Self, ()>) {
        self.start_primary(ctx.actor_ref()).await;
    }

    #[message(ctx)]
    async fn process_supervisor(&mut self, ctx: &mut Context<Self, String>) -> String {
        println!("handling process in supervisor");

        if let Some((_, primary_ref)) = self.primary.as_ref()
            && primary_ref.is_alive()
        {
            println!("forwarding process in supervisor");

            primary_ref.ask(Process).await.expect("process fail")
        } else {
            println!("panicking in process in supervisor");

            panic!("primary ref is dead");
        }
    }
}

impl Actor for Supervisor {
    type Args = String;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("on_start for supervisor: {args}");

        Ok(Self::new(args))
    }

    async fn on_link_died(
        &mut self,
        actor_ref: WeakActorRef<Self>,
        id: ActorId,
        reason: ActorStopReason,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        println!("supervisor link died. id: {id} - reason: {reason} (in {actor_ref:?})");

        if let Some((primary, _)) = self.primary
            && primary == id
            && let Some(actor_ref) = actor_ref.upgrade()
        {
            println!("restarting failed primary");

            self.start_primary(&actor_ref).await;
        }

        Ok(ControlFlow::Continue(()))
    }

    async fn on_panic(
        &mut self,
        actor_ref: WeakActorRef<Self>,
        err: PanicError,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        println!("got panic in supervisor: {err} (in {actor_ref:?}");

        Ok(ControlFlow::Continue(()))
    }
}

// Primary

#[derive(new)]
struct Primary {
    _message: String,
}

#[messages]
impl Primary {
    #[allow(clippy::unused_self)]
    #[message]
    fn kill_primary(&self) {
        panic!("primary killed!");
    }

    #[allow(clippy::unused_self)]
    #[message]
    fn process(&self) -> String {
        println!("handling process in primary");

        "message from primary".into()
    }
}

impl Actor for Primary {
    type Args = String;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        println!("on_start in primary: {args}");

        Ok(Self::new(args))
    }
}

// Secondary

#[derive(new)]
struct Secondary {
    message: String,
}

#[messages]
impl Secondary {
    #[allow(clippy::unused_self)]
    #[message]
    fn kill_secondary(&self) {
        panic!("secondary killed!");
    }
}

impl Actor for Secondary {
    type Args = String;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(Self::new(args))
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let supervisor = Supervisor::spawn("test".into());

    supervisor.register("supervisor")?;

    println!("sending start message to supervisor");

    supervisor.ask(Start).await?;

    let primary = ActorRef::<Primary>::lookup("primary")?;

    println!("got primary ref: {primary:?}");

    if let Some(primary) = primary {
        println!("sending kill message to primary");

        primary.tell(KillPrimary).await?;
    }

    println!("sending process to supervisor");

    let reply = supervisor.ask(ProcessSupervisor).await?;

    println!("received: {reply}");

    signal::ctrl_c().await?;

    Ok(())
}
