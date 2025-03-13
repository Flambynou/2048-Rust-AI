// An ultra optimized implementation of 2048 based on lookup tables and precomputed moves

// Computing a lookup table of every possible left row move
use crate::Random;
use crate::game::Direction;

const MAX_BLOCK_EXPONENT: usize = 16;
const TABLE_SIZE: usize = 104976;
static LEFT_MOVE_TABLE: [Result; TABLE_SIZE] = compute_left_move_table();

#[derive(Copy)]
#[derive(Clone)]
struct Result {
	new_state : u32,
	changed : bool,
	score : u32,
}
const fn compute_left_move_table() -> [Result; TABLE_SIZE] {
    let mut table = [Result { new_state: 0, changed: false, score: 0 }; TABLE_SIZE];
    let mut a = 0;
    
    while a <= MAX_BLOCK_EXPONENT {
        let mut b = 0;
        while b <= MAX_BLOCK_EXPONENT {
            let mut c = 0;
            while c <= MAX_BLOCK_EXPONENT {
                let mut d = 0;
                while d <= MAX_BLOCK_EXPONENT {
                    let row = ((a << 15) | (b << 10) | (c << 5) | d) as usize;
                    if row < TABLE_SIZE { // Safety check in case we exceed table size
                        table[row] = const_compute_move_left(row);
                    }
                    d += 1;
                }
                c += 1;
            }
            b += 1;
        }
        a += 1;
    }
    table
}

const fn const_compute_move_left(row: usize) -> Result {
    let mut target: u32 = 0;
    let mut score: u32 = 0;
    let mut new_row: u32 = 0;
    let mut i = 0;
    
    while i < 4 {
        let value: u32 = ((row >> (i * 5)) & 0x1F) as u32; // 5 bits mask (0x1F)
        if value != 0 {
            if (new_row >> (target * 5)) & 0x1F == 0 {
                new_row |= value << (target * 5);
            } else if (new_row >> (target * 5)) & 0x1F == value {
                new_row &= !(0x1F << (target * 5)); // Clear 5 bits
                new_row |= (value + 1) << (target * 5);
                score += 1 << (value + 1);
                target += 1;
            } else {
                target += 1;
                new_row &= !(0x1F << (target * 5));
                new_row |= value << (target * 5);
            }
        }
        i += 1;
    }
    
    Result {
        new_state: new_row,
        changed: new_row != row as u32,
        score: score,
    }
}

// Implementation of the game logic

fn move_row_left(row: &u32) -> (u32,u32) {
	let result = LEFT_MOVE_TABLE[*row as usize];

	if !result.changed {
		return (*row, 0);
	}

	(result.new_state, result.score)

}

fn reverse_row(row:u32) -> u32 {
    // Extract each tile
    let a = (row >> 15) & 0x1F;
    let b = (row >> 10) & 0x1F;
    let c = (row >> 5) & 0x1F;
    let d = row & 0x1F;
    
    // Reverse the tiles
    return (d << 15) | (c << 10) | (b << 5) | a;
}

fn move_row_right(row: u32) -> (u32, u32) {
    
    
    // Perform the left move
    let (moved, score) = move_row_left(&reverse_row(row));
    
    // Reverse back
    let result_a = (moved >> 15) & 0x1F;
    let result_b = (moved >> 10) & 0x1F;
    let result_c = (moved >> 5) & 0x1F;
    let result_d = moved & 0x1F;
    
    ((result_d << 15) | (result_c << 10) | (result_b << 5) | result_a, score)
}

fn move_grid_left(grid:[u32;4]) -> ([u32;4],u32) {
	let mut new_grid = [0;4];
	let mut score = 0;

	for i in 0..4 {
		let (new_row, row_score) = move_row_left(&grid[i]);
		new_grid[i] = new_row;
		score += row_score;
	}

	(new_grid, score)
}

fn move_grid_right(grid:[u32;4]) -> ([u32;4],u32) {
	let mut new_grid = [0;4];
	let mut score = 0;

	for i in 0..4 {
		let (new_row, row_score) = move_row_right(grid[i]);
		new_grid[i] = new_row;
		score += row_score;
	}

	(new_grid, score)
}

