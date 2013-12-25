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
    if depth <= 1 {
        1.0
    } else {
        1.0 - 1.0 / (1.0 + f64::exp(-(depth as f64 - falloff) / falloff))
    }
}

fn minimax(player: PlayerIndex, game: &GameState, depth: uint, minimize: bool, start_time: u64) -> f64 {
    if game.status.is_over() {
        if game.winner() == player {
            return f64::INFINITY;
        } else {
            return -f64::INFINITY;
        }
    }
    let time_progress = ((precise_time_ns() - start_time) as f64) / (TARGET_ACT_TIME as f64);
    if time_progress > 0.9 || !random_bernoulli(explore_probability(depth, (1.0 - time_progress) * 4.0)) {
        let our_pos = game.players[player].position;
        let other_player = game.player_after(player);
        let their_pos = game.players[other_player].position;
        return (flood_count(our_pos, game) as f64) - (flood_count(their_pos, game) as f64) / 2.0;
    }
    let init = if minimize { f64::INFINITY } else { -f64::INFINITY };
    let foldfn = if minimize { min } else { max };
    let actions = ~[MoveForward, TurnLeft, TurnRight];
    actions.iter().map(|action| {
        let new_game = game_apply_action(game, *action);
        minimax(player, new_game, depth + 1, !minimize, start_time)
    }).fold(init, foldfn)
}

impl PlayerBehaviour for Minimax {
    fn act(&mut self, game: &GameState) -> Action {
        let player_index = game.current_player();
        let player = &game.players[player_index];
        let forward_pos = player.direction.apply_to(player.position);
        let left_pos = player.direction.left().apply_to(player.position);
        let right_pos = player.direction.right().apply_to(player.position);
        let forward_free = game.can_move_to(forward_pos);
        let left_free = game.can_move_to(left_pos);
        let right_free = game.can_move_to(right_pos);

        if forward_free && !left_free && !right_free {
            return MoveForward;
        } else if !forward_free && !left_free {
            return TurnRight;
        } else if !forward_free && !right_free {
            return TurnLeft;
        }

        let actions = if random_bernoulli(0.9) {
            ~[MoveForward, TurnLeft, TurnRight]
        } else if random_bernoulli(0.5) {
            ~[TurnLeft, TurnRight, MoveForward]
        } else {
            ~[TurnRight, TurnLeft, MoveForward]
        };
        let (action, _) = actions.iter().fold(
            (MoveForward, -f64::INFINITY),
            |(maxa, maxf), action| {
                let new_game = game_apply_action(game, *action);
                let f = minimax(player_index, new_game, 0, true, precise_time_ns());
                if f > maxf {
                    (*action, f)
                } else {
                    (maxa, maxf)
                }
            }
        );
        action
    }
}
