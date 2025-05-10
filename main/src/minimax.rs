use crate::{fastgame::FastGame, game::is_lost};
use rayon::prelude::*;
use crate::game;
use std::collections::HashMap;

struct TTEntryMini {
        depth: usize,
        value: f32,
        flag: NodeType,
}

struct TTEntryExpecti {
    depth : usize,
    value : f32,
}

enum NodeType {
    Exact,
    Lowerbound,
    Upperbound,
}

pub fn get_best_direction_minimax(game: &FastGame, grid: [u32; 4], search_depth: usize) -> game::Direction {
    // Returns the direction with the best minimax evaluation
    let best_direction = game.get_possible_directions(&grid)
        .par_iter()
        .map(|direction| {
            // Create a thread-local transposition table for each parallel evaluation
            let mut tt = HashMap::new();
            let (new_grid, _) = game.make_move(&grid, &direction);
            let score = minimax(
                game,
                new_grid,
                search_depth,
                false,
                f32::NEG_INFINITY,
                f32::INFINITY,
                &mut tt,
            );
            (direction, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(direction, _)| direction)
        .unwrap_or(&game::Direction::None).clone();
        return best_direction;
}


pub fn get_best_direction_expectimax(game: &FastGame, grid: [u32; 4], search_depth: usize) -> game::Direction {
    // Returns the direction with the best expectimax evaluation
    let best_direction = game.get_possible_directions(&grid)
        .par_iter()
        .map(|direction| {
            // Create a thread-local transposition table for each parallel evaluation
            let mut tt = HashMap::new();
            let (new_grid, _) = game.make_move(&grid, &direction);
            let score = expectimax(
                game,
                new_grid,
                search_depth,
                false,
                &mut tt,
                0,
            );
            (direction, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(direction, _)| direction)
        .unwrap_or(&game::Direction::None).clone();
        return best_direction;
}

pub fn evaluate(grid: [u32; 4]) -> f32 {
    let flat_grid = FastGame::to_flat_array(grid);

    let big_values_infl:f32 = flat_grid.iter().map(|&value| {(1 << value) as f32}).sum();

    // Monotonicity: measure how aligned tiles are in a single direction
    let _monotonicity_horizontal = 
        (0..4).map(|row| {
            let start = row * 4;
            let row_values = &flat_grid[start..start+4];
            row_values.windows(2)
                .map(|w| (w[1] as f32 - w[0] as f32).abs())
                .sum::<f32>()
        }).sum::<f32>();

    let _monotonicity_vertical = 
        (0..4).map(|col| {
            let column_values = [
                flat_grid[col],
                flat_grid[col + 4],
                flat_grid[col + 8],
                flat_grid[col + 12]
            ];
            column_values.windows(2)
                .map(|w| (w[1] as f32 - w[0] as f32).abs())
                .sum::<f32>()
        }).sum::<f32>();

    // Smoothness: measure the difference between adjacent tiles
    let smoothness_horizontal = 
        (0..4).map(|row| {
            let start = row * 4;
            let row_values = &flat_grid[start..start+4];
            row_values.windows(2)
                .map(|w| ((2.0_f32).powf(w[0] as f32) - (2.0_f32).powf(w[1] as f32)).abs())
                .sum::<f32>()
        }).sum::<f32>();

    let smoothness_vertical = 
        (0..4).map(|col| {
            let column_values = [
                flat_grid[col],
                flat_grid[col + 4],
                flat_grid[col + 8],
                flat_grid[col + 12]
            ];
            column_values.windows(2)
                .map(|w| ((2.0_f32).powf(w[0] as f32) - (2.0_f32).powf(w[1] as f32)).abs())
                .sum::<f32>()
        }).sum::<f32>();

    // Empty cells bonus
    let empty_cells_bonus = FastGame::empty_list(&grid).len() as f32;
    return 10.0*empty_cells_bonus
           + 10.0*big_values_infl
           - 1.0*smoothness_vertical
           - 1.0*smoothness_horizontal;
}

pub fn _basile_heuristique(grid: [u32; 4]) -> f32 {
    // First, compute the grid sorted in a zigzag towards the up left
    let flat_grid = FastGame::to_flat_array(grid);
    if is_lost(&flat_grid) {return -1000.0}
    let mut sorted_grid = flat_grid.clone();
    sorted_grid.sort();
    sorted_grid.reverse();
    let zig_zig_index_mapping:[usize;16] = [0,1,2,3,7,6,5,4,8,9,10,11,15,14,13,12];
    let mut remapped_sorted_grid = [0;16];
    for (old_index,new_index) in zig_zig_index_mapping.iter().enumerate() {
        remapped_sorted_grid[old_index] = sorted_grid[*new_index];
    };
    // Then, compare the real grid with the sorted grid
    let mut total_reward = 0.0;
    for i in 0..16 {
        let real_value = flat_grid[i];
        if real_value == sorted_grid[i] {
            total_reward += (real_value as f32).powf(4.0);
        } else {break}
    }
    return total_reward;
}

fn minimax(
    game: &FastGame,
    grid: [u32; 4],
    depth: usize,
    is_player: bool,
    mut alpha: f32,
    mut beta: f32,
    tt: &mut HashMap<[u32; 4], TTEntryMini>,
) -> f32 {
    // Returns the minimax value of the board with a grid that has been moved in the direction but no block added

    // Check if the grid is in the transposition table
    if let Some(entry) = tt.get(&grid) {
        if entry.depth >= depth {
            match entry.flag {
                NodeType::Exact => return entry.value,
                NodeType::Lowerbound => alpha = alpha.max(entry.value),
                NodeType::Upperbound => beta = beta.min(entry.value),
            }
            if alpha >= beta {
                return entry.value;
            }
        }
    }
    // Store the original alpha-beta for determining the node type
    let original_alpha = alpha;
    let original_beta = beta;

    // If node is final, return its evaluation
    if game.is_lost(&grid){
        return evaluate(grid) + f32::NEG_INFINITY;
    }
    if depth == 0 {
        return evaluate(grid);
    }

    let mut value;
    let flag;

    // If node is player, playout all the potential moves
    if is_player {
        value = f32::NEG_INFINITY;
        for direction in game.get_possible_directions(&grid) {
            let (new_grid, _score) = game.make_move(&grid, &direction);
            value = value.max(minimax(game, new_grid, depth - 1, false, alpha, beta, tt));
            alpha = alpha.max(value);
            if alpha >= beta {
                // Beta cutoff
                break;
            }
        }
    }
    // If the node is a block spawn, playout all the possible spawns
    else {
        value = f32::INFINITY;
        for empty in FastGame::empty_list(&grid) {
            // Spawn a 2
            let new_grid = game.place_block(grid, empty, 1);
            value = value.min(minimax(game, new_grid, depth - 1, true, alpha, beta, tt));
            beta = beta.min(value);
            if beta <= alpha {
                // Alpha cutoff
                break;
            }
            // Spawn a 4
            let new_grid = game.place_block(grid, empty, 2);
            value = value.min(minimax(game, new_grid, depth - 1, true, alpha, beta, tt));
            let beta = beta.min(value);
            if beta <= alpha {
                // Alpha cutoff
                break;
            }
        }
    }

    // Determine node type
    if value <= original_alpha {
        flag = NodeType::Upperbound;
    } else if value >= original_beta {
        flag = NodeType::Lowerbound;
    } else {
        flag = NodeType::Exact;
    }

    tt.insert(grid, TTEntryMini { depth, value, flag });

    return value;
}

fn expectimax(
    game: &FastGame,
    grid: [u32; 4],
    depth: usize,
    is_player: bool,
    tt: &mut HashMap<[u32; 4], TTEntryExpecti>,
    branch_score: u32,
) -> f32 {
    if let Some(entry) = tt.get(&grid) {
        if entry.depth >= depth {    
            return entry.value;
        }
    }


    if game.is_lost(&grid) {
        return -1000.0;
    }
    if depth <= 0 {
        return evaluate(grid) + branch_score as f32;
    }
    let value:f32;
    if is_player {
        // Player's turn: maximize over possible moves
        value = game.get_possible_directions(&grid)
            .iter()  // Use Rayon's parallel iterator
            .map(|direction| {
                let (new_grid, score) = game.make_move(&grid, &direction);
                expectimax(game, new_grid, depth - 1, false, tt, branch_score + score)
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    } else {
        // Block spawn turn: calculate expected value
        let empty_cells = FastGame::empty_list(&grid);
        let total_cells = empty_cells.len();
        
        value = empty_cells.iter()
            .flat_map(|&empty| [
                // Probability of 2 spawn (90%)
                expectimax(game, game.place_block(grid, empty, 1), depth - 1, true, tt, branch_score) * 0.9,
                // Probability of 4 spawn (10%)
                expectimax(game, game.place_block(grid, empty, 2), depth - 1, true, tt, branch_score) * 0.1
            ])
            .sum::<f32>() / (total_cells * 2) as f32
    }
    tt.insert(grid, TTEntryExpecti {depth, value});
    return value;
}