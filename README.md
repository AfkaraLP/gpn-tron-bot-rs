# gpn-tron

A small Rust client library for the [GPN Tron] game. Built for **GPN 24** in
Karlsruhe.

[GPN Tron]: https://github.com/freehuntx/gpn-tron

## Run the demo bot

The crate ships with a [`DemoBot`](demobot::DemoBot) that just walks straight
and turns when it would crash:

```rust,no_run
use gpn_tron::{GpnTronBot, demobot::DemoBot};

#[tokio::main]
async fn main() {
    let mut bot = DemoBot::new();
    bot.start("localhost", 4000, "myname", "mypassword")
        .await
        .unwrap();
}
```

## Write your own bot

Implement [`GpnTronBot::handle_packet`] and react to ticks. Here is a bot that
just runs in a circle:

```rust,no_run
use gpn_tron::{
    Bot, GpnTronBot,
    packets::{MoveDirection, ServerPacket},
};

struct CircleBot {
    direction: MoveDirection,
}

impl GpnTronBot for CircleBot {
    fn handle_packet(&mut self, bot: &mut Bot, packet: ServerPacket) {
        if let ServerPacket::Tick = packet {
            self.direction = match self.direction {
                MoveDirection::Up => MoveDirection::Right,
                MoveDirection::Right => MoveDirection::Down,
                MoveDirection::Down => MoveDirection::Left,
                MoveDirection::Left => MoveDirection::Up,
            };
            bot.do_move(self.direction);
        }
    }
}

#[tokio::main]
async fn main() {
    let mut bot = CircleBot { direction: MoveDirection::Up };
    bot.start("localhost", 4000, "circler", "supersecret")
        .await
        .unwrap();
}
```

