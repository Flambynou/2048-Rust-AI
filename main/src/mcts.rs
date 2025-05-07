use crate::{fastgame, minimax};
use crate::game::{self};
use rand::Rng;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::time::{Instant, Duration};

// Restructured node implementation

#[derive(Clone)]
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
#[derive(Clone)]
enum TypeInfo {
    Spawn(SpawnInfo),
    Move(MoveInfo),
}
#[derive(Clone)]
struct SpawnInfo {
    move_made: game::Direction,
    two_block_spawns_left: Vec<(usize,usize)>,
    four_block_spawns_left: Vec<(usize,usize)>,
    total_value: f32,
    total_squares: f32,
}
#[derive(Clone)]
struct MoveInfo {
    actions_left: Vec<game::Direction>,
    probability: f32,
}

use std::cell::RefCell;
pub struct MonteCarloTree {
    nodes: RefCell<Vec<Node>>,
    generation_iteration_count: usize,
    inherited_node_count: usize,
}

const EXPLORATION_CONSTANT:f32 = 3.5;
//const POWER_MEAN_PARAMETER:f32 = 2.0;
const VARIANCE_CONSTANT:f32 = 0.05;
impl MonteCarloTree {
    #[time_graph::instrument]
    pub fn new(fast: &fastgame::FastGame, root_state:[u32;4]) -> Self {
        let possible_directions = fast.get_possible_directions(&root_state);
        let rootnode = Node{
            game_state: root_state,
            parent_index: None,
            visit_count: 0,
            is_terminal: possible_directions.is_empty(),
            children_indices: Vec::new(),
            specific_information: TypeInfo::Move(MoveInfo { actions_left: possible_directions, probability: 1.0 }),
            move_number: 0,
            score: 0,
        };
        Self { nodes: RefCell::new(vec![rootnode]), generation_iteration_count: 0, inherited_node_count: 0}
    }

    #[time_graph::instrument]
    fn exploration_function(&self) -> f32 {
        let root = self.nodes.borrow()[0].clone();
        return (root.score as f32 + 1.0).log2();
    }

    #[time_graph::instrument]
    pub fn find_new_root(&self, new_root_state: [u32;4]) -> Option<usize> {
        for (index, node) in self.nodes.borrow().iter().enumerate() {
            if node.game_state == new_root_state && match node.specific_information{ TypeInfo::Move(_) => true, _ => false} {
                return Some(index);
            }
        }
        // If the new root is not found as a child of the old root's children, return None
        return None;
    }

