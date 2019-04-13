use address::*;
use communication::*;
use piece_etc::*;
use position::*;
use rpm_conv::thread::rpm_note::*;
// use rpm_conv::thread::rpm_note_operation::*;
use rpm_for_json::rpm_book_file::*;
use usi_conv::usi_move::*;

/// １手分。
#[derive(Debug)]
pub struct RpmMove {
    pub notes: Vec<RpmNote>,
}
/*
impl fmt::Display for RpmMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = String::new();

        let size = self.operation_notes.len();
        for i in 0..size {
            text = format!("{} ({} {})", text, self.operation_notes[i], self.piece_number_notes[i]).to_string()
        }

        write!(f, "{}", text)
    }
}
*/
impl RpmMove {
    fn new() -> RpmMove {
        RpmMove {
            notes: Vec::new(),
        }
    }

    /// 1手分解析。
    pub fn parse_1move(comm:&Communication, record_for_json:&RpmRecordForJson, note_start:&mut usize, board_size:BoardSize) -> Option<RpmMove> {
        let mut rmove = RpmMove::new();

        let size = record_for_json.body.operation.len();
        if size == 1 {
            panic!("操作トラックが 1ノート ということは無いはず。 {:?}", record_for_json.body.operation)
        }

        // comm.print(&format!("P1M: note_start: {}, size: {}.", *note_start, size));
        let mut is_first = true;

        // 次のフェーズ・チェンジまで読み進める。
        'j_loop: loop {
            if *note_start < size {
                // トラックの終わり。
                // comm.print("Break: End of track.");
                break 'j_loop;
            }

            // comm.print(&format!("P1Mb: note_start: {}.", note_start));

            let note_opt = RpmNote::parse_1note(comm, record_for_json, note_start, board_size);

            match note_opt {
                Some(note) => {
                    if note.is_phase_change() {
                        if is_first {

                        } else {
                            comm.print("Break: Phase change.");
                            break 'j_loop;
                        }
                    }

                    comm.print(&format!("Push: {:?}.", note));
                    rmove.notes.push(note);
                },
                None => {
                    comm.print("Break: None.");
                    break 'j_loop;
                },
            }

            is_first = false;
        }

        if rmove.is_empty() {
            None
        } else if rmove.len() == 1 {
            panic!("指し手が 1ノート ということは無いはず。 {:?}", record_for_json.body.operation)
        } else {
            Some(rmove)
        }
    }

    pub fn len(&self) -> usize {
        self.notes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    /// この指し手が、どの駒が動いたものによるものなのか、またどこにあった駒なのかを返します。
    pub fn to_first_touch_piece_id(&self, board_size:BoardSize) -> (PieceIdentify, Address) {
        // とりあえず USI move に変換するついでに、欲しい情報を得る。
        let (_umove, first_touch_id, first_touch_addr) = self.to_usi_move(board_size);

        (first_touch_id, first_touch_addr)
    }

    /// # Returns
    /// 
    /// Usi move,
    /// どの駒を動かした一手か,
    /// どこの駒を動かした一手か,
    pub fn to_usi_move(&self, board_size:BoardSize) -> (UsiMove, PieceIdentify, Address) {
        // 順番は決まっている。
        let mut i_token = 0;

        let mut src_opt = None;
        let mut dst_opt = None;
        let mut promotion = false;
        let mut drop_opt = None;
        // first touch piece.
        let mut ftp_id = None;
        let mut ftp_addr = None;
        for note in &self.notes {
            // 数が入っているとき。
            if let Some(address) = note.get_ope().address {
                if let Some(piece) = address.get_hand_piece() {
                    // 駒台
                    if i_token == 0 {
                        drop_opt = Some(piece_to_piece_type(piece));
                        ftp_id = Some(note.get_id());
                        ftp_addr = Some(address);
                        i_token += 1;
                    }
                } else {
                    // 盤上
                    match i_token {
                        0 => {
                            src_opt = Some(board_size.address_to_cell(address.get_index()));
                            ftp_id = Some(note.get_id());
                            ftp_addr = Some(address);
                            i_token += 1;
                        },
                        1 => {
                            dst_opt = Some(board_size.address_to_cell(address.get_index()));
                            // ２つ目に出てくる場合、１つ目は取った相手の駒の動き。
                            ftp_id = Some(note.get_id());
                            ftp_addr = Some(address);
                            i_token += 1;
                        },
                        _ => {},
                    }
                }

            } else if note.get_ope().sky_turn {
                // +
                promotion = true;
            } else if note.get_ope().sky_rotate {
                // -
            }
        }

        let umove = if let Some(drop) = drop_opt {
            UsiMove::create_drop(
                dst_opt.unwrap(),
                drop,
                board_size)
        } else if let Some(dst) = dst_opt {
            UsiMove::create_walk(
                src_opt.unwrap(),
                dst,
                promotion,
                board_size)
        } else {
            panic!("Unexpected dst. move.len: '{}' > 1, move: '{:?}'.", self.len(), self)
        };

        // USIの指し手が作れれば、 first touch が分からないことはないはず。
        (umove, PieceIdentify::from_number(ftp_id.unwrap()).unwrap(), ftp_addr.unwrap())
    }

    pub fn to_operation_string(&self, board_size:BoardSize) -> String {
        let mut text = String::new();

        for i in 0..self.len() {
            let mut ply = -1;
            text = format!("{} {}", text, &self.notes[i].get_ope().to_sign(board_size, &mut ply));
        }

        text
    }

    pub fn to_identify_string(&self) -> String {
        let mut text = String::new();

        for i in 0..self.len() {
            text = format!("{} {}", text, &self.notes[i].get_id());
        }

        text
    }
}