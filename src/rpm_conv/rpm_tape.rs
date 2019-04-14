/// Reversible physical move.
use board_size::*;
//use communication::*;
//use piece_etc::*;
use rpm_conv::thread::rpm_note::*;
//use rpm_conv::thread::rpm_note_operation::*;

const NONE_VALUE:i8 = -1;

#[derive(Default)]
pub struct RpmTape {
    pub notes: Vec<RpmNote>,
}
impl RpmTape {
    pub fn default() -> Self {
        RpmTape {
            notes: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.notes.clear();
    }

    /// 連結。
    pub fn append_tape(&mut self, tape:&mut RpmTape) {
        self.notes.append(&mut tape.notes);
    }

    fn up_count_retry(&mut self, cursor:i16, ply:&mut i16) {
        let rpm_note = &self.notes[cursor as usize];
        if rpm_note.is_phase_change() {
            *ply += 1;
        }
    }

    fn down_count(&mut self, note:&RpmNote, ply:&mut i16) {
        if note.get_ope().is_phase_change() {
            *ply -= 1;
        }
    }

    fn down_count_retry(&mut self, cursor:&mut i16, ply:&mut i16) {
        // フェーズ切り替えがあったら、手目を１つ減らす。
        let rpm_note = &self.notes[*cursor as usize];
        if rpm_note.is_phase_change() {
            *ply -= 1;
        }

        *cursor -= 1;
        if *cursor < 0 {
            // 何も記録していない内部状態に相当。
            return;
        }
    }

    pub fn add_note(&mut self, note:RpmNote, cursor:&mut i16, ply:&mut i16) {
        // 追加しようとしたとき、すでに後ろの要素がある場合は、後ろの要素を削除する。
        if (*cursor + 1) < self.notes.len() as i16 {
            println!("後ろの要素を削除。 {}, {}.", *cursor, self.notes.len());
            self.notes.truncate((*cursor + 1) as usize)
        };

        if self.notes.len() == (*cursor + 1) as usize {
            *cursor += 1;

            if note.get_ope().is_phase_change() {
                *ply += 1;
            }

            // 追加。
            self.notes.push(note);

        } else {
            panic!("Unexpected add: cursor: {}, len: {}.", *cursor, self.notes.len());
        }
    }

    /// カーソルが指している要素を返す。
    pub fn get_current_note(&self, cursor:i16) -> Option<RpmNote> {
        if cursor == -1 {
            None
        } else {
            Some(self.notes[cursor as usize])
        }
    }

    pub fn pop_current(&mut self, cursor:&mut i16, ply:&mut i16) -> Option<RpmNote> {
        // 後ろの要素がある場合は、削除する。
        if (*cursor + 1) < self.notes.len() as i16 {
            println!("後ろの要素を削除。 {}, {}.", *cursor, self.notes.len());
            self.notes.truncate((*cursor + 1) as usize)
        };

        if let Some(deleted_note) = self.notes.pop() {
            *cursor -= 1;
            self.down_count(&deleted_note, ply);
            Some(deleted_note)
        } else {
            // Empty.
            None
        }
    }

    /// カーソルだけ進める。
    pub fn forward(&mut self, cursor:&mut i16, ply:&mut i16) -> bool {
        if self.notes.len() as i16 <= (*cursor + 1) {
            // 進めない。
            false
        } else {
            *cursor += 1;
            self.up_count_retry(*cursor, ply);
            true
        }
    }

    /// カーソルだけ戻す。
    pub fn back(&mut self, cursor:&mut i16, ply:&mut i16) {
        if *cursor < 0 {
            // 戻れない。
            return
        };

        self.down_count_retry(cursor, ply);
    }

    /// コマンドライン入力形式。
    /// 
    /// # Returns
    /// 
    /// 駒の背番号, 操作。
    pub fn to_sign(&self, board_size:BoardSize, ply:&mut i16) -> (String, String) {
        let mut numbers = "".to_string();
        let mut operations = "".to_string();

        for note in &self.notes {
            numbers = format!("{} {}", numbers, if let Some(pid) = note.get_id() {pid.get_number().to_string()} else {NONE_VALUE.to_string()});
            operations = format!("{} {}", operations, note.get_ope().to_sign(board_size, ply));
        }

        (numbers, operations)
    }

    /// JSONファイル保存形式。
    /// 
    /// # Returns
    /// 
    /// 駒の背番号, 操作。
    pub fn to_json(&self, board_size:BoardSize, ply:&mut i16) -> (String, String) {
        let mut numbers = "".to_string();
        let mut operations = "".to_string();
        
        let mut iter = self.notes.iter();

        // 最初はカンマなし。
        if !self.notes.is_empty() {
            let note = iter.next().unwrap();
            numbers = format!("{} {}", numbers, if let Some(pid) = note.get_id() {pid.get_number().to_string()} else {NONE_VALUE.to_string()});
            operations = format!("{} \"{}\"", operations, note.get_ope().to_sign(board_size, ply));
        }

        for _index in 1..self.notes.len() {
            let note = iter.next().unwrap();
            numbers = format!("{}, {}", numbers, if let Some(pid) = note.get_id() {pid.get_number().to_string()} else {NONE_VALUE.to_string()});
            operations = format!("{}, \"{}\"", operations, note.get_ope().to_sign(board_size, ply));
        }
        
        (numbers.trim_start().to_string(), operations.trim_start().to_string())
    }
}