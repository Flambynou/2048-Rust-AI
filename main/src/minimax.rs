use crate::fastgame::FastGame;
use rayon::prelude::*;
use crate::game;
use std::collections::HashMap;

struct TTEntry {
    depth: usize,
    value: f32,
    flag: NodeType,
}

enum NodeType {
    Exact,
    Lowerbound,
    Upperbound,
}

pub fn get_best_direction(game: &FastGame, grid: [u32; 4], search_depth: usize) -> game::Direction {
    // Returns the direction with the best minimax evaluation

    let mut best_direction = game::Direction::None;
    let mut _best_score = f32::NEG_INFINITY;
    let mut _tt: HashMap<[u32;4],TTEntry> = HashMap::new();

    // Try every possible direction
    best_direction = game.get_possible_directions(&grid)
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
            );
            (direction, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(direction, _)| direction)
        .unwrap_or(&game::Direction::None).clone();
        return best_direction;
}

fn evaluate(game: &FastGame, grid: [u32; 4]) -> f32 {
    let mut total = 0.0;
    let flat_array = game.to_flat_array(grid);
    for value in flat_array {
        if value != 0 {
            total += ((1 << value) as f32) * (value as f32);
        }
    }
    return total;
}

fn minimax(
    game: &FastGame,
    grid: [u32; 4],
    depth: usize,
    is_player: bool,
    mut alpha: f32,
    mut beta: f32,
    tt: &mut HashMap<[u32; 4], TTEntry>,
    branch_score: usize,
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
        return branch_score as f32;
    }
    if depth == 0 {
        return branch_score as f32 * (game.empty_list(&grid).len() as f32 + 1.0).powf(4.0);
    }

    let mut value;
    let flag;

    // If node is player, playout all the potential moves
    if is_player {
        value = f32::NEG_INFINITY;
        for direction in game.get_possible_directions(&grid) {
            let (new_grid, score) = game.make_move(&grid, &direction);
            value = value.max(minimax(game, new_grid, depth - 1, false, alpha, beta, tt, branch_score + score as usize));
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
        for empty in game.empty_list(&grid) {
            // Spawn a 2
            let new_grid = game.place_block(grid, empty, 1);
            value = value.min(minimax(game, new_grid, depth - 1, true, alpha, beta, tt, branch_score));
            beta = beta.min(value);
            if beta <= alpha {
                // Alpha cutoff
                break;
            }
            // Spawn a 4
            let new_grid = game.place_block(grid, empty, 2);
            value = value.min(minimax(game, new_grid, depth - 1, true, alpha, beta, tt, branch_score));
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

    tt.insert(grid, TTEntry { depth, value, flag });

    return value;
}

fn expectimax(
    game: &FastGame,
    grid: [u32; 4],
    depth: usize,
    is_player: bool,
    tt: &mut HashMap<[u32; 4], TTEntry>,
) -> f32 {
    if let Some(entry) = tt.get(&grid) {
        if entry.depth >= depth {    
            return entry.value;
        }
    }


    if game.is_lost(&grid) {
        return f32::NEG_INFINITY;
    }
    if depth <= 0 {
        return evaluate(game, grid);
    }

    if is_player {
        // Player's turn: maximize over possible moves
        game.get_possible_directions(&grid)
            .iter()  // Use Rayon's parallel iterator
            .map(|direction| {
                let (new_grid, _score) = game.make_move(&grid, &direction);
                expectimax(game, new_grid, depth - 1, false, tt)
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(f32::NEG_INFINITY)
    } else {
        // Block spawn turn: calculate expected value
        let empty_cells = game.empty_list(&grid);
        let total_cells = empty_cells.len();
        
        empty_cells.iter()
            .flat_map(|&empty| [
                // Probability of 2 spawn (90%)
                expectimax(game, game.place_block(grid, empty, 1), depth - 1, true, tt) * 0.9,
                // Probability of 4 spawn (10%)
                expectimax(game, game.place_block(grid, empty, 2), depth - 1, true, tt) * 0.1
            ])
            .sum::<f32>() / (total_cells * 2) as f32
    }
}