enum Direction {
    Left,
    Right,
    Up,
    Down,
}

enum


fn move_left_single(row:&mut [u8;GRID_SIZE]) -> [u8;GRID_SIZE] {
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

fn move_right(game_state: &mut [u8; GRID_SIZE * GRID_SIZE]) {
    for row_chunk in game_state.chunks_exact_mut(GRID_SIZE) {
        let row_array: &mut [u8; GRID_SIZE] = row_chunk.try_into().unwrap();
        row_array.reverse();
        move_left_single(row_array);
        row_array.reverse();
    }
}

fn move_up(state: &mut [u8; GRID_SIZE * GRID_SIZE]) {
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = state[row * GRID_SIZE + col];
        }
        move_left_single(&mut temp);
        for row in 0..GRID_SIZE {
            state[row * GRID_SIZE + col] = temp[row];
        }
    }
}

fn move_down(state: &mut [u8; GRID_SIZE * GRID_SIZE]) {
    for col in 0..GRID_SIZE {
        let mut temp = [0; GRID_SIZE];
        for row in 0..GRID_SIZE {
            temp[row] = state[(GRID_SIZE - 1 - row) * GRID_SIZE + col];
        }
        move_left_single(&mut temp);
        for row in 0..GRID_SIZE {
            state[(GRID_SIZE - 1 - row) * GRID_SIZE + col] = temp[row];
        }
    }
}

fn make_move(&mut game_state, direction:Direction) {
    // Copy the game state to compare after the movement
    let mut old_game_state = game_state.clone();
    match direction {
        Direction::Left => move_left(&mut game_state),
        Direction::Right => move_right(&mut game_state),
        Direction::Up => move_up(&mut game_state),
        Direction::Down => move_down(&mut game_state),
    }
    if old_game_state == game_state { // The move is not valid
        return true;
    }
    else {
        // Add random blocks
    }
    // Check if the game is lost
    if !game_state.contains(0) {
        test_game_state = game_state.clone();
        move_left(&mut test_game_state);
        move_right(&mut test_game_state);
        move_up(&mut test_game_state);
        move_down(&mut test_game_state);
        if test_game_state == game_state {
            return false;
        }
    }

}
