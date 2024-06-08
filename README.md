# mctser
An incrediblely easy-to-use library for Monte Carlo Tree Search.

All you need to do is to implement four required traits in this library for corresponding types in your game.
## Usage
Add the dependency to your Cargo.toml
```sh
cargo add mctser
```

To use this library, two traits `mctser::GameState` and `mctser::Action`, and two marking traits `mctser::EndStatus` and `mctser::Action` need to be implemented for corresponding types in your game. The definations of the four traits are as follows.
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
Here is an example for tic tac toe. Some details of the example are hiden and [click here](/examples/tictactoe.rs) to see the full example. Clone this repository and `cargo run --example tictactoe` to see the game playing between two `mctser` bots.

To use this crate, four types are needed:
1. A type representing the status of the game. For tic tac toe, it would be the situation on the borad, the player of next move, if the game ends and who wins the game when it ends.
2. A type representing the players. For tic tac toe, we can use an `enum` to representing the two players.
3. A type representing possible actions. For tic tac toe, it would be the coordination of a move.
4. A type representing the status of end of the game. For tic tac toe, it would be player1 win, player2 win, or tie.

For these types, we have four corresponding traits in this crate, namely `GameState`, `Player`, `Action` and `EndStatus`, which you need to implement for your types.

As a start, we can define the four needed types as follows:
```rust
#[derive(Clone, Copy)]
pub enum EndStatus {
    Win(Player),
    Tie,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    Player0,
    Player1,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Action(pub usize, pub usize);

pub struct TictactoeGame {
    pub board: [[Option<Player>; 3]; 3],
    pub player: Player,
    pub end_status: Option<EndStatus>,
}
```

To make the game playable, we need to implement a few necessary methods. [Here](/examples/tictactoe.rs#L61) to see the detailed implementation. Then we can implement the corresponding traits for these types:
```rust, ignore
impl mctser::EndStatus for EndStatus {}
impl mctser::Action for Action {}

impl mctser::Player<EndStatus> for Player {
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

impl mctser::GameState<Player, EndStatus, Action> for TictactoeGame {
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

Then you can build a `mcts::SearchTree` and start to search. To make search records of preceding searches reused in the next move, using `mcts::SearchTree::renew` to step forward.
```rust, ignore
fn main() {
    // We use `RC` to store the game status. Create a new game and pass it to the search tree through `RC::clone()`.
    let mut game = Rc::new(TictactoeGame::new());
    let mut search_tree = mctser::SearchTree::new(game.clone());

    while game.end_status.is_none() {
        // Make 1000 simulations to find the best move
        let selected = search_tree.search(1000).unwrap();
        // Step forward to the next state using the action provided by the search tree
        search_tree.renew(&selected).unwrap();
        // Get current game state after the move
        game = search_tree.get_game_state();
        game.draw_board();
    }
}
```

The usage of this library is quite easy, isn't it?

## Todo
- [ ] Add test cases
- [ ] Support custom tree policy
- [ ] Add parallel search

## Contribution
All kind of contributions are welcome. Feel free to open an issue or a pull request.