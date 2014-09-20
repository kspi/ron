use std::vec::Vec;
use std::string::String;

pub type Position = (uint, uint);

#[deriving(PartialEq, Eq, Show, Clone)]
pub enum Direction {
    North,
    West,
    South,
    East
}

impl Direction {
    pub fn left(&self) -> Direction {
        match *self {
            North => West,
            East => North,
            South => East,
            West => South
        }
    }

    pub fn right(&self) -> Direction {
        match *self {
            North => East,
            East => South,
            South => West,
            West => North
        }
    }

    pub fn apply_to(&self, position: Position) -> Position {
        match position {
            (r, c) => match *self {
                North => (r - 1, c),
                West => (r, c - 1),
                South => (r + 1, c),
                East => (r, c + 1)
            }
        }
    }

    pub fn action_for(&self, target: Direction) -> Option<Action> {
        if *self == target {
            Some(MoveForward)
        } else if self.left() == target {
            Some(TurnLeft)
        } else if self.right() == target {
            Some(TurnRight)
        } else {
            None
        }
    }
}

#[deriving(PartialEq, Eq, Show, Clone)]
pub enum Action {
    MoveForward,
    TurnLeft,
    TurnRight
}

impl Action {
    fn apply_to(&self, direction : Direction) -> Direction {
        match *self {
            MoveForward => direction,
            TurnLeft => direction.left(),
            TurnRight => direction.right()
        }
    }
}

pub type PlayerIndex = uint;

#[deriving(PartialEq, Eq, Show, Clone)]
pub struct Player {
    pub name: String,
    pub position: Position,
    pub direction: Direction,
    pub is_alive: bool
}

pub struct Behaviour {
    sender: Sender<GameState>,
    pub receiver: Receiver<(uint, Action)>
}

impl Behaviour {
    pub fn make(body: proc (Receiver<GameState>, Sender<(uint, Action)>): Send) -> Behaviour {
        let (state_sender, state_receiver) = channel::<GameState>();
        let (action_sender, action_receiver) = channel::<(uint, Action)>();
        spawn(proc () {
            body(state_receiver, action_sender);
        });
        Behaviour {
            sender: state_sender,
            receiver: action_receiver
        }
    }

    pub fn send_state(&self, game: &GameState) {
        self.sender.send(game.clone());
    }
}

#[deriving(PartialEq, Eq, Show, Clone)]
pub enum Tile {
    Empty,
    PlayerWall(PlayerIndex),
    PlayerHead(PlayerIndex),
    Crash
}

impl Tile {
    pub fn is_passable(&self) -> bool {
        match *self {
            Empty => true,
            _ => false
        }
    }
}

#[deriving(PartialEq, Eq, Show, Clone)]
pub enum GameStatus {
    PlayerTurn(PlayerIndex),
    Won(PlayerIndex)
}

impl GameStatus {
    pub fn is_over(&self) -> bool {
        match *self {
            PlayerTurn(_) => false,
            _ => true
        }
    }
}

#[deriving(Clone, Show, PartialEq, Eq)]
pub struct GameState {
    pub turn: uint,
    pub players: Vec<Player>,
    pub alive_count: uint,
    pub status: GameStatus,
    pub board_width: uint,
    pub board_height: uint,
    pub board: Vec<Vec<Tile>>
}

impl GameState {
    pub fn new(board_width: uint, board_height: uint, players: Vec<Player>) -> GameState {
        let mut s = GameState {
            turn: 0,
            players: players,
            alive_count: 2,
            status: PlayerTurn(0),
            board_width: board_width,
            board_height: board_height,
            board: Vec::from_elem(board_height, Vec::from_elem(board_width, Empty))
        };
        // Place initial walls.
        for i in range(0, s.alive_count) {
            let pos = s.players[i].position;
            s.board_set(pos, PlayerHead(i))
        };
        s
    }

    fn board_set(&mut self, position: Position, tile: Tile) {
        match position {
            (r, c) => self.board.get_mut(r).grow_set(c, &Empty, tile)
        }
    }

    fn move_to(&mut self, player: PlayerIndex, position: Position) {
        let old_pos = self.players[player].position;
        self.board_set(old_pos, PlayerWall(player));
        self.board_set(position, PlayerHead(player));
        self.players.get_mut(player).position = position;
    }

    pub fn can_move_to(&self, position: Position) -> bool {
        match position {
            (row, column) => {
                row < self.board_height &&
                column < self.board_width &&
                self.board[row][column].is_passable()
            }
        }
    }

    pub fn player_after(&self, current: PlayerIndex) -> PlayerIndex {
        assert!(self.alive_count >= 1);
        let mut cur = (current + 1) % self.players.len();
        while !self.players[cur].is_alive {
            cur = (cur + 1) % self.players.len();
        }
        cur
    }

    pub fn is_over(&self) -> bool {
        match self.status {
            PlayerTurn(_) => false,
            _ => true
        }
    }

    pub fn current_player(&self) -> PlayerIndex {
        match self.status {
            PlayerTurn(x) => x,
            _ => fail!("GameState::current_player called after game over")
        }
    }

    pub fn winner(&self) -> PlayerIndex {
        match self.status {
            Won(x) => x,
            _ => fail!("GameState::winner called before game over")
        }
    }

    pub fn do_turn(&mut self, action: Action) {
        let current = self.current_player();
        let cur_direction = self.players[current].direction;
        let new_direction = action.apply_to(cur_direction);
        self.players.get_mut(current).direction = new_direction;
        let cur_position = self.players[current].position;
        let new_position = new_direction.apply_to(cur_position);

        if self.can_move_to(new_position) {
            self.move_to(current, new_position);
        } else {
            self.board_set(cur_position, Crash);
            self.players.get_mut(current).is_alive = false;
            self.alive_count -= 1;
        }

        let next_player = self.player_after(current);
        if self.alive_count == 1 {
            self.status = Won(next_player)
        } else {
            self.status = PlayerTurn(next_player)
        }

        self.turn += 1;
    }
}
