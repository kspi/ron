use game::*;
use behaviour::static_action::StaticAction;
use std::f64;
use std::vec;
use std::rand::random;

#[deriving(ToStr)]
pub struct Minimax;

impl Minimax {
    pub fn new() -> ~PlayerBehaviour {
        ~Minimax as ~PlayerBehaviour
    }
}

static ALL_ACTIONS: [Action, ..3] = [MoveForward, TurnLeft, TurnRight];

fn random_bernoulli(p: f64) -> bool {
    let x: f64 = random();
    x < p
}

static EXPLORE_DEPTH_FALLOFF: f64 = 4.0;
fn explore_probability(depth: uint) -> f64 {
    if depth <= 1 {
        1.0
    } else {
        1.0 - 1.0 / (1.0 + f64::exp(-(depth as f64 - EXPLORE_DEPTH_FALLOFF) / EXPLORE_DEPTH_FALLOFF))
    }
}

fn minimize(player: PlayerIndex, game: &GameState, depth: uint) -> (Action, f64) {
    if game.status.is_over() {
        if game.winner() == player {
            return (MoveForward, 1.0);
        } else {
            return (MoveForward, -1.0);
        }
    }
    if depth >= 20 || !random_bernoulli(explore_probability(depth)) {
        return (MoveForward, 0.0);
    }
    let mut m = f64::INFINITY;
    let mut maction = MoveForward;
    for action in ALL_ACTIONS.iter() {
        let mut new_game = game.clone();
        new_game.do_turn(vec::from_elem(game.players.len(), *action).map(|a| StaticAction::new(*a)));
        let (_, result) = maximize(player, &new_game, depth + 1);
        if result < m {
            m = result;
            maction = *action;
        }
    }
    (maction, m)
}

fn maximize(player: PlayerIndex, game: &GameState, depth: uint) -> (Action, f64) {
    if game.status.is_over() {
        if game.winner() == player {
            return (MoveForward, 1.0);
        } else {
            return (MoveForward, -1.0);
        }
    }
    if depth >= 20 || !random_bernoulli(explore_probability(depth)) {
        return (MoveForward, 0.0);
    }
    let mut m = -f64::INFINITY;
    let mut maction = MoveForward;
    for action in ALL_ACTIONS.iter() {
        let mut new_game = game.clone();
        new_game.do_turn(vec::from_elem(game.players.len(), *action).map(|a| StaticAction::new(*a)));
        let (_, result) = minimize(player, &new_game, depth + 1);
        if result > m {
            m = result;
            maction = *action;
        }
    }
    (maction, m)
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

        let (action, result) = maximize(game.current_player(), game, 0);
        action
    }
}
