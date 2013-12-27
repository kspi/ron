use game::*;
use behaviour::static_action::StaticAction;
use std::f64;
use std::vec;
use std::rand::random;
use std::cmp::{min, max};
use extra::time::precise_time_ns;

#[deriving(ToStr)]
pub struct Minimax;

static TARGET_ACT_TIME : u64 = 50000000;

impl Minimax {
    pub fn new() -> ~PlayerBehaviour {
        ~Minimax as ~PlayerBehaviour
    }
}

fn flood_count(position: Position, game: &GameState) -> uint {
    let mut flooded = vec::from_elem(game.board_height, vec::from_elem(game.board_width, false));
    let mut count = 0u;
    fn fill(pos: Position, game: &GameState, flooded: &mut ~[~[bool]], count: &mut uint) {
        let (r, c) = pos;
        if game.can_move_to(pos) && !flooded[r][c] {
            flooded[r][c] = true;
            *count += 1;
            fill(North.apply_to(pos), game, flooded, count);
            fill(East.apply_to(pos), game, flooded, count);
            fill(South.apply_to(pos), game, flooded, count);
            fill(West.apply_to(pos), game, flooded, count);
        }
    };
    fill(North.apply_to(position), game, &mut flooded, &mut count);
    fill(East.apply_to(position), game, &mut flooded, &mut count);
    fill(South.apply_to(position), game, &mut flooded, &mut count);
    fill(West.apply_to(position), game, &mut flooded, &mut count);
    count
}

fn random_bernoulli(p: f64) -> bool {
    let x: f64 = random();
    x < p
}

fn game_apply_action(game: &GameState, action: Action) -> ~GameState {
    let mut new_game = game.clone();
    new_game.do_turn(vec::from_elem(game.players.len(), action).map(|a| StaticAction::new(*a)));
    ~new_game
}

fn explore_probability(depth: uint, falloff: f64) -> f64 {
    let d = depth as f64;
    if d <= falloff {
        1.0
    } else {
        1.0 - 1.0 / (1.0
                     + f64::exp(falloff * 2.0
                                - f64::pow(d, 1.0/1.1)))
    }
}

fn position_distance(a: Position, b: Position) -> int {
    let (ar, ac) = a;
    let (br, bc) = b;
    max((ar - br).abs(), (ac - bc).abs())
}

static ACTIONS: [Action, ..3] = [MoveForward, TurnLeft, TurnRight];

fn minimax(player: PlayerIndex, game: &GameState, depth: uint, minimize: bool, start_time: u64) -> f64 {
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
        if (our_space > their_space) {
            return 100.0 * our_space;
        } else if (our_space < their_space) {
            return -1000.0 / our_space;
        } else {
            return our_space / position_distance(our_pos, their_pos) as f64;
        }
    }
    let init = if minimize { f64::INFINITY } else { -f64::INFINITY };
    let foldfn = if minimize { min } else { max };
    ACTIONS.iter().map(|action| {
        let new_game = game_apply_action(game, *action);
        minimax(player, new_game, depth + 1, !minimize, start_time)
    }).fold(init, foldfn)
}

impl PlayerBehaviour for Minimax {
    fn act(&mut self, game: &GameState) -> Action {
        let player_index = game.current_player();

        let best_action = *ACTIONS.iter().max_by(|action| {
            let new_game = game_apply_action(game, **action);
            minimax(player_index, new_game, 0, true, precise_time_ns())
        }).unwrap();
        best_action
    }
}
