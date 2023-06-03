#![no_std]

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Sender};
use core::clone::Clone;

pub trait Actor {
    type Msg: 'static;
    type Ctx;

    fn handle(&mut self, msg: Self::Msg, ctx: &mut Self::Ctx);
}

pub struct Ctx<A: Actor, const SIZE: usize> {
    pub addr: Addr<A, SIZE>, // TODO: make private
}

impl<A: Actor, const SIZE: usize> Ctx<A, SIZE> {
    pub fn address(&self) -> Addr<A, SIZE> {
        self.addr.clone()
    }
}

pub struct Addr<A: Actor, const SIZE: usize> {
    pub sender: Sender<'static, NoopRawMutex, A::Msg, SIZE>, // TODO: make private
}

impl<A: Actor, const SIZE: usize> Clone for Addr<A, SIZE> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A: Actor, const SIZE: usize> Addr<A, SIZE> {
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

        let addr = Addr { sender };
        let ctx = Ctx { addr: addr.clone() };

        #[embassy_executor::task]
        async fn task(
            mut actor: $actor_type,
            receiver: Receiver<'static, NoopRawMutex, Message, $size>,
            mut ctx: Ctx<$actor_type, $size>,
        ) {
            loop {
                let msg = receiver.recv().await;
                actor.handle(msg, &mut ctx);
            }
        }

        $spawner.spawn(task($actor, receiver, ctx)).map(|_| addr)
    }};
}
