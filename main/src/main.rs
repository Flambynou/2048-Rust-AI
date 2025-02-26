use game::add_block;
use seeded_random::{Random, Seed};

mod renderer;
mod game;


const GRID_SIZE: usize = 7;

fn main() {
    let rand = Random::from_seed(Seed::unsafe_new(512));
    let mut game_state: [u8; GRID_SIZE*GRID_SIZE] = [0; GRID_SIZE*GRID_SIZE];
    add_block(&mut game_state, &rand);
    renderer::render(game_state);
    do_a_barrel_roll(game_state, &rand);
    return;
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
        if !game::make_move(&mut game_state, direction, &rand) {
            renderer::render(game_state);
            println!("You lost !");
            break;
        }
        renderer::render(game_state);
    }
    
}


fn do_a_barrel_roll(mut game_state: [u8; GRID_SIZE*GRID_SIZE], rand: &Random) {
    let mut direction: game::Direction = game::Direction::Left;
    loop {
        if !game::make_move(&mut game_state, direction.clone(), &rand) {
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
}