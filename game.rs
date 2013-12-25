use std::vec;

type Position = (int, int);

#[deriving(Eq, ToStr, Clone)]
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

    fn apply_to(&self, position: Position) -> Position {
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

#[deriving(ToStr, Clone)]
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

type PlayerIndex = uint;

#[deriving(ToStr, Clone)]
pub struct Player {
    name: ~str,
    position: Position,
    direction: Direction,
    is_alive: bool
}

pub trait PlayerBehaviour {
    fn act(&mut self, &GameState) -> Action;
}

#[deriving(ToStr, Clone)]
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

#[deriving(ToStr, Clone)]
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

#[deriving(Clone, ToStr)]
pub struct GameState {
    turn: uint,
    players: ~[Player],
    alive_count: uint,
    status: GameStatus,
    board_width: uint,
    board_height: uint,
    board: ~[~[Tile]]
}

impl GameState {
    pub fn new(board_width: uint, board_height: uint, players: ~[Player]) -> GameState {
        let mut s = GameState {
            turn: 0,
            players: players,
            alive_count: 2,
            status: PlayerTurn(0),
            board_width: board_width,
            board_height: board_height,
            board: vec::from_elem(board_height, vec::from_elem(board_width, Empty))
        };
        // Place initial walls.
        for i in range(0, s.alive_count) {
            s.board_set(s.players[i].position, PlayerHead(i))
        };
        s
    }

    fn board_set(&mut self, position: Position, tile: Tile) {
        match position {
            (r, c) => self.board[r][c] = tile
        }
    }

    fn move_to(&mut self, player: PlayerIndex, position: Position) {
        self.board_set(self.players[player].position, PlayerWall(player));
        self.board_set(position, PlayerHead(player));
        self.players[player].position = position;
    }

    fn can_move_to(&self, position: Position) -> bool {
        match position {
            (row, column) => {
                0 <= row &&
                row < self.board_height as int &&
                0 <= column &&
                column < self.board_width as int &&
                self.board[row][column].is_passable()
            }
        }
    }

    fn player_after(&self, current: PlayerIndex) -> PlayerIndex {
        assert!(self.alive_count >= 1);
        let mut cur = (current + 1) % self.players.len();
        while !self.players[cur].is_alive {
            cur = (cur + 1) % self.players.len();
        }
        cur
    }

    pub fn current_player(&self) -> PlayerIndex {
        match self.status {
            PlayerTurn(x) => x,
            _ => fail!("GameState::current_player called after game over")
        }
    }

    pub fn do_turn(&mut self, behaviours: &mut [~PlayerBehaviour]) {
        let current = self.current_player();
        let cur_direction = self.players[current].direction;
        let action = behaviours[current].act(self);
        let new_direction = action.apply_to(cur_direction);
        self.players[current].direction = new_direction;
        let cur_position = self.players[current].position;
        let new_position = new_direction.apply_to(cur_position);

        if self.can_move_to(new_position) {
            self.move_to(current, new_position);
        } else {
            self.board_set(cur_position, Crash);
            self.players[current].is_alive = false;
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
