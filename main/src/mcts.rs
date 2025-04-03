use crate::fastgame;
use crate::game;
use seeded_random::{Random,Seed};


fn imediate_score_policy(fast: &fastgame::FastGame, grid: [u32;4], score: u32, rand:&Random) -> ([u32;4],u32) {
	if fast.is_lost(&grid) {
		return (grid,score); 
	}
	let (new_grid,move_score) = fast.get_possible_directions(&grid)
        .iter()
        .map(|direction| {
            let (new_grid, move_score) = fast.play_move(grid, direction.clone(), rand);
            (new_grid, move_score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap_or((grid,score)).clone();
    return (new_grid,score+move_score);
}

pub fn loop_policy(fast: fastgame::FastGame) -> ([u32;4],u32) {
    let rand = Random::from_seed(Seed::unsafe_new(2));
    let mut game_state = [0;4];
    game_state = fast.add_random_block(game_state, &rand);
    game_state = fast.add_random_block(game_state, &rand);
    let mut score = 0;
    while !fast.is_lost(&game_state) {
        (game_state, score) = imediate_score_policy(&fast, game_state, score, &rand);
    }
    return (game_state,score);
}