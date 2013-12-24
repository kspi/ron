pub static board_width : int = 80;
pub static board_height : int = 24;

type Vec = (int, int);

enum Direction {
    North,
    West,
    South,
    East
}

impl Direction {
    fn apply_to(&self, position: Vec) -> Vec {
        match position {
            (r, c) => match *self {
                North => (r - 1, c),
                West => (r, c + 1),
                South => (r + 1, c),
                East => (r, c - 1)
            }
        }
    }
}

type PlayerIndex = uint;

struct Player {
    name: ~str,
    position: Vec,
    direction: Direction,
    is_alive: bool,
    get_action: ~fn(&GameState) -> Direction
}

pub enum Tile {
    Empty,
    PlayerWall(PlayerIndex)
}

impl Tile {
    pub fn is_passable(&self) -> bool {
        match *self {
            Empty => true,
            _ => false
        }
    }
}

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

pub type Board = [[Tile, ..board_width], ..board_height];

pub struct GameState {
    turn: uint,
    players: ~[Player],
    alive_count: uint,
    status: GameStatus,
    board: Board
}

impl GameState {
    pub fn new() -> GameState {
        let mut s = GameState {
            turn: 0,
            players: ~[
                Player { name: ~"Player 1", position: (10, 10), direction: North, is_alive: true, get_action: |_| { North } },
                Player { name: ~"Player 2", position: (10, 20), direction: South, is_alive: true, get_action: |_| { West } }
            ],
            alive_count: 2,
            status: PlayerTurn(0),
            board: [[Empty, ..board_width], ..board_height]
        };
        // Place initial walls.
        for player_i in range(0, s.alive_count) {
            s.place_wall(player_i)
        };
        s
    }

    fn place_wall(&mut self, owner: PlayerIndex) {
        match self.players[owner].position {
            (r, c) => self.board[r][c] = PlayerWall(owner)
        }
    }

    fn can_move_to(&self, position: Vec) -> bool {
        match position {
            (row, column) => {
                0 <= row &&
                row < board_height &&
                0 <= column &&
                column < board_width &&
                self.board[row][column].is_passable()
            }
        }
    }

    fn player_after(&self, current: PlayerIndex) -> PlayerIndex {
        assert!(self.alive_count > 1);
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

    pub fn do_turn(&mut self) {
        let current = self.current_player();
        let direction = (self.players[current].get_action)(self);
        self.players[current].direction = direction;
        let new_position = direction.apply_to(self.players[current].position);

        if self.can_move_to(new_position) {
            self.players[current].position = new_position;
            self.place_wall(current);
        } else {
            self.players[current].is_alive = false;
            self.alive_count -= 1;
        }

        if self.alive_count == 1 {
            self.status = Won(current)
        } else {
            self.status = PlayerTurn(self.player_after(current))
        }

        self.turn += 1;
    }
}
