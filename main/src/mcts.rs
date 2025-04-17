use crate::fastgame;
use crate::game;
use rand::Rng;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::seq::SliceRandom;
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
    score: u32,
}

enum TypeInfo {
    Spawn(SpawnInfo),
    Move(MoveInfo),
}

struct SpawnInfo {
    move_made: game::Direction,
    two_block_spawns_left: Vec<(usize,usize)>,
    four_block_spawns_left: Vec<(usize,usize)>,
    total_value: f32,
}

struct MoveInfo {
    actions_left: Vec<game::Direction>,
    probability: f32,
}

use std::cell::RefCell;
pub struct MonteCarloTree {
    nodes: RefCell<Vec<Node>>,
}

const EXPLORATION_CONSTANT:f32 = 100000.0;

impl MonteCarloTree {
    #[time_graph::instrument]
    pub fn new(fast: &fastgame::FastGame, root_state:[u32;4], starting_move_number: usize, starting_score: u32) -> Self {
        let possible_directions = fast.get_possible_directions(&root_state);
        let rootnode = Node{
            game_state: root_state,
            parent_index: None,
            visit_count: 0,
            is_terminal: possible_directions.is_empty(),
            children_indices: Vec::new(),
            specific_information: TypeInfo::Move(MoveInfo { actions_left: possible_directions, probability: 1.0 }),
            move_number: starting_move_number,
            score: starting_score,
        };
        Self { nodes: RefCell::new(vec![rootnode])}
    }

    fn get_info(&self) {
        println!("Nodes : {}", self.nodes.borrow().len());
    }

    #[time_graph::instrument]
    fn selection(&self, node_index: usize, rng: &mut SmallRng) -> usize {
        let node = &self.nodes.borrow()[node_index];
        let mut children_scores:Vec<f32> = Vec::with_capacity(node.children_indices.len());
        match &node.specific_information {
            TypeInfo::Spawn(spawn_info) => {
                if node.is_terminal || (!spawn_info.two_block_spawns_left.is_empty() && !spawn_info.four_block_spawns_left.is_empty()) || node.children_indices.is_empty() {
                    return node_index;
                }
                // Calculation of children scores (probability/visitcount)
                for child_index in &node.children_indices {
                    let child = &self.nodes.borrow()[*child_index];
                    if child.visit_count == 0 {
                        panic!("Spawn's child visitcount is 0");
                    }
                    match &child.specific_information {
                        TypeInfo::Move(move_info) => {children_scores.push(move_info.probability/child.visit_count as f32)},
                        TypeInfo::Spawn(_) => unreachable!("Spawn is a child of spawn"),
                    }
                }
            },
            TypeInfo::Move(move_info) => {
                if node.is_terminal || !move_info.actions_left.is_empty() {
                    return node_index;
                }
                // Calculation of children scores (uct value)
                for child_index in &node.children_indices {
                    let child = &self.nodes.borrow()[*child_index];
                    if child.visit_count == 0 {
                        panic!("Move's child visitcount is 0");
                    }
                    match &child.specific_information {
                        TypeInfo::Spawn(spawn_info) => {children_scores.push(
                            (spawn_info.total_value / child.visit_count as f32) 
                            + EXPLORATION_CONSTANT * ((node.visit_count as f32).ln() / child.visit_count as f32).sqrt()
                            )},
                        TypeInfo::Move(_) => unreachable!("Move is a child of move"),
                    }
                }
            }
        };
        let highest_score_children = children_scores.iter()
            .enumerate()
            .fold(
                (Vec::new(), f32::NEG_INFINITY),
                |(mut indices, current_max), (i, &val)| {
                    if val > current_max {
                        (vec![i], val) // New max: reset indices
                    } else if val == current_max {
                        indices.push(i);
                        (indices, current_max) // Same max: add index
                    } else {
                        (indices, current_max) // Smaller: ignore
                    }
                },
            )
            .0;
        let random_index = rng.random_range(0..highest_score_children.len());
        let child_index_index: usize =  highest_score_children[random_index];
        return self.selection(node.children_indices[child_index_index], rng);
    }
    #[time_graph::instrument]
    fn expansion(&mut self, fast: &fastgame::FastGame, node_index: usize, rng:&mut SmallRng) -> usize {
        let new_child_index = self.nodes.borrow().len();
        let mut nodes = self.nodes.borrow_mut();
        let node = &mut nodes[node_index];
        let new_child: Node;
        if node.is_terminal {
            return node_index;
        }
        match &mut node.specific_information { // Create a random child
            TypeInfo::Spawn(ref mut spawn_info) => {
                let new_spawn = {
                    let value;
                    let twos = &mut spawn_info.two_block_spawns_left;
                    let fours = &mut spawn_info.four_block_spawns_left;
                    let coords = match (twos.is_empty(), fours.is_empty()) {
                        (false, true) => {
                            value = 1;
                            twos.pop().unwrap()
                        },
                        (true, false) => {
                            value = 2;
                            fours.pop().unwrap()
                        },
                        (false, false) => {
                            if rng.random_bool(0.9) {
                                value = 1;
                                twos.pop().unwrap()
                            } else {
                                value = 2;
                                fours.pop().unwrap()
                            }
                        },
                        _ => unreachable!("Both value's block spawn are empty but node is not terminal")
                    };
                    (coords,value)
                };
                let new_child_probability = if new_spawn.1 == 1 {0.9} else {0.1};
                let new_child_state = fast.place_block(node.game_state, new_spawn.0, new_spawn.1);
                let mut new_child_actions_left = fast.get_possible_directions(&new_child_state);
                new_child_actions_left.shuffle(rng);
                new_child = Node {
                    game_state: new_child_state,
                    parent_index: Some(node_index),
                    visit_count: 0,
                    is_terminal: fast.is_lost(&new_child_state),
                    children_indices: Vec::new(),
                    specific_information: TypeInfo::Move(MoveInfo {
                        actions_left: new_child_actions_left,
                        probability: new_child_probability
                    }),
                    move_number: node.move_number,
                    score: node.score,
                };
            },
            TypeInfo::Move(ref mut move_info) => {
                let new_child_direction = &move_info.actions_left.pop().unwrap();
                let (new_child_state, move_score) = fast.make_move(&node.game_state, new_child_direction);
                let mut new_child_two_spawns = fastgame::FastGame::empty_list(&new_child_state);
                new_child_two_spawns.shuffle(rng);
                let mut new_child_four_spawns = fastgame::FastGame::empty_list(&new_child_state);
                new_child_four_spawns.shuffle(rng);
                new_child = Node {
                    game_state: new_child_state,
                    parent_index: Some(node_index),
                    visit_count: 0,
                    is_terminal: fast.is_lost(&new_child_state),
                    children_indices: Vec::new(),
                    specific_information: TypeInfo::Spawn(SpawnInfo {
                        move_made: new_child_direction.clone(),
                        two_block_spawns_left: new_child_two_spawns,
                        four_block_spawns_left: new_child_four_spawns,
                        total_value: 0.0,
                    }),
                    move_number: node.move_number + 1,
                    score: node.score + move_score,
                }
            },
        }
        // Append its index to the parent's index list
        node.children_indices.push(new_child_index);
        // Add the child to the tree
        nodes.push(new_child);
        return new_child_index;
    }

