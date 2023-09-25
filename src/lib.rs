use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    rc::Rc,
};

pub trait Player<E: EndStatus> {
    fn reward_if_outcome_is(&self, outcome: &E) -> f32;
}

pub trait EndStatus {}

pub trait Selection: Eq + Clone {}

pub trait GameState<P, E, S>
where
    P: Player<E>,
    E: EndStatus,
    S: Selection,
{
    /// To get the next player
    fn player(&self) -> P;
    /// Judge if the game is end; if not, return None; if true, return the status of the game result
    fn end_status(&self) -> Option<E>;

    fn selections(&self) -> Vec<S>;
    fn select(&self, selection: &S) -> Self;
}

pub struct SearchTree<P, G, E, S>
where
    P: Player<E>,
    G: GameState<P, E, S>,
    E: EndStatus,
    S: Selection,
{
    root_node: Rc<RefCell<Node<P, G, E, S>>>,
}

pub struct Node<P, G, E, S>
where
    P: Player<E>,
    G: GameState<P, E, S>,
    E: EndStatus,
    S: Selection,
{
    state: Rc<G>,
    last_selection: Option<S>,
    child_nodes: RefCell<Vec<Rc<RefCell<Node<P, G, E, S>>>>>,

    /// times of win
    wi: Cell<f32>,
    /// times of selection
    ni: Cell<f32>,

    /// policy used to select the child node; the three parameters are wi, ni, and np, which is ni of parent node
    selection_policy: Rc<dyn Fn(f32, f32, f32) -> f32>,
}

impl<P, G, E, S> SearchTree<P, G, E, S>
where
    P: Player<E>,
    G: GameState<P, E, S>,
    E: EndStatus,
    S: Selection,
{
    pub fn new(state: Rc<G>) -> Self {
        SearchTree {
            root_node: Rc::new(RefCell::new(Node::new(state, Rc::new(uct)))),
        }
    }

    pub fn with_selection_policy(
        self,
        selection_policy: impl Fn(f32, f32, f32) -> f32 + 'static,
    ) -> Self {
        let mut root_node_borrow = self.root_node.borrow_mut();
        root_node_borrow.selection_policy = Rc::new(selection_policy);
        drop(root_node_borrow);
        self
    }

    pub fn search(&self, n: u32) -> Option<S> {
        let root_node = self.root_node.borrow();
        for _ in 0..n {
            root_node.simulate(&root_node.state.player());
        }
        let selected_node = root_node.select_most_visited();
        selected_node
            .map(|v| v.borrow().last_selection.clone())
            .flatten()
    }

    pub fn renew(&mut self, selected: &S) -> Result<(), String> {
        let root_node = self.root_node.borrow_mut();
        root_node.expand();
        drop(root_node);

        let root_node = self.root_node.borrow();
        let new_root_node = root_node.find_child(selected);

        drop(root_node);

        if let Some(node) = new_root_node {
            self.root_node = node;
            return Ok(());
        }
        Err("The state is not a child of the root node".to_string())
    }

    pub fn get_game_state(&self) -> Rc<G> {
        self.root_node.borrow().state.clone()
    }

    pub fn root_node(&self) -> Rc<RefCell<Node<P, G, E, S>>> {
        self.root_node.clone()
    }
}

impl<P, G, E, S> Debug for Node<P, G, E, S>
where
    P: Player<E>,
    G: GameState<P, E, S> + Debug,
    E: EndStatus + Debug,
    S: Selection,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("state", &self.state)
            .field("wi", &self.wi)
            .field("ni", &self.ni)
            .finish()
    }
}

impl<P, G, E, S> Node<P, G, E, S>
where
    P: Player<E>,
    G: GameState<P, E, S>,
    E: EndStatus,
    S: Selection,
{
    fn new(state: Rc<G>, selection_policy: Rc<dyn Fn(f32, f32, f32) -> f32>) -> Self {
        Node {
            state,
            last_selection: None,
            child_nodes: RefCell::new(vec![]),
            wi: Cell::new(0.),
            ni: Cell::new(0.),
            selection_policy,
        }
    }

    fn derive_child(&self, selection: S) -> Rc<RefCell<Node<P, G, E, S>>> {
        Rc::new(RefCell::new(Node {
            state: Rc::new(self.state.select(&selection)),
            last_selection: Some(selection),
            child_nodes: RefCell::new(vec![]),
            wi: Cell::new(0.),
            ni: Cell::new(0.),
            selection_policy: self.selection_policy.clone(),
        }))
    }

    fn find_child(&self, selection: &S) -> Option<Rc<RefCell<Node<P, G, E, S>>>> {
        for node in self.child_nodes.borrow().iter() {
            if node.borrow().last_selection == Some(selection.clone()) {
                return Some(node.clone());
            }
        }
        None
    }

    fn select(&self) -> Option<Rc<RefCell<Node<P, G, E, S>>>> {
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
                (self.selection_policy)(node_borrow.wi.get(), node_borrow.ni.get(), self.ni.get());
            if value > max_value {
                max_value = value;
                selected_node = Some(node.clone());
            }
        }

        selected_node
    }

    fn select_most_visited(&self) -> Option<Rc<RefCell<Node<P, G, E, S>>>> {
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
        for selections in self.state.selections().iter() {
            self.child_nodes
                .borrow_mut()
                .push(self.derive_child(selections.clone()));
        }
    }

    fn is_expanded(&self) -> bool {
        self.child_nodes.borrow().len() > 0
    }

    fn backpropagate(&self, player: &P, outcome: &E) {
        self.ni.set(self.ni.get() + 1.);
        self.wi
            .set(self.wi.get() + player.reward_if_outcome_is(outcome));
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

    pub fn child_nodes(&self) -> Vec<Rc<RefCell<Node<P, G, E, S>>>> {
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
