use crate::game;
use crate::fastgame::FastGame;
use std::collections::HashMap;

struct TTEntry {
	depth: usize,
	value: f32,
	flag:  NodeType,
}

enum NodeType {
	Exact,
	Lowerbound,
	Upperbound,
}

fn get_best_direction(game: &FastGame, grid:[u32;4], search_depth:usize) -> game::Direction {
	// Returns the direction with the best minimax evaluation

	let mut best_direction = game::Direction::None;
	let mut best_score = f32::NEG_INFINITY;
	let mut tt = HashMap::new();

	// Try every possible direction
	for direction in game.get_possible_directions(grid) {
		let (new_grid, _) = game.make_move(grid, &direction);
		let score = minimax(game, new_grid, search_depth, false, f32::NEG_INFINITY, f32::INFINITY, &mut tt);
		if score > best_score {
			best_score = score;
			best_direction = direction;
		}
	}
	return best_direction;
}

fn evaluate(grid:[u32;4]) -> f32 {
	// Temporary function to have functional code
	let mut total = 0.0;
	for row in grid {
		total += row as f32
	}
	total
}

fn minimax(game:&FastGame, grid:[u32;4], depth:usize, is_player:bool, mut alpha: f32, mut beta:f32, tt: &mut HashMap<[u32;4], TTEntry>) -> f32 {
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
	if depth == 0 || game.is_lost(grid) {
		return evaluate(grid);
	}

	let mut value;
	let flag;

	// If node is player, playout all the potential moves
	if is_player {
		value = f32::NEG_INFINITY;
		for direction in game.get_possible_directions(grid) {
			let (new_grid, _score) = game.make_move(grid, &direction);
			value = value.max(minimax(game, new_grid, depth - 1, false, alpha, beta, tt));
			let alpha = alpha.max(value);
			if alpha >= beta {
				// Beta cutoff
				break;
			}
			
		}
	}

	// If the node is a block spawn, playout all the possible spawns
	else {
		value = f32::INFINITY;
		for empty in game.empty_list(grid) {
			// Spawn a 2
			let new_grid = game.place_block(grid, empty, 1);
			value = value.min(minimax(game,new_grid, depth - 1, true, alpha, beta, tt));
			let beta = beta.min(value);
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
	}
	else if value >= original_beta {
		flag = NodeType::Lowerbound;
	}
	else {
		flag = NodeType::Exact;
	}

	tt.insert(grid, TTEntry {depth, value, flag});

	return value;

}