    #[time_graph::instrument]
    pub fn reroot(&mut self,fast: &fastgame::FastGame, gained_score: u32, new_root_state: [u32;4]) {
        let mut new_nodes: Vec<Node> = Vec::new();
        let mut new_tree_old_indices = Vec::new();
        if let Some(new_root_old_index) = self.find_new_root(new_root_state) {
            let old_nodes = self.nodes.borrow();
            // First, traverse trough every node in the new tree (from the new root)
            let mut stack = Vec::new();
            let mut was_visited = vec![false;old_nodes.len()];
            stack.push(new_root_old_index);
            was_visited[new_root_old_index] = true;
            while let Some(old_index) = stack.pop() {
                new_tree_old_indices.push(old_index);
                let node = &old_nodes[old_index];
                for &child_old_index in &node.children_indices {
                    if !was_visited[child_old_index]{
                        was_visited[child_old_index] = true;
                        stack.push(child_old_index);
                    }
                }
            }

            // Then, map old indices to new indices
            let mut index_map = vec![None;old_nodes.len()];
            for (new_index, &old_index) in new_tree_old_indices.iter().enumerate() {
                index_map[old_index] = Some(new_index);
            }
            // Lastly, create the new tree and change the indices
            for &old_index in &new_tree_old_indices {
                let old_node = &old_nodes[old_index];
                let new_parent = old_node.parent_index.and_then(|old_parent_index| index_map[old_parent_index]);
                let new_children: Vec<usize> = old_node.children_indices
                    .iter()
                    .filter_map(|&old_child_index| index_map[old_child_index])
                    .collect();
                // Create the new nodes with corrected parent/children indices and add them to the new vec
                let mut new_node = old_node.clone();
                new_node.parent_index = new_parent;
                new_node.children_indices = new_children;
                new_nodes.push(new_node);
            }
        }
        else {
            let old_nodes = self.nodes.borrow();
            let old_root = &old_nodes[0];
            let possible_directions = fast.get_possible_directions(&new_root_state);
            let new_node = Node{
                game_state: new_root_state,
                parent_index: None,
                visit_count: 0,
                is_terminal: possible_directions.is_empty(),
                children_indices: Vec::new(),
                specific_information: TypeInfo::Move(MoveInfo { actions_left: possible_directions, probability: 1.0 }),
                move_number: old_root.move_number + 1,
                score: old_root.score + gained_score,
            };
            new_nodes.push(new_node);
        }
        self.nodes = RefCell::new(new_nodes);
        self.generation_iteration_count = 0;
        self.inherited_node_count = new_tree_old_indices.len();
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
                let exploration_factor = EXPLORATION_CONSTANT * (node.visit_count as f32).ln().sqrt();
                let children_scores_squares_visitcounts:Vec<(f32,f32,f32)> = node.children_indices.iter().map(|child_index|{
                    let child = &self.nodes.borrow()[*child_index];
                    match &child.specific_information {
                        TypeInfo::Spawn(spawn_info) => (spawn_info.total_value,spawn_info.total_squares,child.visit_count as f32),
                        TypeInfo::Move(_) => unreachable!("Move is a child of move"),
                        }
                })
                .collect();
                let children_average_scores:Vec<f32> = children_scores_squares_visitcounts.iter().map(|(score,_,visit_count)| (score / visit_count)).collect();
                let children_visitcounts:Vec<f32> = children_scores_squares_visitcounts.iter().map(|(_,_,visit_count)| *visit_count).collect();
                // Normalize the children average scores with local min-max normalization
                let min_child_score:f32 = *children_average_scores.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                let max_child_score:f32 = *children_average_scores.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                let children_score_variance:f32 = (max_child_score - min_child_score).max(1.0); // Add 1 to avoid division by 0
                let children_normalized_scores:Vec<f32> = children_average_scores.iter().map(|average_score|{
                    (average_score - min_child_score) / children_score_variance
                })
                .collect();
                let children_normalized_variance:Vec<f32> = children_scores_squares_visitcounts.iter().map(|(totalscore,totalsquares,visitcount)| {
                    let raw_var:f32 = (totalsquares/visitcount) - (totalscore/visitcount).powf(2.0);
                    raw_var / children_score_variance.powf(2.0)
                }).collect();
                // Get the children's uct scores with their normalized scores
                children_scores = children_visitcounts.iter().enumerate().map(|(index, child_visit_count)| {
                    children_normalized_scores[index] +  (exploration_factor + VARIANCE_CONSTANT * children_normalized_variance[index]) / child_visit_count.sqrt()
                })
                .collect();
                /*/ Without min-max normalization :
                children_scores = node.children_indices.iter()
                    .map(|child_index| {
                        let child = &self.nodes.borrow()[*child_index];
                        let (avg_score, visit_count) = match &child.specific_information {
                            TypeInfo::Spawn(spawn_info) => (spawn_info.total_value / child.visit_count as f32, child.visit_count as f32),
                            TypeInfo::Move(_) => unreachable!(),
                        };
                        avg_score + self.exploration_function() * exploration_factor / visit_count.sqrt()
                    })
                    .collect();*/
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
                        total_squares: 0.0,
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

    #[time_graph::instrument]
    fn random_simulation(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> ([u32;4],usize,u32) {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_score, starting_move_number) = (node.game_state, node.score, node.move_number);
        let mut score = starting_score;
        let mut move_number = starting_move_number;
        match &node.specific_information {
            TypeInfo::Spawn(_) => {
                let empty_list = fastgame::FastGame::empty_list(&game_state);
                let exponent = if rng.random_bool(0.9) {1} else {2};
                let coords = empty_list[rng.random_range(0..empty_list.len())];
                game_state = fast.place_block(game_state, coords, exponent);
            }
            _ => ()
        }
        loop {
            let possible_directions = fast.get_possible_directions(&game_state);
            let direction_number = possible_directions.len();
            if direction_number == 0 {break};
            let (new_game_state,move_score) = {
                let random_direction_index = rng.random_range(0..direction_number);
                fast.make_move(&game_state, &possible_directions[random_direction_index])
            };
            game_state = new_game_state;
            score += move_score;
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
            move_number += 1;
        }
        return (game_state,move_number,score);
    }

    #[time_graph::instrument]
    fn greedy_simulation(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> ([u32;4],usize,u32) {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_score, starting_move_number) = (node.game_state, node.score, node.move_number);
        let mut score = starting_score;
        let mut move_number = starting_move_number;
        match &node.specific_information {
            TypeInfo::Spawn(_) => {
                let empty_list = fastgame::FastGame::empty_list(&game_state);
                let exponent = if rng.random_bool(0.9) {1} else {2};
                let coords = empty_list[rng.random_range(0..empty_list.len())];
                game_state = fast.place_block(game_state, coords, exponent);
            }
            _ => ()
        }
        loop {
            let mut possible_directions = fast.get_possible_directions(&game_state);
            if possible_directions.is_empty() {break};
            let (new_game_state,move_score) = {
                // Shuffle the directions so as not to introduce a bias for certain directions
                possible_directions.shuffle(rng);
                let best = possible_directions
                    .iter()
                    .map(|direction| {
                        let (new_grid, move_score) = fast.make_move(&game_state, &direction);
                        (new_grid, move_score)
                    })
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap_or((game_state,0)).clone();
                    best
            };
            game_state = new_game_state;
            score += move_score;
            move_number += 1;
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
        }
        return (game_state,move_number,score);
    }

    #[time_graph::instrument]
    fn merge_greedy_simulation(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> ([u32;4],usize,u32) {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_score, starting_move_number) = (node.game_state, node.score, node.move_number);
        let mut score = starting_score;
        let mut move_number = starting_move_number;
        match &node.specific_information {
            TypeInfo::Spawn(_) => {
                let empty_list = fastgame::FastGame::empty_list(&game_state);
                let exponent = if rng.random_bool(0.9) { 1 } else { 2 };
                let coords = empty_list[rng.random_range(0..empty_list.len())];
                game_state = fast.place_block(game_state, coords, exponent);
            }
            _ => (),
        }
        loop {
            let mut possible_directions = fast.get_possible_directions(&game_state);
            if possible_directions.is_empty() {
                break;
            }
            let empty_before = fastgame::FastGame::empty_list(&game_state).len();
            // Shuffle to randomize order of evaluation
            possible_directions.shuffle(rng);
            // Precompute all possible moves with their merge counts
            let (new_game_state,move_score) = {
                let best = possible_directions
                    .iter()
                    .map(|direction| {
                        let (new_state, move_score) = fast.make_move(&game_state, direction);
                        let empty_after = fastgame::FastGame::empty_list(&new_state).len();
                        let merges = empty_after - empty_before;
                        (new_state, move_score, merges)
                    })
                    .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                    .unwrap_or((game_state,0,0)).clone();
                (best.0, best.1)
                };
            // Update game state and score
            game_state = new_game_state;
            score += move_score;
            move_number += 1;
            // Place new block
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) { 1 } else { 2 };
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
        }
        (game_state, move_number, score)
    }

