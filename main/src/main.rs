use std::{thread, time::Duration};

use game::add_block;
use seeded_random::{Random, Seed};

mod renderer;
mod game;


const GRID_SIZE: usize = 4;

fn main() {
    let mut rand = Random::from_seed(Seed::unsafe_new(512));
    //test_movements();
    let mut game_state: [u8; 16] = [0; 16];
    add_block(&mut game_state, &rand);
    renderer::render(game_state);
    loop {
        thread::sleep(Duration::from_millis(1000));
        game::make_move(&mut game_state, game::Direction::Left, &rand);
        renderer::render(game_state);
    }
}


fn test_movements() { // A function to test the movements by initializing a testing game state and displaying with simple prints
    let test_state = vec![1, 1, 1, 1,0, 2, 0, 1, 0, 1, 1, 0,0, 0, 1, 0];

    println!("Moving right");
    let moved_right_state = 2;

}
