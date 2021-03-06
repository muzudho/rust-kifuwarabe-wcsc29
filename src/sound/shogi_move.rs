use audio_compo::audio_rack::*;
//use audio_compo::cassette_deck::CassetteDeck;
use audio_compo::cassette_deck::Slot;
use instrument::piece_etc::*;
use media::cassette_tape_box::CassetteTapeBox;
use musician::best_move::*;
use sheet_music_format::kifu_usi::usi_move::*;
use std::fmt;
use studio::application::Application;
use studio::board_size::*;
use studio::common::caret::*;
use studio::common::closed_interval::*;

/// カセット・テープ上の閉区間（１手分）。
///
/// 右端をフェーズ・チェンジとして含む。
/// 最後のムーブのみ、フェーズチェンジを含まなくても構わない。
pub struct ShogiMove {
    // カセット・テープ上の閉区間。
    pub caret_closed_interval: ClosedInterval,
}
impl fmt::Display for ShogiMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.caret_closed_interval.to_human_presentable())
    }
}
impl ShogiMove {
    // ###############
    // # Constructor #
    // ###############

    pub fn new_facing_right_move() -> Self {
        ShogiMove {
            caret_closed_interval: ClosedInterval::new_facing_right(),
        }
    }

    pub fn from_closed_interval(closed_interval: ClosedInterval) -> Self {
        ShogiMove {
            caret_closed_interval: closed_interval,
        }
    }

    // #####
    // # G #
    // #####

    pub fn get_start(&self) -> i16 {
        self.caret_closed_interval.get_start()
    }

    pub fn get_end(&self) -> i16 {
        self.caret_closed_interval.get_end()
    }

    // #####
    // # I #
    // #####

    pub fn is_empty(&self) -> bool {
        self.caret_closed_interval.is_empty()
    }

    // #####
    // # L #
    // #####

    pub fn len(&self) -> usize {
        self.caret_closed_interval.len()
    }

    // #####
    // # P #
    // #####

    /// 次の1手分解析。
    ///
    /// # Arguments
    ///
    /// (parsed_note_count, move_opt)
    ///
    /// parsed_note_count は巻き戻すのに使う。
    pub fn parse_1move(
        tape_box: &CassetteTapeBox,
        caret: &mut Caret,
        _board_size: BoardSize,
        app: &Application,
    ) -> Option<ShogiMove> {
        let brandnew = ShogiMove::new_facing_right_move();

        let mut is_first = true;

        // 次のフェーズ・チェンジまで読み進める。
        'j_loop: loop {
            if tape_box.is_before_caret_overflow_of_tape() {
                // トラックの終わり。
                //comm.print("Break: End of track.");
                break 'j_loop;
            }

            let note = if let (_taken_overflow, _awareness, Some(note)) =
                tape_box.seek_a_note_with_othre_caret(caret, &app)
            {
                note
            } else {
                // パースできるノートが無かった。
                //comm.print("Break: None.");
                break 'j_loop;
            };

            if note.is_phase_change() && !is_first {
                // フェーズの変わり目で終わり。
                //comm.print("Break: Phase change.");
                break 'j_loop;
            }

            //comm.print(&format!("Push: {:?}.", note));
            //closed_interval.intersect_closed_interval(sub_closed_interval);

            is_first = false;
        }

