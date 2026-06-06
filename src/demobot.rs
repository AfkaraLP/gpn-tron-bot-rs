use crate::{
    Bot, GpnTronBot,
    packets::{MoveDirection, ServerPacket},
};

/// # Demo bot for `GPN tron`.
///
/// Picks a safe direction every tick. Prefers continuing straight, otherwise
/// turns to any non-blocked neighbour. If fully boxed in it just charges
/// forward and accepts its fate.
pub struct DemoBot {
    direction: MoveDirection,
    ticks: usize,
}

impl DemoBot {
    pub const fn new() -> Self {
        Self {
            direction: MoveDirection::Up,
            ticks: 0,
        }
    }

    /// Choose the next move based on the current world state in `bot`.
    fn choose_direction(&self, bot: &Bot) -> MoveDirection {
        let safe = bot.safe_directions();
        if safe.is_empty() {
            return self.direction;
        }
        if safe.contains(&self.direction) {
            return self.direction;
        }
        for d in [
            MoveDirection::Up,
            MoveDirection::Right,
            MoveDirection::Down,
            MoveDirection::Left,
        ] {
            if safe.contains(&d) {
                return d;
            }
        }
        self.direction
    }
}

impl GpnTronBot for DemoBot {
    fn handle_packet(&mut self, bot: &mut Bot, packet: ServerPacket) {
        match packet {
            ServerPacket::Tick => {
                self.ticks += 1;
                self.direction = self.choose_direction(bot);
                if self.ticks == 1 {
                    bot.chat("Demo bot. gpn-tron on crates.io");
                }
                bot.do_move(self.direction);
            }
            ServerPacket::Game { .. } => {
                self.direction = MoveDirection::Up;
                self.ticks = 0;
            }
            _ => {}
        }
    }
}
