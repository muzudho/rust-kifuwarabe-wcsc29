use audio_compo::cassette_deck::*;
use instrument::position::*;
use sheet_music_format::kifu_usi::usi_move::*;
use sheet_music_format::kifu_usi::usi_tape::*;
use sound::shogi_note_operation::*;
use studio::address::*;
use studio::application::Application;

pub struct UsiConverter {}
impl UsiConverter {
    /// # Arguments
    ///
    /// * 'position' - USIレコードと 初期局面を合わせてください。
    ///
    pub fn play_out_usi_tape(
        position: &mut Position,
        utape: &UsiTape,
        deck: &mut CassetteDeck,
        app: &Application,
    ) {
        let board_size = position.get_board_size();

        // 局面を動かしながら変換していく。
        let mut ply = 1;
        for umove in &utape.moves {
            let rnote_opes = UsiConverter::convert_move(*umove, position, ply, &app);
            //comm.println(&format!("Pmoves len: {}.", rpm_move.len()));

            for rnote_ope in rnote_opes {
                position.touch_1note_ope(deck, &rnote_ope, false, board_size, &app);
            }

            ply += 1;
        }
    }

    pub fn convert_move(
        umove: UsiMove,
        position: &Position,
        ply: i16,
        app: &Application,
    ) -> Vec<ShogiNoteOpe> {
        let mut rpm_move = Vec::new();

        // change-phase
        rpm_move.push(ShogiNoteOpe::change_phase(ply));

        if umove.is_resign() {
            rpm_move.push(ShogiNoteOpe::resign());

            // change-phase
            rpm_move.push(ShogiNoteOpe::change_phase(ply));
            return rpm_move;
        }

        let destination_address = Address::from_cell(
            umove
                .destination
                .unwrap_or_else(|| panic!(app.comm.panic("Fail. umove.destination."))),
            position.get_board_size(),
        );

        match umove.get_drop() {
            Some(drop) => {
                // 駒を打つ動きの場合

                // hand-off
                let hand_off = ShogiNoteOpe::from_address(Address::from_hand_ph_pt(
                    position.get_phase().get_value(),
                    drop,
                ));
                rpm_move.push(hand_off);

                // hand-on
                let hand_on = ShogiNoteOpe::from_address(destination_address);
                rpm_move.push(hand_on);
            }
            None => {
                // 駒を進める動きの場合
                if let Some(id_piece) =
                    position.get_id_piece_by_address(destination_address.get_index())
                {
                    // 駒を取る動きが入る場合

                    // hand-off
                    let hand_off = ShogiNoteOpe::from_address(destination_address);
                    rpm_move.push(hand_off);

                    // hand-rotate
                    let hand_rotate = ShogiNoteOpe::rotate();
                    rpm_move.push(hand_rotate);

                    // hand-turn
                    if id_piece.is_promoted() {
                        let hand_turn = ShogiNoteOpe::turn_over();
                        rpm_move.push(hand_turn);
                    }

                    // hand-on
                    let up = id_piece.get_type();
                    let hand_on = ShogiNoteOpe::from_address(Address::from_hand_ph_pt(
                        position.get_phase().get_value(),
                        up,
                    ));
                    rpm_move.push(hand_on);
                }

                // board-off
                let board_off = ShogiNoteOpe::from_address(Address::from_cell(
                    umove
                        .source
                        .unwrap_or_else(|| panic!(app.comm.panic("Fail. umove.source."))),
                    position.get_board_size(),
                ));
                rpm_move.push(board_off);

                // board-turn-over
                if umove.promotion {
                    let board_turn = ShogiNoteOpe::turn_over();
                    rpm_move.push(board_turn);
                }

                // board-on
                let board_on = ShogiNoteOpe::from_address(destination_address);
                rpm_move.push(board_on);
            }
        }

        // change-phase
        rpm_move.push(ShogiNoteOpe::change_phase(ply));

        rpm_move
    }
}
