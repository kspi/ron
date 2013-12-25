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

#[deriving(ToStr)]
pub struct KeyboardControlled {
    maybe_direction: Option<Direction>
}

impl KeyboardControlled {
    pub fn new(d: Option<Direction>) -> ~PlayerBehaviour {
        ~KeyboardControlled {
            maybe_direction: d
        } as ~PlayerBehaviour
    }
}

impl PlayerBehaviour for KeyboardControlled {
    fn act(&mut self, game: &GameState) -> Action {
        match self.maybe_direction {
            None => MoveForward,
            Some(direction) => {
                (game.players[game.current_player()]
                                .direction.action_for(direction)
                                .unwrap_or(MoveForward))
            }
        }
    }
}

fn main() {
    let mut g = GameState::new(40, 20, ~[
        Player { name: ~"Player 1", position: (10, 12), direction: North, is_alive: true },
        Player { name: ~"Player 2", position: (10, 28), direction: South, is_alive: true }
    ]);

    let mut behaviours = ~[
        Minimax::new(),
        Minimax::new()
    ];

    let all_args = os::args();
    let options = all_args.slice(0, all_args.len());
    let keyboard_control = options.iter().any(|x| *x == ~"-k" || *x == ~"--keyword");

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

    let mut key_dir: Option<Direction> = None;

    let mut timer = Timer::new().unwrap();
    let sleeper = timer.periodic(100);

    while !g.status.is_over() {
        move(0, 0);

        {
            let mut key = getch();
            while key != ERR {
                if key == 113 { // q
                    endwin();
                    return;
                }
                key_direction(key).map(|dir| { key_dir = Some(dir); });
                key = getch();
            }
        }

        if (keyboard_control) {
            behaviours[0] = KeyboardControlled::new(key_dir);
        }

        g.do_turn(behaviours);

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
