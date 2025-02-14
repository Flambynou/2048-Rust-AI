const GRID_SIZE: i32 = 4;
const BLOCK_SIDE: i32 = 5;
fn main() {
    println!("Hello, world!");
    test_movements();
}

fn draw_game(game_state : Vec<Vec<i32>>) {
    
    //Draws the current state of the game (a grid of GRID_SIZE squared of blocks of size BLOCK_SIDE)
    //Should use ANSI escape codes to redraw over the last frame
    //Uses background color to distinguish between empty and existing blocks
}

fn move_right(current_state : Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    // Returns the game state after having moved to the right
    // current_state is a square matrix (list of list) of GRID_SIZE dimension
    let mut new_state = current_state.clone();
    for i in 0..GRID_SIZE {         // Loop over every line
        let mut pointer = GRID_SIZE-1;         // A pointer for the current right-most free block
        let mut noted_value = -1;    // A variable noting the value of the first block encountered to know if the next block can be fused with it
        let mut noted_index:usize= 0;
        let line = &mut new_state[i as usize];
        for j in 0..GRID_SIZE { // Loop over every space in the line
            let index = (GRID_SIZE - j - 1) as usize;
            if noted_value == -1 && line[index] != 0 { // Change block_value if it's -1 and the value of the current block is not 0
                noted_value += 1+line[index];
                noted_index = index
            }
            else { // Move the noted block at pointer when encountering another block
                if line[index] != 0 { // If encountering a new block
                    if line[index] == noted_value { // Fuse them if they have the same value
                        line[pointer as usize] = noted_value+1;
                        pointer -= 1;
                        noted_value = -1;
                        line[noted_index] = 0;
                        line[index] = 0
                    }
                    else { // Otherwise, move the noted block at pointer and note the new block
                        line[pointer as usize] = noted_value;
                        pointer -= 1;
                        line[noted_index] = 0;
                        noted_value = line[index]
                    }
                }
                else if j == GRID_SIZE-1 { // If at the end of the line, move the noted block at pointer
                    line[pointer as usize] = noted_value;
                    line[noted_index] = 0;
                    break
                }
            }
        }
    }
    return new_state;
}

fn test_movements() { // A function to test the movements by initializing a testing game state and displaying with simple prints
    let test_state = vec![vec![1, 1, 1, 1], vec![0, 2, 0, 1], vec![0, 1, 1, 0], vec![0, 0, 1, 0]];
    test_display(&test_state);
    println!("Moving right");
    let moved_right_state = move_right(test_state);
    test_display(&moved_right_state);

}

fn test_display(game_state:&Vec<Vec<i32>>) { // A simple function to display game states by printing each line out
    for line in game_state {
        println!("{:?}", line)
    }
}