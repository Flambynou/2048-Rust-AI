// An ultra optimized implementation of 2048 based on lookup tables and precomputed moves

// Computing a lookup table of every possible left row move
use seeded_random::Random;
use crate::game;

const MAX_BLOCK_EXPONENT: u32 = 17;
const TABLE_SIZE: u32 = (MAX_BLOCK_EXPONENT<<15) + (MAX_BLOCK_EXPONENT<<10) + (MAX_BLOCK_EXPONENT<<5) + (MAX_BLOCK_EXPONENT) + 1;

#[derive(Copy, Clone, Debug)]
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
        let mut target: usize = 0;
        let mut score: u32 = 0;
        let mut row_array = [(row >> 15) & 0x1F, (row >> 10) & 0x1F, (row >> 5) & 0x1F, row & 0x1F];

        for i in 1..4 {
            if row_array[i] == 0 {
                continue;
            }
            if row_array[target] == 0 {
                row_array[target] = row_array[i];
                row_array[i] = 0;
                continue;
            }
            if row_array[target] == row_array[i] {
                row_array[target] += 1;
                score += 1 << row_array[target];
                row_array[i] = 0;
                target += 1;
            } else {
                target += 1;
                if target != i {
                    row_array[target] = row_array[i];
                    row_array[i] = 0;
                }
            }
        }
        let new_row:u32 = (row_array[0] << 15) | (row_array[1] << 10) | (row_array[2] << 5) | row_array[3];

        Result {
            new_state: new_row,
            changed: new_row != row,
            score: score,
        }
    }

    // Implementation of the game logic

    fn move_row_left(&self, row: u32) -> (u32, u32) {
        let result = self.table[row as usize];

        if !result.changed {
            return (row, 0);
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
        let (moved, score) = self.move_row_left(Self::reverse_row(row));

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

    #[inline]
    pub fn move_grid_left(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
        let r0 = self.table[grid[0] as usize];
        let r1 = self.table[grid[1] as usize];
        let r2 = self.table[grid[2] as usize];
        let r3 = self.table[grid[3] as usize];
        
        (
            [r0.new_state, r1.new_state, r2.new_state, r3.new_state],
            r0.score + r1.score + r2.score + r3.score
        )
    }

    #[inline]
    pub fn move_grid_right(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
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
            column |= ((grid[i] >> ((3-col) * 5)) & 0x1F) << ((3-i) * 5);
        }
        column
    }

    fn update_column(grid: &mut [u32; 4], col_num: usize, column: u32) {
        for i in 0..4 {
            grid[i] &= !(0x1F << ((3-col_num) * 5));
            grid[i] |= ((column >> ((3-i) * 5)) & 0x1F) << ((3-col_num) * 5);
        }
    }

    pub fn move_grid_up(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
        let mut new_grid = [0; 4];
        let mut score = 0;

        for i in 0..4 {
            let column = Self::extract_column(grid, i);
            let (new_column, column_score) = self.move_row_left(column);
            Self::update_column(&mut new_grid, i, new_column);
            score += column_score;
        }

        (new_grid, score)
    }

    pub fn move_grid_down(&self, grid: &[u32; 4]) -> ([u32; 4], u32) {
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
    #[inline]
    fn can_go_left(&self, grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            if self.table[grid[i] as usize].changed {
                return true;
            }
        }
        return false;
    }
    #[inline]
    fn can_go_right(&self, grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            if self.table[Self::reverse_row(grid[i]) as usize].changed {
                return true;
            }
        }
        return false;
    }
    #[inline]
    fn can_go_up(&self, new_grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            let column = Self::extract_column(new_grid, i);
            if self.table[column as usize].changed {
                return true;
            }
        }
        return false;
    }
    #[inline]
    fn can_go_down(&self, new_grid: &[u32; 4]) -> bool {
        for i in 0..4 {
            let column = Self::extract_column(new_grid, i);
            if self.table[Self::reverse_row(column) as usize].changed {
                return true;
            }
        }
        return false;
    }
    #[inline]
    pub fn is_lost(&self, grid: &[u32; 4]) -> bool {
        !(self.can_go_left(grid)
            || self.can_go_right(grid)
            || self.can_go_up(grid)
            || self.can_go_down(grid))
    }
    #[inline]
    pub fn get_possible_directions(&self, grid: &[u32; 4]) -> Vec<game::Direction> {
        let mut directions = Vec::with_capacity(4);
        if self.can_go_left(grid) {
            directions.push(game::Direction::Left);
        }
        if self.can_go_down(grid) {
            directions.push(game::Direction::Down);
        }
        if self.can_go_right(grid) {
            directions.push(game::Direction::Right);
        }
        if self.can_go_up(grid) {
            directions.push(game::Direction::Up);
        }
        directions
    }
    #[inline]
    pub fn make_move(&self, grid: &[u32; 4], direction: &game::Direction) -> ([u32; 4], u32) {
        let (new_grid, score) = match direction {
            game::Direction::Left => self.move_grid_left(grid),
            game::Direction::Right => self.move_grid_right(grid),
            game::Direction::Up => self.move_grid_up(grid),
            game::Direction::Down => self.move_grid_down(grid),
            game::Direction::None => return (*grid, 0),
        };
        (new_grid, score)
    }
    #[inline]
    pub fn empty_list(grid: &[u32; 4]) -> Vec<(usize, usize)> {
        let mut empty = Vec::with_capacity(16);
        for (i, &row) in grid.iter().enumerate() {
            if row & 0x0000001F == 0 {
                empty.push((i, 0));
            }
            if row & 0x00003E0 == 0 {
                empty.push((i, 1));
            }
            if row & 0x0007C00 == 0 {
                empty.push((i, 2));
            }
            if row & 0x00F8000 == 0 {
                empty.push((i, 3));
            }
        }
            empty
    }

    pub fn place_block(&self, grid: [u32; 4], pos: (usize, usize), value: u32) -> [u32; 4] {
        let mut new_grid = grid;
        new_grid[pos.0] |= value << (pos.1 * 5);
        return new_grid
    }

    pub fn add_random_block(&self, grid: [u32; 4], rand: &Random) -> [u32;4] {
        // Adds a block of random value at a random place
        let empty = FastGame::empty_list(&grid);
        if empty.len() == 0 {
            return grid;
        }
        let value: u8 = if rand.gen::<f32>() < 0.9 { 1 } else { 2 };
        let index = (empty.len() as f32 * rand.gen::<f32>()) as usize;
        let pos = empty[index];
        return self.place_block(grid, pos, value as u32);
    }

    pub fn to_flat_array(grid: [u32;4]) -> [u8; 16] {
        let mut flat = [0; 16];
        for i in 0..4 {
            for j in 0..4 {
                flat[i * 4 + j] = ((grid[i] >> ((3-j) * 5)) & 0x1F) as u8;
            }
        }
        return flat;
    }

    pub fn play_move(&self, mut grid: [u32; 4], direction: game::Direction, rand: &Random) -> ([u32; 4],u32) {
        if direction == game::Direction::None {
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
