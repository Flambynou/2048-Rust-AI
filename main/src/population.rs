use rayon::prelude::*;
use seeded_random::{Random, Seed};
use crate::neural_network;
use crate::neural_network::NeuralNetwork;
use crate::GRID_SIZE;
use crate::game;


pub const RUNS_PER_AGENT: usize = 50;

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
            neural_network: neural_network::NeuralNetwork::new(vec![(GRID_SIZE as u32) * (GRID_SIZE as u32), 64, 32, 16, 4], 3, 5, (-1.0,1.0), (-0.1,0.1)),
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
                self.score += self.best as usize * 100;
                break;
            }
            // If not, execute the move chosen by the ai
            let move_score = game::execute_move(&mut self.game_state, direction, rand);
            // Get the best block
            match self.game_state.iter().max() {
                Some(&max) => self.best = max,
                _ => continue,
            }
            // Update the score
            self.score += get_score(&self.game_state, move_score, self.best);

            self.move_number += 1;
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
        // Then create a vec with each index corresponding to the directions ranked by the neural network
        let mut indices: Vec<usize> = (0..outputs.len()).collect();
        indices.sort_by(|&i, &j| outputs[j].partial_cmp(&outputs[i]).unwrap());
        // Then loop through the indices to get the first valid move
        for index in indices {
            let direction = match index {
                0 => game::Direction::Up,
                1 => game::Direction::Down,
                2 => game::Direction::Left,
                3 => game::Direction::Right,
                _ => game::Direction::None,
            };
            if match direction {
                game::Direction::Up => game::can_up(&self.game_state),
                game::Direction::Down => game::can_down(&self.game_state),
                game::Direction::Left => game::can_left(&self.game_state),
                game::Direction::Right => game::can_right(&self.game_state),
                game::Direction::None => false,
            } {
                return direction;

            }
        }

        return game::Direction::None;
    }
}

fn get_score(game_state: &[u8; GRID_SIZE*GRID_SIZE], move_score: i32, best: u8) -> usize {
    // Count the number of empty cells
    let mut empty_cells = 0;
    for i in 0..game_state.len() {
        if game_state[i] == 0 {
            empty_cells += 1;
        }
    }
    
    return empty_cells
    + move_score as usize
    + best as usize * 5
    + smoothness(game_state) as usize * 2
    + 10;
}

fn smoothness(game_state: &[u8;GRID_SIZE*GRID_SIZE]) -> i32 {
    // Evaluate the smoothness of the game state, ie. the total of the absolute value of the differences between each adjacent tiles
    let mut smoothness = 0;
    for line in game_state.chunks_exact(GRID_SIZE) {
        for i in 0..GRID_SIZE-1 {
            smoothness += (line[i] as i32 - line[i+1] as i32).abs();
        }
    }
    for column in 0..GRID_SIZE {
        for i in 0..GRID_SIZE-1 {
            smoothness += (game_state[i*GRID_SIZE+column] as i32 - game_state[(i+1)*GRID_SIZE+column] as i32).abs();
        }
    }
    return smoothness;
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

