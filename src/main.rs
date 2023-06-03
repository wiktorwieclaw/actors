#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use panic_probe as _;

pub mod example {
    use actors::{Actor, Addr, Ctx};

    pub struct Counter {
        pub count: i32,
    }

    pub enum Message {
        Inc,
        Dec,
    }

    pub const QUEUE_SIZE: usize = 128;
    pub type Address = Addr<Counter, QUEUE_SIZE>;

    impl Actor for Counter {
        type Msg = Message;
        type Ctx = Ctx<Self, QUEUE_SIZE>;

        fn handle(&mut self, msg: Self::Msg, _ctx: &mut Self::Ctx) {
            match msg {
                Message::Inc => self.count += 1,
                Message::Dec => self.count -= 1,
            }
        }
    }
}

use example::{Address, Counter, Message};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let address: Address = actors::spawn!(spawner, Counter, 128, Counter { count: 0 }).unwrap();

    join3(
        address.send(Message::Inc),
        address.send(Message::Inc),
        address.send(Message::Dec),
    )
    .await;
}

mod counterexample {}