fn extract_column(grid:[u32;4], col:usize) -> u32 {
    let mut column = 0;
    for i in 0..4 {
        column |= ((grid[i] >> (col * 5)) & 0x1F) << (i * 5);
    }

    column
}

fn update_column(grid:&mut [u32;4], col_num:usize, column:u32) {
    for i in 0..4 {
        grid[i] &= !(0x1F << (col_num * 5));
        grid[i] |= (column >> (i * 5)) & 0x1F;
    }
}

fn move_grid_up(grid:[u32;4]) -> ([u32;4],u32) {
    let mut new_grid = [0;4];
    let mut score = 0;

    for i in 0..4 {
        let column = extract_column(grid, i);
        let (new_column, column_score) = move_row_left(&column);
        update_column(&mut new_grid, i, new_column);
        score += column_score;
    }

    (new_grid, score)
}

fn move_grid_down(grid:[u32;4]) -> ([u32;4],u32) {
    let mut new_grid = [0;4];
    let mut score = 0;

    for i in 0..4 {
        let column = extract_column(grid, i);
        let (new_column, column_score) = move_row_right(column);
        update_column(&mut new_grid, i, new_column);
        score += column_score;
    }

    (new_grid, score)
}

fn add_block(mut grid: [u32;4], (index, value):(usize,u8)) {
    // Add a block of value at the index'th empty position
    let mut empty_count = 0;
    for i in 0..4 {
        if (grid[i] & 0x1F) == 0 {
            if empty_count == index {
                grid[i] |= value as u32;
                break;
            }
            empty_count += 1;
        }
    }
}

fn can_go_left(grid:[u32;4]) -> bool {
    for i in 0..4 {
        if LEFT_MOVE_TABLE[grid[i] as usize].changed {
            return true;
        }
    }
    return false;
}

fn can_go_right(grid:[u32;4]) -> bool {
    for i in 0..4 {
        if LEFT_MOVE_TABLE[reverse_row(grid[i]) as usize].changed {
            return true;
        }
    }
    return false;
}

fn can_go_up(new_grid:[u32;4]) -> bool {
    for i in 0..4 {
        let column = extract_column(new_grid, i);
        if LEFT_MOVE_TABLE[column as usize].changed {
            return true;
        }
    }
    return false;
}

fn can_go_down(new_grid:[u32;4]) -> bool {
    for i in 0..4 {
        let column = extract_column(new_grid, i);
        if LEFT_MOVE_TABLE[reverse_row(column) as usize].changed {
            return true;
        }
    }
    return false;
}

fn is_lost(grid:[u32;4]) -> bool {
    !(can_go_left(grid) || can_go_right(grid) || can_go_up(grid) || can_go_down(grid))
}

fn get_possible_directions(grid:[u32;4]) -> Vec<Direction> {
    let mut directions = Vec::new();
    if can_go_left(grid) {
        directions.push(Direction::Left);
    }
    if can_go_right(grid) {
        directions.push(Direction::Right);
    }
    if can_go_up(grid) {
        directions.push(Direction::Up);
    }
    if can_go_down(grid) {
        directions.push(Direction::Down);
    }
    directions
}

fn make_move(mut grid: [u32;4], direction:Direction, rand:Random) -> u32 {
    if get_possible_directions(grid).contains(&direction) {
        return 0;
    }
    let (new_grid, score) = match direction {
        Direction::Left => move_grid_left(grid),
        Direction::Right => move_grid_right(grid),
        Direction::Up => move_grid_up(grid),
        Direction::Down => move_grid_down(grid),
        Direction::None => return 0,
    };
    grid = new_grid;
    add_block(grid, random_block(empty_count(grid), &rand));

    score
}

fn random_block(empty_count:usize, rand:&Random) -> (usize,u8) {
    let index = (empty_count as f32 * rand.gen::<f32>()) as usize;
    let value = if rand.gen::<f32>() < 0.9 {1} else {2};
    (index, value)
}

fn empty_count(grid:[u32;4]) -> usize {
    // Count the number of empty blocks (keeping in mind each line is stored as a u32)
    let mut count = 0;
    for i in 0..4 {
        for j in 0..4 {
            if (grid[i] >> (j * 5)) & 0x1F == 0 {
                count += 1;
            }
        }
    }
    count
}


