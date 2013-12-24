use std::io::stdout;

use game::*;

mod game;

fn main() {
    let mut g = GameState::new(~[
        Player { name: ~"Player 1", position: (10, 10), direction: North, is_alive: true, get_action: |_| { North } },
        Player { name: ~"Player 2", position: (10, 20), direction: South, is_alive: true, get_action: |_| { West } }
    ]);

    while !g.status.is_over() {
        g.do_turn();

        print("\x1b[2J\x1b[1;1H");
        stdout().flush();
        for row in g.board.iter() {
            for tile in row.iter() {
                print(match *tile {
                    PlayerWall(x) => fmt!("%u", x),
                    Empty => ~"."
                })
            }
            println("");
        }
        println(fmt!("Turn: %u, status: %?", g.turn, g.status))
    }
}
