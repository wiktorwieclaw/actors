#![no_std]

use core::clone::Clone;
use embassy_sync::channel::DynamicSender;

pub trait Actor: Sized {
    type Msg: 'static;

    fn handle(&mut self, msg: Self::Msg, ctx: &mut Ctx<Self>);
}

pub struct Ctx<A: Actor> {
    pub addr: Addr<A>, // TODO: make private
}

impl<A: Actor> Ctx<A> {
    pub fn address(&self) -> Addr<A> {
        self.addr.clone()
    }
}

pub struct Addr<A: Actor> {
    pub sender: DynamicSender<'static, A::Msg>, // TODO: make private
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A: Actor> Addr<A> {
    pub async fn send(&self, msg: A::Msg) {
        self.sender.send(msg).await
    }
}

#[macro_export]
macro_rules! spawn {
    ($spawner:expr, $actor_type:ty, $size:expr, $actor:expr) => {{
        use ::actors::{Actor, Addr, Ctx};
        use embassy_sync::{
            blocking_mutex::raw::NoopRawMutex,
            channel::{Channel, Receiver},
        };
        use static_cell::StaticCell;

        type Message = <$actor_type as Actor>::Msg;

        static CHANNEL: StaticCell<Channel<NoopRawMutex, Message, $size>> = StaticCell::new();
        let channel = CHANNEL.init(Channel::new());
        let (sender, receiver) = (channel.sender(), channel.receiver());

        let addr = Addr {
            sender: sender.into(),
        };
        let ctx = Ctx { addr: addr.clone() };

        #[embassy_executor::task]
        async fn task(
            mut actor: $actor_type,
            receiver: Receiver<'static, NoopRawMutex, Message, $size>,
            mut ctx: Ctx<$actor_type>,
        ) {
            loop {
                let msg = receiver.recv().await;
                actor.handle(msg, &mut ctx);
            }
        }

        $spawner.spawn(task($actor, receiver, ctx)).map(|_| addr)
    }};
}
