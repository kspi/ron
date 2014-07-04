use game::{Action, GameState, PlayerBehaviour};
use std::owned::Box;

#[deriving(Show)]
pub struct StaticAction {
    action: Action
}

impl StaticAction {
    pub fn new(action: Action) -> Box<PlayerBehaviour> {
        box StaticAction {
            action: action
        } as Box<PlayerBehaviour>
    }
}

impl PlayerBehaviour for StaticAction {
    fn act(&mut self, _: &GameState) -> Action {
        self.action
    }
}
