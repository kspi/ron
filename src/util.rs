use std::rand::random;
use game::{Position, GameState};
use game::{North, East, South, West};

pub fn random_bernoulli(p: f64) -> bool {
    let x: f64 = random();
    x < p
}

pub fn flood_count(position: Position, game: &GameState) -> uint {
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

