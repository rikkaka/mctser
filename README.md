# mcts
An incrediblely easy-to-use library for Monte Carlo Tree Search.

All you need to do is to implement traits `mcts::GameState` and `mcts::Action` and mark traits `mcts::EndStatus` and `mcts::Action` for corresponding types in your game.
## Usage
Add the dependency to your Cargo.toml
```toml
mcts = { git = "https://github.com/rikkaka/mcts"}
```

To use the library, you should implement traits `mcts::GameState` and `mcts::Action` and mark traits `mcts::EndStatus` and `mcts::Action` for corresponding types in your game. 
```rust
/// The trait for the end status of the game, like player1 wins, player2 wins, or tie
pub trait EndStatus {}

/// The trait for the action. For example, in tictactoe, the action is the coordinate of the next move
pub trait Action: Eq + Clone {}

/// The trait for the player
pub trait Player<E: EndStatus> {
    fn reward_when_outcome_is(&self, outcome: &E) -> f32;
}

/// The trait for the game state
pub trait GameState<P, E, A>
where
    P: Player<E>,
    E: EndStatus,
    A: Action,
{
    /// To get the next player
    fn player(&self) -> P;
    /// Judge if the game is over; if not, return None; if true, return the status of the game result
    fn end_status(&self) -> Option<E>;
    /// Get all possible actions for the player at the current state
    fn possible_actions(&self) -> Vec<A>;
    /// Get the next state after the player takes the action
    fn act(&self, action: &A) -> Self;
}
```
Here is an example for tic tac toe. The implemention of the mod `game` is hiden. [Here](/examples/tictactoe.rs) to see the full example.
```rust, ignore
use game::{EndStatus, Player, Action, TictactoeGame};

impl mcts::EndStatus for EndStatus {}
impl mcts::Action for Action {}

impl mcts::Player<EndStatus> for Player {
    fn reward_when_outcome_is(&self, outcome: &EndStatus) -> f32 {
        match outcome {
            EndStatus::Win(winner) => {
                if self == winner {
                    1.
                } else {
                    0.
                }
            }
            EndStatus::Tie => 0.5,
        }
    }
}

impl mcts::GameState<Player, EndStatus, Action> for TictactoeGame {
    fn end_status(&self) -> Option<EndStatus> {
        self.end_status
    }

    fn player(&self) -> Player {
        return self.player;
    }

    fn possible_actions(&self) -> Vec<Action> {
        let mut possible_actions = Vec::new();
        for row in 0..3 {
            for col in 0..3 {
                if !self.occupied(Action(row, col)) {
                    possible_actions.push(Action(row, col));
                }
            }
        }
        possible_actions
    }

    fn act(&self, selection: &Action) -> Self {
        self.place(&selection).unwrap()
    }
}
```
Then you can build a `mcts::SearchTree` and start to search. To make search records reused in the next move, using `mcts::SearchTree::renew(&mut self, selected: &A)` to step forward.
```rust, ignore
fn main() {
    let mut game = Rc::new(TictactoeGame::new());
    let mut search_tree = mcts::SearchTree::new(game.clone());

    while game.end_status.is_none() {
        let selected = search_tree.search(1000).unwrap();
        search_tree.renew(&selected).unwrap();
        game = search_tree.get_game_state();
        game.draw_board();
    }
}
```