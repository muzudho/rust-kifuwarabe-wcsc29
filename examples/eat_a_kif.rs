extern crate getopts;
extern crate kifuwarabe_wcsc29_lib;
extern crate regex;
use getopts::Options;
use kifuwarabe_wcsc29_lib::application::*;
use kifuwarabe_wcsc29_lib::kifu_kif::kif_converter::*;
use kifuwarabe_wcsc29_lib::object_rpm::*;
use kifuwarabe_wcsc29_lib::object_rpm::cassette_deck::*;
use std::env;

#[derive(Debug)]
struct Args {
    path: Option<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("p", "path", "set input csa file name.", "NAME");

    let matches = opts
        .parse(&args[1..])
        .unwrap_or_else(|f| panic!(f.to_string()));

    Args {
        path: matches.opt_str("path"),
    }
}

pub fn main() {
    // Command line arguments.
    let args = parse_args();

    // The application contains all immutable content.
    let app = Application::new();

    let path = args.path.unwrap();
    app.comm.println(&format!("args.path = '{}'.", path));

    // Record.
    let mut tape_box_conveyer = CassetteDeck::new_empty();
    tape_box_conveyer.choice_box_manually("sheet.txt");
    let mut recorder = CassetteTapeEditor::new_cassette_tape_editor();

    KifConverter::convert_kif_tape_fragment(&path, &mut tape_box_conveyer, &mut recorder, &app);
}
