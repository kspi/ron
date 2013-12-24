use std::io::stdout;

mod game;

fn main() {
    let mut g = game::GameState::new();

    while !g.status.is_over() {
        g.do_turn();

        print("\x1b[2J\x1b[1;1H");
        stdout().flush();
        for row in g.board.iter() {
            for tile in row.iter() {
                print(match *tile {
                    game::PlayerWall(x) => fmt!("%u", x),
                    game::Empty => ~"."
                })
            }
            println("");
        }
        println(fmt!("Turn: %u, status: %?", g.turn, g.status))
    }
}
