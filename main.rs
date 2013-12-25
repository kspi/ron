#[feature(globs)];

use game::*;
use behaviour::stupid_random::StupidRandom;
use behaviour::static_action::StaticAction;
use std::io::timer::sleep;

mod game;
mod behaviour {
    pub mod static_action;
    pub mod stupid_random;
}

fn direction_char(direction: Direction) -> ~str {
    match direction {
        North => ~"^",
        East => ~">",
        South => ~"v",
        West => ~"<"
    }
}

fn main() {
    let mut g = GameState::new(80, 30, ~[
        Player { name: ~"Player 1", position: (15, 30), direction: North, is_alive: true },
        Player { name: ~"Player 2", position: (15, 50), direction: South, is_alive: true }
    ]);

    let mut behaviours = ~[
        StupidRandom::new(5.0),
        StaticAction::new(MoveForward)
    ];

    while !g.status.is_over() {
        g.do_turn(behaviours);

        print("\x1b[2J\x1b[1;1H");
        for row in g.board.iter() {
            for tile in row.iter() {
                print(match *tile {
                    PlayerHead(p) => direction_char(g.players[p].direction),
                    PlayerWall(x) => format!("{:u}", x),
                    Crash => ~"X",
                    Empty => ~"."
                })
            }
            println("");
        }
        println(format!("Turn: {}, status: {}", g.turn, g.status.to_str()));
        sleep(100);
    }
}
