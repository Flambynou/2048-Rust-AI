mod renderer;
mod game;
mod population;
mod neural_network;

use seeded_random::{Random,Seed};
use std::path::Path;

const GRID_SIZE: usize = 4;

const POPULATION_SIZE: usize = 3000;

const SEED: u64 = 512;


fn main() {
    // Ask user for playing / training / ai mode
    println!("Choose a mode :");
    println!("1. Play");
    println!("2. Train");
    println!("3. AI");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.pop();
    match line.as_str() {
        "1" => play(),
        "2" => train(),
        "3" => ai(),
        _ => println!("Invalid mode")
    }
}


fn play() {
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    let mut game_state: [u8; GRID_SIZE*GRID_SIZE] = [0; GRID_SIZE*GRID_SIZE];
    game::add_block(&mut game_state, &rand);
    game::add_block(&mut game_state, &rand);
    renderer::render(game_state);
    let mut total_score = 0;
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line.pop();
        if line == "" { continue }
        let direction: game::Direction = match line.as_str() {
            "z" => game::Direction::Up,
            "s" => game::Direction::Down,
            "q" => game::Direction::Left,
            "d" => game::Direction::Right,
            _ => continue
        };
        let score = game::try_move(&mut game_state, direction, &rand);
        if game::is_lost(&game_state) {
            renderer::render(game_state);
            println!("You lost !");
            break;
        }
        renderer::render(game_state);
        total_score += score;
        println!("Score: {}", total_score);
    }
}


fn train() {
    // Ask user for network name (if it exists load, else create)
    println!("Enter a network name :");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.pop();
    // Check if networks/line exists
    let path = format!("networks/{}.ntwk", line);

    // Load or create the population
    let mut gen_count: u64 = 1;
    let mut population = if !Path::new(&path).exists() {
        population::create_population(POPULATION_SIZE, 0)
    } else {
        let (network, gen) = neural_network::NeuralNetwork::load(&path);
        // Print some info about the network
        println!("Weights: {}", network.weights.len());
        println!("Biases: {}", network.bias.len());
        gen_count = gen as u64;
        population::load_population(POPULATION_SIZE, gen as u64 * population::RUNS_PER_AGENT as u64, network)
    };

    loop {
        // Run the population
        population::run_all(&mut population);
        // Get the best agent
        let mut best_score = 0;
        let mut best_agent = 0;
        for i in 0..population.len() {
            if population[i].score >= best_score {
                best_score = population[i].score;
                best_agent = i;
            }
        }
        let best_network = population[best_agent].neural_network.clone();

        // Save the best network
        best_network.save(&path, gen_count as usize);

        // Print the best agent's score
        println!("Generation {}: {}     Best block : {}     Moves : {}", gen_count, population[best_agent].score, 1 << population[best_agent].bestbest,population[best_agent].total_moves);
        // Create the next generation
        population::clone_population(&mut population, best_network, gen_count * population::RUNS_PER_AGENT as u64, 0.15, 0.5 * (0.97f32).powi(gen_count as i32 / 25));
        gen_count += 1;
    }
}


fn ai() {
    // Ask user for network name
    println!("Enter a network name :");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.pop();
    // Check if networks/line exists
    let path = format!("networks/{}.ntwk", line);
    if !Path::new(&path).exists() {
        println!("Network not found");
        return;
    }
    let (network, _) = neural_network::NeuralNetwork::load(&path);
    let mut agent = population::Agent::from(network, SEED);
    
    let rand = Random::from_seed(Seed::unsafe_new(0));
    game::add_block(&mut agent.game_state, &rand);
    game::add_block(&mut agent.game_state, &rand);
    renderer::render(agent.game_state);
    let mut total_score = 0;
    loop {
        // Wait for a bit
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Get the direction from the neural network
        let direction = agent.get_direction();
        let score = game::try_move(&mut agent.game_state, direction, &rand);
        if game::is_lost(&agent.game_state) {
            renderer::render(agent.game_state);
            println!("You lost !");
            break;
        }
        renderer::render(agent.game_state);
        total_score += score;
        println!("Score: {}", total_score);
    }
}


/*fn do_a_barrel_roll(mut game_state: [u8; GRID_SIZE*GRID_SIZE], rand: &Random) {
    let mut direction: game::Direction = game::Direction::Left;
    loop {
        if !game::make_move(&mut game_state, direction.clone(), &rand).0 {
            renderer::render(game_state);
            println!("You lost !");
            break;
        }
        //renderer::render(game_state);
        direction = match direction.clone() {
            game::Direction::Up => game::Direction::Right,
            game::Direction::Right => game::Direction::Down,
            game::Direction::Down => game::Direction::Left,
            game::Direction::Left => game::Direction::Up
        };
    }
}*/