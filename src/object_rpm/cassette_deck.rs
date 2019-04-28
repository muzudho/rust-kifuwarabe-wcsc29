use application::Application;
use board_size::BoardSize;
use human::human_interface::*;
use object_rpm::cassette_tape::CassetteTape;
use object_rpm::cassette_tape_box::*;
use object_rpm::shogi_move::ShogiMove;
use object_rpm::shogi_note::ShogiNote;
use shogi_ban::position::*;

pub enum Slot {
    /// トレーニング。
    Training,
    /// ラーニング。
    Learning,
}

pub struct CassetteSlot {
    /// 何も指していない状態で 1。
    /// TODO 本将棋の大橋流の最初の玉は Ply=-39 にしたい。
    /// トレーニング・テープの 手目。
    pub ply: i16,

    /// テープ・ボックス。
    /// 選んでいるテープを示すインデックスを持つ。
    pub tape_box: Option<CassetteTapeBox>,
}
impl CassetteSlot {
    pub fn new_t() -> Self {
        CassetteSlot {
            ply: 1,
            tape_box: None,
        }
    }

    pub fn new_l(app: &Application) -> Self {
        CassetteSlot {
            ply: 1,
            tape_box: Some(CassetteTapeBox::new_empty(&app)),
        }
    }
}

/// カセット・デッキ。
pub struct CassetteDeck {
    // カセットのスロット。トレーニング、ラーニングの順。
    pub slots: [CassetteSlot; 2],
}
impl CassetteDeck {
    pub fn new_empty(app: &Application) -> Self {
        CassetteDeck {
            slots: [CassetteSlot::new_t(), CassetteSlot::new_l(&app)],
        }
    }

    /// トレーニング・テープを交換するぜ☆（＾～＾）
    ///
    /// 棋譜読取の際は、テープ・ボックスの中にテープ・インデックスが指定されてある☆（＾～＾）
    pub fn change(
        &mut self,
        training_tape_box_opt: Option<CassetteTapeBox>,
        board_size: BoardSize,
        app: &Application,
    ) {
        // トレーニング・テープは読み取り専用なんで、べつに保存とかしない☆（＾～＾）
        self.slots[Slot::Training as usize].tape_box = training_tape_box_opt;
        self.slots[Slot::Training as usize].ply = 1;

        if let Some(learning_box) = self.slots[Slot::Learning as usize].tape_box {
            // ラーニング・テープの、テープ・ボックスを外部ファイルに保存する（今までのラーニング・テープは、このテープ・ボックスに入っている）。
            let file_name = learning_box.get_file_name();
            learning_box
                .to_rpm(board_size)
                .write(file_name, board_size, &app.comm);

            if 499 < learning_box.len() {
                // TODO 満杯になったら次のボックスを新しく作りたい☆（＾～＾）
                self.slots[Slot::Learning as usize].tape_box =
                    Some(CassetteTapeBox::new_empty(&app));
            }
        } else {
            panic!("Get l_box none.")
        }

        // 新しいラーニング・テープに差し替える。
        if let Some(learning_box) = self.slots[Slot::Learning as usize].tape_box {
            learning_box.change(&app);
            self.slots[Slot::Learning as usize].ply = 1;
        } else {
            panic!("Get l_box none.")
        }
    }

    pub fn get_mut_tape(&mut self, slot: Slot, app: &Application) -> &mut CassetteTape {
        if let Some(ref mut tape_box) = self.slots[slot as usize].tape_box {
            &mut tape_box.get_current_tape()
        } else {
            panic!("t_tape none.");
        }
    }

    /// テープ・フラグメント単位で書き込めるぜ☆（*＾～＾*）スロットは ラーニング限定☆（＾～＾）
    pub fn write_tape_fragment(&mut self, board_size: BoardSize, app: &Application) {
        if let Some(tape_box) = self.slots[Slot::Learning as usize].tape_box {
            tape_box.write_tape_fragment_of_current_tape(board_size, &app);
        } else {
            panic!("tape box none.");
        }
    }

    pub fn write_tape_box(&mut self, board_size: BoardSize, app: &Application) {
        if let Some(tape_box) = self.slots[Slot::Learning as usize].tape_box {
            tape_box.write_tape_box(board_size, &app);
        } else {
            panic!("tape box none.");
        }
    }

    /// テープの文字化。
    pub fn to_mut_tape_sign(
        &mut self,
        slot: Slot,
        board_size: BoardSize,
        app: &Application,
    ) -> (String, String) {
        self.get_mut_tape(slot, &app).to_sign(board_size)
    }

