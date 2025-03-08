use rayon::prelude::*;
use seeded_random::{Random, Seed};
use crate::neural_network;
use crate::neural_network::NeuralNetwork;
use crate::GRID_SIZE;
use crate::game;


pub const RUNS_PER_AGENT: usize = 10;

pub struct Agent {
    pub neural_network: neural_network::NeuralNetwork,
    pub game_state: [u8; GRID_SIZE*GRID_SIZE],
    fitness: [f32; RUNS_PER_AGENT],
    pub highest_tile: u8,
    seed: u64
}

impl Agent {
    pub fn new(seed: u64) -> Self {
        return Agent {
            neural_network: neural_network::NeuralNetwork::new(vec![(GRID_SIZE as u32) * (GRID_SIZE as u32), 512, 512, 512, 4], 3, 5, (-1.0,1.0), (-0.1,0.1)),
            game_state: [0; GRID_SIZE*GRID_SIZE],
            fitness: [0.0; RUNS_PER_AGENT],
            highest_tile: 0,
            seed: seed
        }
    }
    pub fn from(neural_network: neural_network::NeuralNetwork, seed: u64) -> Self {
        return Agent {
            neural_network: neural_network,
            game_state: [0; GRID_SIZE*GRID_SIZE],
            fitness: [0.0; RUNS_PER_AGENT],
            highest_tile: 0,
            seed: seed
        }
    }
    pub fn run(self: &mut Self) {
        for i in 0..RUNS_PER_AGENT {
            self.fitness[i] = self.run_once(&Random::from_seed(Seed::unsafe_new(self.seed)));
            self.seed += 1;
        }
    }

    pub fn run_once(self: &mut Self, rand: &Random) -> f32 {
        self.game_state = [0; GRID_SIZE*GRID_SIZE];
        // Add two block to the game state
        game::add_block(&mut self.game_state, rand);
        game::add_block(&mut self.game_state, rand);
        let mut move_number = 0;
        let mut max_tile = 0;
        let mut total_score = 0;
        let mut total_empty = 0;
        let mut total_smoothness = 0;
        let mut total_monotonicity = 0;
        loop {
            // Get the direction from the neural network
            let direction = self.get_direction();
            // If the position is unplayable, break
            if direction == game::Direction::None {
                self.highest_tile = max_tile as u8;
                return Agent::get_final_fitness(move_number, max_tile, total_score, total_empty, total_smoothness, total_monotonicity);
            }
            // If not, execute the move chosen by the ai
            let move_score = game::execute_move(&mut self.game_state, direction, rand);
            // Update the fitness variables
            move_number += 1;
            match self.game_state.iter().max() {
                Some(&max) => max_tile = max as i32,
                _ => continue,
            }
            total_score += move_score as i32;
            total_empty += self.game_state.iter().filter(|&&x| x == 0).count() as i32;
            total_smoothness += self.smoothness();
            total_monotonicity += self.monotonicity();
        }
    }

    fn get_final_fitness(move_number:i32, max_tile:i32, total_score:i32, total_empty:i32, total_smoothness:i32, total_monotonicity:i32) -> f32 {
    10.0 * max_tile as f32
    + 1.0 * total_score as f32
    + 0.5 * total_empty as f32 / move_number as f32
    + -0.1 * total_smoothness as f32 / move_number as f32
    + 0.5 * total_monotonicity as f32 / move_number as f32
    + -0.05 * move_number as f32
    }
    fn smoothness(self: &mut Self) -> i32 {
        let mut sum = 0;  
        for i in 0..GRID_SIZE*GRID_SIZE {  
            let row = i / GRID_SIZE;  
            let col = i % GRID_SIZE;  

            // Check right neighbor (same row, next column)  
            if col < GRID_SIZE - 1 {  
                let right = i + 1;  
                sum += (self.game_state[i] as i32 - self.game_state[right] as i32).abs();  
            }  

            // Check bottom neighbor (same column, next row)  
            if row < GRID_SIZE - 1 {  
                let bottom = i + GRID_SIZE;  
                sum += (self.game_state[i] as i32 - self.game_state[bottom] as i32).abs();  
            }  
        }  
        return sum;
    }

    fn monotonicity(self: &mut Self) -> i32 {
        let mut total = 0;  
        // Check rows  
        for row in self.game_state.chunks_exact(GRID_SIZE) {  
            let (mut inc, mut dec) = (0, 0);  
            for j in 0..GRID_SIZE - 1 {  
                if row[j] <= row[j + 1] { inc += 1; }  
                if row[j] >= row[j + 1] { dec += 1; }  
            }  
            total += inc.max(dec);  
        }  

        // Check columns  
        for col in 0..GRID_SIZE {  
            let (mut inc, mut dec) = (0, 0);  
            for row in 0..GRID_SIZE - 1 {  
                let idx = col + row * GRID_SIZE;  
                let next_idx = col + (row + 1) * GRID_SIZE;  
                if self.game_state[idx] <= self.game_state[next_idx] { inc += 1; }  
                if self.game_state[idx] >= self.game_state[next_idx] { dec += 1; }  
            }  
            total += inc.max(dec);  
        }  

        total  
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

    pub fn geometric_mean(self: &mut Self) -> f32 {
    // Calculate the the geometric mean of the scores by computing the arithmetic mean of logarithms of the scores
    return (self.fitness.iter().map(|&x| x as f32).fold(0.0, |acc, x| acc + x.ln()) / RUNS_PER_AGENT as f32).exp();
    }
    pub fn get_worst(self: &mut Self) -> f32 {
    // Get the minimum score
    return *self.fitness.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
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

