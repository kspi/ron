#[feature(globs)];

extern mod extra;
extern mod ncurses;

use ncurses::*;
use game::*;
use behaviour::minimax::Minimax;
use std::io::Timer;
use std::os;

mod game;
mod behaviour {
    pub mod static_action;
    pub mod stupid_random;
    pub mod minimax;
}

fn direction_char(direction: Direction) -> ~str {
    match direction {
        North => ~"^",
        East => ~">",
        South => ~"v",
        West => ~"<"
    }
}

pub struct KeyboardControlled {
    port: Port<Direction>
}

impl KeyboardControlled {
    pub fn new(port: Port<Direction>) -> ~PlayerBehaviour {
        ~KeyboardControlled {
            port: port
        } as ~PlayerBehaviour
    }
}

impl PlayerBehaviour for KeyboardControlled {
    fn act(&mut self, game: &GameState) -> Action {
        match self.port.try_recv() {
            None => MoveForward,
            Some(direction) => (game.players[game.current_player()]
                                .direction.action_for(direction)
                                .unwrap_or(MoveForward))
        }
    }
}

fn getch_each(f: |i32|) {
    let mut key = getch();
    while key != ERR {
        f(key);
        key = getch();
    }
}

fn key_direction(key: i32) -> Option<Direction> {
    match key {
        KEY_UP => Some(North),
        107 => Some(North), // k
        KEY_RIGHT => Some(East),
        108 => Some(East), // l
        KEY_DOWN => Some(South),
        106 => Some(South), // j
        KEY_LEFT => Some(West),
        104 => Some(West), // h
        _ => None
    }
}

fn main() {
    let mut g = GameState::new(40, 20, ~[
        Player { name: ~"Player 1", position: (10, 12), direction: North, is_alive: true },
        Player { name: ~"Player 2", position: (10, 28), direction: South, is_alive: true }
    ]);

    let all_args = os::args();
    let options = all_args.slice(0, all_args.len());
    let keyboard_control = options.iter().any(|x| *x == ~"-k" || *x == ~"--keyword");

    let (dir_port, dir_chan) = Chan::new();

    let mut behaviours = if keyboard_control {
        ~[
            KeyboardControlled::new(dir_port),
            Minimax::new()
        ]
    } else {
        ~[
            Minimax::new(),
            Minimax::new()
        ]
    };


    // Curses init.
    initscr();
    raw();
    keypad(stdscr, true);
    noecho();
    timeout(0);
    curs_set(CURSOR_INVISIBLE);
    start_color();
    init_pair(1, COLOR_RED, COLOR_BLACK);
    init_pair(2, COLOR_CYAN, COLOR_BLACK);

    let mut timer = Timer::new().unwrap();
    let sleeper = timer.periodic(100);

    while !g.status.is_over() {
        getch_each(|key| {
            if key == 113 { // q
                endwin();
                return;
            }
            if (keyboard_control) {
                key_direction(key).map(|dir| {
                    dir_chan.send(dir);
                });
            }
        });

        g.do_turn(behaviours);

        move(0, 0);
        for row in g.board.iter() {
            for tile in row.iter() {
                match *tile {
                    PlayerHead(p) => {
                        attron(A_BOLD());
                        attron(COLOR_PAIR(p as i16 + 1));
                        printw(direction_char(g.players[p].direction));
                        attroff(COLOR_PAIR(p as i16 + 1));
                        attroff(A_BOLD());
                    }
                    PlayerWall(x) => {
                        attron(COLOR_PAIR(x as i16 + 1));
                        printw("#");
                        attroff(COLOR_PAIR(x as i16 + 1));
                    }
                    Crash => { printw("X"); }
                    Empty => { printw("."); }
                }
            }
            printw("\n");
        }
        printw(format!("Turn: {}, status: {}\n", g.turn, g.status.to_str()));
        refresh();
        sleeper.recv();
    }

    timeout(-1);
    printw("Press any key to exit.");
    getch();

    endwin();

    print(format!("Turn: {}, status: {}\n", g.turn, g.status.to_str()));
}
