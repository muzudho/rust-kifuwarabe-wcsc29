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

/// extern crate は main.rs か lib.rs に入れる。
/// 参考: https://github.com/serde-rs/json |シリアライズ、デシリアライズ。
extern crate serde_json;

use std::io;

mod address;
mod common_operation;
mod communication;
mod fen;
mod logical_move;
mod logical_record;
mod parser;
mod physical_move;
mod physical_record;
mod position;
// mod position_file;
mod record_converter;
mod thought;

use address::*;
use common_operation::*;
use communication::*;
use fen::*;
use logical_record::*;
use physical_move::*;
use physical_record::*;
use position::*;
use record_converter::*;
use thought::Thought;

/*
fn test(cursor:&mut usize) {
    *cursor += 13;
}
*/
fn main() {
    let comm = Communication::new();
    let mut physical_record = PhysicalRecord::new();
    let mut board = Position::default();

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
        // ########
        // # 記号 #
        // ########
        if line.starts_with('1') || 
            line.starts_with('2') ||
            line.starts_with('3') ||
            line.starts_with('4') ||
            line.starts_with('5') ||
            line.starts_with('6') ||
            line.starts_with('7') ||
            line.starts_with('8') ||
            line.starts_with('9') ||
            line.starts_with('+') ||
            line.starts_with('-') ||
            line.starts_with('|')
        {
            read_tape(&comm, &line, &mut physical_record, &mut board);

        // #####
        // # B #
        // #####
        } else if line.starts_with("bo") {
            // board.
            CommonOperation::bo(&comm, &physical_record, &board);

        } else if line.starts_with('B') {
            read_tape(&comm, &line, &mut physical_record, &mut board);

        // #####
        // # D #
        // #####
        } else if line == "d" {
            // Delete.
            CommonOperation::detouch(&comm, &mut physical_record, &mut board);

        // #####
        // # G #
        // #####
        } else if line.starts_with("go") {
            let thought = Thought::new();
            let best_logical_move = thought.get_best_move(
                &board,
                &mut physical_record);
            // Examples.
            // println!("bestmove 7g7f");
            // println!("bestmove win");
            // println!("bestmove resign");
            comm.println(&format!("bestmove {}", best_logical_move.to_sign()));

            let best_physical_moves = RecordConverter::convert_logical_move(
                best_logical_move,
                &board);
            for physical_move in best_physical_moves {
                CommonOperation::go(&comm, &mut physical_record, &physical_move, &mut board);
            }

        } else if line.starts_with('G') {
            read_tape(&comm, &line, &mut physical_record, &mut board);
            
        // #####
        // # I #
        // #####
        } else if line == "isready" {
            comm.println("readyok");

        // #####
        // # K #
        // #####
        } else if line.starts_with('K') {
            read_tape(&comm, &line, &mut physical_record, &mut board);

        // #####
        // # L #
        // #####
        } else if line.starts_with('L') {
            read_tape(&comm, &line, &mut physical_record, &mut board);

        // #####
        // # N #
        // #####
        } else if line.starts_with('N') {
            read_tape(&comm, &line, &mut physical_record, &mut board);

        // #####
        // # P #
        // #####
        } else if line.starts_with('P') {
            read_tape(&comm, &line, &mut physical_record, &mut board);

        // #####
        // # Q #
        // #####
        } else if line == "quit" {
            break;

        // #####
        // # S #
        // #####
        } else if line.starts_with('S') {
            read_tape(&comm, &line, &mut physical_record, &mut board);

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
            if Fen::parse_position(&line, &mut start, &mut board) {
                if let Some(lrecords) = Fen::parse_moves(&comm, &line, &mut start, &mut board) {
                    logical_record = lrecords;
                };
            }

            RecordConverter::convert_logical_record_to_physical_record(
                &comm,
                &mut board,
                &logical_record,
                &mut physical_record);
        // #####
        // # R #
        // #####
        } else if line.starts_with('R') {
            read_tape(&comm, &line, &mut physical_record, &mut board);
        }
    }
}

/// 
fn read_tape(comm:&Communication, line:&str, physical_record:&mut PhysicalRecord, board:&mut Position) {
    let mut start = 0;

    loop {
        if line.len() <= start {
            return;
        }

        let ch1 = line[start..=start].chars().nth(0).unwrap();
        let pmove_opt = match ch1 {
            ' ' => {
                comm.print(&ch1.to_string());
                start += 1;
                None
            }
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                // セル
                start += 1;
                let ch2 = line[start..=start].chars().nth(0).unwrap();
                start += 1;
                comm.print(&format!("{}{}", ch1, ch2));
                let file = file_char_to_i8(ch1);
                let rank = rank_char_to_i8(ch2);
                let address = Address::create_by_cell(file, rank, board.get_board_size());
                Some(PhysicalMove::create_by_address(address))
            },
            '+' => {
                // 成り。
                comm.print(&ch1.to_string());
                start += 1;
                Some(PhysicalMove::turn_over())
            },
            '-' => {
                // １８０°回転。
                comm.print(&ch1.to_string());
                start += 1;
                Some(PhysicalMove::rotate())
            },
            '|' => {
                // フェーズ交代。
                comm.print(&ch1.to_string());
                start += 1;
                Some(PhysicalMove::change_phase())
            },
            'K' | 'R' | 'B' | 'G' | 'S' | 'N' | 'L' | 'P' => {
                // ドロップ。
                start += 1;
                let ch2 = line[start..=start].chars().nth(0).unwrap();
                start += 1;
                if ch2 != '*' {
                    panic!("Unexpected drop '{}'.", line)
                };
                comm.print(&format!("{}{}", ch1, ch2));
                let piece_type = sign_to_piece_type(ch1.to_string());
                let address = Address::create_by_hand(Some(board.get_phase()), piece_type);
                comm.println(&format!("address index = {}.", address.get_index()));
                Some(PhysicalMove::create_by_address(address))
            },
            _ => {
                panic!("Unexpected line '{}'.", line)
            }
        };

        if let Some(pmove) = pmove_opt {
            CommonOperation::touch(comm, physical_record, &pmove, board);
        }
    }
}
