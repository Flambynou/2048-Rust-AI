mod fastgame;
mod game;
mod minimax;
mod neural_network;
mod population;
mod renderer;
mod mcts;
use fastgame::FastGame;
use seeded_random::{Random, Seed};
use std::path::Path;
use rayon::prelude::*;
const GRID_SIZE: usize = 4;

const POPULATION_SIZE: usize = 2000;

const SEED: u64 = 0;


const MINIMAX_DEPTH: usize = 15;
const EXPECTIMAX_DEPTH: usize = 2;
// MCTS will search until either the time or iteration limit is reached
// Time limit for MCTS simulation in seconds
const MCTS_TIME_LIMIT: f32 = 0.01;
const MCTS_ITERATION_LIMIT: usize = 1_000_000_000_000;

fn main() {
    // Ask user for playing / training / ai mode
    println!("Choose a mode :");
    println!("1. Play");
    println!("2. Play fast");
    println!("3. Train");
    println!("4. AI");
    println!("5. Minimax");
    println!("6. Expectimax");
    println!("7. Monte Carlo tree search");
    println!("8. Test MCTS optimization");
    println!("9. Test MCTS strength");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let line = line.trim();
    match line {
        "1" => play(),
        "2" => playfast(),
        "3" => train(),
        "4" => ai(),
        "5" => use_mini_expecti_max(true),
        "6" => use_mini_expecti_max(false),
        "7" => use_mcts(),
        "8" => mcts_optimization_test(),
        "9" => mcts_strength_test(false),
        "10" => rollout_verification(),
        _ => println!("Invalid mode"),
    }
}

fn play() {
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    let mut game_state: [u8; GRID_SIZE * GRID_SIZE] = [0; GRID_SIZE * GRID_SIZE];
    game::add_block(&mut game_state, &rand);
    game::add_block(&mut game_state, &rand);
    renderer::render(game_state);
    let mut total_score = 0;
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let line = line.trim();
        if line == "" {
            continue;
        }
        let direction: game::Direction = match line {
            "z" => game::Direction::Up,
            "s" => game::Direction::Down,
            "q" => game::Direction::Left,
            "d" => game::Direction::Right,
            _ => continue,
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
    let line = line.trim();
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
        population::load_population(
            POPULATION_SIZE,
            gen as u64 * population::RUNS_PER_AGENT as u64,
            network,
        )
    };

    loop {
        // Run the population
        population::run_all(&mut population);
        // Get the best agent
        let mut best_score = 0.0;
        let mut best_agent = 0;
        for i in 0..population.len() {
            if population[i].geometric_mean() >= best_score {
                best_score = population[i].geometric_mean();
                best_agent = i;
            }
        }
        let best_network = population[best_agent].neural_network.clone();

        // Save the best network
        best_network.save(&path, gen_count as usize);

        // Print the best agent's score
        println!(
            "Generation {}: {}     Best block accross all games : {}",
            gen_count,
            best_score,
            1 << population[best_agent].highest_tile
        );
        // Create the next generation
        population::clone_population(
            &mut population,
            best_network,
            gen_count * population::RUNS_PER_AGENT as u64,
            0.25,
            0.5,
        );
        gen_count += 1;
    }
}

fn ai() {
    // Ask user for network name
    println!("Enter a network name :");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let line = line.trim();

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

fn use_mini_expecti_max(mini: bool) {
    // Generate an empty grid
    let mut game_state = [0u32;4];
    // Compute the lookup table
    let fast = fastgame::FastGame::new();
    // Add two random blocks
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    game_state = fast.add_random_block(game_state, &rand);
    game_state = fast.add_random_block(game_state, &rand);
    let mut score:usize = 0;
    // Render the game
    renderer::render(FastGame::to_flat_array(game_state));
    // Main loop: print, compute best move, play
    loop {
        // Get the best direction, play it, and add a random block
        let best_direction;
        if mini {
            best_direction = minimax::get_best_direction_minimax(&fast, game_state, MINIMAX_DEPTH);
        } else {
            best_direction = minimax::get_best_direction_expectimax(&fast, game_state, EXPECTIMAX_DEPTH);
        }
        if best_direction == game::Direction::None {
            println!("No possible move !");
            break;
        }
        let (new_game_state, move_score) = fast.make_move(&game_state, &best_direction);
        score += move_score as usize;
        game_state = new_game_state;
        game_state = fast.add_random_block(game_state, &rand);
        // Check for loss
        if fast.is_lost(&game_state) {
            renderer::render(FastGame::to_flat_array(game_state));
            println!("Final score : {}",score);
            println!("You lost !");
            break;
        }
        // Render the game and score
        renderer::render(FastGame::to_flat_array(game_state));
        println!("Score: {}", score);
    }
}


fn playfast() {
    // Initialize the LUT
    let fast = fastgame::FastGame::new();
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    let mut game_state = [0u32;4];
    game_state = fast.add_random_block(game_state, &rand);
    game_state = fast.add_random_block(game_state, &rand);
    renderer::render(FastGame::to_flat_array(game_state));
    let mut total_score = 0;
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let line = line.trim();
        if line == "" {
            continue;
        }
        let direction: game::Direction = match line {
            "z" => game::Direction::Up,
            "s" => game::Direction::Down,
            "q" => game::Direction::Left,
            "d" => game::Direction::Right,
            _ => continue,
        };
        let (new_game_state,score) = fast.play_move(game_state, direction, &rand);
        game_state = new_game_state;
        if fast.is_lost(&game_state) {
            renderer::render(FastGame::to_flat_array(game_state));
            println!("You lost !");
            break;
        }
        renderer::render(FastGame::to_flat_array(game_state));
        total_score += score;
        println!("Score: {}", total_score);
    }
}

