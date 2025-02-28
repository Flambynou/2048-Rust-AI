use std::io::{Read, Write};

use rand::Rng;

#[derive(Clone)]
pub struct NeuralNetwork {
    pub weights: Vec<f32>,
    pub bias: Vec<f32>,
    layers: Vec<u32>,
    activation_func: fn(f32) -> f32,
    weight_range: (f32, f32),
    bias_range: (f32, f32)
}

impl NeuralNetwork {
    pub fn new(layers: Vec<u32>, activation_func: usize, weight_range: (f32, f32), bias_range: (f32, f32)) -> NeuralNetwork {
        let mut weights = Vec::new();
        for i in 0..layers.len() - 1 {
            for _ in 0..layers[i] * layers[i + 1] {
                weights.push(rand::rng().random_range(weight_range.0..weight_range.1));
            }
        }

        let mut bias = Vec::new();
        for i in 1..layers.len() {
            for _ in 0..layers[i] {
                bias.push(rand::rng().random_range(bias_range.0..bias_range.1));
            }
        }

        NeuralNetwork {
            weights,
            bias,
            layers,
            activation_func: ACTIVATION_FUNCTIONS[activation_func],
            weight_range,
            bias_range
        }
    }

    pub fn load(path: &str) -> (NeuralNetwork, usize) {
        let mut file = std::fs::File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let contents: Vec<&str> = contents.split("\n").collect();

        let generation: usize = contents[0].parse().unwrap();
        let layers_len = contents[1].parse().unwrap();
        let mut layers = Vec::new();
        for i in 0..layers_len {
            layers.push(contents[i + 2].parse().unwrap());
        }
        let activation_func: usize = contents[layers_len + 2].parse().unwrap();
        let weight_range = (contents[layers_len + 3].parse().unwrap(), contents[layers_len + 4].parse().unwrap());
        let bias_range = (contents[layers_len + 5].parse().unwrap(), contents[layers_len + 6].parse().unwrap());

        // Calculate the number of weights and bias
        let mut weight_len = 0;
        for i in 0..layers.len() - 1 {
            weight_len += (layers[i] * layers[i + 1]) as usize;
        }
        let mut bias_len = 0;
        for i in 1..layers.len() {
            bias_len += layers[i] as usize;
        }

        let mut weights = Vec::new();
        for i in 0..weight_len {
            weights.push(contents[layers_len + 7 + i].parse().unwrap());
        }

        let mut bias = Vec::new();
        for i in 0..bias_len {
            bias.push(contents[layers_len + 7 + weight_len + i].parse().unwrap());
        }

        (NeuralNetwork {
            weights,
            bias,
            layers,
            activation_func: ACTIVATION_FUNCTIONS[activation_func],
            weight_range,
            bias_range
        },
        generation)
    }

    pub fn save(&self, path: &str, generation: usize) {
        let mut file = std::fs::File::create(path).unwrap();
        // First save all metadata (generation, layers, weight_range, bias_range)
        file.write_all(generation.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        file.write_all(self.layers.len().to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        for i in 0..self.layers.len() {
            file.write_all(self.layers[i].to_string().as_bytes()).unwrap();
            file.write_all("\n".as_bytes()).unwrap();
        }
        file.write_all(ACTIVATION_FUNCTIONS.iter().position(|&x| x == self.activation_func).unwrap().to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        file.write_all(self.weight_range.0.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        file.write_all(self.weight_range.1.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        file.write_all(self.bias_range.0.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        file.write_all(self.bias_range.1.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();

        // Save the weights and bias
        let mut weights = String::new();
        for i in 0..self.weights.len() {
            weights.push_str(&self.weights[i].to_string());
            weights.push_str("\n");
        }
        file.write_all(weights.as_bytes()).unwrap();

        let mut bias = String::new();
        for i in 0..self.bias.len() {
            bias.push_str(&self.bias[i].to_string());
            bias.push_str("\n");
        }
        file.write_all(bias.as_bytes()).unwrap();
    }




    pub fn feed_forward(&self, inputs: Vec<f32>) -> Vec<f32> {
        let mut weight_index = 0;
        let mut bias_index = 0;
        let mut current_layer = inputs.clone();
        for i in 0..self.layers.len() - 1 {
            // First check if the number of inputs is equal to the number of neurons in the current layer
            if current_layer.len() != self.layers[i] as usize {
                panic!("The number of inputs is not equal to the number of neurons in the input layer");
            }

            let mut outputs = Vec::new();
            for _ in 0..self.layers[i + 1] {
                let mut sum: f32 = 0.0;
                for k in 0..self.layers[i] {
                    sum += self.weights[weight_index] * current_layer[k as usize];
                    weight_index += 1;
                }
                // Normalize the sum
                sum /= self.layers[i] as f32;
                // Push the output to the outputs vector
                outputs.push((self.activation_func)(sum) + self.bias[bias_index]);
                bias_index += 1;
            }
            current_layer = outputs;
        }

        return current_layer
    }


    pub fn mutate(&mut self, mutation_rate: f32, mutation_strength: f32) {
        for i in 0..self.weights.len() {
            if rand::rng().random_range(0.0..1.0) < mutation_rate {
                self.weights[i] += rand::rng().random_range(-mutation_strength..mutation_strength);
                // Clamp the weights to -1.0 and 1.0
                if self.weights[i] > self.weight_range.1 {
                    self.weights[i] = self.weight_range.1;
                } else if self.weights[i] < self.weight_range.0 {
                    self.weights[i] = self.weight_range.0;
                }
            }
        }

        for i in 0..self.bias.len() {
            if rand::rng().random_range(0.0..1.0) < mutation_rate {
                self.bias[i] += rand::rng().random_range(-mutation_strength..mutation_strength);
                // Clamp the bias to -1.0 and 1.0
                if self.bias[i] > self.bias_range.1 {
                    self.bias[i] = self.bias_range.1;
                } else if self.bias[i] < self.bias_range.0 {
                    self.bias[i] = self.bias_range.0;
                }
            }
        }
    }
}


// Array of activation functions
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
pub fn tanh(x: f32) -> f32 {
    x.tanh()
}
pub fn relu(x: f32) -> f32 {
    if x > 0.0 {
        return x;
    } else {
        return 0.0;
    }
}
pub fn leaky_relu(x: f32) -> f32 {
    if x > 0.0 {
        return x;
    } else {
        return 0.01 * x;
    }
}
pub fn elu(x: f32) -> f32 {
    if x > 0.0 {
        return x;
    } else {
        return 0.01 * (x.exp() - 1.0);
    }
}
pub fn linear(x: f32) -> f32 {
    return x;
}

const ACTIVATION_FUNCTIONS: [fn(f32) -> f32; 6] = [sigmoid, tanh, relu, leaky_relu, elu, linear];