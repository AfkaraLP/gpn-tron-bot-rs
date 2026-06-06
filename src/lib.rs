#![doc = include_str!("../README.md")]

use tokio::{io::AsyncWriteExt as _, net::TcpStream};

use tokio_stream::StreamExt as _;
use tokio_util::codec::{FramedRead, LinesCodec};

use crate::packets::{ClientPacket, MoveDirection, ServerPacket};

pub mod demobot;
pub mod errors;
pub mod packets;

pub struct Bot {
    instruction_queue: Vec<ClientPacket>,
    world: WorldMap,
    username: String,
    user_id: Option<usize>,
    current_position: Option<(usize, usize)>,
}

/// Grid of the playing field with None being empty fields and Some if it's occupied with the occupant's user id.
pub struct WorldMap {
    cells: Vec<Vec<Option<usize>>>,
    width: usize,
    height: usize,
}

impl WorldMap {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![None; width]; height],
            width,
            height,
        }
    }

    /// Returns (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Get returns either None if the field is empty or Some with the id of the user occupying it.
    pub fn get(&self, x: usize, y: usize) -> Option<usize> {
        if self.width == 0 || self.height == 0 {
            return None;
        }
        let x = x % self.width;
        let y = y % self.height;
        self.cells[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, val: usize) {
        if self.width == 0 || self.height == 0 {
            return;
        }
        let x = x % self.width;
        let y = y % self.height;
        self.cells[y][x] = Some(val);
    }

    /// Clear every cell occupied by `player_id`.
    pub fn clear_player(&mut self, player_id: usize) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                if *cell == Some(player_id) {
                    *cell = None;
                }
            }
        }
    }

    /// Returns the (x, y) coordinate after stepping once in a direction.
    pub fn step(&self, x: usize, y: usize, direction: MoveDirection) -> (usize, usize) {
        let (width, height) = (self.width, self.height);
        match direction {
            MoveDirection::Up => (x, (y + height - 1) % height),
            MoveDirection::Down => (x, (y + 1) % height),
            MoveDirection::Left => ((x + width - 1) % width, y),
            MoveDirection::Right => ((x + 1) % width, y),
        }
    }

    /// True if the cell in `direction` from `(x, y)` is occupied.
    pub fn is_blocked(&self, x: usize, y: usize, direction: MoveDirection) -> bool {
        let (nx, ny) = self.step(x, y, direction);
        self.get(nx, ny).is_some()
    }
}

impl Bot {
    fn new(username: impl Into<String>) -> Self {
        let username = username.into();
        Self {
            instruction_queue: Vec::new(),
            world: WorldMap::new(0, 0),
            username,
            user_id: None,
            current_position: None,
        }
    }

    /// Read-only access to the world map.
    pub fn world(&self) -> &WorldMap {
        &self.world
    }

    /// The bot's player id, once received from the server.
    pub fn user_id(&self) -> Option<usize> {
        self.user_id
    }

    /// The bot's current position, once received from the server.
    pub fn position(&self) -> Option<(usize, usize)> {
        self.current_position
    }

    /// Returns every direction whose neighbouring cell is currently empty.
    pub fn safe_directions(&self) -> Vec<MoveDirection> {
        let Some((x, y)) = self.current_position else {
            return Vec::new();
        };
        [
            MoveDirection::Up,
            MoveDirection::Down,
            MoveDirection::Left,
            MoveDirection::Right,
        ]
        .into_iter()
        .filter(|d| !self.world.is_blocked(x, y, *d))
        .collect()
    }

    /// True if moving in `direction` from the current position would hit a trail.
    /// Returns `true` when the position is unknown.
    pub fn is_blocked(&self, direction: MoveDirection) -> bool {
        match self.current_position {
            Some((x, y)) => self.world.is_blocked(x, y, direction),
            None => true,
        }
    }

    /// Write a raw packet to the server.
    pub fn write_packet(&mut self, packet: ClientPacket) {
        self.instruction_queue.push(packet);
    }

    /// Send a message in chat, make sure to not get rate limited.
    pub fn chat(&mut self, message: impl Into<String>) {
        self.write_packet(ClientPacket::Chat(message.into()));
    }

    /// Set a move direction. (I think you should only do one per tick I dir not test several)
    pub fn do_move(&mut self, direction: MoveDirection) {
        self.write_packet(ClientPacket::Move(direction));
    }
}

pub trait GpnTronBot: Sized {
    fn handle_packet(&mut self, bot: &mut Bot, packet: ServerPacket);

    #[allow(async_fn_in_trait)]
    async fn start(
        &mut self,
        url: impl std::fmt::Display,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> crate::errors::Result<()> {
        let addr = format!("{url}:{port}");
        println!("connecting to {addr}");
        let stream =
            TcpStream::connect(&addr)
                .await
                .map_err(|source| crate::errors::Error::Connect {
                    addr: addr.clone(),
                    source: std::sync::Arc::new(source),
                })?;
        let (read, mut write) = stream.into_split();
        let username = username.into();
        let join = ClientPacket::join(username.clone(), password).to_string();
        write.write_all(join.as_bytes()).await?;
        let decoder = LinesCodec::new();
        let mut read = FramedRead::new(read, decoder);
        let mut bot = Bot::new(username);
        loop {
            while let Some(next) = read.next().await {
                let string = next?;
                let packet = ServerPacket::parse(string)?;
                match &packet {
                    ServerPacket::Game { width, height, .. } => {
                        bot.world = WorldMap::new(*width, *height);
                        bot.current_position = None;
                    }
                    ServerPacket::Position { player_id, x, y } => {
                        bot.world.set(*x, *y, *player_id);
                        if Some(*player_id) == bot.user_id {
                            bot.current_position = Some((*x, *y));
                        }
                    }
                    ServerPacket::Player { id, name } => {
                        if name == &bot.username {
                            bot.user_id.replace(*id);
                        }
                    }
                    ServerPacket::Die { user_ids } => {
                        for id in user_ids {
                            bot.world.clear_player(*id);
                            if Some(*id) == bot.user_id {
                                bot.current_position = None;
                            }
                        }
                    }
                    _ => {}
                }
                self.handle_packet(&mut bot, packet);
                for packet in &bot.instruction_queue {
                    write.write_all(packet.to_string().as_bytes()).await?;
                }
                bot.instruction_queue.clear();
            }
        }
    }
}
