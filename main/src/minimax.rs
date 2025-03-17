

use crate::fastgame::FastGame;

fn get_best_direction() {
	// evaluate every direction with minimax algorithm
}

fn evaluate(grid:[u32;4]) -> f32 {
	let mut total = 0.0;
	for row in grid {
		total += row as f32
	}
	total
}

fn minimax(game:&FastGame, grid:[u32;4], depth:usize, is_player:bool) -> f32 {
	// Evaluate the board with minimax with a grid that has been moved in the direction but no block added
	// Return the score of the board
	if depth == 0 || game.is_lost(grid) {
		return evaluate(grid);
	}
	if is_player {
		// Compute all the possible directions
		let mut value:f32 = f32::NEG_INFINITY;
		for direction in game.get_possible_directions(grid) {
			value = value.max(minimax(game, game.make_move(grid, direction).0, depth-1, false) as f32);
		}
		value
	}
	else {
		// Compute all possible block appearances
		let mut value:f32 = f32::INFINITY;
		for empty in game.empty_list(grid) {
			value = value.min(minimax(game, game.place_block(grid, empty, 1), depth-1, true) as f32);
			value = value.min(minimax(game, game.place_block(grid, empty, 2), depth-1, true) as f32);
		}
		value
	}
}