    #[inline]
    fn supplementary_scoring_function(game_state: [u32;4]) -> f32 {
        *fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32
    }

    #[time_graph::instrument]
    fn simulation1(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> f32 {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_move_number) = (node.game_state, node.move_number);
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
            move_number += 1;
        }
        let score_bias = Self::supplementary_scoring_function(game_state);
        return move_number as f32 * score_bias;
    }

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
    fn simulation2(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> f32 {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_move_number) = (node.game_state, node.move_number);
        let mut move_number = starting_move_number;
        while !fast.is_lost(&game_state) {
            game_state = Self::greedy_policy(&fast, game_state);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
            move_number += 1;
        }
        return move_number as f32 * (*fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32);
    }
    #[time_graph::instrument]
    fn backpropagation(&mut self, node_index: usize, score: f32) {
        let parent_index = {
            let mut nodes = self.nodes.borrow_mut();
            let root_move_number = nodes[0].move_number as f32;
            let node = &mut nodes[node_index];
            node.visit_count += 1;
            match node.specific_information {
                TypeInfo::Spawn(ref mut spawn_info) => {
                    spawn_info.total_value += score / (root_move_number+1.0);
                },
                _ => (),
            }
            node.parent_index
        };
        if let Some(parent_index) = parent_index {
            self.backpropagation(parent_index, score);
        }
    }
    #[time_graph::instrument]
    pub fn get_best_direction(&mut self, fast: &fastgame::FastGame, time_limit: f32, iteration_limit: usize) -> game::Direction {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let time_limit = Duration::from_secs_f32(time_limit);
        let start_time = std::time::Instant::now();
        let mut iteration_count = 0;
        while Instant::now() - start_time < time_limit && iteration_count < iteration_limit {
            let selected_node_index = self.selection(0, &mut rng);
            let chosen_node_index = self.expansion(&fast, selected_node_index, &mut rng);
            let rollout_score = self.simulation2(&fast, chosen_node_index, &mut rng);
            self.backpropagation(chosen_node_index, rollout_score);
            iteration_count += 1;
        }
        return self.nodes.borrow()[0].children_indices.iter()
            .map(|child_index| {
                let child = &self.nodes.borrow()[*child_index];
                match &child.specific_information {
                    TypeInfo::Spawn(spawn_info) => {
                        (spawn_info.total_value, spawn_info.move_made.clone())
                    },
                    _ => unreachable!(),
                }
            })
            .max_by(|a,b| a.0.partial_cmp(&b.0).expect("Could not order moves"))
            .map(|(_, direction)| direction)
            .expect("Could not choose best direction");
    }
}