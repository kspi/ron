use game::*;

#[deriving(ToStr)]
pub struct StaticAction {
    action: Action
}

impl StaticAction {
    pub fn new(action: Action) -> ~PlayerBehaviour {
        ~StaticAction {
            action: action
        } as ~PlayerBehaviour
    }
}

impl PlayerBehaviour for StaticAction {
    fn act(&mut self, _: &GameState) -> Action {
        self.action
    }
}
