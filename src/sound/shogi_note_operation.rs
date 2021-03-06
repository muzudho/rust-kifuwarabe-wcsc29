//
// Rpm棋譜のノートの操作属性。
//
// 局面から独立しています。
//
use instrument::piece_etc::PieceType;
use instrument::piece_etc::*;
use std::fmt;
use studio::address::{Address, Cell};
use studio::application::Application;
use studio::board_size::BoardSize;
use studio::common::caret::*;
use studio::common::closed_interval::*;
use studio::parser::Parser;

/// Vector に入れるときコピーする。
//#[derive(Debug)]
#[derive(Clone, Copy, PartialEq)]
pub struct ShogiNoteOpe {
    pub address: Option<Address>,
    /// +
    pub fingertip_turn: bool,
    /// -
    pub fingertip_rotate: bool,
    /// フェーズ・チェンジなら Ply、数が省略されている場合は -1。フェーズ・チェンジでないなら None。
    phase_change: Option<i16>,
    resign: bool,
}
impl fmt::Display for ShogiNoteOpe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.address {
            Some(address) => write!(f, "{}", address.get_index()),
            None => {
                if self.fingertip_turn {
                    write!(f, "+")
                } else if self.fingertip_rotate {
                    write!(f, "-")
                } else if let Some(ply) = self.phase_change {
                    if ply > -1 {
                        write!(f, "[{}]", ply)
                    } else {
                        write!(f, "|")
                    }
                } else if self.resign {
                    write!(f, "%resign")
                } else {
                    write!(f, "PANIC!")
                }
            }
        }
    }
}
impl ShogiNoteOpe {
    // ###############
    // # Constructor #
    // ###############

    pub fn from_address(address: Address) -> Self {
        Self {
            address: Some(address),
            fingertip_turn: false,
            fingertip_rotate: false,
            phase_change: None,
            resign: false,
        }
    }

    pub fn turn_over() -> Self {
        Self {
            address: None,
            fingertip_turn: true,
            fingertip_rotate: false,
            phase_change: None,
            resign: false,
        }
    }

    pub fn rotate() -> Self {
        Self {
            address: None,
            fingertip_turn: false,
            fingertip_rotate: true,
            phase_change: None,
            resign: false,
        }
    }

    pub fn change_phase(ply: i16) -> Self {
        Self {
            address: None,
            fingertip_turn: false,
            fingertip_rotate: false,
            phase_change: Some(ply),
            resign: false,
        }
    }

    pub fn resign() -> Self {
        Self {
            address: None,
            fingertip_turn: false,
            fingertip_rotate: false,
            phase_change: None,
            resign: true,
        }
    }

    // #####
    // # G #
    // #####

    pub fn get_phase_change(&self) -> Option<i16> {
        self.phase_change
    }

    // #####
    // # I #
    // #####

    pub fn is_phase_change(&self) -> bool {
        if let Some(_ply) = self.phase_change {
            true
        } else {
            false
        }
    }

    /// Position に変更を与えずに行える動作☆（＾～＾）
    pub fn is_resign(&self) -> bool {
        self.resign
    }

    // #####
    // # P #
    // #####

