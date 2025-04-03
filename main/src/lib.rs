/*#![feature(test)]

extern crate test;
use seeded_random::{Random, Seed};
mod fastgame;
mod game;

fn barrel_roll(fast: &fastgame::FastGame, mut grid:[u32;4], random:&Random) -> ([u32;4],u32) {
    let mut score = 0;
    let mut move_score;
    for _ in 0..1_000 {
        (grid,score) = fast.move_grid_left(&grid);
        (grid,move_score) = fast.move_grid_up(&grid);
        score += move_score;
        (grid,move_score) = fast.move_grid_right(&grid);
        score += move_score;
        (grid,move_score) = fast.move_grid_down(&grid);
        score += move_score;
        grid = fast.add_random_block(grid, random)
    }
    return (grid,score);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_barrel_roll(b: &mut Bencher) {
        let random = Random::from_seed(Seed::unsafe_new(0));
        let fast = fastgame::FastGame::new();
        let grid = [1,0,0,0];
        b.iter(|| barrel_roll(&fast, grid, &random))
    }
}*/