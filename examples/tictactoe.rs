use std::rc::Rc;

use game::{EndStatus, Player, Action, TictactoeGame};

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
        self.player
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
        self.place(selection).unwrap()
    }
}

fn main() {
    let mut game = Rc::new(TictactoeGame::new());
    let mut search_tree = mctser::SearchTree::new(game.clone());

    while game.end_status.is_none() {
        let selected = search_tree.search(1000).unwrap();
        search_tree.renew(&selected).unwrap();
        game = search_tree.get_game_state();
        game.draw_board();
    }
}

mod game {
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

    impl Player {
        pub fn next(&self) -> Player {
            match self {
                Player::Player0 => Player::Player1,
                Player::Player1 => Player::Player0,
            }
        }
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct Action(pub usize, pub usize);

    pub struct TictactoeGame {
        pub board: [[Option<Player>; 3]; 3],
        pub player: Player,
        pub end_status: Option<EndStatus>,
    }

    impl TictactoeGame {
        pub fn new() -> Self {
            Self {
                board: [[None; 3]; 3],
                player: Player::Player0,
                end_status: None,
            }
        }

        fn check_end_status(&mut self) {
            for row in 0..3 {
                if self.board[row][0].is_some()
                    && self.board[row][0] == self.board[row][1]
                    && self.board[row][1] == self.board[row][2]
                {
                    let winner = self.board[row][0].unwrap();
                    self.end_status = Some(EndStatus::Win(winner));
                    return;
                }
            }

            for col in 0..3 {
                if self.board[0][col].is_some()
                    && self.board[0][col] == self.board[1][col]
                    && self.board[1][col] == self.board[2][col]
                {
                    let winner = self.board[0][col].unwrap();
                    self.end_status = Some(EndStatus::Win(winner));
                    return;
                }
            }

            if self.board[1][1].is_some()
                && ((self.board[0][0] == self.board[1][1] && self.board[1][1] == self.board[2][2])
                    || (self.board[0][2] == self.board[1][1]
                        && self.board[1][1] == self.board[2][0]))
            {
                let winner = self.board[1][1].unwrap();
                self.end_status = Some(EndStatus::Win(winner));
                return;
            }

            if self.board.iter().flatten().all(|p| p.is_some()) {
                self.end_status = Some(EndStatus::Tie);
            } else {
                self.end_status = None;
            }
        }

        pub fn occupied(&self, selection: Action) -> bool {
            let Action(row, col) = selection;
            self.board[row][col].is_some()
        }

        pub fn place(&self, selection: &Action) -> Result<TictactoeGame, &str> {
            let Action(row, col) = *selection;
            match self.board[row][col] {
                Some(_) => Err("invalid place"),
                None => {
                    let mut board = self.board;
                    board[row][col] = Some(self.player);
                    let mut game = TictactoeGame {
                        board,
                        player: self.player.next(),
                        end_status: None,
                    };
                    game.check_end_status();
                    Ok(game)
                }
            }
        }

        pub fn draw_board(&self) {
            println!("\u{250C}\u{2500}\u{2500}\u{2500}\u{2510}");
            for row in 0..3 {
                print!("\u{2502}");
                for col in 0..3 {
                    match self.board[row][col] {
                        Some(Player::Player0) => print!("O"),
                        Some(Player::Player1) => print!("X"),
                        None => print!("-"),
                    }
                }
                println!("\u{2502}");
            }
            println!("\u{2514}\u{2500}\u{2500}\u{2500}\u{2518}");
        }
    }
}
