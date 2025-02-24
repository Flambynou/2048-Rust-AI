mod renderer;


const GRID_SIZE: usize = 4;
const BLOCK_SIDE: usize = 5;

fn main() {
    println!("Hello, world!");
    //test_movements();
    let mut test_state: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let now = std::time::Instant::now();
    let mut counter = 0;
    loop {
        renderer::render(test_state, GRID_SIZE);
        // Barrel shift test_state
        let mut new_test_state = test_state.clone();
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                if i*GRID_SIZE + j + 1 == GRID_SIZE*GRID_SIZE {
                    new_test_state[i*GRID_SIZE + j] = test_state[0];
                }
                else {
                    new_test_state[i*GRID_SIZE + j] = test_state[i*GRID_SIZE + j + 1];
                }
            }
        }
        test_state = new_test_state;

        // Calculate the fps and print it
        let elapsed = now.elapsed();
        let fps = counter as f64 / elapsed.as_secs_f64();
        println!("FPS: {}", fps);

        counter += 1;
        // Sleep for some time according to target frame rate
        std::thread::sleep(std::time::Duration::from_millis(1000 / 3));
    }
    
}


fn test_movements() { // A function to test the movements by initializing a testing game state and displaying with simple prints
    let test_state = vec![1, 1, 1, 1,0, 2, 0, 1, 0, 1, 1, 0,0, 0, 1, 0];

    println!("Moving right");
    let moved_right_state = 2;

}

fn move_left_single(row:&mut [u8;GRID_SIZE]) -> [u8;GRID_SIZE] {
    // a more optimized version to move a single row to the left
    let mut target:u8 = 0;
    for i in 1..GRID_SIZE {
        if row[i] == 0 {
            continue;
        }
        if row[target as usize] == 0 {
            row[target as usize] = row[i];
            row[i] = 0;
            continue;
        }
        if row[target as usize] == row[i] {
            row[target as usize] += 1;
            row[i] = 0;
            target += 1;
        }
        else {
            target += 1;
            row[target as usize] = row[i];
            row[i] = 0;
        }
    }
    return *row;
}

fn move_left(game_state: &mut [u8; GRID_SIZE * GRID_SIZE]) {
    for row_chunk in game_state.chunks_exact_mut(GRID_SIZE) {
        let row_array: &mut [u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        move_left_single(row_array);
    }
}