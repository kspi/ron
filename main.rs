#[feature(globs)];

use std::io::stdout;
use game::*;
use behaviour::stupid_random::*;
use std::io::timer::sleep;

mod game;
mod behaviour {
    pub mod stupid_random;
}

fn main() {
    let mut g = GameState::new(~[
        Player { name: ~"Player 1", position: (20, 30), direction: North, is_alive: true, behaviour: StupidRandom::new(5.0) },
        Player { name: ~"Player 2", position: (20, 50), direction: South, is_alive: true, behaviour: StupidRandom::new(1.0) }
    ]);

    while !g.status.is_over() {
        g.do_turn();

        print("\x1b[2J\x1b[1;1H");
        stdout().flush();
        for row in g.board.iter() {
            for tile in row.iter() {
                print(match *tile {
                    PlayerWall(x) => format!("{:u}", x),
                    Empty => ~"."
                })
            }
            println("");
        }
        println(format!("Turn: {}, status: {}", g.turn, g.status.to_str()));
        sleep(100);
    }
}
