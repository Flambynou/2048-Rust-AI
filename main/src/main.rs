const GRID_SIZE: usize = 4;
const BLOCK_SIDE: usize = 5;
fn main() {
    println!("Hello, world!");
    test_movements();
}

fn draw_game(_game_state : Vec<Vec<i32>>) {
    
    //Draws the current state of the game (a grid of GRID_SIZE squared of blocks of size BLOCK_SIDE)
    //Should use ANSI escape codes to redraw over the last frame
    //Uses background color to distinguish between empty and existing blocks
}

fn move_right(current_state : Vec<i32>) -> Vec<i32> {
    // Returns the game state after having moved to the right
    // current_state is a list of GRID_SIZE² length
    let mut new_state = current_state.clone();
    for i in 0..GRID_SIZE {         // Loop over every line
        println!("Line number : {:?}", i);
        let mut pointer = GRID_SIZE-1;  // A pointer for the current right-most free block
        let mut noted_value = -1;   // A variable noting the value of the first block encountered to know if the next block can be fused with it
        let mut noted_index:usize= 0;
        let line = &mut new_state[GRID_SIZE*(i-1) as usize..i as usize*GRID_SIZE+1]; // Get the current line
       
        for j in 0..GRID_SIZE { // Loop over every space in the line
            let index = (GRID_SIZE - j - 1) as usize; // Adjusting index because of the direction
            if noted_value == -1 && line[index] != 0 { // Change noted_value if it's -1 and the value of the current block is not 0
                noted_value = line[index];
                noted_index = index;
            }
            else { // If the noted_value has not just been set
                if line[index] != 0 { // If encountering a new block
                    if line[index] == noted_value { // Fuse them if they have the same value
                        println!("Fused blocks of value {:?} at index {:?} and {:?} to pointer {:?}", noted_value, noted_index, index, pointer);
                        line[pointer as usize] = noted_value+1;
                        pointer -= 1;
                        noted_value = -1;
                        line[noted_index] *= !(noted_index == pointer as usize) as i32; // If noted_index == pointer, then the block was already replaced by the new block
                        line[index] *= !(index == pointer as usize) as i32; // If index == pointer, then the block was already replaced by the new block
                   
                    }
                    else { // Otherwise, move the noted block at pointer and note the new block
                        line[pointer as usize] = noted_value;
                        pointer -= 1;
                        line[noted_index] = 0;
                        noted_value = line[index];
                        println!("Moved block (other)");
                    }
                }
                else if j == GRID_SIZE-1 { // If at the end of the line, move the noted block at pointer
                    line[pointer as usize] = noted_value;
                    line[noted_index] = 0;
                    println!("Moved block (end)");
                    break;
                }
            }
        }
    }
    return new_state;
}

fn test_movements() { // A function to test the movements by initializing a testing game state and displaying with simple prints
    let test_state = vec![1, 1, 1, 1,0, 2, 0, 1, 0, 1, 1, 0,0, 0, 1, 0];
    test_display(&test_state);
    println!("Moving right");
    let moved_right_state = move_right(test_state);
    test_display(&moved_right_state);

}

fn test_display(game_state:&Vec<i32>) { // A simple function to display game states by printing each line out
    for i in 0..GRID_SIZE {
        let line_start = (i * GRID_SIZE) as usize;
        let line_end = line_start + GRID_SIZE as usize;
        println!("{:?}", &game_state[line_start..line_end]);
   
    }
}