    /// 次のノート１つ読取☆（＾～＾）
    ///
    /// # Arguments
    ///
    /// * `caret` - Token caret.
    ///
    /// # Returns
    ///
    /// (last_used_caret, note_ope_opt)
    pub fn parse_1ope(
        line: &str,
        caret: &mut Caret,
        board_size: BoardSize,
        app: &Application,
    ) -> (ClosedInterval, Option<ShogiNoteOpe>) {
        let mut closed_interval = ClosedInterval::new_facing_right();

        let mut n0 = caret
            .seek_a_note(&app)
            .index
            .unwrap_or_else(|| panic!(app.comm.panic("n0 fail.")));
        closed_interval.intersect_caret_number(n0 as i16);

        let mut ch0 = line[n0..=n0]
            .chars()
            .nth(0)
            .unwrap_or_else(|| panic!(app.comm.panic("Fail. n0")));
        match ch0 {
            ' ' => (closed_interval, None),
            '0' => {
                // 駒台。
                let mut n1 = caret
                    .seek_a_note(&app)
                    .index
                    .unwrap_or_else(|| panic!(app.comm.panic("n1 fail.")));
                let mut ch1 = line[n1..=n1]
                    .chars()
                    .nth(0)
                    .unwrap_or_else(|| panic!(app.comm.panic("Fail. n0.")));

                if 2 < line.len() {
                    match ch1 {
                        '+' => {
                            // TODO 成り駒を駒台に置いた、という記号 + は読み飛ばします。この経路では 1つずれます。

                            n1 = caret
                                .seek_a_note(&app)
                                .index
                                .unwrap_or_else(|| panic!(app.comm.panic("n1 fail.")));
                            ch1 = line[n1..=n1]
                                .chars()
                                .nth(0)
                                .unwrap_or_else(|| panic!(app.comm.panic("Fail. n0.")));
                        }
                        _ => {}
                    };
                }

                // 駒の種類、フェーズ。
                let piece = Piece::from_sign(ch1.to_string());

                //comm.print(&format!("{}{}{}", ch1, text15, ch2));
                let address =
                    Address::from_hand_ph_pt(piece.get_phase(), PieceType::from_piece(piece));
                //comm.println(&format!("address index = {}.", address.get_index()));

                closed_interval.intersect_caret_number(n1 as i16);
                (closed_interval, Some(ShogiNoteOpe::from_address(address)))
            }
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                // セル
                let mut n1 = caret
                    .seek_a_note(&app)
                    .index
                    .unwrap_or_else(|| panic!(app.comm.panic("n1 fail.")));
                let mut ch1 = line[n1..=n1]
                    .chars()
                    .nth(0)
                    .unwrap_or_else(|| panic!(app.comm.panic("Fail. n0.")));

                // comm.print(&format!("Parse1Op: '{}', '{}'.", ch0, ch1));
                let address = Address::from_cell(
                    Cell::from_file_rank(
                        Parser::file_char_to_i8(ch0),
                        Parser::rank_char_to_i8(ch1),
                    ),
                    board_size,
                );

                closed_interval.intersect_caret_number(n1 as i16);
                (closed_interval, Some(ShogiNoteOpe::from_address(address)))
            }
            '+' => {
                // 成り。
                //comm.print(&ch1.to_string());
                (closed_interval, Some(ShogiNoteOpe::turn_over()))
            }
            '-' => {
                // １８０°回転。
                //comm.print(&ch1.to_string());
                (closed_interval, Some(ShogiNoteOpe::rotate()))
            }
            '|' => {
                // フェーズ交代。Ply は分からない。
                (closed_interval, Some(ShogiNoteOpe::change_phase(-1)))
            }
            '[' => {
                // フェーズ交代。 ']' まで読み飛ばす。
                let mut ply = 0;
                loop {
                    if line.len() <= n0 {
                        break;
                    }

                    n0 = caret
                        .seek_a_note(&app)
                        .index
                        .unwrap_or_else(|| panic!(app.comm.panic("n0 fail.")));
                    ch0 = line[n0..=n0]
                        .chars()
                        .nth(0)
                        .unwrap_or_else(|| panic!(app.comm.panic("Fail. n0.")));
                    closed_interval.intersect_caret_number(n0 as i16);

                    if ch0 == ']' {
                        break;
                    }

                    // Ply カウント。
                    let num: i16 = ch0
                        .to_string()
                        .parse::<i16>()
                        .unwrap_or_else(|err| panic!(app.comm.println(&format!("{}", err))));
                    ply *= 10;
                    ply += num;
                }
                (closed_interval, Some(ShogiNoteOpe::change_phase(ply)))
            }
            _ => {
                let last = line.len();
                panic!("Unexpected line '{}'.", &line[n0..last]);
            }
        }
    }

    // #####
    // # T #
    // #####

    pub fn to_human_presentable(&self, board_size: BoardSize, app: &Application) -> String {
        match self.address {
            Some(address) => {
                // 人に読みやすいセル表記にします。
                if address.is_fingertip() {
                    "FT".to_string()
                } else if address.is_on_board(board_size) {
                    board_size
                        .address_to_cell(address.get_index())
                        .to_human_presentable()
                } else if address.is_hand() {
                    address
                        .get_hand_piece()
                        .unwrap_or_else(|| panic!(app.comm.panic("Fail. get_hand_piece.")))
                        .to_human_presentable_4width()
                } else {
                    panic!(
                        "Unexpected address: {}.",
                        address.to_human_presentable(board_size)
                    )
                }
            }
            None => {
                if self.fingertip_turn {
                    "+".to_string()
                } else if self.fingertip_rotate {
                    "-".to_string()
                } else if let Some(ply) = self.phase_change {
                    if ply > -1 {
                        format!("[{}]", ply).to_string()
                    } else {
                        "|".to_string()
                    }
                } else if self.resign {
                    "%resign".to_string()
                } else {
                    "PANIC!".to_string()
                }
            }
        }
    }

    /// 幅は5。
    pub fn to_human_presentable_5width(&self, board_size: BoardSize, app: &Application) -> String {
        match self.address {
            Some(address) => {
                // 人に読みやすいセル表記にします。
                if address.is_fingertip() {
                    "Fingt".to_string()
                } else if address.is_on_board(board_size) {
                    board_size
                        .address_to_cell(address.get_index())
                        .to_human_presentable_5width()
                } else if address.is_hand() {
                    address
                        .get_hand_piece()
                        .unwrap_or_else(|| panic!(app.comm.panic("Fail. get_hand_piece.")))
                        .to_human_presentable_5width()
                } else {
                    panic!(
                        "Unexpected address: {}.",
                        address.to_human_presentable(board_size)
                    )
                }
            }
            None => {
                if self.fingertip_turn {
                    "  +  ".to_string()
                } else if self.fingertip_rotate {
                    "  -  ".to_string()
                } else if let Some(ply) = self.phase_change {
                    if ply > -1 {
                        format!("[{:>3}]", ply).to_string()
                    } else {
                        "[ - ]".to_string()
                    }
                } else if self.resign {
                    "%resi".to_string()
                } else {
                    "PANI!".to_string()
                }
            }
        }
    }

    pub fn to_sign(&self, board_size: BoardSize) -> String {
        match self.address {
            Some(address) => address.to_physical_sign(board_size),
            None => {
                if self.fingertip_turn {
                    "+".to_string()
                } else if self.fingertip_rotate {
                    "-".to_string()
                } else if let Some(ply) = self.phase_change {
                    if ply > -1 {
                        format!("[{}]", ply)
                    } else {
                        "|".to_string()
                    }
                } else if self.resign {
                    "%resign".to_string()
                } else {
                    panic!("Unexpected physical move print.")
                }
            }
        }
    }
}
