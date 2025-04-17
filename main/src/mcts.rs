use crate::fastgame;
use crate::game;
use rand::Rng;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::time::{Instant, Duration};
use std::collections::HashSet;

// Restructured node implementation

struct Node {
    // Functionnality variables
    game_state: [u32;4],
    parent_index: Option<usize>,
    visit_count: usize,
    is_terminal: bool,
    children_indices: Vec<usize>,
    specific_information: TypeInfo,
    // Additional information for score/display
    move_number: usize,
}

enum TypeInfo {
    Spawn(SpawnInfo),
    Move(MoveInfo),
}

struct SpawnInfo {
    move_made: game::Direction,
    block_spawns_left: Vec<((usize,usize),u32)>,
    total_value: f32,
}

struct MoveInfo {
    actions_left: Vec<game::Direction>,
    probability: f32,
}

use std::cell::RefCell;
pub struct MonteCarloTree2 {
    nodes: RefCell<Vec<Node>>,
}

impl MonteCarloTree2 {
    pub fn new(fast: &fastgame::FastGame, root_state:[u32;4], starting_move_number: usize) -> Self {
        let possible_directions = fast.get_possible_directions(&root_state);
        let rootnode = Node{
            game_state: root_state,
            parent_index: None,
            visit_count: 0,
            is_terminal: possible_directions.is_empty(),
            children_indices: Vec::new(),
            specific_information: TypeInfo::Move(MoveInfo { actions_left: possible_directions, probability: 1.0 }),
            move_number: starting_move_number,
        };
        Self { nodes: RefCell::new(vec![rootnode])}
    }

    fn selection(&self, node_index: usize) -> usize {
        let node = &self.nodes.borrow()[node_index];
        let children_scores:Vec<f32> = Vec::with_capacity(node.children_indices.len());
        match &node.specific_information {
            TypeInfo::Spawn(spawn_info) => {
                if !spawn_info.block_spawns_left.is_empty() {
                    return node_index;
                }
                // Todo: add calculation of children scores (probability/visitcount)  
            },
            TypeInfo::Move(move_info) => {
                if !move_info.actions_left.is_empty() {
                    return node_index;
                }
                // Todo: add calculation of children scores (uct value)
            }
        };
        let child_index_index: usize = children_scores
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .unwrap();
        return self.selection(node.children_indices[child_index_index]);
    }

    fn expansion(&self, node_index: usize) -> usize {
        let node = &self.nodes.borrow()[node_index];
        match &node.specific_information {
            TypeInfo::Spawn(spawn_info) => {

            },
            TypeInfo::Move(move_info) => {

            },
        }


        return 0;
    }
}


const EXPLORATION_CONSTANT:f32 = 0.5;

#[derive(Clone,Debug)]
pub enum NodeType {
    Spawn(SpawnNode),
    Move(MoveNode),
}
#[derive(Clone,Debug)]
pub struct SpawnNode {
    game_state: [u32;4],
    parent_index: Option<usize>,
    children_indices_blockexponent_and_isexplored: Vec<(usize,u32,bool)>,
    move_made: game::Direction,
    visit_count: f32,
    total_value: f32,
    is_terminal: bool,
    move_number: f32,
}
#[derive(Clone,Debug)]
pub struct MoveNode {
    game_state: [u32;4],
    parent_index: Option<usize>,
    children_indices: Vec<usize>,
    actions_left: Vec<game::Direction>,
    visit_count: f32,
    is_terminal: bool,
    move_number: f32,
}

pub struct MonteCarloTree {
    node_vec: Vec<Box<NodeType>>,
}

