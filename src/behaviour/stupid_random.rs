use game::{Action, MoveForward, TurnLeft, TurnRight, Behaviour, GameState};
use util::random_bernoulli;

fn random_turn() -> Action {
    if random_bernoulli(0.5) {
        TurnLeft
    } else {
        TurnRight
    }
}

pub fn stupid_random(stability: f64) -> Behaviour {
    Behaviour::make(proc(state_receiver, action_sender) {
        let mut turns_since_change: uint = 0;

        let act = |game: &GameState| {
            let player_index = game.current_player();
            let player = &game.players[player_index];
            let forward_pos = player.direction.apply_to(player.position);
            let left_pos = player.direction.left().apply_to(player.position);
            let right_pos = player.direction.right().apply_to(player.position);
            let forward_free = game.can_move_to(forward_pos);
            let left_free = game.can_move_to(left_pos);
            let right_free = game.can_move_to(right_pos);
            let change_probability = 1f64 - (-(turns_since_change as f64) / stability).exp();

            if forward_free && ((!left_free && !right_free) || !random_bernoulli(change_probability)) {
                turns_since_change += 1;
                MoveForward
            } else {
                turns_since_change = 0;
                if !left_free {
                    TurnRight
                } else if !right_free {
                    TurnLeft
                } else {
                    random_turn()
                }
            }
        };

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