    pub fn get_ply(&self, slot: Slot) -> i16 {
        self.slots[slot as usize].ply
    }

    pub fn turn_caret_to_opponent(&self, slot: Slot) {
        if let Some(tape_box) = self.slots[slot as usize].tape_box {
            tape_box.turn_caret_to_opponent();
        } else {
            panic!("tape box none.");
        }
    }

    pub fn put_1move(&mut self, slot: Slot, rmove: &ShogiMove, app: &Application) {
        for note in rmove.notes.iter() {
            self.put_1note(slot, *note, app);
            if let Some(ply) = note.get_ope().get_phase_change() {
                // フェーズ更新。
                self.slots[slot as usize].ply = ply;
            }
        }
    }

    /// キャレット位置に、ノートを上書き、または追加をするぜ☆（＾～＾）
    pub fn put_1note(&mut self, slot: Slot, note: ShogiNote, app: &Application) {
        if let Some(tape_box) = self.slots[slot as usize].tape_box {
            let (is_positive, index) = tape_box.get_current_tape().caret.to_index();

            if is_positive {
                // 正のテープ。
                // 最先端かどうか判断。
                if tape_box.get_current_tape().is_positive_peak()
                    && !tape_box.get_current_tape().caret.is_facing_left()
                {
                    // 正の絶対値が大きい方の新しい要素を追加しようとしている。
                    if let Some(tape_box) = self.slots[slot as usize].tape_box {
                        tape_box.push_note_to_positive_of_current_tape(note);
                        tape_box.go_caret_to_next(&app);
                    }
                } else {
                    // 先端でなければ、上書き。
                    if let Some(tape_box) = self.slots[slot as usize].tape_box {
                        tape_box.set_note_to_positive_of_current_tape(index, note);
                        tape_box.go_caret_to_next(&app);

                        // 仮のおわり を更新。
                        let (_is_positive, index) = tape_box.get_caret_index_of_current_tape();
                        tape_box.truncate_positive_of_current_tape(index);
                    }
                }
            } else {
                // 負のテープ。
                // 最先端かどうか判断。
                if tape_box.get_current_tape().is_negative_peak()
                    && tape_box.get_current_tape().caret.is_facing_left()
                {
                    // 負の絶対値が大きい方の新しい要素を追加しようとしている。
                    if let Some(tape_box) = self.slots[slot as usize].tape_box {
                        tape_box.push_note_to_negative_of_current_tape(note);
                        tape_box.go_caret_to_next(&app);
                    }
                } else {
                    // 先端でなければ、上書き。
                    if let Some(tape_box) = self.slots[slot as usize].tape_box {
                        tape_box.set_note_to_negative_of_current_tape(index, note);
                        tape_box.go_caret_to_next(&app);

                        // 仮のおわり を更新。
                        let (_is_positive, index) = tape_box.get_caret_index_of_current_tape();
                        tape_box.truncate_negative_of_current_tape(index);
                    }
                }
            }
        } else {
            panic!("Recording tape is none.")
        };
    }

    /// # Returns
    /// TODO ply が変わることがある。
    ///
    /// 削除したノート。
    pub fn delete_1note(&mut self, slot: Slot, app: &Application) -> Option<ShogiNote> {
        if let Some(tape_box) = self.slots[slot as usize].tape_box {
            tape_box.delete_1note(&app)
        } else {
            None
        }
    }

    /// 棋譜のカーソルが指している要素を削除して、１つ戻る。
    /// TODO ply が変わることがある。
    ///
    /// # Returns
    ///
    /// 削除したノート。
    pub fn pop_1note(
        &mut self,
        slot: Slot,
        position: &mut Position,
        app: &Application,
    ) -> Option<ShogiNote> {
        HumanInterface::show_position(&app.comm, -1, position);

        if let Some(rpm_note) = self.delete_1note(slot, &app) {
            let board_size = position.get_board_size();
            let (_is_legal_touch, _piece_identify_opt) =
                position.touch_beautiful_1note(&rpm_note.get_ope(), &app.comm, board_size);
            Some(rpm_note)
        } else {
            None
        }
    }

    /// 1手削除する。
    ///
    /// TODO ply が変わる。
    pub fn pop_1move(&mut self, slot: Slot, position: &mut Position, app: &Application) {
        let mut count = 0;
        // 開始前に達したら終了。
        while let Some(rpm_note) = self.pop_1note(slot, position, app) {
            if count != 0 && rpm_note.is_phase_change() {
                // フェーズ切り替えしたら終了。（ただし、初回除く）
                break;
            }

            // それ以外は繰り返す。
            count += 1;
        }
    }
}