fn use_mcts(){
    time_graph::enable_data_collection(true);
    let fast = fastgame::FastGame::new();
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    let mut game_state = [0;4];
    game_state = fast.add_random_block(game_state, &rand);
    game_state = fast.add_random_block(game_state, &rand);
    //game_state = [163840,229376,327680,427008];
    let mut game_score = 0;
    renderer::render(FastGame::to_flat_array(game_state));
    println!("Score: {:?}", game_score);
    let start_time = std::time::Instant::now();
    let mut mcts = mcts::MonteCarloTree::new(&fast, game_state);
    loop {
        mcts.grow_tree(&fast, MCTS_TIME_LIMIT, MCTS_ITERATION_LIMIT);
        let best_direction = mcts.get_best_direction();
        let (new_game_state, move_score) = fast.play_move(game_state, best_direction.clone(), &rand);
        game_score += move_score;
        game_state = new_game_state;
        if fast.is_lost(&game_state) {
            renderer::render(FastGame::to_flat_array(game_state));
            println!("Final score : {}", game_score);
            println!("You lost !");
            let graph = time_graph::get_full_graph();
            println!("{}", graph.as_table());
            break;
        }
        renderer::render(FastGame::to_flat_array(game_state));
        println!("Score: {:?}", game_score);
        println!("Time spent since the begining of the game : {:?}", std::time::Instant::now() - start_time);
        mcts.get_info(&best_direction);
        mcts.reroot(&fast, move_score, game_state);
    }
}

fn mcts_optimization_test(){
    // Test performance by running a few tests in a standard situation, to assess time per iteration in a controled way
    time_graph::enable_data_collection(true);
    let fast = fastgame::FastGame::new();
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    let mut game_state = [0;4];
    game_state = fast.add_random_block(game_state, &rand);
    game_state = fast.add_random_block(game_state, &rand);
    //game_state = [163840,229376,327680,427008];
    let mut game_score = 0;
    renderer::render(FastGame::to_flat_array(game_state));
    println!("Score: {:?}", game_score);
    let start_time = std::time::Instant::now();
    let mut mcts = mcts::MonteCarloTree::new(&fast, game_state);
    mcts.grow_tree(&fast, 5.0, usize::max_value());
    let best_direction = mcts.get_best_direction();
    let (new_game_state, move_score) = fast.play_move(game_state, best_direction.clone(), &rand);
    game_score += move_score;
    game_state = new_game_state;
    renderer::render(FastGame::to_flat_array(game_state));
    println!("Score: {:?}", game_score);
    println!("Time spent since the begining of the game : {:?}", std::time::Instant::now() - start_time);
    mcts.get_info(&best_direction);
    mcts.reroot(&fast, move_score, game_state);
    let graph = time_graph::get_full_graph();
    println!("{}", graph.as_table());
}

