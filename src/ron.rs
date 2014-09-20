#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate ncurses;
extern crate time;

use game::{Direction, North, East, South, West, MoveForward};
use game::{GameState, Player, Behaviour, PlayerTurn, PlayerHead, PlayerWall, Crash, Empty};
use std::io::Timer;
use std::time::Duration;
use std::io::stdio::print;
use std::os;
use std::comm::{channel, Receiver, Select};

pub mod game;
pub mod util;
pub mod behaviour {
    pub mod static_action;
    pub mod stupid_random;
    pub mod minimax;
//    pub mod minimax_memory;
}

static FRAME_DELAY_MS: i64 = 1000;

fn direction_str(direction: Direction) -> &'static str {
    match direction {
        North => "^",
        East => ">",
        South => "v",
        West => "<"
    }
}

fn keyboard_controlled(direction_receiver: Receiver<Direction>) -> Behaviour {
    Behaviour::make(proc(state_receiver, action_sender) {
        loop {
            let game = state_receiver.recv();
            if game.is_over() {
                debug!("Game is over, quitting.");
                break;
            };
            
            let mut timer = Timer::new().unwrap();
            let timeout = timer.oneshot(Duration::milliseconds(FRAME_DELAY_MS - 20));
            let action = select! {
                direction = direction_receiver.recv() =>
                    (game.players[game.current_player()]
                     .direction.action_for(direction)
                     .unwrap_or(MoveForward)),
                () = timeout.recv() => MoveForward
            };

            debug!("Sending action {}", action);
            action_sender.send((game.turn, action));
        }
    })
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
    let mut game = GameState::new(40, 20, vec![
        Player { name: "Player 1".to_string(), position: (10, 12), direction: North, is_alive: true },
        Player { name: "Player 2".to_string(), position: (10, 28), direction: South, is_alive: true }
    ]);

    let all_args = os::args();
    let options = all_args.slice(0, all_args.len());
    let keyboard_control = options.iter().any(|x| *x == "-k".to_string() );

    let (direction_sender, direction_receiver) = channel::<Direction>();

    let behaviours = if keyboard_control {
        vec![
            keyboard_controlled(direction_receiver),
            behaviour::minimax::minimax()
        ]
    } else {
        vec![
            behaviour::minimax::minimax(),
            behaviour::minimax::minimax()
        ]
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


    let mut quit = false;
    while !game.status.is_over() && !quit {
        getch_each(|key| {
            if key == 113 { // q
                quit = true;
            }
            if keyboard_control {
                key_direction(key).map(|dir| {
                    direction_sender.send(dir);
                });
            }
        });

        debug!("Turn {}, player {}", game.turn, game.current_player());

        {
            let ref behaviour = behaviours[game.current_player()];
            behaviour.send_state(&game);
            let mut timer = Timer::new().unwrap();
            let timeout = timer.oneshot(Duration::milliseconds(FRAME_DELAY_MS));
            let mut action = MoveForward;
            let mut action_set = false;

            let select = Select::new();
            let mut timeout_handle  = select.handle(&timeout);
            let mut behaviour_handle = select.handle(&behaviour.receiver);
            unsafe {
                timeout_handle.add();
                behaviour_handle.add();
            }
            loop {
                let id = select.wait();
                if id == timeout_handle.id() {
                    if !action_set {
                        warn!("Turn {}, player {}: action was not set fast enough.", game.turn, game.current_player());
                    }
                    break;
                } else if id == behaviour_handle.id() {
                    let (turn, a) = behaviour.receiver.recv();
                    if turn == game.turn {
                        action = a;
                        action_set = true;
                    } else {
                        warn!("Turn {}: received action for wrong turn {} from player {}.", game.turn, turn, game.current_player());
                    };
                };
            };
            game.do_turn(action);
        };

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
    }

    ncurses::timeout(-1);
    ncurses::printw("Press any key to exit.");
    ncurses::getch();

    ncurses::endwin();

    print(format!("Turn: {}, status: {}\n", game.turn, game.status).as_slice());
}
