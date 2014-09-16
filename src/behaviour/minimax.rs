use game::{Action, MoveForward, TurnLeft, TurnRight, PlayerBehaviour, GameState, Position, PlayerIndex};
use game::{North, East, South, West};
use behaviour::static_action::StaticAction;
use std::f64;
use std::vec::Vec;
use std::rand::random;
use std::owned::Box;
use time::precise_time_ns;
use std::cmp::max;

#[deriving(Show)]
pub struct Minimax;

static TARGET_ACT_TIME : u64 = 50000000;

impl Minimax {
    pub fn new() -> Box<PlayerBehaviour> {
        box Minimax as Box<PlayerBehaviour>
    }
}

fn flood_count(position: Position, game: &GameState) -> uint {
    let mut flooded = Vec::from_elem(game.board_height, Vec::from_elem(game.board_width, false));
    let mut count = 0u;
    fn fill(pos: Position, game: &GameState, flooded: &mut Vec<Vec<bool>>, count: &mut uint) {
        let (r, c) = pos;
        if game.can_move_to(pos) && !(*flooded)[r][c] {
            flooded.get_mut(r).grow_set(c, &false, true);
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

fn game_apply_action(game: &GameState, action: Action) -> GameState {
    let mut new_game = game.clone();
    let mut behaviours = Vec::from_fn(game.players.len(), |_| StaticAction::new(action));
    new_game.do_turn(behaviours.as_mut_slice());
    new_game
}

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
        let new_game = game_apply_action(game, *action);
        minimax(player, &new_game, depth + 1, !minimize, start_time)
    }).fold(init, foldfn)
}

impl PlayerBehaviour for Minimax {
    fn act(&mut self, game: &GameState) -> Action {
        let player_index = game.current_player();

        let (best_action, _) = ACTIONS.iter().map(|action| {
            let new_game = game_apply_action(game, *action);
            let score = minimax(player_index, &new_game, 0, true, precise_time_ns());
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
}
