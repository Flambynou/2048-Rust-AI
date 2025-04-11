use crate::fastgame;
use crate::game;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::time::{Instant, Duration};

const EXPLORATION_CONSTANT:f32 = 5000.0;

#[derive(Clone,Debug)]
pub enum NodeType {
    Spawn(SpawnNode),
    Move(MoveNode),
}
#[derive(Clone,Debug)]
pub struct SpawnNode {
    game_state: [u32;4],
    score: u32,
    parent_index: Option<usize>,
    children_indices: Vec<usize>,
    move_made: game::Direction,
    visit_count: f32,
    total_value: f32,
    is_terminal: bool,
}
#[derive(Clone,Debug)]
pub struct MoveNode {
    game_state: [u32;4],
    score: u32,
    parent_index: Option<usize>,
    children_indices: Vec<usize>,
    actions_left: Vec<game::Direction>,
    visit_count: f32,
}


fn move_imediate_score_policy(fast: &fastgame::FastGame, grid: [u32;4], score: u32) -> ([u32;4],u32) {
	if fast.is_lost(&grid) {
		return (grid,score); 
	}
	let (new_grid,move_score) = fast.get_possible_directions(&grid)
        .iter()
        .map(|direction| {
            let (new_grid, move_score) = fast.make_move(&grid, &direction);
            (new_grid, move_score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap_or((grid,score)).clone();
    return (new_grid,score+move_score);
}

fn random_move_policy(fast: &fastgame::FastGame, grid: [u32;4], score: u32, rng: &mut ThreadRng) -> ([u32;4],u32) {
    if fast.is_lost(&grid) {
        return (grid,score); 
    }
    let possible_directions = fast.get_possible_directions(&grid);
    let random_direction_index = rng.random_range(0..possible_directions.len());
    let direction = &possible_directions[random_direction_index];
    let (new_grid,move_score) = fast.make_move(&grid, &direction);
    return (new_grid,score+move_score);
}



pub struct MonteCarloTree {
    node_vec: Vec<Box<NodeType>>,
}

impl MonteCarloTree {
    pub fn new(fast: &fastgame::FastGame, rootstate:[u32;4], score: u32) -> MonteCarloTree {
        let rootnode = NodeType::Move(MoveNode{
            game_state: rootstate,
            score: score,
            parent_index: None,
            children_indices: Vec::new(),
            actions_left: fast.get_possible_directions(&rootstate),
            visit_count: 0.0,
        });
        MonteCarloTree { node_vec: vec![Box::new(rootnode)]}
    }

    fn select_recursive(&mut self, node_index: usize, rng: &mut ThreadRng) -> usize {
        match self.node_vec[node_index].as_ref() {
            NodeType::Spawn(spawn_node) => {
                if spawn_node.is_terminal || spawn_node.children_indices.is_empty() {
                    return node_index;
                }
                // Choose a children at random, following the probabilities
                let value_change_index = spawn_node.children_indices.len() / 2;
                let two_probability = 1.0 / (value_change_index as f32 * 0.9);
                let four_probability = 1.0 / (value_change_index as f32 * 0.1);
                let mut prob_vec = vec![0.0];
                for i in 0..(value_change_index * 2) {
                    if i <= value_change_index {
                        prob_vec.push(prob_vec[i] + two_probability);
                    } else {
                        prob_vec.push(prob_vec[i] + four_probability);
                    }
                }
                let random_float = rng.random::<f32>();
                let mut children_list_index = 0;
                for (i, &prob) in prob_vec.iter().enumerate().skip(1) {
                    if prob > random_float {
                        children_list_index = i - 1;
                        break;
                    }
                }
                return self.select_recursive(spawn_node.children_indices[children_list_index], rng);
            },
            NodeType::Move(move_node) => {
                if !move_node.actions_left.is_empty() || move_node.children_indices.is_empty() {
                    return node_index;
                }
                let mut best_children_list_index = 0;
                let mut best_uct = f32::NEG_INFINITY;
                for (i, child_index) in move_node.children_indices.iter().enumerate() {
                    let uct_value = match self.node_vec[*child_index].as_ref() {
                        NodeType::Spawn(spawn_node) => {
                            (spawn_node.total_value / spawn_node.visit_count) + EXPLORATION_CONSTANT * (move_node.visit_count.ln() / spawn_node.visit_count).sqrt()
                        }
                        _ => unreachable!(),
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

    fn expand(&mut self, fast: &fastgame::FastGame, node_index: usize) -> Vec<usize> {
        let mut new_nodes_indices = Vec::new();
        let mut new_children = Vec::new();
        let node_vec_len = self.node_vec.len();
        match self.node_vec[node_index].as_mut() {
            NodeType::Spawn(ref mut spawn_node) => {
                // Add all possible block spawns, with either 2 or 4 as value
                for value in [2u32,4u32].iter() {
                    for empty_space in fastgame::FastGame::empty_list(&spawn_node.game_state) {
                        let new_state = fast.place_block(spawn_node.game_state, empty_space, *value);
                        let new_child = NodeType::Move(MoveNode {
                            game_state: new_state,
                            score: spawn_node.score,
                            parent_index: Some(node_index),
                            children_indices: Vec::new(),
                            actions_left: fast.get_possible_directions(&new_state),
                            visit_count: 0.0,
                        });
                        new_children.push(new_child);
                        let new_child_index = node_vec_len + spawn_node.children_indices.len();
                        spawn_node.children_indices.push(new_child_index);
                        new_nodes_indices.push(new_child_index);
                    }
                }
            },
            NodeType::Move(ref mut move_node) => {
                // Add the child following the first direction in actions_left
                if move_node.actions_left.is_empty() {
                    return new_nodes_indices;
                }
                let direction_taken = move_node.actions_left[0].clone();
                let (state, score) = fast.make_move(&move_node.game_state, &direction_taken);
                let new_child = NodeType::Spawn(SpawnNode {
                    game_state: state,
                    score: score,
                    parent_index: Some(node_index),
                    children_indices: Vec::new(),
                    move_made: direction_taken.clone(),
                    visit_count: 0.0,
                    total_value: 0.0,
                    is_terminal: fast.is_lost(&state),
                });
                new_children.push(new_child);
                move_node.children_indices.push(node_vec_len);
                new_nodes_indices.push(node_vec_len);
                move_node.actions_left.retain(|direction| direction != &direction_taken);
            },
        }
        for new_child in new_children.iter() {
            self.node_vec.push(Box::new(new_child.clone()));
        }
        return new_nodes_indices;
    }

    fn imediate_score_rollout(fast: &fastgame::FastGame, node: &mut NodeType, rng: &mut ThreadRng) -> f32 {
        let (mut game_state, current_score) = match node {
            NodeType::Spawn(spawn_node) => (spawn_node.game_state, spawn_node.score),
            NodeType::Move(move_node) => (move_node.game_state, move_node.score),
        };
        let mut score = current_score;
        while !fast.is_lost(&game_state) {
            (game_state, score) = move_imediate_score_policy(&fast, game_state, score);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let value = if rng.random::<f32>() > 0.9 {4} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, value)
        }
        return score as f32;
    }

    fn random_rollout(fast: &fastgame::FastGame, node: &mut NodeType, rng: &mut ThreadRng) -> f32 {
        let (mut game_state, current_score) = match node {
            NodeType::Spawn(spawn_node) => (spawn_node.game_state, spawn_node.score),
            NodeType::Move(move_node) => (move_node.game_state, move_node.score),
        };
        let mut score = current_score;
        while !fast.is_lost(&game_state) {
            (game_state, score) = random_move_policy(&fast, game_state, score, rng);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let value = if rng.random::<f32>() > 0.9 {4} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, value)
        }
        return score as f32;
        
    }

    fn backpropagate_recursive(&mut self, node_index: usize, score: f32) {
        match self.node_vec[node_index].as_mut() {
            NodeType::Spawn(ref mut spawn_node) => {
                spawn_node.total_value += score;
                spawn_node.visit_count += 1.0;
                if let Some(parent_index) = spawn_node.parent_index {
                    self.backpropagate_recursive(parent_index, score);
                } else {
                    return;
                }
            }
            NodeType::Move(ref mut move_node) => {
                move_node.visit_count += 1.0;
                if let Some(parent_index) = move_node.parent_index {
                    self.backpropagate_recursive(parent_index, score);
                } else {
                    return;
                }
            }
        }
    }

    pub fn get_best_direction(&mut self, fast: &fastgame::FastGame, time_limit: f32, iteration_limit: usize) -> (game::Direction,&NodeType) {
        let mut rng = rand::rng();
        let time_limit = Duration::from_secs_f32(time_limit);
        let start_time = std::time::Instant::now();
        let mut iteration_count = 0;
        while Instant::now() - start_time < time_limit && iteration_count < iteration_limit {
            let selected_node_index = self.select_recursive(0, &mut rng);
            let new_nodes_indices = self.expand(&fast, selected_node_index);
            for new_node_index in new_nodes_indices.iter() {
                let rollout_score = Self::random_rollout(&fast, self.node_vec[*new_node_index].as_mut(), &mut rng);
                self.backpropagate_recursive(*new_node_index, rollout_score);
            }
            //println!("{} new nodes added", new_nodes_indices.len());
            iteration_count += 1;
        }
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
        self.node_vec[0].as_ref());
    }
}