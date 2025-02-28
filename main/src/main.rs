mod renderer;
mod game;
mod population;
mod neural_network;


const GRID_SIZE: usize = 4;

fn main() {
    // Create population
    let mut gen_count = 1;
    let mut population = population::create_population(10000, 0);
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
        // Print the best agent's score
        println!("Generation {}: {}     Best block : {}     Moves : {}", gen_count, population[best_agent].score, 1 << population[best_agent].best,population[best_agent].move_number);
        // Create the next generation
        population::clone_population(&mut population, best_agent, gen_count * population::RUNS_PER_AGENT as u64, 0.01, 15.0/((gen_count as f32 + 0.1).log10()) + 0.5);
        gen_count += 1;
    }

    /*let rand = Random::from_seed(Seed::unsafe_new(512));
    let mut game_state: [u8; GRID_SIZE*GRID_SIZE] = [0; GRID_SIZE*GRID_SIZE];
    add_block(&mut game_state, &rand);
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
        let (lost, score) = game::make_move(&mut game_state, direction, &rand);
        if lost {
            renderer::render(game_state);
            println!("You lost !");
            break;
        }
        renderer::render(game_state);
        total_score += score;
        println!("Score: {}", total_score);
    }*/
    
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