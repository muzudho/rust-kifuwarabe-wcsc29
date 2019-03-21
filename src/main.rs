/// extern crate は main.rs か lib.rs に入れる。
/// 参考: https://github.com/serde-rs/json |シリアライズ、デシリアライズ。
extern crate serde_json;

use std::io;

mod address;
mod board;
mod communication;
mod fen;
mod logical_move;
mod logical_record;
mod parser;
mod physical_move;
mod physical_record;
mod position_file;
mod record_converter;
mod thought;

use address::*;
use board::*;
use communication::*;
use fen::*;
// use logical_move::*;
use logical_record::*;
use physical_move::*;
use physical_record::*;
use record_converter::*;
use thought::Thought;

/// My name is Kifuwarabe.
/// I am a computer shogi engine.
/// I will go to WCSC29 this year.
///
/// Let's explain how to use me.
/// 
/// Windows 10.
/// 
/// [Windows] + [R].
/// `cmd`, [Enter].
/// 
/// ```Shell
/// ### Example.
/// cd C:\muzudho\projects_rust\rust-kifuwarabe-wcsc29
/// cls
/// 
/// ### Compile.
/// cargo clippy
/// 
/// ### Run.
/// cargo run --release
/// ```
/// 
/// Execution file.
/// C:/muzudho/projects_rust/rust-kifuwarabe-wcsc29/target/release/rust-kifuwarabe-wcsc29.exe

/*
fn test(cursor:&mut usize) {
    *cursor += 13;
}
*/
fn main() {
    let mut comm = Communication::new();
    let mut physical_record = PhysicalRecord::new();
    let mut board = Board::default();

    loop {
        // Standard input.
        // Be sure to add "info" before the output message.
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("info Failed: stdin.read_line.");

        // Excludes trailing newlines. The surrounding whitespace deletes.
        line = line.trim()
            .parse()
            .expect("info Failed: stdin parse.");

        // ######
        // # 数 #
        // ######
        if line.starts_with('1') || 
            line.starts_with('2') ||
            line.starts_with('3') ||
            line.starts_with('4') ||
            line.starts_with('5') ||
            line.starts_with('6') ||
            line.starts_with('7') ||
            line.starts_with('8') ||
            line.starts_with('9')
        {
            do_touch_cell_command(&line, &mut physical_record, &mut board);

        // ########
        // # 記号 #
        // ########
        } else if line.starts_with('+') {
            // 成り。
            let pmove = PhysicalMove::turn_over();
            board.touch(&pmove);
            physical_record.add(&pmove);
            board.println(physical_record.get_phase());

        } else if line.starts_with('-') {
            // １８０°回転。
            let pmove = PhysicalMove::rotate();
            board.touch(&pmove);
            physical_record.add(&pmove);
            board.println(physical_record.get_phase());

        // #####
        // # B #
        // #####
        } else if line.starts_with("bo") {
            // board.
            board.println(physical_record.get_phase());
            physical_record.println(&board.get_board_size());

        // #####
        // # G #
        // #####
        } else if line.starts_with("go") {
            /* TODO
            let thought = Thought::new();
            comm.println(&format!("bestmove {}", thought.get_best_move(
                &physical_record.get_position(),
                &mut logical_record).to_sign()));
             */
            // Examples.
            // println!("bestmove 7g7f");
            // println!("bestmove win");
            // println!("bestmove resign");
            
        // #####
        // # I #
        // #####
        } else if line == "isready" {
            comm.println("readyok");
        // #####
        // # Q #
        // #####
        } else if line == "quit" {
            break;
        // #####
        // # U #
        // #####
        } else if line == "usi" {
            comm.println("id name Kifuwarabe Build.11");
            comm.println("id author Satoshi TAKAHASHI");
            comm.println("usiok");
        } else if line == "usinewgame" {
        // #####
        // # P #
        // #####
        } else if line.starts_with("position") {
            let mut logical_record = LogicalRecord::new();
            let mut start = 0;
            if Fen::parse_board(&line, &mut start, &mut board) {
                match Fen::parse_moves(&line, &mut start, &mut board) {
                    Some(lrecords) => {
                        logical_record = lrecords;
                    },
                    None => {
                    },
                };
            }

            RecordConverter::convert_logical_to_physical(
                &mut board,
                &logical_record,
                &mut physical_record);
        }
    }
}

fn do_touch_cell_command(line:&str, physical_record:&mut PhysicalRecord, board:&mut Board) {
    let file = file_char_to_i8(line.to_string().chars().nth(0).unwrap());
    let rank = rank_char_to_i8(line.to_string().chars().nth(1).unwrap());
    let address = Address::create_by_cell(file, rank, &board.get_board_size());
    let pmove = PhysicalMove::create_by_address(address);
    physical_record.add(&pmove);
    if board.touch(&pmove) {
        // Phase change.
        physical_record.add(&PhysicalMove::phase_change());
    }
    board.println(physical_record.get_phase());
    physical_record.println(&board.get_board_size());
}
