extern crate getopts;
extern crate kifuwarabe_wcsc29_lib;

use getopts::Options;
use kifuwarabe_wcsc29_lib::audio_compo::cassette_deck::*;
use kifuwarabe_wcsc29_lib::instrument::position::*;
use kifuwarabe_wcsc29_lib::sheet_music_format::kifu_kif::kif_converter::KifConverter;
use kifuwarabe_wcsc29_lib::sheet_music_format::kifu_kif::kif_tape::*;
use kifuwarabe_wcsc29_lib::studio::application::*;
use kifuwarabe_wcsc29_lib::*;
use std::env;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug)]
pub struct Arguments {
    pub input_file: Option<String>,
    pub output_file: Option<String>,
}
impl Arguments {
    pub fn parse() -> Arguments {
        let args: Vec<String> = env::args().collect();

        let mut opts = Options::new();
        opts.optopt("i", "input", "set input record file name.", "NAME");
        opts.optopt("o", "output", "set output record file name.", "NAME");

        let matches = opts
            .parse(&args[1..])
            .unwrap_or_else(|f| panic!(f.to_string()));

        Arguments {
            input_file: matches.opt_str("input"),
            output_file: matches.opt_str("output"),
        }
    }
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn main() {
    // Command line arguments.
    let args = Arguments::parse();

    // The application contains all immutable content.
    let app = Application::new();

    let in_file = args
        .input_file
        .unwrap_or_else(|| panic!(app.comm.panic("Fail. args.input_file.")));

    // 保存先のテープ・フラグメント名☆（＾～＾）　ラーニング・テープと想定☆（＾～＾）
    let tape_file_name_without_extension = args
        .output_file
        .unwrap_or_else(|| panic!(app.comm.panic("Fail. args.output_file.")));

    // Position.
    let mut position = Position::new_honshogi_origin(&app, 123001);

    // Deck.
    let mut deck = CassetteDeck::new_for_tape_conversion(&tape_file_name_without_extension, &app);

    if !in_file.is_empty() {
        // 棋譜解析。
        let ext = get_extension_from_filename(&in_file)
            .unwrap_or_else(|| panic!(app.comm.panic("Fail. get_extension_from_filename.")))
            .to_uppercase();

        match ext.as_str() {
            "KIF" => {
                // Training data.
                let ktape = KifTape::from_file(&in_file, &app);

                // Play out.
                KifConverter::play_out_kifu_tape(&ktape, &mut position, &mut deck, &app);

                // Write.
                deck.write_leaning_tape_fragment(position.get_board_size(), &app);
            }
            "CSA" => {}
            _ => print!("Pass extension: {}", ext),
        }
    } else {
        main_loop();
    }
}