fn mcts_strength_test(parallel:bool) {
    // Test the strenght of the mcts implementation by running it accros different seeds and with different time limits
    let fast = fastgame::FastGame::new();
    let seeds:[usize;25] = core::array::from_fn(|i| i + 1);
    let time_limits:Vec<f32> = vec![0.001];
    let iteration_count = 1;
    let mut time_limit_average_score = vec![0.0;time_limits.len()];
    if parallel {
        time_limit_average_score = time_limits.par_iter().map(|&time_limit| {
            let seed_average_scores: Vec<_> = seeds.par_iter().map(|&seed| {
                let seed_scores: Vec<_> = (0..iteration_count).into_par_iter().map(|iteration| {
                    let rand = Random::from_seed(Seed::unsafe_new(seed as u64));
                    let mut game_state = [0; 4];
                    game_state = fast.add_random_block(game_state, &rand);
                    game_state = fast.add_random_block(game_state, &rand);
                    let mut game_score = 0;
                    let mut mcts = mcts::MonteCarloTree::new(&fast, game_state);
                    loop {
                        mcts.grow_tree(&fast, time_limit, usize::max_value());
                        let best_direction = mcts.get_best_direction();
                        let (new_game_state, move_score) = fast.play_move(game_state, best_direction.clone(), &rand);
                        game_score += move_score;
                        game_state = new_game_state;
                        if fast.is_lost(&game_state) {
                            println!("Finished simulation with time limit {}, seed {}, iteration {}. Reached score {}", 
                                   time_limit, seed, iteration, game_score);
                            return game_score;
                        }
                        mcts.reroot(&fast, move_score, game_state);
                    }
                }).collect();
                seed_scores.iter().map(|score| (*score as f32).ln()).sum::<f32>()
            }).collect();
            (seed_average_scores.iter().sum::<f32>() / (seeds.len() * iteration_count) as f32).exp()
        }).collect();
    } else {
        for (time_index, &time_limit) in time_limits.iter().enumerate() {
            let mut seed_average_scores = vec![0.0;seeds.len()];
            for (seed_index,&seed) in seeds.iter().enumerate() {
                let mut seed_scores = vec![0;iteration_count];
                for iteration in 0..iteration_count {
                    let rand = Random::from_seed(Seed::unsafe_new(seed as u64));
                    let mut game_state = [0;4];
                    game_state = fast.add_random_block(game_state, &rand);
                    game_state = fast.add_random_block(game_state, &rand);
                    //game_state = [163840,229376,327680,427008];
                    let mut game_score = 0;
                    let mut mcts = mcts::MonteCarloTree::new(&fast, game_state);
                    loop {
                        mcts.grow_tree(&fast, time_limit, usize::max_value());
                        let best_direction = mcts.get_best_direction();
                        let (new_game_state, move_score) = fast.play_move(game_state, best_direction.clone(), &rand);
                        game_score += move_score;
                        game_state = new_game_state;
                        if fast.is_lost(&game_state) {
                            seed_scores[iteration] = game_score;
                            println!("Finnished simulation with time limit {}, seed {}, iteration {}. Reached score {}", time_limit, seed, iteration, game_score);
                            break;
                        }
                        mcts.reroot(&fast, move_score, game_state);
                    }
                }
                seed_average_scores[seed_index] = seed_scores.iter().map(|score| (*score as f32).ln()).sum::<f32>();
            }
            time_limit_average_score[time_index] = (seed_average_scores.iter().sum::<f32>() / (seeds.len() * iteration_count) as f32).exp()
        }
    }
    println!(" Time limits : {:?}", time_limits);
    println!("Average score: {:?}", time_limit_average_score);
}

fn rollout_verification() {
    // Test rollouts
    time_graph::enable_data_collection(true);
    let fast = fastgame::FastGame::new();
    let rand = Random::from_seed(Seed::unsafe_new(SEED));
    let mut game_state = [0;4];
    game_state = fast.add_random_block(game_state, &rand);
    game_state = fast.add_random_block(game_state, &rand);
    //game_state = [163840,229376,327680,427008];
    let mut game_score = 0;
    renderer::render(FastGame::to_flat_array(game_state));
    println!("Score: {:?}", game_score);
    let start_time = std::time::Instant::now();
    let mut mcts = mcts::MonteCarloTree::new(&fast, game_state);
    mcts.grow_tree(&fast, 5.0, 1);
    let best_direction = mcts.get_best_direction();
    let (new_game_state, move_score) = fast.play_move(game_state, best_direction.clone(), &rand);
    game_score += move_score;
    game_state = new_game_state;
    println!("Score: {:?}", game_score);
    println!("Time spent since the begining of the game : {:?}", std::time::Instant::now() - start_time);
    mcts.get_info(&best_direction);
    mcts.reroot(&fast, move_score, game_state);
    let graph = time_graph::get_full_graph();
    println!("{}", graph.as_table());
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
