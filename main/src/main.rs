const GRID_SIZE = 4
const BLOCK_SIDE = 5
fn main() {
    println!("Hello, world!");
}

fn draw_game(game_state) {
    //Draws the current state of the game (a grid of GRID_SIZE squared of blocks of size BLOCK_SIDE)
    //Should use ANSI escape codes to redraw over the last frame
    //Uses background color to distinguish between empty and existing blocks
}

fn move_right(current_state) {
    // Returns the game state after having moved to the right
    // current_state is a square matrix (list of list) of GRID_SIZE dimension
    let new_state = current_state.deepcopy()
    for i in 0..GRID_SIZE {         // Loop over every line
        let mut pointer = 0         // A pointer for the current right-most free block
        let mut noted_value = -1    // A variable noting the value of the first block encountered to know if the next block can be fused with it
        let mut noted_index = 0
        let mut line = new_state[i]
        for j in 0..GRID_SIZE { // Loop over every space in the line
            if noted_value == -1 && line[j] != 0 { // Change block_value if it's -1 and the value of the current block is not 0
                noted_value += 1+line[j]
                noted_index = j 
            }
            else { // Move the noted block at pointer when encountering another block
                if line[j] != 0 { // If encountering a new block
                    if line[j] == noted_value { // Fuse them if they have the same value
                        line[pointer] = block_value+1
                        noted_value = -1
                        line[noted_index] = 0
                        line[j] = 0
                    }
                    else if{ // Otherwise, move the noted block at pointer and note the new block
                        line[pointer] = noted_value
                        line[noted_index] = 0
                        noted_value = line[j]
                    }
                }
                else if j == GRID_SIZE-1 { // If at the end of the line, move the noted block at pointer
                    line[pointer] = noted_value
                    line[noted_index] = 0
                    break
                }
            }
        }

    }
}