/// extern crate は main.rs か lib.rs に入れる。
/// 参考: https://github.com/serde-rs/json |シリアライズ、デシリアライズ。
extern crate serde_json;

use std::io;

mod address;
mod board;
mod communication;
mod record;
mod position_file;
mod position;
mod thought;

use address::*;
use communication::*;
use position::Position;
use record::*;
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
fn main() {

    let mut comm = Communication::new();
    let mut position = Position::new();

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

        // #####
        // # 3 #
        // #####
        if line.starts_with("3") {
            // Sign.
            match line.as_str() {
                "3c" => {
                    let address = Address::create_by_cell(3, 3, &position.board);
                    position.board.touch(&address);
                },
                "3d" => {
                    let address = Address::create_by_cell(3, 4, &position.board);
                    position.board.touch(&address);
                },
                _ => {},
            };
            position.board.print(&position.record.get_current_phase());

        // #####
        // # 4 #
        // #####
        } else if line.starts_with("4") {
            // Sign.
            match line.as_str() {
                "4c" => {
                    let address = Address::create_by_cell(4, 3, &position.board);
                    position.board.touch(&address);
                },
                _ => {},
            };
            position.board.print(&position.record.get_current_phase());

        // #####
        // # 7 #
        // #####
        } else if line.starts_with("7") {
            // Sign.
            match line.as_str() {
                "7g" => {
                    let address = Address::create_by_cell(7, 7, &position.board);
                    position.board.touch(&address);
                },
                "7f" => {
                    let address = Address::create_by_cell(7, 6, &position.board);
                    position.board.touch(&address);
                },
                _ => {},
            };
            position.board.print(&position.record.get_current_phase());

        // #####
        // # B #
        // #####
        } else if line.starts_with("bo") {
            // board.
            position.board.print(&position.record.get_current_phase());

        // #####
        // # G #
        // #####
        } else if line.starts_with("go") {
            let thought = Thought::new();
            comm.println(&format!("bestmove {}", thought.get_best_move(&mut position).to_sign()));
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
            position.parse(&line);
        }
    }
}
