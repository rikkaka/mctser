#![doc = include_str!("../README.md")]

use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    rc::Rc,
};

/// The trait for the end status of the game.
/// Like player1 wins, player2 wins, or tie
pub trait EndStatus {}

/// The trait for the action.
/// For example, in tictactoe, the action is the coordinate of the next move
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

type RcNode<P, G, E, A> = Rc<RefCell<Node<P, G, E, A>>>;

pub struct SearchTree<P, G, E, A>
where
    P: Player<E>,
    G: GameState<P, E, A>,
    E: EndStatus,
    A: Action,
{
    root_node: RcNode<P, G, E, A>,
}

pub struct Node<P, G, E, A>
where
    P: Player<E>,
    G: GameState<P, E, A>,
    E: EndStatus,
    A: Action,
{
    state: Rc<G>,
    last_action: Option<A>,
    child_nodes: RefCell<Vec<RcNode<P, G, E, A>>>,

    /// times of win
    wi: Cell<f32>,
    /// times of selection
    ni: Cell<f32>,

    /// policy used to select the child node; the three parameters are wi, ni, and np, which is ni of parent node
    tree_policy: Rc<dyn Fn(f32, f32, f32) -> f32>,
}

impl<P, G, E, A> SearchTree<P, G, E, A>
where
    P: Player<E>,
    G: GameState<P, E, A>,
    E: EndStatus,
    A: Action,
{
    /// Create a new search tree
    pub fn new(game_state: Rc<G>) -> Self {
        SearchTree {
            root_node: Rc::new(RefCell::new(Node::new(game_state, Rc::new(uct)))),
        }
    }

    /// Set the tree policy
    pub fn with_tree_policy(self, tree_policy: impl Fn(f32, f32, f32) -> f32 + 'static) -> Self {
        let mut root_node_borrow = self.root_node.borrow_mut();
        root_node_borrow.tree_policy = Rc::new(tree_policy);
        drop(root_node_borrow);
        self
    }

    /// Search for the best action
    pub fn search(&self, n: u32) -> Option<A> {
        let root_node = self.root_node.borrow();
        for _ in 0..n {
            root_node.simulate(&root_node.state.player());
        }
        let selected_node = root_node.select_most_visited();
        selected_node.and_then(|v| v.borrow().last_action.clone())
    }

    /// Renew the root node
    pub fn renew(&mut self, action: &A) -> Result<(), String> {
        let root_node = self.root_node.borrow_mut();
        root_node.expand();
        drop(root_node);

        let root_node = self.root_node.borrow();
        let new_root_node = root_node.find_child(action);

        drop(root_node);

        if let Some(node) = new_root_node {
            self.root_node = node;
            return Ok(());
        }
        Err("The state is not a child of the root node".to_string())
    }

    /// Get the current game state
    pub fn get_game_state(&self) -> Rc<G> {
        self.root_node.borrow().state.clone()
    }

    /// Get the root node
    pub fn root_node(&self) -> RcNode<P, G, E, A> {
        self.root_node.clone()
    }
}

impl<P, G, E, A> Debug for Node<P, G, E, A>
where
    P: Player<E>,
    G: GameState<P, E, A> + Debug,
    E: EndStatus + Debug,
    A: Action,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("state", &self.state)
            .field("wi", &self.wi)
            .field("ni", &self.ni)
            .finish()
    }
}

impl<P, G, E, A> Node<P, G, E, A>
where
    P: Player<E>,
    G: GameState<P, E, A>,
    E: EndStatus,
    A: Action,
{
    fn new(state: Rc<G>, tree_policy: Rc<dyn Fn(f32, f32, f32) -> f32>) -> Self {
        Node {
            state,
            last_action: None,
            child_nodes: RefCell::new(vec![]),
            wi: Cell::new(0.),
            ni: Cell::new(0.),
            tree_policy,
        }
    }

    fn derive_child(&self, action: A) -> RcNode<P, G, E, A> {
        Rc::new(RefCell::new(Node {
            state: Rc::new(self.state.act(&action)),
            last_action: Some(action),
            child_nodes: RefCell::new(vec![]),
            wi: Cell::new(0.),
            ni: Cell::new(0.),
            tree_policy: self.tree_policy.clone(),
        }))
    }

    fn find_child(&self, action: &A) -> Option<RcNode<P, G, E, A>> {
        for node in self.child_nodes.borrow().iter() {
            if node.borrow().last_action == Some(action.clone()) {
                return Some(node.clone());
            }
        }
        None
    }

    fn select(&self) -> Option<RcNode<P, G, E, A>> {
        for node in self.child_nodes.borrow().iter() {
            if node.borrow().ni.get() == 0. {
                return Some(node.clone());
            }
        }

        let mut max_value = f32::MIN;
        let mut selected_node = None;
        for node in self.child_nodes.borrow().iter() {
            let node_borrow = node.borrow();
            let value =
                (self.tree_policy)(node_borrow.wi.get(), node_borrow.ni.get(), self.ni.get());
            if value > max_value {
                max_value = value;
                selected_node = Some(node.clone());
            }
        }

        selected_node
    }

    fn select_most_visited(&self) -> Option<RcNode<P, G, E, A>> {
        let mut times_visted_max = f32::MIN;
        let mut selected_node = None;
        for node in self.child_nodes.borrow().iter() {
            let node_borrow = node.borrow();
            let times_visted = node_borrow.ni.get();
            if times_visted > times_visted_max {
                times_visted_max = times_visted;
                selected_node = Some(node.clone());
            }
        }

        selected_node
    }

    fn expand(&self) {
        if self.is_expanded() {
            return;
        }
        for action in self.state.possible_actions().iter() {
            self.child_nodes
                .borrow_mut()
                .push(self.derive_child(action.clone()));
        }
    }

    fn is_expanded(&self) -> bool {
        self.child_nodes.borrow().len() > 0
    }

    fn backpropagate(&self, player: &P, outcome: &E) {
        self.ni.set(self.ni.get() + 1.);
        self.wi
            .set(self.wi.get() + player.reward_when_outcome_is(outcome));
    }

    fn simulate(&self, player: &P) -> E {
        match self.state.end_status() {
            Some(outcome) => {
                self.backpropagate(player, &outcome);
                outcome
            }
            None => {
                self.expand();
                let selected_node = self.select().unwrap();
                let selected_node = selected_node.borrow_mut();
                let outcome = selected_node.simulate(&self.state.player());
                self.backpropagate(player, &outcome);
                outcome
            }
        }
    }

    pub fn state(&self) -> Rc<G> {
        self.state.clone()
    }

    pub fn child_nodes(&self) -> Vec<RcNode<P, G, E, A>> {
        self.child_nodes.borrow().clone()
    }

    pub fn wi(&self) -> f32 {
        self.wi.get()
    }

    pub fn ni(&self) -> f32 {
        self.ni.get()
    }
}

fn uct(wi: f32, ni: f32, np: f32) -> f32 {
    wi / ni + 2_f32.sqrt() * (np.ln() / ni).sqrt()
}
