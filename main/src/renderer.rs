use core::panic;

use crate::GRID_SIZE;

// Colored blocks
const COLORS: [&str; 7] = [
    "\x1b[41m",
    "\x1b[42m",
    "\x1b[43m",
    "\x1b[44m",
    "\x1b[45m",
    "\x1b[46m",
    "\x1b[47m",
];
const PIXEL: &str = "  ";
const BORDER_PIXEL: &str = "\x1b[30m▒▒";

const BLOCK_SIZE: usize = 1; // Real size in pixel : 3 + 2*BLOCK_SIZE


pub fn render(game_state: [u8; GRID_SIZE*GRID_SIZE]) {
    let mut data: Vec<String> = vec![];

    // Top border
    data.push(format!("{}{}\x1b[0m", COLORS[6], PIXEL.repeat((3 + 2*BLOCK_SIZE)*GRID_SIZE)));

    for i in 0..GRID_SIZE {
        let mut line: Vec<String> = create_block(game_state[i*GRID_SIZE], BLOCK_SIZE);
        for j in 1..GRID_SIZE {
            line = hlink(line, create_block(game_state[i*GRID_SIZE + j], BLOCK_SIZE));
        }
        data = vlink(data, line);
    }

    // Bottom border
    data.push(format!("{}{}\x1b[0m", COLORS[6], PIXEL.repeat((3 + 2*BLOCK_SIZE)*GRID_SIZE)));

    // Side borders
    let border: Vec<String> = vec![format!("{}{}\x1b[0m", COLORS[6], PIXEL); (3 + 2*BLOCK_SIZE)*GRID_SIZE + 2];
    data = hlink(border.clone(), data);
    data = hlink(data, border);

    draw(data);
}


fn create_block(value: u8, size: usize) -> Vec<String> {
    if value == 0 {
        let mut block = vec![];
        for _ in 0..(3 + 2*size) {
            block.push(PIXEL.repeat(3 + 2*size));
        }
        return block;
    }
    let mut block = vec![];
    let color = COLORS[(value+1) as usize % 6];
    let number = format!("{}",value);
    let length = number.len();

    // Top border
    block.push(format!("{}{}\x1b[0m",color, BORDER_PIXEL.repeat(3 + 2*size)));
    // First half
    for _ in 0..size {
        block.push(format!("{}{}{}{}\x1b[0m", color, BORDER_PIXEL, PIXEL.repeat(1 + 2*size), BORDER_PIXEL));
    }
    // Middle
    block.push(format!("{}{}{}{}{}{}\x1b[0m", color, BORDER_PIXEL, " ".repeat((4*size + 2 - length) / 2), number, " ".repeat((4*size + 2) - ((4*size + 2 - length) / 2) - length), BORDER_PIXEL));
    // Second half
    for _ in 0..size {
        block.push(format!("{}{}{}{}\x1b[0m", color, BORDER_PIXEL, PIXEL.repeat(1 + 2*size), BORDER_PIXEL));
    }
    // Bottom border
    block.push(format!("{}{}\x1b[0m",color, BORDER_PIXEL.repeat(3 + 2*size)));
    
    return block;
}


fn hlink(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    if a.len() != b.len() {
        panic!("The two blocks must have the same size");
    }
    let mut result = vec![];
    for i in 0..a.len() {
        result.push(format!("{}{}", a[i], b[i]));
    }
    return result;
}

fn vlink(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let mut result = a.clone();
    for i in 0..b.len() {
        result.push(b[i].clone());
    }
    return result;
}


fn draw(data: Vec<String>) {
    // Clear the terminal
    print!("\x1b[2J\x1b[1;1H");
    for line in data {
        println!("{}", line);
    }
}