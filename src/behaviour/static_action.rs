use game::{Action, Behaviour};

pub fn static_action(action: Action) -> Behaviour {
    Behaviour::make(proc(state_receiver, action_sender) {
        loop {
            let game = state_receiver.recv();
            if game.is_over() {
                debug!("Game is over, quitting.");
                break;
            };
            debug!("Sending action {}", action);
            action_sender.send((game.turn, action));
        }
    })
}
