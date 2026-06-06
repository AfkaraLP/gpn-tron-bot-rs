use crate::errors::{Error, Result, ServerError};

/// Packets sent by the server to the client.
#[derive(Clone, Debug)]
pub enum ServerPacket {
    /// Inform about new round.
    Game {
        width: usize,
        height: usize,
        curr_player_id: usize,
    },
    /// Inform about player's position.
    Position {
        player_id: usize,
        x: usize,
        y: usize,
    },
    /// At connection.
    MessageOfTheDay(String),
    /// Share player information.
    Player { id: usize, name: String },
    /// Every time a turn has been done.
    Tick,
    /// Inform about who died.
    Die { user_ids: Vec<usize> },
    /// Inform of a message by another player.
    Message { player_id: usize, content: String },
    /// Inform the client they won.
    Win {
        total_wins: usize,
        total_losses: usize,
    },
    /// Inform the client they lost.
    Lose {
        total_wins: usize,
        total_losses: usize,
    },
    /// An error or warning reported by the server.
    Error(ServerError),
}

impl ServerPacket {
    pub fn parse(s: impl AsRef<str>) -> Result<Self> {
        let input = s.as_ref();
        let mut input = input.split('|');
        let name = input.next().ok_or(Error::NoName)?;
        let packet = match name {
            "game" => {
                let width: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let height: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let curr_player_id: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;

                ServerPacket::Game {
                    width,
                    height,
                    curr_player_id,
                }
            }
            "pos" => {
                let player_id: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let x: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let y: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                ServerPacket::Position { player_id, x, y }
            }
            "motd" => {
                let content = input.collect::<Vec<_>>().join("|");
                if content.is_empty() {
                    return Err(Error::MalformedPacket);
                }
                ServerPacket::MessageOfTheDay(content)
            }
            "error" => {
                let code = input.next().ok_or(Error::MalformedPacket)?;
                ServerPacket::Error(code.parse::<ServerError>()?)
            }
            "player" => {
                let id: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let name = input.next().ok_or(Error::MalformedPacket)?.to_string();
                ServerPacket::Player { id, name }
            }
            "tick" => ServerPacket::Tick,
            "die" => {
                let user_ids = input
                    .map(|s| s.parse::<usize>())
                    .collect::<core::result::Result<Vec<_>, _>>()?;
                if user_ids.is_empty() {
                    return Err(Error::MalformedPacket);
                }
                ServerPacket::Die { user_ids }
            }
            "message" => {
                let player_id: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let content = input.collect::<Vec<_>>().join("|");
                if content.is_empty() {
                    return Err(Error::MalformedPacket);
                }
                ServerPacket::Message { player_id, content }
            }
            "win" => {
                let total_wins: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let total_losses: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                ServerPacket::Win {
                    total_wins,
                    total_losses,
                }
            }
            "lose" => {
                let total_wins: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                let total_losses: usize = input.next().ok_or(Error::MalformedPacket)?.parse()?;
                ServerPacket::Lose {
                    total_wins,
                    total_losses,
                }
            }
            _ => return Err(Error::MalformedPacket),
        };
        Ok(packet)
    }
}

/// Packets sent by the client to the server.
pub enum ClientPacket {
    /// First packet sent.
    Join { username: String, password: String },
    /// Decide where to move.
    Move(MoveDirection),
    /// Send a message to chat.
    Chat(String),
}

impl ClientPacket {
    pub fn join(username: impl Into<String>, password: impl Into<String>) -> Self {
        let username = username.into();
        let password = password.into();
        Self::Join { username, password }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for MoveDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveDirection::Up => write!(f, "up"),
            MoveDirection::Down => write!(f, "down"),
            MoveDirection::Left => write!(f, "left"),
            MoveDirection::Right => write!(f, "right"),
        }
    }
}
impl std::fmt::Display for ClientPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientPacket::Join { username, password } => writeln!(f, "join|{username}|{password}"),
            ClientPacket::Move(move_direction) => writeln!(f, "move|{move_direction}"),
            ClientPacket::Chat(msg) => writeln!(f, "chat|{msg}"),
        }
    }
}

impl std::fmt::Display for ServerPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerPacket::Game {
                width,
                height,
                curr_player_id,
            } => writeln!(f, "game|{width}|{height}|{curr_player_id}"),
            ServerPacket::Position { player_id, x, y } => writeln!(f, "pos|{player_id}|{x}|{y}"),
            ServerPacket::MessageOfTheDay(content) => writeln!(f, "motd|{content}"),
            ServerPacket::Player { id, name } => writeln!(f, "player|{id}|{name}"),
            ServerPacket::Tick => writeln!(f, "tick"),
            ServerPacket::Die { user_ids } => {
                writeln!(
                    f,
                    "die{}",
                    user_ids.iter().fold(String::new(), |mut acc, v| {
                        use std::fmt::Write as _;
                        _ = write!(&mut acc, "|{v}");
                        acc
                    })
                )
            }
            ServerPacket::Message { player_id, content } => {
                writeln!(f, "message|{player_id}|{content}")
            }
            ServerPacket::Win {
                total_wins,
                total_losses,
            } => writeln!(f, "win|{total_wins}|{total_losses}"),
            ServerPacket::Lose {
                total_wins,
                total_losses,
            } => writeln!(f, "lose|{total_wins}|{total_losses}"),
            ServerPacket::Error(err) => writeln!(f, "error|{}", err.code()),
        }
    }
}
