#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use actors::{Actor, Addr, Ctx};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use panic_probe as _;

pub struct Counter {
    pub count: i32,
}

pub enum Message {
    Inc,
    Dec,
}

impl Actor for Counter {
    type Msg = Message;

    fn handle(&mut self, msg: Message, _ctx: &mut Ctx<Self>) {
        match msg {
            Message::Inc => self.count += 1,
            Message::Dec => self.count -= 1,
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let address: Addr<Counter> =
        actors::spawn!(spawner, Counter, 128, Counter { count: 0 }).unwrap();

    join3(
        address.send(Message::Inc),
        address.send(Message::Inc),
        address.send(Message::Dec),
    )
    .await;
}
