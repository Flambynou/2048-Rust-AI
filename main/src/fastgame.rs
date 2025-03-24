// An ultra optimized implementation of 2048 based on lookup tables and precomputed moves

// Computing a lookup table of every possible left row move
use seeded_random::Random;
use crate::game::Direction;

const MAX_BLOCK_EXPONENT: u32 = 17;
const TABLE_SIZE: u32 = (MAX_BLOCK_EXPONENT<<15) + (MAX_BLOCK_EXPONENT<<10) + (MAX_BLOCK_EXPONENT<<5) + (MAX_BLOCK_EXPONENT) + 1;

#[derive(Copy, Clone)]
struct Result {
    new_state: u32,
    changed: bool,
    score: u32,
}

pub struct FastGame {
    table: Box<[Result]>,
}

impl FastGame {
    pub fn new() -> FastGame {
        FastGame {
            table: Self::compute_left_move_table(),
        }
    }
    fn compute_left_move_table() -> Box<[Result]> {
        let mut table = vec![
            Result {
                new_state: 0,
                changed: false,
                score: 0,
            }; 
            TABLE_SIZE as usize
        ].into_boxed_slice();
        let mut a: u32 = 0;

        while a <= MAX_BLOCK_EXPONENT {
            let mut b: u32 = 0;
            while b <= MAX_BLOCK_EXPONENT {
                let mut c: u32 = 0;
                while c <= MAX_BLOCK_EXPONENT {
                    let mut d: u32 = 0;
                    while d <= MAX_BLOCK_EXPONENT {
                        let row: u32 = (a << 15) | (b << 10) | (c << 5) | d;
                        if row < TABLE_SIZE as u32 {
                            // Safety check in case we exceed table size
                            table[row as usize] = Self::compute_move_left(row);
                        }
                        else {
                            println!("Error: row {} out of bounds", row);
                            break;
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

    fn compute_move_left(row: u32) -> Result {
        let mut target: u32 = 0;
        let mut score: u32 = 0;
        let mut new_row = row;
        let mut i = 1;

        while i < 4 {
            let value: u32 = ((new_row >> (i * 5)) & 0x1F) as u32;
            let target_value: u32 = ((new_row >> (target * 5)) & 0x1F) as u32;
            if value != 0 {
                if (new_row >> (target * 5)) & 0x1F == 0 { // Move the value to the target if target is 0
                    new_row |= value << (target * 5);
                    new_row &= !(0x1F << (i * 5));
                } else if target_value == value { // If the target is the same as value, increase the target by 1 and erase the value
                    new_row += 1 << (target * 5);
                    score += 1 << (target_value + 1);
                    new_row &= !(0x1F << (i * 5));
                    target += 1;
                } else { // If both the target and the value are non-zero but different, move the value to the tile next to the target if it is not already on it
                    target += 1;
                    if target != i {
                        new_row &= !(0x1F << (target * 5));
                        new_row |= value << (target * 5);
                        new_row &= !(0x1F << (i * 5));
                    }
                }
            }
            i += 1;
        }

        Result {
            new_state: row,
            changed: new_row != row as u32,
            score: score,
        }
    }

    // Implementation of the game logic

    fn move_row_left(&self, row: &u32) -> (u32, u32) {
        let result = self.table[*row as usize];

        if !result.changed {
            return (*row, 0);
        }

        (result.new_state, result.score)
    }

    fn reverse_row(row: u32) -> u32 {
        // Extract each tile
        let a = (row >> 15) & 0x1F;
        let b = (row >> 10) & 0x1F;
        let c = (row >> 5) & 0x1F;
        let d = row & 0x1F;

        // Reverse the tiles
        return (d << 15) | (c << 10) | (b << 5) | a;
    }

    fn move_row_right(&self, row: u32) -> (u32, u32) {
        // Perform the left move
        let (moved, score) = self.move_row_left(&Self::reverse_row(row));

        // Reverse back
        let result_a = (moved >> 15) & 0x1F;
        let result_b = (moved >> 10) & 0x1F;
        let result_c = (moved >> 5) & 0x1F;
        let result_d = moved & 0x1F;

        (
            (result_d << 15) | (result_c << 10) | (result_b << 5) | result_a,
            score,
        )
    }

    fn move_grid_left(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
        let mut new_grid = [0; 4];
        let mut score = 0;

        for i in 0..4 {
            let (new_row, row_score) = self.move_row_left(&grid[i]);
            new_grid[i] = new_row;
            score += row_score;
        }

        (new_grid, score)
    }

    fn move_grid_right(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
        let mut new_grid = [0; 4];
        let mut score = 0;

        for i in 0..4 {
            let (new_row, row_score) = self.move_row_right(grid[i]);
            new_grid[i] = new_row;
            score += row_score;
        }

        (new_grid, score)
    }

    fn extract_column(grid: &[u32; 4], col: usize) -> u32 {
        let mut column = 0;
        for i in 0..4 {
            column |= ((grid[i] >> (col * 5)) & 0x1F) << (i * 5);
        }

        column
    }

    fn update_column(grid: &mut [u32; 4], col_num: usize, column: u32) {
        for i in 0..4 {
            grid[i] &= !(0x1F << (col_num * 5));
            grid[i] |= (column >> (i * 5)) & 0x1F;
        }
    }

    fn move_grid_up(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
        let mut new_grid = [0; 4];
        let mut score = 0;

        for i in 0..4 {
            let column = Self::extract_column(grid, i);
            let (new_column, column_score) = self.move_row_left(&column);
            Self::update_column(&mut new_grid, i, new_column);
            score += column_score;
        }

        (new_grid, score)
    }

    fn move_grid_down(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
        let mut new_grid = [0; 4];
        let mut score = 0;

        for i in 0..4 {
            let column = FastGame::extract_column(grid, i);
            let (new_column, column_score) = self.move_row_right(column);
            Self::update_column(&mut new_grid, i, new_column);
            score += column_score;
        }

        (new_grid, score)
    }

    fn can_go_left(&self, grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            if self.table[grid[i] as usize].changed {
                return true;
            }
        }
        return false;
    }

    fn can_go_right(&self, grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            if self.table[Self::reverse_row(grid[i]) as usize].changed {
                return true;
            }
        }
        return false;
    }

    fn can_go_up(&self, new_grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            let column = Self::extract_column(new_grid, i);
            if self.table[column as usize].changed {
                return true;
            }
        }
        return false;
    }

    fn can_go_down(&self, new_grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            let column = Self::extract_column(new_grid, i);
            if self.table[Self::reverse_row(column) as usize].changed {
                return true;
            }
        }
        return false;
    }

    pub fn is_lost(&self, grid: &[u32; 4]) -> bool {
        !(self.can_go_left(grid)
            || self.can_go_right(grid)
            || self.can_go_up(grid)
            || self.can_go_down(grid))
    }

    pub fn get_possible_directions(&self, grid: &[u32; 4]) -> Vec<Direction> {
        let mut directions = Vec::new();
        if self.can_go_left(grid) {
            directions.push(Direction::Left);
        }
        if self.can_go_right(grid) {
            directions.push(Direction::Right);
        }
        if self.can_go_up(grid) {
            directions.push(Direction::Up);
        }
        if self.can_go_down(grid) {
            directions.push(Direction::Down);
        }
        directions
    }

    pub fn make_move(&self, grid: &[u32; 4], direction: &Direction) -> ([u32; 4], u32) {
        let (new_grid, score) = match direction {
            Direction::Left => self.move_grid_left(grid),
            Direction::Right => self.move_grid_right(grid),
            Direction::Up => self.move_grid_up(grid),
            Direction::Down => self.move_grid_down(grid),
            Direction::None => return (*grid, 0),
        };
        (new_grid, score)
    }

    pub fn empty_list(&self, grid: &[u32; 4]) -> Vec<(usize, usize)> {
        let mut empty = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if (grid[i] >> (j * 5)) & 0x1F == 0 {
                    empty.push((i, j));
                }
            }
        }
        empty
    }