impl MonteCarloTree {
    pub fn new(fast: &fastgame::FastGame, rootstate:[u32;4], starting_move: f32) -> MonteCarloTree {
        let rootnode = NodeType::Move(MoveNode{
            game_state: rootstate,
            parent_index: None,
            children_indices: Vec::new(),
            actions_left: fast.get_possible_directions(&rootstate),
            visit_count: 0.0,
            is_terminal: fast.is_lost(&rootstate),
            move_number: starting_move,
        });
        MonteCarloTree { node_vec: vec![Box::new(rootnode)]}
    }
    // ----------- Selection function ----------
    #[time_graph::instrument]
    fn select_recursive(&mut self, node_index: usize, rng: &mut SmallRng) -> usize {
        match self.node_vec[node_index].as_ref() {
            NodeType::Spawn(spawn_node) => {
                let mut unexplored_children_vec = spawn_node.children_indices_blockexponent_and_isexplored.clone();
                unexplored_children_vec.retain(|(_, _, isexplored)| !isexplored);
                if spawn_node.is_terminal || spawn_node.children_indices_blockexponent_and_isexplored.is_empty() || !unexplored_children_vec.is_empty() {
                    return node_index;
                }
                let mut children_vec = spawn_node.children_indices_blockexponent_and_isexplored.clone();
                let random_exponent = if rng.random_bool(0.9) { 1 } else { 2 };
                children_vec.retain(|(_, exponent, _)| *exponent == random_exponent);
                let random_index = rng.random_range(0..children_vec.len());
                return self.select_recursive(children_vec[random_index].0, rng);
            },
            NodeType::Move(move_node) => {
                if move_node.is_terminal || !move_node.actions_left.is_empty() || move_node.children_indices.is_empty() {
                    return node_index;
                }
                let mut best_children_list_index = 0;
                let mut best_uct = f32::NEG_INFINITY;
                for (i, child_index) in move_node.children_indices.iter().enumerate() {
                    let uct_value = match self.node_vec[*child_index].as_ref() {
                        NodeType::Spawn(spawn_node) => {
                            (spawn_node.total_value / spawn_node.visit_count) + EXPLORATION_CONSTANT * (move_node.visit_count.ln() / spawn_node.visit_count).sqrt()
                        }
                        _ => unreachable!("Move is a child of move"),
                    };
                    if uct_value > best_uct {
                        best_uct = uct_value;
                        best_children_list_index = i;
                    }
                }
                return self.select_recursive(move_node.children_indices[best_children_list_index], rng);
            },
        }
    }

