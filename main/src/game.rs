use crate::GRID_SIZE;
use seeded_random::Random;

#[derive(PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    None,
}
impl Clone for Direction {
    fn clone(&self) -> Self {
        match self {
            Self::Left => Self::Left,
            Self::Right => Self::Right,
            Self::Up => Self::Up,
            Self::Down => Self::Down,
            Self::None => Self::None,
        }
    }
}

#[inline]
fn move_left_single(row: &mut [u8; GRID_SIZE]) -> i32 {
    let mut target: u8 = 0;
    let mut score: i32 = 0;
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
            score += 1 << row[target as usize];
            row[i] = 0;
            target += 1;
        } else {
            target += 1;
            if target as usize != i {
                row[target as usize] = row[i];
                row[i] = 0;
            }
        }
    }
    return score;
}

fn move_left(game_state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> i32 {
    let mut score: i32 = 0;
    for row_chunk in game_state.chunks_exact_mut(GRID_SIZE) {
        let row_array: &mut [u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        score += move_left_single(row_array);
    }
    return score;
}

fn move_right(game_state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> i32 {
    let mut score: i32 = 0;
    for row_chunk in game_state.chunks_exact_mut(GRID_SIZE) {
        let row_array: &mut [u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        row_array.reverse();
        score += move_left_single(row_array);
        row_array.reverse();
    }
    return score;
}

fn move_up(state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> i32 {
    let mut score: i32 = 0;
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = state[row * GRID_SIZE + col];
        }
        score += move_left_single(&mut temp);
        for row in 0..GRID_SIZE {
            state[row * GRID_SIZE + col] = temp[row];
        }
    }
    return score;
}

fn move_down(state: &mut [u8; GRID_SIZE * GRID_SIZE]) -> i32 {
    let mut score: i32 = 0;
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = state[(GRID_SIZE - 1 - row) * GRID_SIZE + col];
        }
        score += move_left_single(&mut temp);
        for row in 0..GRID_SIZE {
            state[(GRID_SIZE - 1 - row) * GRID_SIZE + col] = temp[row];
        }
    }
    return score;
}

pub fn try_move(
    game_state: &mut [u8; GRID_SIZE * GRID_SIZE],
    direction: Direction,
    rand: &Random,
) -> i32 {
    let score = match direction {
        Direction::Left => {
            if can_left(game_state) {
                move_left(game_state)
            } else {
                return 0;
            }
        }
        Direction::Right => {
            if can_right(game_state) {
                move_right(game_state)
            } else {
                return 0;
            }
        }
        Direction::Up => {
            if can_up(game_state) {
                move_up(game_state)
            } else {
                return 0;
            }
        }
        Direction::Down => {
            if can_down(game_state) {
                move_down(game_state)
            } else {
                return 0;
            }
        }
        Direction::None => -1,
    };
    add_block(game_state, rand);
    return score;
}

pub fn execute_move(
    game_state: &mut [u8; GRID_SIZE * GRID_SIZE],
    direction: Direction,
    rand: &Random,
) -> i32 {
    let score = match direction {
        Direction::Left => move_left(game_state),
        Direction::Right => move_right(game_state),
        Direction::Up => move_up(game_state),
        Direction::Down => move_down(game_state),
        Direction::None => -1,
    };
    add_block(game_state, rand);
    return score;
}

pub fn add_block(game_state: &mut [u8; GRID_SIZE * GRID_SIZE], rand: &Random) {
    // Select which block is going to be placed (1 or 2 which corresponds to 2 or 4)
    let value: u8 = if rand.gen::<f32>() < 0.9 { 1 } else { 2 };
    // Count the number of empty cells
    let count: usize = game_state.iter().filter(|&n| *n == 0).count();
    let mut index: usize = (count as f32 * rand.gen::<f32>()) as usize;
    // Loop through game_state
    for i in 0..(GRID_SIZE * GRID_SIZE) {
        if game_state[i] == 0 {
            if index == 0 {
                game_state[i] = value;
                return;
            }
            index -= 1;
        }
    }
}

pub fn is_lost(game_state: &[u8; GRID_SIZE * GRID_SIZE]) -> bool {
    return !(can_left(game_state)
        || can_right(game_state)
        || can_up(game_state)
        || can_down(game_state));
}

fn can_left_single(row: &[u8; GRID_SIZE]) -> bool {
    for i in 1..GRID_SIZE {
        // If the right cell is empty, we cannot say anything so skip this window
        if row[i] == 0 {
            continue;
        }
        // If the two cells are the same, fusion is possible
        if row[i] == row[i - 1] {
            return true;
        }
        // If the left cell is empty, moving is possible
        if row[i - 1] == 0 {
            return true;
        }
    }
    return false;
}

pub fn can_left(game_state: &[u8; GRID_SIZE * GRID_SIZE]) -> bool {
    for row_chunk in game_state.chunks_exact(GRID_SIZE) {
        let row_array: &[u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        if can_left_single(row_array) {
            return true;
        }
    }
    return false;
}

pub fn can_right(game_state: &[u8; GRID_SIZE * GRID_SIZE]) -> bool {
    for row_chunk in game_state.chunks_exact(GRID_SIZE) {
        let row_array: &[u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        let mut temp = row_array.clone();
        temp.reverse();
        if can_left_single(&temp) {
            return true;
        }
    }
    return false;
}

pub fn can_up(game_state: &[u8; GRID_SIZE * GRID_SIZE]) -> bool {
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = game_state[row * GRID_SIZE + col];
        }
        if can_left_single(&temp) {
            return true;
        }
    }
    return false;
}

pub fn can_down(game_state: &[u8; GRID_SIZE * GRID_SIZE]) -> bool {
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = game_state[(GRID_SIZE - 1 - row) * GRID_SIZE + col];
        }
        if can_left_single(&temp) {
            return true;
        }
    }
    return false;
}
