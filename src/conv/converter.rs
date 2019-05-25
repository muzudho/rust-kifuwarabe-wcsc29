use audio_compo::audio_rack::*;
use audio_compo::cassette_deck::*;
use instrument::position::*;
use sheet_music_format::kifu_csa::csa_converter::CsaConverter;
use sheet_music_format::kifu_csa::csa_tape::*;
use sheet_music_format::kifu_kif::kif_converter::KifConverter;
use sheet_music_format::kifu_kif::kif_tape::*;
use std::ffi::OsStr;
use std::path::Path;
use studio::application::*;

pub struct Converter {}

impl Converter {
    fn get_extension_from_file_path(file_path: &str) -> Option<&str> {
        Path::new(file_path).extension().and_then(OsStr::to_str)
    }

    fn get_file_stem_from_file_path(file_path: &str) -> Option<&str> {
        Path::new(file_path).file_stem().and_then(OsStr::to_str)
    }

    pub fn convert(
        in_file: String,
        rack: &mut AudioRack,
        position: &mut Position,
        app: &Application,
    ) {
        let file_stem = Converter::get_file_stem_from_file_path(&in_file)
            .unwrap_or_else(|| panic!(app.comm.panic("Fail. get_file_stem_from_file_path.")));

        let extension = Converter::get_extension_from_file_path(&in_file)
            .unwrap_or_else(|| panic!(app.comm.panic("Fail. get_extension_from_file_path.")))
            .to_uppercase();

        match extension.as_str() {
            "KIF" => {
                // Training data.
                let mut ktape = KifTape::from_file(&in_file, &app);

                // Play out.
                KifConverter::play_out_kifu_tape(&ktape, rack, position, &app);

                // Tape label
                rack.set_source_file_of_tape_label(Slot::Learning, file_stem.to_string());

                // Write.
                rack.write_leaning_tapes_fragment(position.get_board_size(), &app);
            }
            "CSA" => {
                // Training data.
                let mut ctape = CsaTape::from_file(&in_file, &app);

                if app.is_debug() {
                    app.comm
                        .println(&format!("Ctape: '{}'", ctape.to_human_presentable()));
                }

                // Play out.
                CsaConverter::play_out_csa_tape(&ctape, rack, position, &app);

                // Tape label
                rack.set_source_file_of_tape_label(Slot::Learning, file_stem.to_string());

                // Write.
                rack.write_leaning_tapes_fragment(position.get_board_size(), &app);
            }
            _ => print!("Pass extension: {}", extension),
        }
    }
}
