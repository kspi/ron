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

fn random_turn_for(direction: Direction) -> Direction {
    let x: f64 = random();
    if x < 0.5 {
        direction.left()
    } else {
        direction.right()
    }
}

impl PlayerBehaviour for StupidRandom {
    fn decide_direction(&mut self, g: &GameState) -> Direction {
        let change_probability = 1f64 - exp(-(self.turns_since_change as f64) / self.stability);
        let x: f64 = random();
        if x < change_probability {
            self.turns_since_change = 0;
            random_turn_for(g.players[g.current_player()].direction)
        } else {
            self.turns_since_change += 1;
            g.players[g.current_player()].direction
        }
    }
}
