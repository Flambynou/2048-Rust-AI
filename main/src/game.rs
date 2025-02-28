use crate::GRID_SIZE;
use seeded_random::Random;

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Clone for Direction{
    fn clone(&self) -> Self {
        match self {
            Self::Left => Self::Left,
            Self::Right => Self::Right,
            Self::Up => Self::Up,
            Self::Down => Self::Down,
        }
    }
}

#[inline]
fn move_left_single(row:&mut [u8;GRID_SIZE]) -> (bool,i32) {
    let mut target:u8 = 0;
    let mut changed:bool = false;
    let mut score:i32 = 0;
    for i in 1..GRID_SIZE {
        if row[i] == 0 {
            continue;
        }
        if row[target as usize] == 0 {
            row[target as usize] = row[i];
            row[i] = 0;
            changed = true;
            continue;
        }
        if row[target as usize] == row[i] {
            row[target as usize] += 1;
            score += 1 << row[target as usize];
            row[i] = 0;
            target += 1;
            changed = true; 
        }
        else {
            target += 1;
            if target as usize != i {
                row[target as usize] = row[i];
                row[i] = 0;
                changed = true;
            }
        }
    }
    return (changed,score);
}

fn move_left(game_state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> (bool,i32) {
    let mut changed:bool = false;
    let mut score:i32 = 0;
    for row_chunk in game_state.chunks_exact_mut(GRID_SIZE) {
        let row_array: &mut [u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        let (tempchanged,tempscore) = move_left_single(row_array);
        changed = changed || tempchanged;
        score += tempscore;
    }
    return (changed,score);
}

fn move_right(game_state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> (bool,i32) {
    let mut changed:bool = false;
    let mut score:i32 = 0;
    for row_chunk in game_state.chunks_exact_mut(GRID_SIZE) {
        let row_array: &mut [u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        row_array.reverse();
        let (tempchange,tempscore) = move_left_single(row_array);
        changed = changed || tempchange;
        score += tempscore;
        row_array.reverse();
    }
    return (changed,score);
}

fn move_up(state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> (bool,i32) {
    let mut changed:bool = false;
    let mut score:i32 = 0;
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = state[row * GRID_SIZE + col];
        }
        let (tempchanged,tempscore) = move_left_single(&mut temp);
        changed = changed || tempchanged;
        score += tempscore;
        if tempchanged {
            for row in 0..GRID_SIZE {
            state[row * GRID_SIZE + col] = temp[row];
            }
        }
    }
    return (changed,score);
}

fn move_down(state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> (bool,i32) {
    let mut changed:bool = false;
    let mut score:i32 = 0;
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = state[(GRID_SIZE - 1 - row) * GRID_SIZE + col];
        }
        let (tempchanged,tempscore) = move_left_single(&mut temp);
        changed = changed || tempchanged;
        score += tempscore;
        if tempchanged {
            for row in 0..GRID_SIZE {
                state[(GRID_SIZE - 1 - row) * GRID_SIZE + col] = temp[row];
            }
        }
    }
    return (changed,score);
}

pub fn make_move(game_state: &mut [u8; GRID_SIZE*GRID_SIZE], direction:Direction, rand: &Random) -> (bool,i32) {
    // Copy the game state to compare after the movement
    let (changed,score) = match direction {
        Direction::Left => move_left(game_state),
        Direction::Right => move_right(game_state),
        Direction::Up => move_up(game_state),
        Direction::Down => move_down(game_state),
    };
    if !changed {
        return (false,-1);
    }
    // Add random blocks
    add_block(game_state, &rand);
    // Check if the game is lost
    if !game_state.contains(&0) {
        let mut test_game_state = game_state.clone();
        if !(move_left(&mut test_game_state).0||move_right(&mut test_game_state).0||move_up(&mut test_game_state).0||move_down(&mut test_game_state).0) {
            return (true,score);
        }
    }
    return (false,score);
}


pub fn add_block(game_state: &mut [u8; GRID_SIZE*GRID_SIZE], rand: &Random) {
    // Select which block is going to be placed (1 or 2 which corresponds to 2 or 4)
    let value: u8 = if rand.gen::<f32>() < 0.9 {1} else {2};
    // Count the number of empty cells
    let count: usize = game_state.iter().filter(|&n| *n == 0).count();
    let mut index: usize = (count as f32 * rand.gen::<f32>()) as usize;
    // Loop through game_state
    for i in 0..(GRID_SIZE*GRID_SIZE) {
        if game_state[i] == 0 {
            if index == 0 {
                game_state[i] = value;
                return;
            }
            index -= 1;
        }
    }
}