    // ----------- Expand function ----------
    #[time_graph::instrument]
    fn expand(&mut self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> usize {
        let chosen_node_index;
        let mut new_children = Vec::new();
        let node_vec_len = self.node_vec.len();
        match self.node_vec[node_index].as_mut() {
            NodeType::Spawn(ref mut spawn_node) => {
                if spawn_node.is_terminal {return node_index;}
                if spawn_node.children_indices_blockexponent_and_isexplored.is_empty() {
                    // Add all possible block spawns if the node has no children
                    for exponent in [1u32,2u32].iter() {
                        let empty_space_vec = fastgame::FastGame::empty_list(&spawn_node.game_state);
                        if empty_space_vec.is_empty() {
                            return node_index;
                        }
                        for empty_space in empty_space_vec {
                        let new_state = fast.place_block(spawn_node.game_state, empty_space, *exponent);
                        let new_child = NodeType::Move(MoveNode {
                            game_state: new_state,
                            parent_index: Some(node_index),
                            children_indices: Vec::new(),
                            actions_left: fast.get_possible_directions(&new_state),
                            visit_count: 0.0,
                            is_terminal: fast.is_lost(&new_state),
                            move_number: spawn_node.move_number,
                        });
                        let new_child_index = node_vec_len + new_children.len();
                        new_children.push(new_child);
                        spawn_node.children_indices_blockexponent_and_isexplored.push((new_child_index,*exponent,false));
                        }
                    }
                }
                // Choose a random unexplored child
                let mut random_child_vec = spawn_node.children_indices_blockexponent_and_isexplored.clone();
                random_child_vec.retain(|(_, _, isexplored)| !isexplored);
                let exponents: HashSet<u32> = random_child_vec.iter().map(|(_, exp, _)| *exp).collect();
                let random_exponent = if exponents.len() == 2 {
                    if rng.random_bool(0.9) { 1 } else { 2 }
                } else {
                    *exponents.iter().next().expect("There are no exponents")
                };
                random_child_vec.retain(|(_, exponent, _)| *exponent == random_exponent);
                let random_index = rng.random_range(0..random_child_vec.len());
                chosen_node_index = random_child_vec[random_index].0;
                // Change the status of the random child as explored
                let child_entry = spawn_node.children_indices_blockexponent_and_isexplored
                    .iter_mut()
                    .find(|(index, _ , _)| *index == chosen_node_index)
                    .expect("Chosen child not found");
                child_entry.2 = true;
            },
            NodeType::Move(ref mut move_node) => {
                // Add the child following the first direction in actions_left
                if move_node.is_terminal {
                    return node_index;
                }
                let random_direction_index = rng.random_range(0..move_node.actions_left.len());
                let direction_taken = move_node.actions_left[random_direction_index].clone();
                let (state, _) = fast.make_move(&move_node.game_state, &direction_taken);
                let new_child = NodeType::Spawn(SpawnNode {
                    game_state: state,
                    parent_index: Some(node_index),
                    children_indices_blockexponent_and_isexplored: Vec::new(),
                    move_made: direction_taken.clone(),
                    visit_count: 0.0,
                    total_value: 0.0,
                    is_terminal: fast.is_lost(&state),
                    move_number: move_node.move_number+1.0,
                });
                new_children.push(new_child);
                move_node.children_indices.push(node_vec_len);
                move_node.actions_left.retain(|direction| direction != &direction_taken);
                chosen_node_index = node_vec_len;
            },
        }
        for new_child in new_children.iter() {
            self.node_vec.push(Box::new(new_child.clone()));
        }
        return chosen_node_index;
    }

    // ----------- Rollout anb policy functions ----------
    #[time_graph::instrument]
    fn greedy_policy(fast: &fastgame::FastGame, grid: [u32;4]) -> [u32;4] {
        if fast.is_lost(&grid) {
            return grid; 
        }
        let (new_grid, _) = fast.get_possible_directions(&grid)
            .iter()
            .map(|direction| {
                let (new_grid, move_score) = fast.make_move(&grid, &direction);
                (new_grid, move_score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or((grid,0)).clone();
        return new_grid;
    }
    #[time_graph::instrument]
    fn greedy_rollout_move_number(fast: &fastgame::FastGame, node: &mut NodeType, rng: &mut SmallRng) -> f32 {
        let (mut game_state, starting_move_number) = match node {
            NodeType::Spawn(spawn_node) => (spawn_node.game_state,spawn_node.move_number),
            NodeType::Move(move_node) => (move_node.game_state,move_node.move_number),
        };
        let mut move_number = starting_move_number;
        while !fast.is_lost(&game_state) {
            game_state = Self::greedy_policy(&fast, game_state);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
            move_number += 1.0;
        }
        return move_number * (*fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32);
    }
    #[time_graph::instrument]
    fn greedy_rollout_best_tile(fast: &fastgame::FastGame, node: &mut NodeType, rng: &mut SmallRng) -> f32 {
        let mut game_state = match node {
            NodeType::Spawn(spawn_node) => spawn_node.game_state,
            NodeType::Move(move_node) => move_node.game_state,
        };
        while !fast.is_lost(&game_state) {
            game_state = Self::greedy_policy(&fast, game_state);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
             let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
        }
        return *fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32;
    }
    #[time_graph::instrument]
    fn random_move_policy(fast: &fastgame::FastGame, grid: [u32;4], rng: &mut SmallRng) -> [u32;4] {
        if fast.is_lost(&grid) {
            return grid; 
        }
        let possible_directions = fast.get_possible_directions(&grid);
        let random_direction_index = rng.random_range(0..possible_directions.len());
        let direction = &possible_directions[random_direction_index];
        let (new_grid,_) = fast.make_move(&grid, &direction);
        return new_grid;
    }
    #[time_graph::instrument]
    fn random_rollout_move_number(fast: &fastgame::FastGame, node: &mut NodeType, rng: &mut SmallRng) -> f32 {
        let (mut game_state, starting_move_number) = match node {
            NodeType::Spawn(spawn_node) => (spawn_node.game_state,spawn_node.move_number),
            NodeType::Move(move_node) => (move_node.game_state,move_node.move_number),
        };
        let mut move_number = starting_move_number;
        while !fast.is_lost(&game_state) {
            game_state = Self::random_move_policy(&fast, game_state, rng);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
            move_number += 1.0;
        }
        return move_number * *fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32;
    }
    #[time_graph::instrument]
    fn random_rollout_move_number2(fast: &fastgame::FastGame, node: &mut NodeType, rng: &mut SmallRng) -> f32 {
        let (mut game_state, starting_move_number) = match node {
            NodeType::Spawn(spawn_node) => (spawn_node.game_state,spawn_node.move_number),
            NodeType::Move(move_node) => (move_node.game_state,move_node.move_number),
        };
        let mut move_number = starting_move_number;
        let mut possible_directions;
        loop {
            possible_directions = fast.get_possible_directions(&game_state);
            let direction_number = possible_directions.len();
            if direction_number == 0 {break};
            game_state = {
                let random_direction_index = rng.random_range(0..direction_number);
                fast.make_move(&game_state, &possible_directions[random_direction_index]).0
            };
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
            move_number += 1.0;
        }
        return move_number * *fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32;
    }

    // ----------- Backpropagation function ----------
    #[time_graph::instrument]
    fn backpropagate_recursive(&mut self, node_index: usize, score: f32, root_move_number: f32) {
        match self.node_vec[node_index].as_mut() {
            NodeType::Spawn(ref mut spawn_node) => {
                spawn_node.total_value += score  / (root_move_number+1.0);
                spawn_node.visit_count += 1.0;
                if let Some(parent_index) = spawn_node.parent_index {
                    self.backpropagate_recursive(parent_index, score, root_move_number);
                } else {
                    return;
                }
            }
            NodeType::Move(ref mut move_node) => {
                move_node.visit_count += 1.0;
                if let Some(parent_index) = move_node.parent_index {
                    self.backpropagate_recursive(parent_index, score, root_move_number);
                } else {
                    return;
                }
            }
        }
    }


    // ----------- Main function ----------
    #[time_graph::instrument]
    pub fn get_best_direction(&mut self, fast: &fastgame::FastGame, time_limit: f32, iteration_limit: usize, root_move_number: f32) -> (game::Direction,usize) {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let time_limit = Duration::from_secs_f32(time_limit);
        let start_time = std::time::Instant::now();
        let mut iteration_count = 0;
        while Instant::now() - start_time < time_limit && iteration_count < iteration_limit {
            let selected_node_index = self.select_recursive(0, &mut rng);
            let chosen_node_index = self.expand(&fast, selected_node_index, &mut rng);
            let rollout_score = Self::random_rollout_move_number2(&fast, self.node_vec[chosen_node_index].as_mut(), &mut rng);
            self.backpropagate_recursive(chosen_node_index, rollout_score, root_move_number);
            iteration_count += 1;
        }
        let node_number = self.node_vec.len();
        return (match self.node_vec[0].as_ref() {
            NodeType::Spawn(_) => unreachable!("Root is a spawn node"),
            NodeType::Move(move_node) => {
                move_node.children_indices.iter()
                    .map(|child_index| {
                        let child = self.node_vec[*child_index].as_ref();
                        match child {
                            NodeType::Spawn(spawn_node) => {
                                (spawn_node.total_value, spawn_node.move_made.clone())
                            },
                            _ => unreachable!(),
                        }
                    })
                    .max_by(|a,b| a.0.partial_cmp(&b.0).expect("Could not order moves"))
                    .map(|(_, direction)| direction)
                    .expect("Could not choose best direction")
            }
        },
        node_number)
    }
}