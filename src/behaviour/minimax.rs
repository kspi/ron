use game::{Action, MoveForward, TurnLeft, TurnRight, Behaviour, GameState, Position, PlayerIndex};
use std::f64;
use time::precise_time_ns;
use std::cmp::max;
use util::{random_bernoulli, flood_count};

static TARGET_ACT_TIME : u64 = 50000000;

fn explore_probability(depth: uint, falloff: f64) -> f64 {
    let d = depth as f64;
    if d <= falloff {
        1.0
    } else {
        1.0 - 1.0 / (1.0
                     + (falloff * 2.0 - d.powf(1.0/1.1)).exp())
    }
}

fn position_distance(a: Position, b: Position) -> int {
    let (ar, ac) = a;
    let (br, bc) = b;
    max((ar as int - br as int).abs(), (ac as int - bc as int).abs())
}

static ACTIONS: [Action, ..3] = [MoveForward, TurnLeft, TurnRight];

fn explore(player: PlayerIndex, game: &GameState, depth: uint, minimize: bool, start_time: u64) -> f64 {
    if game.status.is_over() {
        if game.winner() == player {
            return f64::INFINITY;
        } else {
            return -f64::INFINITY;
        }
    }
    let time_progress = ((precise_time_ns() - start_time) as f64) / (TARGET_ACT_TIME as f64);
    if time_progress > 0.9 || !random_bernoulli(explore_probability(depth, (1.0 - time_progress) * 9.0)) {
        let our_pos = game.players[player].position;
        let other_player = game.player_after(player);
        let their_pos = game.players[other_player].position;
        let our_space = flood_count(our_pos, game) as f64;
        let their_space = flood_count(their_pos, game) as f64;
        if our_space > their_space {
            return 100.0 * our_space;
        } else if our_space < their_space {
            return -1000.0 / our_space;
        } else {
            return our_space / position_distance(our_pos, their_pos) as f64;
        }
    }
    let init = if minimize { f64::INFINITY } else { -f64::INFINITY };
    let foldfn = if minimize {
        |a: f64, b: f64| if a < b { a } else { b }
    } else {
        |a: f64, b: f64| if a > b { a } else { b }
    };
    ACTIONS.iter().map(|action| {
        let new_game = game.apply_action(*action);
        explore(player, &new_game, depth + 1, !minimize, start_time)
    }).fold(init, foldfn)
}

fn act(game: &GameState) -> Action {
    let player_index = game.current_player();

    let (best_action, _) = ACTIONS.iter().map(|action| {
        let new_game = game.apply_action(*action);
        let score = explore(player_index, &new_game, 0, true, precise_time_ns());
        (*action, score)
    }).fold((MoveForward, -f64::INFINITY), |(xa, xs), (ya, ys)|
        if xs > ys {
            (xa, xs)
        } else {
            (ya, ys)
        }
    );

    best_action
}

pub fn minimax() -> Behaviour {
    Behaviour::make(proc(state_receiver, action_sender) {
        loop {
            let game = state_receiver.recv();
            if game.is_over() {
                debug!("Game is over, quitting.");
                break;
            };
            let action = act(&game);
            debug!("Sending action {}", action);
            action_sender.send((game.turn, action));
        }
    })
}

