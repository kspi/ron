#[feature(globs)];

use std::io::stdout;
use game::*;

mod game;

fn main() {
    let mut g = GameState::new(~[
        Player { name: ~"Player 1", position: (10, 10), direction: North, is_alive: true, behaviour: ~GoNorth as ~PlayerBehaviour },
        Player { name: ~"Player 2", position: (10, 20), direction: South, is_alive: true, behaviour: ~GoNorth as ~PlayerBehaviour }
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
        println(format!("Turn: {}, status: {}", g.turn, g.status.to_str()))
    }
}
