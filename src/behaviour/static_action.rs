use game::{Action, GameState, Behaviour};
use std::owned::Box;

#[deriving(Show)]
pub struct StaticAction {
    action: Action
}

impl StaticAction {
    pub fn new(action: Action) -> Box<Behaviour> {
        box StaticAction {
            action: action
        } as Box<Behaviour>
    }
}

impl Behaviour for StaticAction {
    fn act(&mut self, _: &GameState) -> Action {
        self.action
    }
}
