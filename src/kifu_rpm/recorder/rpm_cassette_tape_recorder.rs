use board_size::*;
use common::caret::*;
use communication::*;
use human::human_interface::*;
use kifu_rpm::object::rpm_cassette_tape::*;
use kifu_rpm::play::rpm_note_player::*;
use kifu_rpm::thread::rpm_move::*;
use kifu_rpm::thread::rpm_note::*;
use kifu_rpm::thread::rpm_note_operation::*;
use position::*;
use std::*;

pub struct RpmCassetteTapeRecorder {
    /// 何も指していない状態で 1。
    /// TODO 本将棋の大橋流の最初の玉は Ply=-39 にしたい。
    pub ply: i16,
    pub cassette_tape: RpmCassetteTape,
}
impl RpmCassetteTapeRecorder {
    pub fn new_cassette_tape_recorder() -> Self {
        RpmCassetteTapeRecorder {
            ply: 1,
            cassette_tape: RpmCassetteTape::new_facing_right_cassette_tape(),
        }
    }

    pub fn from_cassette_tape(ply_num: i16, inner_cassette_tape: RpmCassetteTape) -> Self {
        RpmCassetteTapeRecorder {
            ply: ply_num,
            cassette_tape: inner_cassette_tape,
        }
    }

    pub fn clear_recorder(&mut self) {
        self.ply = 1;
        self.cassette_tape.clear();
    }

    pub fn reset_caret(&mut self) {
        self.cassette_tape.reset_caret();
    }

    /// キャレット位置に、ノートを上書き、または追加をするぜ☆（＾～＾）
    pub fn record_note(&mut self, note: RpmNote, comm: &Communication) {
        let (is_positive, index) = self.cassette_tape.caret.to_index();

        if is_positive {
            // 正のテープ。
            // 最先端かどうか判断。
            if self.cassette_tape.is_positive_peak() && !self.cassette_tape.caret.is_facing_left() {
                // 正の絶対値が大きい方の新しい要素を追加しようとしている。
                self.cassette_tape.tape.positive_notes.push(note);
                self.cassette_tape.caret.go_next(comm, "record_note+new");
            } else {
                // 先端でなければ、上書き。
                self.cassette_tape.tape.positive_notes[index] = note;
                self.cassette_tape.caret.go_next(comm, "record_note+exists");

                // 仮のおわり を更新。
                let (_is_positive, index) = self.cassette_tape.caret.to_index();
                self.cassette_tape.tape.positive_notes.truncate(index);
            }
        } else {
            // 負のテープ。
            // 最先端かどうか判断。
            if self.cassette_tape.is_negative_peak() && self.cassette_tape.caret.is_facing_left() {
                // 負の絶対値が大きい方の新しい要素を追加しようとしている。
                self.cassette_tape.tape.negative_notes.push(note);
                self.cassette_tape.caret.go_next(comm, "record_note-new");
            } else {
                // 先端でなければ、上書き。
                self.cassette_tape.tape.negative_notes[index] = note;
                self.cassette_tape.caret.go_next(comm, "record_note-exists");

                // 仮のおわり を更新。
                let (_is_positive, index) = self.cassette_tape.caret.to_index();
                self.cassette_tape.tape.negative_notes.truncate(index);
            }
        }
    }

    pub fn record_move(&mut self, rmove: &RpmMove, comm: &Communication) {
        for note in rmove.notes.iter() {
            self.record_note(*note, comm);
            if let Some(recorded_ply) = note.get_ope().get_phase_change() {
                self.ply = recorded_ply;
            }
        }
    }

    pub fn delete(&mut self) -> Option<RpmNote> {
        let (new_tape, removed_note_opt) = self
            .cassette_tape
            .tape
            .new_truncated_tape(&self.cassette_tape.caret);
        self.cassette_tape.tape = new_tape;

        if let Some(removed_note) = removed_note_opt {
            if let Some(recorded_ply) = removed_note.get_ope().get_phase_change() {
                self.ply = recorded_ply;
            }

            Some(removed_note)
        } else {
            None
        }
    }

    /// Human presentable large log.
    pub fn to_human_presentable(&self, board_size: BoardSize) -> String {
        self.cassette_tape.to_human_presentable(board_size)
    }

    /// 棋譜読取。
    pub fn read_tape(&mut self, comm: &Communication, line: &str, position: &mut Position) {
        let mut caret = Caret::new_facing_right_caret();

        loop {
            if caret.is_greater_than_or_equal_to(line.len() as i16) {
                return;
            }

            let tuple = RpmNoteOpe::parse_1ope(&line, &mut caret, position.get_board_size(), &comm);

            if let (_last_used_caret, Some(rnote_ope)) = tuple {
                comm.println("rpm_cassette_tape_recorder.rs:read_tape: touch_brandnew_note");
                RpmNotePlayer::touch_brandnew_note(&rnote_ope, position, self, comm);

                let ply = if let Some(ply) = rnote_ope.get_phase_change() {
                    ply
                } else {
                    -1
                };
                HumanInterface::bo(comm, &self.cassette_tape, ply, &position);
            }
        }
    }
}