use rayon::prelude::*;
use seeded_random::{Random, Seed};

use crate::neural_network;
use crate::GRID_SIZE;
use crate::game;


pub struct Agent {
    pub neural_network: neural_network::NeuralNetwork,
    pub game_state: [u8; GRID_SIZE*GRID_SIZE],
    pub score: usize,
    pub best: u8,
    rand: Random,
}

impl Agent {
    pub fn new(seed: u64) -> Self {
        return Agent {
            neural_network: neural_network::NeuralNetwork::new(vec![(GRID_SIZE as u32) * (GRID_SIZE as u32), 50, 100, 100, 50, 4], 1, (-4.0,4.0), (-1.0,1.0)),
            game_state: [0; GRID_SIZE*GRID_SIZE],
            score: 0,
            best: 0,
            rand: Random::from_seed(Seed::unsafe_new(seed))
        }
    }
    pub fn from(neural_network: neural_network::NeuralNetwork, seed: u64) -> Self {
        return Agent {
            neural_network: neural_network,
            game_state: [0; GRID_SIZE*GRID_SIZE],
            score: 0,
            best: 0,
            rand: Random::from_seed(Seed::unsafe_new(seed))
        }
    }
    pub fn run(self: &mut Self) {
        loop {
            // Add a block to the game state
            game::add_block(&mut self.game_state, &self.rand);
            // First divide the game state by 10 and convert it to a Vec<f32>
            let mut game_state = Vec::new();
            let mut n_zeros = 0;
            for i in 0..self.game_state.len() {
                game_state.push(self.game_state[i] as f32 - 0.5);
                // If bigger than best, update best
                if self.game_state[i] > self.best {
                    self.best = self.game_state[i];
                }
                // If the value is 0, increment n_zeros
                if self.game_state[i] == 0 {
                    n_zeros += 1;
                }
            }
            // First get the 4 outputs from the neural network
            let outputs = self.neural_network.feed_forward(game_state);
            // Then get the index of the highest output
            let mut max_index = 0;
            for i in 1..outputs.len() {
                if outputs[i] > outputs[max_index] {
                    max_index = i;
                }
            }
            // Then convert the index to a direction
            let direction = match max_index {
                0 => game::Direction::Up,
                1 => game::Direction::Down,
                2 => game::Direction::Left,
                3 => game::Direction::Right,
                _ => panic!("You fucked up something with the ai's output")
            };
            // Then make the move
            let (lost, score) = game::make_move(&mut self.game_state, direction, &self.rand);
            // Check if the move wasn't valid
            if score == -1 {
                break;
            }
            // Add the score to the agent's score
            self.score += score as usize;
            self.score += n_zeros * 2;
            self.score += self.best as usize * 30;
            // If the agent lost, break
            if lost {
                break;
            }
        }
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

pub fn clone_population(agents: &mut Vec<Agent>, best: usize, seed: u64, mutation_rate: f32, mutation_strength: f32) {
    // Get size
    let size = agents.len();
    // Get best neural network (at index best)
    let best_neural_network = agents[best].neural_network.clone();
    // Clear the agents vector
    agents.clear();
    // Add the best neural network to the agents vector
    agents.push(Agent::from(best_neural_network.clone(), seed));
    // Add the rest of the agents (parallelized)
    let new_agents: Vec<Agent> = (1..size).into_par_iter().map(|i| {
        // Clone the best neural network
        let mut neural_network = best_neural_network.clone();
        // Mutate the neural network
        neural_network.mutate(mutation_rate, mutation_strength);
        // Create a new agent
        Agent::from(neural_network, seed)
    }).collect();
    // Extend the agents vector with the new agents
    agents.extend(new_agents);
}