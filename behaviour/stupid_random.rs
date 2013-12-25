use game::*;
use std::f64::exp;
use std::rand::random;

#[deriving(ToStr)]
pub struct StupidRandom {
    stability: f64,
    turns_since_change: int
}

impl StupidRandom {
    pub fn new(stability: f64) -> ~PlayerBehaviour {
        ~StupidRandom {
            stability: stability,
            turns_since_change: 0
        } as ~PlayerBehaviour
    }
}

fn random_bernoulli(p: f64) -> bool {
    let x: f64 = random();
    x < p
}

fn random_turn() -> Action {
    if random_bernoulli(0.5) {
        TurnLeft
    } else {
        TurnRight
    }
}

impl PlayerBehaviour for StupidRandom {
    fn act(&mut self, game: &GameState) -> Action {
        let player_index = game.current_player();
        let player = &game.players[player_index];
        let forward_pos = player.direction.apply_to(player.position);
        let left_pos = player.direction.left().apply_to(player.position);
        let right_pos = player.direction.right().apply_to(player.position);
        let change_probability = 1f64 - exp(-(self.turns_since_change as f64) / self.stability);

        if !game.can_move_to(forward_pos) || random_bernoulli(change_probability){
            self.turns_since_change = 0;
            if !game.can_move_to(left_pos) {
                TurnRight
            } else if !game.can_move_to(right_pos) {
                TurnLeft
            } else {
                random_turn()
            }
        } else {
            self.turns_since_change += 1;
            MoveForward
        }
    }
}