        if brandnew.caret_closed_interval.is_empty() {
            None
        } else if brandnew.caret_closed_interval.len() == 1 {
            panic!("指し手が 1ノート ということは無いはず。")
        } else {
            Some(brandnew)
        }
    }

    // #####
    // # T #
    // #####

    /// 一手。フェーズ・チェンジ・ノートや「ほこり取り」は含まない。
    /// 決まっている並びをしているものとする。
    ///
    /// オーバーフローを含むか、決まった並びをしていないなどの場合、Noneを返す。
    /// 頻繁に含まれるので、強制終了はしない。
    ///
    /// # Returns
    ///
    /// (Usi move, どの駒を動かした一手か, どこの駒を動かした一手か, あれば取った駒，取った駒の番地)
    pub fn to_best_move(
        &self,
        rack: &mut AudioRack,
        slot: Slot,
        board_size: BoardSize,
        app: &Application,
    ) -> Option<BestMove> {
        /*
        if app.is_debug() {
            app.comm.println("[#To best move: Begin]");
        }
        */
        // 動作の主体。
        let mut subject_pid_opt;
        let mut subject_address_opt;
        let mut subject_promotion = false;
        // 動作に巻き込まれる（駒を取られる）方。
        let mut captured_pid_opt = None;
        let mut captured_address_opt = None;
        let mut _captured_promotion = false;
        // 基本情報。
        let mut src_opt = None;
        let mut dst_opt = None;
        let mut drop_opt = None;

        let mut caret =
            Caret::new_facing_right_caret_with_number(self.caret_closed_interval.get_start());
        /*
        if app.is_debug() {
            app.comm
                .println(&format!("[Caret: {}]", caret.to_human_presentable(&app)));
        }
        */

        // ノートを取る。
        let mut note = if let (_taken_overflow, _awareness, Some(note)) =
            rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
        {
            note
        } else {
            if app.is_debug() {
                app.comm.println("Note fail(1).");
            }
            return None;
        };
        /*
        if app.is_debug() {
            app.comm.println(&format!(
                "[#Move.Note: {}]",
                note.to_human_presentable(board_size, &app)
            ));
        }
        */

        // ####################
        // # (0) Change phase #
        // ####################
        if !note.is_phase_change() {
            panic!(
                "[#１つ目がフェーズ・チェンジでない指し手はエラー☆（＾～＾）]"
            );
        }

        // ノートを取る。
        note = if let (_taken_overflow, _awareness, Some(note)) =
            rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
        {
            note
        } else {
            /*
            if app.is_debug() {
                app.comm.println("Note fail(1b).");
            }
            */
            return None;
        };
        /*
        if app.is_debug() {
            app.comm.println(&format!(
                "[#Note: {}]",
                note.to_human_presentable(board_size, &app)
            ));
        }
        */

        // 盤上の自駒 or 盤上の相手の駒 or 駒台の自駒
        if let Some(address) = note.get_ope().address {
            /*
            if app.is_debug() {
                app.comm.println(&format!(
                    "[#Address: {}]",
                    address.to_human_presentable(board_size)
                ));
            }
            */

            if let Some(piece) = address.get_hand_piece() {
                /*
                if app.is_debug() {
                    app.comm
                        .println(&format!("[HandPiece: {}]", piece.to_human_presentable()));
                }
                */

                // 駒台なら必ず自駒のドロップ。

                // #################
                // # (1d) Hand off #
                // #################
                subject_pid_opt = note.get_id();
                subject_address_opt = Some(address);
                drop_opt = Some(PieceType::from_piece(piece));

                // ノートを取る。
                note = if let (_taken_overflow, _awareness, Some(note)) =
                    rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                {
                    note
                } else {
                    /*
                    if app.is_debug() {
                        app.comm.println("Note fail(2).");
                    }
                    */
                    return None;
                };
                /*
                if app.is_debug() {
                    app.comm.println(&format!(
                        "[Note: {}]",
                        note.to_human_presentable(board_size, &app)
                    ));
                }
                */

                // ################
                // # (4d) Hand on #
                // ################
                if let Some(address) = note.get_ope().address {
                    dst_opt = Some(board_size.address_to_cell(address.get_index()));
                } else {
                    panic!(
                        "Unexpected 1st note of move(30): {}.",
                        note.to_human_presentable(board_size, &app)
                    );
                }

            // その次は無い。

            // 盤上
            } else {
                // これが盤上の自駒か、相手の駒かは、まだ分からない。仮に入れておく。

                // #################    #################
                // # (1c) Hand off # or # (5) Board off #
                // #################    #################
                subject_pid_opt = note.get_id();
                subject_address_opt = Some(address);
                src_opt = Some(board_size.address_to_cell(address.get_index()));
                /*
                if app.is_debug() {
                    app.comm.println("[Not HandPiece]");
                }
                */

                // ノートを取る。
                note = if let (_taken_overflow, _awareness, Some(note)) =
                    rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                {
                    note
                } else {
                    /*
                    if app.is_debug() {
                        app.comm.println("Note fail(3).");
                    }
                    */
                    return None;
                };
                /*
                if app.is_debug() {
                    app.comm.println(&format!(
                        "[Note: {}]",
                        note.to_human_presentable(board_size, &app)
                    ));
                }
                */

                // #################    #######################
                // # (2) Hand turn # or # (6) Board turn over #
                // #################    #######################
                if note.get_ope().fingertip_turn {
                    // +。駒を裏返した。自駒を成ったのか、取った成り駒を表返したのかは、まだ分からない。仮に成ったことにしておく。
                    subject_promotion = true;

                    // 次。
                    note = if let (_taken_overflow, _awareness, Some(note)) =
                        rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                    {
                        note
                    } else {
                        /*
                        if app.is_debug() {
                            app.comm.println("Note fail(4).");
                        }
                        */
                        return None;
                    };
                    /*
                    if app.is_debug() {
                        app.comm.println(&format!(
                            "[Note: {}]",
                            note.to_human_presentable(board_size, &app)
                        ));
                    }
                    */
                }

                // ###################
                // # (3) Hand rotate #
                // ###################
                if note.get_ope().fingertip_rotate {
                    // -。向きを変えているようなら、相手の駒を取ったようだ。いろいろキャンセルする。
                    captured_pid_opt = subject_pid_opt;
                    captured_address_opt = subject_address_opt;
                    _captured_promotion = subject_promotion;
                    subject_pid_opt = None;
                    subject_address_opt = None;
                    src_opt = None;
                    subject_promotion = false;

                    // ノートを取る。
                    note = if let (_taken_overflow, _awareness, Some(note)) =
                        rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                    {
                        note
                    } else {
                        /*
                        if app.is_debug() {
                            app.comm.println("Note fail(5).");
                        }
                        */
                        return None;
                    };
                    /*
                    if app.is_debug() {
                        app.comm.println(&format!(
                            "[Note: {}]",
                            note.to_human_presentable(board_size, &app)
                        ));
                    }
                    */

                    // ################
                    // # (4c) Hand on #
                    // ################
                    // 自分の駒台に置く動きは USI符号に書かれない。
                    if let Some(_address) = note.get_ope().address {
                        // ノートを取る。
                        note = if let (_taken_overflow, _awareness, Some(note)) =
                            rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                        {
                            note
                        } else {
                            /*
                            if app.is_debug() {
                                app.comm.println("Note fail(6).");
                            }
                            */
                            return None;
                        };
                        /*
                        if app.is_debug() {
                            app.comm.println(&format!(
                                "[Note: {}]",
                                note.to_human_presentable(board_size, &app)
                            ));
                        }
                        */

                        // 5～7は、盤上の駒を進める動き。

                        // #################
                        // # (5) Board off #
                        // #################
                        if let Some(address) = note.get_ope().address {
                            subject_pid_opt = note.get_id();
                            subject_address_opt = Some(address);
                            src_opt = Some(board_size.address_to_cell(address.get_index()));

                            // ノートを取る。
                            note = if let (_taken_overflow, _awareness, Some(note)) =
                                rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                            {
                                note
                            } else {
                                /*
                                if app.is_debug() {
                                    app.comm.println("Note fail(7).");
                                }
                                */
                                return None;
                            };
                            /*
                            if app.is_debug() {
                                app.comm.println(&format!(
                                    "[Note: {}]",
                                    note.to_human_presentable(board_size, &app)
                                ));
                            }
                            */
                        }

                        // #######################
                        // # (6) Board turn over #
                        // #######################
                        if note.get_ope().fingertip_turn {
                            // +。盤上の自駒が成った。
                            subject_promotion = true;

                            // ノートを取る。
                            note = if let (_taken_overflow, _awareness, Some(note)) =
                                rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                            {
                                note
                            } else {
                                /*
                                if app.is_debug() {
                                    app.comm.println("Note fail(8).");
                                }
                                */
                                return None;
                            };
                            /*
                            if app.is_debug() {
                                app.comm.println(&format!(
                                    "[Note: {}]",
                                    note.to_human_presentable(board_size, &app)
                                ));
                            }
                            */
                        }
                    } else {
                        panic!(
                            "Unexpected 1st note of move(70): {}.",
                            note.to_human_presentable(board_size, &app)
                        );
                    }
                } else {
                    // 盤上の自駒を触ったのだと確定した。
                }

                // ################
                // # (7) Board on #
                // ################
                if let Some(address) = note.get_ope().address {
                    // 行き先に盤上の自駒を進めた。
                    dst_opt = Some(board_size.address_to_cell(address.get_index()));

                    // ノートを取る。
                    note = if let (_taken_overflow, _awareness, Some(note)) =
                        rack.seek_a_note_with_othre_caret(slot, &mut caret, &app)
                    {
                        note
                    } else {
                        /*
                        if app.is_debug() {
                            app.comm.println("Note fail(7b).");
                        }
                        */
                        return None;
                    };
                    /*
                    if app.is_debug() {
                        app.comm.println(&format!(
                            "[Note: {}]",
                            note.to_human_presentable(board_size, &app)
                        ));
                    }
                    */
                }

                // ####################
                // # (8) Change phase #
                // ####################
                if !note.is_phase_change() {
                    panic!(
                        "[#最後がフェーズ・チェンジでない指し手はエラー☆（＾～＾）]"
                    );
                }
            }
        } else {
            panic!(
                "Unexpected 1st note of move(100): {}.",
                note.to_human_presentable(board_size, &app)
            );
        }

        /*
        if i + 1 < self.notes.len() {
            panic!(
                "Unexpected 1st note of move(110): 余り。 {}/{}, {}",
                i,
                self.notes.len(),
                self.to_human_presentable(board_size)
            );
        }
        */

        let umove = if let Some(drop) = drop_opt {
            UsiMove::create_drop(
                dst_opt.unwrap_or_else(|| panic!(app.comm.panic("Fail. dst_opt."))),
                drop,
                board_size,
            )
        } else if let Some(dst) = dst_opt {
            UsiMove::create_walk(
                src_opt.unwrap_or_else(|| panic!(app.comm.panic("Fail. src_opt."))),
                dst,
                subject_promotion,
                board_size,
            )
        } else {
            // 目的地の分からない指し手☆（＾～＾）
            panic!(
                "Unexpected dst. Drop-none, Dst-none, move.len: '{}' > 1, move: '{}'. Slot: {:?}.",
                self.len(),
                self.to_human_presentable(rack, slot, board_size, &app),
                slot,
            )
            // , Tape file name: '{}', Tape index: {}.
            //deck.get_file_name_of_tape_box(slot),
            //match deck.get_tape_index(slot){Some(tape_index)=>{tape_index.to_string()},None=>{"".to_string()}}
        };

        // USIの指し手が作れれば、 動作の主体 が分からないことはないはず。
        if let Some(subject_idp) = subject_pid_opt {
            /*
            if app.is_debug() {
                app.comm.println("[#To best move: End]");
            }
            */
            Some(BestMove {
                usi_move: umove,
                subject_pid: subject_idp,
                subject_addr: subject_address_opt
                    .unwrap_or_else(|| panic!(app.comm.panic("Fail. subject_address_opt."))),
                capture_pid: captured_pid_opt,
                capture_addr: captured_address_opt,
            })
        } else {
            panic!("Unexpected rpm move. id fail.")
        }
    }

    pub fn to_operation_string(
        &self,
        tape_box: &CassetteTapeBox,
        board_size: BoardSize,
        app: &Application,
    ) -> String {
        let mut caret =
            Caret::new_facing_right_caret_with_number(self.caret_closed_interval.get_start());

        let mut text = String::new();
        if app.is_debug() {
            app.comm.println(&format!(
                "OpeStr step in: ({}).",
                self.caret_closed_interval.get_start()
            )); // TODO
        }
        while caret.while_to(&self.caret_closed_interval, &app) {
            if let (_taken_overflow, _awareness, Some(note)) =
                tape_box.seek_a_note_with_othre_caret(&mut caret, &app)
            {
                text = format!("{} {}", text, &note.get_ope().to_sign(board_size));
            } else {
                break;
            }
        }

        text
    }

    pub fn to_identify_string(&self, tape_box: &CassetteTapeBox, app: &Application) -> String {
        let mut caret =
            Caret::new_facing_right_caret_with_number(self.caret_closed_interval.get_start());

        let mut text = String::new();
        if app.is_debug() {
            app.comm.println(&format!(
                "IdStr ({})",
                self.caret_closed_interval.get_start()
            )); // TODO
        }
        while caret.while_to(&self.caret_closed_interval, &app) {
            if let (_taken_overflow, _awareness, Some(note)) =
                tape_box.seek_a_note_with_othre_caret(&mut caret, &app)
            {
                text = format!(
                    "{} {}",
                    text,
                    match &note.get_id() {
                        Some(pid) => pid.get_number().to_string(),
                        None => "-1".to_string(),
                    }
                );
            } else {
                break;
            }
        }

        text
    }

    /// Human presentable.
    pub fn to_human_presentable(
        &self,
        rack: &mut AudioRack,
        slot: Slot,
        board_size: BoardSize,
        app: &Application,
    ) -> String {
        let mut text = String::new();
        let mut other_caret =
            Caret::new_facing_right_caret_with_number(self.caret_closed_interval.get_start());
        if app.is_debug() {
            app.comm.print(&format!(
                "{} ({})",
                text,
                self.caret_closed_interval.get_start()
            ));
        }

        while other_caret.while_to(&self.caret_closed_interval, &app) {
            if let (_taken_overflow, _awareness, Some(note)) =
                rack.seek_a_note_with_othre_caret(slot, &mut other_caret, &app)
            {
                text = format!("{} {}", text, note.to_human_presentable(board_size, &app))
            } else {
                break;
            }
        }

        // TODO スタートが181で、エンドが1だったりするのはなんでだぜ☆（＾～＾）？
        format!(
            "[Move:{}{}]",
            self.caret_closed_interval.to_human_presentable(),
            text
        )
    }
}
