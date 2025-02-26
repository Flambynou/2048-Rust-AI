use std::{fs::DirBuilder, thread, time::Duration};

use game::add_block;
use seeded_random::{Random, Seed};

mod renderer;
mod game;


const GRID_SIZE: usize = 4;

fn main() {
    let mut rand = Random::from_seed(Seed::unsafe_new(512));
    let mut game_state: [u8; 16] = [0; 16];
    add_block(&mut game_state, &rand);
    renderer::render(game_state);
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
