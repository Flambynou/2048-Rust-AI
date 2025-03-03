use rayon::prelude::*;
use seeded_random::{Random, Seed};
use crate::neural_network;
use crate::neural_network::NeuralNetwork;
use crate::GRID_SIZE;
use crate::game;


pub const RUNS_PER_AGENT: usize = 20;

pub struct Agent {
    pub neural_network: neural_network::NeuralNetwork,
    pub game_state: [u8; GRID_SIZE*GRID_SIZE],
    pub score: usize,
    move_number: usize,
    pub total_moves: usize,
    best: u8,
    pub bestbest: u8,
    seed: u64
}

impl Agent {
    pub fn new(seed: u64) -> Self {
        return Agent {
            neural_network: neural_network::NeuralNetwork::new(vec![(GRID_SIZE as u32) * (GRID_SIZE as u32), 64, 128, 32, 4], 3, 5, (-1.0,1.0), (-0.1,0.1)),
            game_state: [0; GRID_SIZE*GRID_SIZE],
            score: 0,
            move_number: 0,
            total_moves: 0,
            best: 0,
            bestbest: 0,
            seed: seed
        }
    }
    pub fn from(neural_network: neural_network::NeuralNetwork, seed: u64) -> Self {
        return Agent {
            neural_network: neural_network,
            game_state: [0; GRID_SIZE*GRID_SIZE],
            score: 0,
            move_number: 0,
            total_moves: 0,
            best: 0,
            bestbest: 0,
            seed: seed
        }
    }
    pub fn run(self: &mut Self) {
        for _ in 0..RUNS_PER_AGENT {
            self.run_once(&Random::from_seed(Seed::unsafe_new(self.seed)));
            self.total_moves += self.move_number;
            if self.best > self.bestbest {
                self.bestbest = self.best;
            }
            self.seed += 1;
        }
    }

    pub fn run_once(self: &mut Self, rand: &Random) {
        self.game_state = [0; GRID_SIZE*GRID_SIZE];
        // Add two block to the game state
        game::add_block(&mut self.game_state, rand);
        game::add_block(&mut self.game_state, rand);
        self.move_number = 0;
        self.best = 0;
        loop {
            // Get the direction from the neural network
            let direction = self.get_direction();
            // If the position is unplayable, break
            if direction == game::Direction::None {
                match self.game_state.iter().max() {
                    Some(&max) => self.best = max,
                    None => continue,
                }
                self.score += self.best as usize * 10;
                break;
            }
            // If not, execute the move chosen by the ai
            let move_score = game::execute_move(&mut self.game_state, direction, rand);
            // Update the score
            self.move_number += 1;
            self.score += move_score as usize + 1;
        }
    }

    pub fn get_direction(self: &mut Self) -> game::Direction {
        // Transform the game_state into an input for the network
        let mut input_game_state = Vec::with_capacity(GRID_SIZE*GRID_SIZE);
        for i in 0..self.game_state.len() {
            if self.game_state[i] == 0 {
                input_game_state.push(0.0);
                continue;
            }
            input_game_state.push((self.game_state[i] as f32 + 2.0) / 10.0);
        }
        // First get the 4 outputs from the neural network
        let outputs = self.neural_network.feed_forward(input_game_state);
        // Then get the index of the highest playable output
        let mut max_index = 0;
        let mut tried = 0;
        for i in 1..outputs.len() {
            if outputs[i] > outputs[max_index] {
                match i {
                0 => if game::can_up(&self.game_state) {max_index = i},
                1 => if game::can_down(&self.game_state) {max_index = i},
                2 => if game::can_left(&self.game_state) {max_index = i},
                3 => if game::can_right(&self.game_state) {max_index = i},
                _ => panic!("You fucked up something with the ai's output")
                };
            }
            tried += 1;
            if tried >= 4 {
                max_index = 4;
            }
        }
        // Then convert the index to a direction
        let direction = match max_index {
            0 => game::Direction::Up,
            1 => game::Direction::Down,
            2 => game::Direction::Left,
            3 => game::Direction::Right,
            4 => game::Direction::None,
            _ => panic!("You fucked up something with the ai's output")
        };
        return direction;
    }
}


pub fn run_all(agents: &mut Vec<Agent>) {
    agents.par_iter_mut().enumerate().for_each(|(_, agent)| {
        agent.run();
    });
}


pub fn create_population(size: usize, seed: u64) -> Vec<Agent> {
    let mut agents = Vec::new();
    for _ in 0..size {
        agents.push(Agent::new(seed as u64));
    }
    return agents;
}

pub fn load_population(size: usize, seed: u64, neural_network: NeuralNetwork) -> Vec<Agent> {
    let mut agents = Vec::new();
    for _ in 0..size {
        agents.push(Agent::from(neural_network.clone(), seed as u64));
    }
    return agents;
}

pub fn clone_population(agents: &mut Vec<Agent>, best: NeuralNetwork, seed: u64, mutation_rate: f32, mutation_strength: f32) {
    // Get size
    let size = agents.len();
    // Clear the agents vector
    agents.clear();
    // Add the best neural network to the agents vector
    agents.push(Agent::from(best.clone(), seed));
    // Add the rest of the agents (parallelized)
    let new_agents: Vec<Agent> = (1..size).into_par_iter().map(|_| {
        // Clone the best neural network
        let mut neural_network = best.clone();
        // Mutate the neural network
        neural_network.mutate(mutation_rate, mutation_strength);
        // Create a new agent
        Agent::from(neural_network, seed)
    }).collect();
    // Extend the agents vector with the new agents
    agents.extend(new_agents);
}