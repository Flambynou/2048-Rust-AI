use crate::fastgame;
use crate::game;
use rand::rngs::ThreadRng;
use rand::Rng;

const EXPLORATION_CONSTANT:f32 = 1.4142;

#[derive(Clone)]
enum NodeType {
    Spawn(SpawnNode),
    Move(MoveNode),
}
#[derive(Clone)]
struct SpawnNode {
    GameState: [u32;4],
    Score: u32,
    Parent: Option<Box<NodeType>>,
    Children: Vec<Box<NodeType>>,
    MoveMade: game::Direction,
    VisitCount: f32,
    TotalValue: f32,
    IsTerminal: bool,
}
#[derive(Clone)]
struct MoveNode {
    GameState: [u32;4],
    Score: u32,
    Parent: Option<Box<NodeType>>,
    Children: Vec<Box<NodeType>>,
    ActionsLeft: Vec<game::Direction>,
    VisitCount: f32,
}


fn move_imediate_score_policy(fast: &fastgame::FastGame, grid: [u32;4], score: u32) -> ([u32;4],u32,game::Direction) {
	if fast.is_lost(&grid) {
		return (grid,score,game::Direction::None); 
	}
	let (new_grid,move_score,direction) = fast.get_possible_directions(&grid)
        .iter()
        .map(|direction| {
            let (new_grid, move_score) = fast.make_move(&grid, &direction);
            (new_grid, move_score, direction.clone())
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap_or((grid,score,game::Direction::None)).clone();
    return (new_grid,score+move_score,direction);
}



pub struct MonteCarloTree {
    Root: Box<NodeType>,
}

impl MonteCarloTree {
    pub fn new(fast: &fastgame::FastGame, rootstate:[u32;4], score: u32) -> MonteCarloTree {
        let rootnode = NodeType::Move(MoveNode{
            GameState: rootstate,
            Score: score,
            Parent: None,
            Children: Vec::new(),
            ActionsLeft: fast.get_possible_directions(&rootstate),
            VisitCount: 0.0,
        });
        MonteCarloTree { Root: Box::new(rootnode)}
    }
    fn make_subtree_main(mut self, new_root: NodeType) {
        match new_root.clone() {
            NodeType::Spawn(_) => {
                println!{"Spawn node cannot be root"}
            }
            NodeType::Move(mut move_node) => {
                move_node.Parent = None;
                self.Root = Box::new(new_root);
            }
        }
    }
    fn backpropagate(&mut self, start_node: NodeType, score: f32) {
        let mut current_node = Some(start_node);
        while let Some(node) = current_node {
            match node {
                NodeType::Spawn(mut spawn_node) => {
                    spawn_node.TotalValue += score;
                    spawn_node.VisitCount += 1.0;
                    if let Some(parent_box) = spawn_node.Parent {
                        current_node = Some(parent_box.as_ref().clone());
                    } else {
                        current_node = None;
                    }
                }
                NodeType::Move(mut move_node) => {
                    move_node.VisitCount += 1.0;
                    if let Some(parent_box) = &move_node.Parent {
                        current_node = Some(parent_box.as_ref().clone());
                    } else {
                        current_node = None;
                    }
                }
            }
        }
    }
    fn expand(&self, fast: &fastgame::FastGame, node: NodeType) -> Vec<NodeType> {
        let mut new_nodes = Vec::new();
        match node {
            NodeType::Spawn(mut spawn_node) => {
                // Add all possible block spawns, with either 2 or 4 as value
                for value in [2u32,4u32].iter() {
                    for empty_space in fastgame::FastGame::empty_list(&spawn_node.GameState) {
                        let new_state = fast.place_block(spawn_node.GameState, empty_space, *value);
                        let new_child = NodeType::Move(MoveNode {
                            GameState: new_state,
                            Score: spawn_node.Score,
                            Parent: Some(Box::new(NodeType::Spawn(spawn_node.clone()))),
                            Children: Vec::new(),
                            ActionsLeft: fast.get_possible_directions(&new_state),
                            VisitCount: 0.0,
                        });
                        spawn_node.Children.push(Box::new(new_child.clone()));
                        new_nodes.push(new_child);
                    }
                }
            },
            NodeType::Move(mut move_node) => {
                // Add the child according to the policy (optional, could be random)
                let (state, score, directiontaken) = move_imediate_score_policy(&fast, move_node.GameState, move_node.Score);
                let new_child = NodeType::Spawn(SpawnNode {
                    GameState: state,
                    Score: score,
                    Parent: Some(Box::new(NodeType::Move(move_node.clone()))),
                    Children: Vec::new(),
                    MoveMade: directiontaken.clone(),
                    VisitCount: 0.0,
                    TotalValue: 0.0,
                    IsTerminal: fast.is_lost(&state),
                });
                move_node.Children.push(Box::new(new_child.clone()));
                new_nodes.push(new_child);
                move_node.ActionsLeft.retain(|direction| direction != &directiontaken);
            },
        }
        return new_nodes;
    }

    fn selection(&self, rng: &mut ThreadRng) -> NodeType {
        let mut current_node = self.Root.as_ref();
        loop {
            match current_node {
                NodeType::Spawn(spawn_node) => {
                    if spawn_node.IsTerminal || spawn_node.Children.len() == 0 {
                        return current_node.clone();
                    }
                    else {
                        let value_change_index = spawn_node.Children.len() / 2;
                        let two_probability = 1.0 / (value_change_index as f32 * 0.9);
                        let four_probability = 1.0 / (value_change_index as f32 * 0.1);
                        let mut prob_vec = vec![0.0];
                        for i in 0..(value_change_index * 2) {
                            if i < value_change_index {
                                prob_vec.push(prob_vec[i-1]+two_probability);
                            }
                            else {
                                prob_vec.push(prob_vec[i-1]+four_probability);
                            }
                        }
                        let mut index = 0;
                        let random_float = rng.random::<f32>();
                        for i in 0..(value_change_index*2) {
                            if prob_vec[i] > random_float {
                                index = i-1;
                                break;
                            }
                        }
                        current_node = spawn_node.Children[index].as_ref();
                    }
                }
                NodeType::Move(move_node) => {
                    if move_node.ActionsLeft.len() > 0 {
                        return current_node.clone();
                    }
                    let uct_values = move_node.Children.iter()
                        .map(|child| {
                            let child = child.as_ref();
                            let uct_value = match child {
                                NodeType::Spawn(spawn_node) => {
                                    (spawn_node.TotalValue/spawn_node.VisitCount) + EXPLORATION_CONSTANT * (move_node.VisitCount.ln()/spawn_node.VisitCount).sqrt()
                                }
                                _ => {println!("A move is a child of move"); 0.0}
                            };
                            (uct_value, child)
                        });
                    current_node = uct_values.max_by(|a,b| a.0.partial_cmp(&b.0).unwrap()).map(|(_, child)| child).unwrap();
                }
            }
        }
    }

    fn rollout(fast: &fastgame::FastGame, node: NodeType, rng: &mut ThreadRng) -> f32 {
        let (mut game_state, current_score) = match node {
            NodeType::Spawn(spawn_node) => (spawn_node.GameState, spawn_node.Score),
            NodeType::Move(move_node) => (move_node.GameState, move_node.Score),
        };
        let mut score = current_score;
        while !fast.is_lost(&game_state) {
            (game_state, score, _) = move_imediate_score_policy(&fast, game_state, score);
            let empty_list = fastgame::FastGame::empty_list(&game_state);
            let value = if rng.random::<f32>() > 0.9 {4} else {2};
            let coords = empty_list[rng.random_range(0..empty_list.len())];
            game_state = fast.place_block(game_state, coords, value)
        }
        return score as f32;
    }

    pub fn get_best_direction(&mut self, fast: &fastgame::FastGame, iteration_count: usize) -> game::Direction {
        let mut rng = rand::rng();
        for _ in 0..iteration_count {
            let new_nodes = self.expand(&fast, self.selection(&mut rng));
            for new_node in new_nodes.iter() {
                self.backpropagate(new_node.clone(), Self::rollout(&fast, new_node.clone(), &mut rng));
            }
        }
        return match self.Root.as_ref() {
            NodeType::Spawn(_) => {println!{"Root is spawn type node, not normal"}; game::Direction::None},
            NodeType::Move(move_node) => {
                move_node.Children.iter()
                    .map(|child| {
                        let child = child.as_ref();
                        match child {
                            NodeType::Spawn(spawn_node) => {
                                (spawn_node.TotalValue, spawn_node.MoveMade.clone())
                            },
                            _ => {println!("A move is a child of root"); (0.0,game::Direction::None)},
                        }
                    })
                    .max_by(|a,b| a.0.partial_cmp(&b.0).unwrap())
                    .map(|(_, direction)| direction)
                    .unwrap()
            }
        };
    }
}