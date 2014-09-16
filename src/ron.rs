extern crate ncurses;
extern crate time;

use behaviour::minimax_memory::MinimaxMemory;
use game::{Direction, North, East, South, West};
use game::{Action, MoveForward};
use game::{GameState, Player, PlayerBehaviour, PlayerTurn, PlayerHead, PlayerWall, Crash, Empty};
use std::owned::Box;
use std::io::Timer;
use std::time::Duration;
use std::io::stdio::print;
use std::os;
use std::comm::{channel, Receiver};

pub mod game;
pub mod behaviour {
    pub mod static_action;
    pub mod stupid_random;
    pub mod minimax;
    pub mod minimax_memory;
}

fn direction_str(direction: Direction) -> &'static str {
    match direction {
        North => "^",
        East => ">",
        South => "v",
        West => "<"
    }
}

pub struct KeyboardControlled {
    port: Receiver<Direction>
}

impl KeyboardControlled {
    pub fn new(port: Receiver<Direction>) -> Box<PlayerBehaviour> {
        box KeyboardControlled {
            port: port
        } as Box<PlayerBehaviour>
    }
}

impl PlayerBehaviour for KeyboardControlled {
    fn act(&mut self, game: &GameState) -> Action {
        match self.port.try_recv() {
            Ok(direction) => (game.players[game.current_player()]
                              .direction.action_for(direction)
                              .unwrap_or(MoveForward)),
            _ => MoveForward
        }
    }
}

fn getch_each(f: |i32|) {
    let mut key = ncurses::getch();
    while key != ncurses::ERR {
        f(key);
        key = ncurses::getch();
    }
}

fn key_direction(key: i32) -> Option<Direction> {
    match key {
        ncurses::KEY_UP => Some(North),
        107 => Some(North), // k
        ncurses::KEY_RIGHT => Some(East),
        108 => Some(East), // l
        ncurses::KEY_DOWN => Some(South),
        106 => Some(South), // j
        ncurses::KEY_LEFT => Some(West),
        104 => Some(West), // h
        _ => None
    }
}

fn main() {
    let mut game = GameState::new(40, 20, vec!(
        Player { name: "Player 1".to_string(), position: (10, 12), direction: North, is_alive: true },
        Player { name: "Player 2".to_string(), position: (10, 28), direction: South, is_alive: true }
    ));

    let all_args = os::args();
    let options = all_args.slice(0, all_args.len());
    let keyboard_control = options.iter().any(|x| *x == "-k".to_string() || *x == "--keyword".to_string());

    let (sender, receiver) = channel();

    let mut behaviours = if keyboard_control {
        vec!(
            KeyboardControlled::new(receiver),
            MinimaxMemory::new()
        )
    } else {
        vec!(
            MinimaxMemory::new(),
            MinimaxMemory::new()
        )
    };


    // Curses init.
    ncurses::initscr();
    ncurses::raw();
    ncurses::keypad(ncurses::stdscr, true);
    ncurses::noecho();
    ncurses::timeout(0);
    ncurses::curs_set(ncurses::CURSOR_INVISIBLE);
    ncurses::start_color();
    ncurses::init_pair(1, ncurses::COLOR_RED, ncurses::COLOR_BLACK);
    ncurses::init_pair(2, ncurses::COLOR_CYAN, ncurses::COLOR_BLACK);

    let mut timer = Timer::new().unwrap();
    let sleeper = timer.periodic(Duration::milliseconds(100));

    let mut quit = false;
    while !game.status.is_over() && !quit {
        getch_each(|key| {
            if key == 113 { // q
                quit = true;
            }
            if keyboard_control {
                key_direction(key).map(|dir| {
                    sender.send(dir);
                });
            }
        });

        game.do_turn(behaviours.as_mut_slice());

        if game.status.is_over() || game.status == PlayerTurn(0) {
            ncurses::move(0, 0);
            for row in game.board.iter() {
                for tile in row.iter() {
                    match *tile {
                        PlayerHead(p) => {
                            ncurses::attron(ncurses::A_BOLD());
                            ncurses::attron(ncurses::COLOR_PAIR(p as i16 + 1));
                            ncurses::printw(direction_str(game.players[p].direction));
                            ncurses::attroff(ncurses::COLOR_PAIR(p as i16 + 1));
                            ncurses::attroff(ncurses::A_BOLD());
                        }
                        PlayerWall(x) => {
                            ncurses::attron(ncurses::COLOR_PAIR(x as i16 + 1));
                            ncurses::printw("#");
                            ncurses::attroff(ncurses::COLOR_PAIR(x as i16 + 1));
                        }
                        Crash => { ncurses::printw("X"); }
                        Empty => { ncurses::printw("."); }
                    }
                }
                ncurses::printw("\n");
            }
            ncurses::printw(format!("Turn: {}, status: {}\n", game.turn, game.status).as_slice());
            ncurses::refresh();
        }
        sleeper.recv();
    }

    ncurses::timeout(-1);
    ncurses::printw("Press any key to exit.");
    ncurses::getch();

    ncurses::endwin();

    print(format!("Turn: {}, status: {}\n", game.turn, game.status).as_slice());
}