    pub fn place_block(&self, grid: [u32; 4], pos: (usize, usize), value: u32) -> [u32; 4] {
        let mut new_grid = grid;
        new_grid[pos.0] |= value << (pos.1 * 5);
        new_grid
    }

    pub fn add_random_block(&self, grid: [u32; 4], rand: &Random) -> [u32;4] {
        // Adds a block of random value at a random place
        let empty = self.empty_list(&grid);
        if empty.len() == 0 {
            return grid;
        }
        let value: u8 = if rand.gen::<f32>() < 0.9 { 1 } else { 2 };
        let index = (empty.len() as f32 * rand.gen::<f32>()) as usize;
        let pos = empty[index];
        self.place_block(grid, pos, value as u32)
    }

    pub fn to_flat_array(&self, grid: [u32;4]) -> [u8; 16] {
        let mut flat = [0; 16];
        for i in 0..4 {
            for j in 0..4 {
                flat[i * 4 + j] = ((grid[i] >> (j * 5)) & 0x1F) as u8;
            }
        }
        return flat;
    }

    pub fn play_move(&self, mut grid: [u32; 4], direction: Direction, rand: &Random) -> ([u32; 4],u32) {
        if direction == Direction::None {
            return (grid,0);
        }
        if self.get_possible_directions(&grid).contains(&direction) {
            let (new_grid, score) = self.make_move(&grid, &direction);
            grid = self.add_random_block(new_grid, rand);
        return (grid,score);
        }
        return (grid,0);
    }
}
