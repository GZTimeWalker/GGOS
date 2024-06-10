#![no_std]
#![no_main]

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use lib::*;

extern crate alloc;
extern crate lib;

// tic-tac-toe game

#[derive(PartialEq, Eq)]
enum CellState {
    Empty,
    Player,
    Computer,
}

fn main() -> isize {
    let mut state = [
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
        CellState::Empty,
    ];

    println!("Let's play Tic-Tac-Toe!");

    let rng = lib::Random::new();

    loop {
        // draw game board
        draw_board(&state);

        // player takes turn
        let guess = get_player_input(&state);
        if guess == 10 {
            println!("Invalid choice");
            continue;
        }

        state[guess] = CellState::Player;

        // check for player win
        if check_win(&state, CellState::Player) {
            println!("\nPLAYER WINS!");
            break;
        }

        // check for draw
        if check_draw(&state) {
            println!("\nDRAW!");
            break;
        }

        // computer takes turn
        get_computer_input(&mut state, &rng);

        // check for computer win
        if check_win(&state, CellState::Computer) {
            println!("\nCOMPUTER WINS!");
            break;
        }
    }

    // draw final game board
    draw_board(&state);

    0
}

fn check_win(state: &[CellState; 9], who: CellState) -> bool {
    // check rows
    for row in 0..3 {
        let idx: usize = row * 3;
        if who == state[idx] && who == state[idx + 1] && who == state[idx + 2] {
            return true;
        }
    }

    // check columns
    for col in 0..3 {
        let idx: usize = col;
        if who == state[idx] && who == state[idx + 3] && who == state[idx + 6] {
            return true;
        }
    }

    // check diagonals
    if who == state[4]
        && ((who == state[0] && who == state[8]) || (who == state[2] && who == state[6]))
    {
        return true;
    }

    // failed
    false
}

fn get_computer_input(state: &mut [CellState; 9], rng: &Random) {
    let mut options: Vec<usize> = Vec::new();

    for (i, s) in state.iter().enumerate() {
        if *s == CellState::Empty {
            options.push(i);
        }
    }

    let sel = rng.next_u32() as usize % options.len();

    if let CellState::Empty = state[options[sel]] {
        state[options[sel]] = CellState::Computer
    }
}

fn check_draw(state: &[CellState; 9]) -> bool {
    for s in state.iter() {
        if *s == CellState::Empty {
            return false;
        }
    }

    true
}

fn draw_board(state: &[CellState; 9]) {
    let mut board = Vec::new();

    for (i, s) in state.iter().enumerate() {
        match s {
            CellState::Player => board.push(String::from("X")),
            CellState::Computer => board.push(String::from("O")),
            CellState::Empty => board.push((i + 1).to_string()),
        }
    }

    println!("/-----------\\");
    println!("| {} | {} | {} |", board[0], board[1], board[2]);
    println!("|---|---|---|");
    println!("| {} | {} | {} |", board[3], board[4], board[5]);
    println!("|---|---|---|");
    println!("| {} | {} | {} |", board[6], board[7], board[8]);
    println!("\\-----------/");
}

fn get_player_input(state: &[CellState; 9]) -> usize {
    print!("Enter the # for your choice (X): ");

    let guess = lib::stdin().read_line();

    let guess = guess.as_bytes()[0];
    if !(49..=57).contains(&guess) {
        return 10; // invalid choice
    }

    let guess: usize = (guess - 49).into();

    if state[guess] == CellState::Empty {
        return guess;
    }

    10 // invalid choice
}

entry!(main);
