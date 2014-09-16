extern crate ncurses;

use game::{Action, MoveForward, TurnLeft, TurnRight, PlayerBehaviour, GameState, Position, PlayerIndex};
use game::{North, East, South, West};
use behaviour::static_action::StaticAction;
use std::vec::Vec;
use std::owned::Box;
use std::cmp::max;
use std::rand::random;
use time::precise_time_ns;

fn flood_count(position: Position, game: &GameState) -> uint {
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

fn game_apply_action(game: &GameState, action: Action) -> GameState {
    let mut new_game = game.clone();
    let mut behaviours = Vec::from_fn(game.players.len(), |_| StaticAction::new(action));
    new_game.do_turn(behaviours.as_mut_slice());
    new_game
}

fn position_distance(a: Position, b: Position) -> int {
    let (ar, ac) = a;
    let (br, bc) = b;
    max((ar as int - br as int).abs(), (ac as int - bc as int).abs())
}

fn state_score(game: &GameState, player: PlayerIndex) -> f64 {
    let mut score: f64 = 0.0;

    if game.is_over() {
        if game.winner() == player {
            score += 100.0;
        } else {
            score -= 100.0;
        }
    }

    let our_pos = game.players[player].position;
    let other_player = game.player_after(player);
    let their_pos = game.players[other_player].position;
    let board_size = (game.board_width * game.board_height) as f64;
    let our_space = flood_count(our_pos, game) as f64 / board_size;
    let their_space = flood_count(their_pos, game) as f64 / board_size;
    if our_space > their_space {
        score += 1.0 + our_space;
    } else if our_space < their_space {
        score -= 1.0 / our_space
    } else {
        score += our_space / position_distance(our_pos, their_pos) as f64;
    }

    score
}

#[deriving(Show, Clone)]
struct GameNode {
    action: Action,
    player: PlayerIndex, // Player that is taking the action.
    game: GameState, // Game state after the action
    score: f64, // Score of the game state.
    children: Vec<GameNode> // Explored sub-GameNodes.
}

static ACTIONS: [Action, ..3] = [MoveForward, TurnLeft, TurnRight];

static TARGET_ACT_TIME : u64 = 50000000;

fn random_bernoulli(p: f64) -> bool {
    let x: f64 = random();
    x < p
}

fn explore_tree(game: &GameState, tree: &mut Vec<GameNode>, start_time: u64, depth: uint) {
    if game.is_over() {
        return;
    }

    if tree.is_empty() {
        *tree = ACTIONS.iter().map(|&action| {
            GameNode::new(game, action)
        }).collect();
    }

    let time_progress = ((precise_time_ns() - start_time) as f64) / (TARGET_ACT_TIME as f64);
    if time_progress < 1.0 && random_bernoulli(1.3f64.powf(-time_progress * (depth as f64))) {
        for node in tree.mut_iter() {
            explore_tree(&node.game, &mut node.children, start_time, depth + 1);
        }
    } else {
        debug!("Stopping exploration at depth {}", depth);
    }
}

impl GameNode {
    pub fn new(game: &GameState, action: Action) -> GameNode {
        let new_game = game_apply_action(game, action);
        let player = game.current_player();
        GameNode {
            action: action,
            player: player,
            score: state_score(&new_game, player),
            game: new_game,
            children: vec!()
        }
    }
}


#[deriving(Show)]
pub struct MinimaxMemory {
    tree: Vec<GameNode>,
    chosen_tree: Vec<GameNode>
}

impl MinimaxMemory {
    pub fn new() -> Box<PlayerBehaviour> {
        box MinimaxMemory { 
            tree: vec!(),
            chosen_tree: vec!()
        } as Box<PlayerBehaviour>
    }
}

impl PlayerBehaviour for MinimaxMemory {
    fn act(&mut self, game: &GameState) -> Action {
        if !self.chosen_tree.is_empty() {
            for node in self.chosen_tree.iter() {
                if node.game == *game {
                    self.tree = node.children.clone();
                    break
                }
            };
        }

        let start_time = precise_time_ns();
        explore_tree(game, &mut self.tree, start_time, 0);

        let tree = self.tree.clone();
        let best_node = tree.iter().fold(&tree[0], |a, b|
            if a.score > b.score {
                a
            } else {
                b
            }
        );

        self.chosen_tree = best_node.children.clone();
        self.tree.clear();
                
        best_node.action
    }
}
