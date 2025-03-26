use core::panic;

use crate::GRID_SIZE;

// Colored blocks
/*const COLORS: [&str; 7] = [
    "\x1b[41m", "\x1b[42m", "\x1b[43m", "\x1b[44m", "\x1b[45m", "\x1b[46m", "\x1b[47m",
];*/
// RGB colors
const COLORS: [&str; 18] = [
    "\x1b[48;2;255;0;0m", "\x1b[48;2;255;127;0m", "\x1b[48;2;255;255;0m", "\x1b[48;2;127;255;0m", "\x1b[48;2;0;255;0m", "\x1b[48;2;0;255;127m", "\x1b[48;2;0;255;255m", "\x1b[48;2;0;127;255m", "\x1b[48;2;0;0;255m", "\x1b[48;2;127;0;255m", "\x1b[48;2;255;0;255m", "\x1b[48;2;255;0;127m", "\x1b[48;2;255;255;255m", "\x1b[48;2;127;127;127m", "\x1b[48;2;0;0;0m", "\x1b[48;2;255;255;255m", "\x1b[48;2;127;127;127m", "\x1b[47m"
];
/*const COLORS: [&str; 14] = [
    "\x1b[48;2;0;0;139m", "\x1b[48;2;0;51;111m", "\x1b[48;2;0;102;83m", "\x1b[48;2;0;153;56m", "\x1b[48;2;0;204;28m", "\x1b[48;2;0;255;0m", "\x1b[48;2;64;255;0m", "\x1b[48;2;128;255;0m", "\x1b[48;2;255;255;0m", "\x1b[48;2;255;210;0m", "\x1b[48;2;255;165;0m", "\x1b[48;2;239;171;128m", "\x1b[48;2;224;176;255m", "\x1b[47m"
];*/


const PIXEL: &str = "  ";
const BORDER_PIXEL: &str = "\x1b[30m▒▒";

const BLOCK_SIZE: usize = 1; // Real size in pixel : 3 + 2*BLOCK_SIZE

pub fn render(game_state: [u8; GRID_SIZE * GRID_SIZE]) {
    let mut data: Vec<String> = vec![];

    // Top border
    data.push(format!(
        "{}{}\x1b[0m",
        COLORS[COLORS.len()-1],
        PIXEL.repeat((3 + 2 * BLOCK_SIZE) * GRID_SIZE)
    ));

    for i in 0..GRID_SIZE {
        let mut line: Vec<String> = create_block(game_state[i * GRID_SIZE], BLOCK_SIZE);
        for j in 1..GRID_SIZE {
            line = hlink(
                line,
                create_block(game_state[i * GRID_SIZE + j], BLOCK_SIZE),
            );
        }
        data = vlink(data, line);
    }

    // Bottom border
    data.push(format!(
        "{}{}\x1b[0m",
        COLORS[COLORS.len()-1],
        PIXEL.repeat((3 + 2 * BLOCK_SIZE) * GRID_SIZE)
    ));

    // Side borders
    let border: Vec<String> =
        vec![format!("{}{}\x1b[0m", COLORS[COLORS.len()-1], PIXEL); (3 + 2 * BLOCK_SIZE) * GRID_SIZE + 2];
    data = hlink(border.clone(), data);
    data = hlink(data, border);

    draw(data);
}

fn create_block(value: u8, size: usize) -> Vec<String> {
    if value == 0 {
        let mut block = vec![];
        for _ in 0..(3 + 2 * size) {
            block.push(PIXEL.repeat(3 + 2 * size));
        }
        return block;
    }
    let mut block = vec![];
    let color = COLORS[(value-1) as usize % COLORS.len()];
    let number = format!("{}", 1 << value);
    let length = number.len();

    // Top border
    block.push(format!(
        "{}{}\x1b[0m",
        color,
        BORDER_PIXEL.repeat(3 + 2 * size)
    ));
    // First half
    for _ in 0..size {
        block.push(format!(
            "{}{}{}{}\x1b[0m",
            color,
            BORDER_PIXEL,
            PIXEL.repeat(1 + 2 * size),
            BORDER_PIXEL
        ));
    }
    // Middle
    block.push(format!(
        "{}{}{}{}{}{}\x1b[0m",
        color,
        BORDER_PIXEL,
        " ".repeat((4 * size + 2 - length) / 2),
        number,
        " ".repeat((4 * size + 2) - ((4 * size + 2 - length) / 2) - length),
        BORDER_PIXEL
    ));
    // Second half
    for _ in 0..size {
        block.push(format!(
            "{}{}{}{}\x1b[0m",
            color,
            BORDER_PIXEL,
            PIXEL.repeat(1 + 2 * size),
            BORDER_PIXEL
        ));
    }
    // Bottom border
    block.push(format!(
        "{}{}\x1b[0m",
        color,
        BORDER_PIXEL.repeat(3 + 2 * size)
    ));

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