    #[time_graph::instrument]
    fn not_worst_simulation(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> ([u32;4],usize,u32) {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_score, starting_move_number) = (node.game_state, node.score, node.move_number);
        let mut score = starting_score;
        let mut move_number = starting_move_number;
        match &node.specific_information {
            TypeInfo::Spawn(_) => {
                let empty_list = fastgame::FastGame::empty_list(&game_state);
                let exponent = if rng.random_bool(0.9) {1} else {2};
                let coords = empty_list[rng.random_range(0..empty_list.len())];
                game_state = fast.place_block(game_state, coords, exponent);
            }
            _ => ()
        }
        loop {
            let mut possible_directions = fast.get_possible_directions(&game_state);
            let (new_game_state,move_score) = match possible_directions.len() {
                0 => break,
                1 => fast.make_move(&game_state, &possible_directions[0]),
                _ => {
                    possible_directions.shuffle(rng);
                    let worst_move = possible_directions.iter().clone().map(|direction| {
                        let (_, move_score) = fast.make_move(&game_state, &direction);
                        (direction, move_score)
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap().0;
                    let other_random_direction = possible_directions.iter().filter(|direction| direction != &worst_move).collect::<Vec<_>>()[0];
                    fast.make_move(&game_state, other_random_direction)
                }
            };
            game_state = new_game_state;
            score += move_score;
            move_number += 1;
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
        }
        return (game_state,move_number,score);
    }
    #[time_graph::instrument]
    fn expectimax_simulation(&self, fast: &fastgame::FastGame, node_index: usize, rng: &mut SmallRng) -> ([u32;4],usize,u32) {
        let node = &self.nodes.borrow()[node_index];
        let (mut game_state, starting_score, starting_move_number) = (node.game_state, node.score, node.move_number);
        let mut score = starting_score;
        let mut move_number = starting_move_number;
        match &node.specific_information {
            TypeInfo::Spawn(_) => {
                let empty_list = fastgame::FastGame::empty_list(&game_state);
                let exponent = if rng.random_bool(0.9) {1} else {2};
                let coords = empty_list[rng.random_range(0..empty_list.len())];
                game_state = fast.place_block(game_state, coords, exponent);
            }
            _ => ()
        }
        loop {
            let possible_directions = fast.get_possible_directions(&game_state);
            if possible_directions.is_empty() {break}
            let best_direction = minimax::get_best_direction_expectimax(fast, game_state, 1);
            let (new_game_state,move_score) = fast.make_move(&game_state, &best_direction);
            game_state = new_game_state;
            score += move_score;
            move_number += 1;
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let exponent = if rng.random_bool(0.9) {1} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, exponent);
        }
        return (game_state,move_number,score);
    }

    #[time_graph::instrument]
    fn backpropagation(&mut self, node_index: usize, (game_state, move_count, score): ([u32;4],usize,u32)) {
        let parent_index = {
            let mut nodes = self.nodes.borrow_mut();
            let node = &mut nodes[node_index];
            node.visit_count += 1;
            match node.specific_information {
                TypeInfo::Spawn(ref mut spawn_info) => {
                    let computed_score = move_count as f32 * minimax::evaluate(node.game_state) * (*fastgame::FastGame::to_flat_array(game_state).iter().max().unwrap() as f32).max(1.0);
                    spawn_info.total_value += computed_score;
                    spawn_info.total_squares += computed_score.powf(2.0)
                },
                _ => (),
            };
            node.parent_index
        };
        if let Some(parent_index) = parent_index {
            self.backpropagation(parent_index, (game_state, move_count, score));
        }
    }

    pub fn grow_tree(&mut self, fast: &fastgame::FastGame, time_limit: f32, iteration_limit: usize) {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let time_limit = Duration::from_secs_f32(time_limit);
        let start_time = std::time::Instant::now();
        let start_iteration_count = self.generation_iteration_count;
        let mut iterations = 0;
        while Instant::now() - start_time < time_limit && self.generation_iteration_count - start_iteration_count + iterations < iteration_limit {
            let selected_node_index = self.selection(0, &mut rng);
            let chosen_node_index = self.expansion(&fast, selected_node_index, &mut rng);
            let rollout_info = self.random_simulation(&fast, chosen_node_index, &mut rng);
            self.backpropagation(chosen_node_index, rollout_info);
            iterations += 1;
        }
        self.generation_iteration_count += iterations;
    }

    pub fn get_info(&self, best_direction: &game::Direction) {
        let nodes = self.nodes.borrow();
        let node_count = nodes.len();
        let iteration_count = self.generation_iteration_count;
        let root_children = &nodes[0].children_indices;
        let mut expected_node_value = 0.0;
        for child_index in root_children {
            let child = &nodes[*child_index];
            match &child.specific_information {
                TypeInfo::Spawn(spawn_info) => {
                    if &spawn_info.move_made == best_direction {
                        expected_node_value = spawn_info.total_value / child.visit_count as f32;
                    }
                },
                _ => unreachable!("Move is a child of spawn"),
            }
        };
        let move_number = nodes[0].move_number;
        println!("Number of nodes: {}",node_count);
        println!("Iterations: {}", iteration_count);
        println!("Expected node value: {}", expected_node_value);
        println!("Move number: {}", move_number);
        println!("Nodes inherited: {}", self.inherited_node_count);
    }

    #[time_graph::instrument]
    pub fn get_best_direction(&self) -> game::Direction {
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
            .unwrap_or(game::Direction::None);
    }
}
