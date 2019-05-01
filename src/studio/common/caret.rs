use studio::application::Application;
use studio::common::closed_interval::ClosedInterval;
use studio::communication::*;

/// 負の方のキャレット番地を符号を反転して１引いて配列のインデックスを作る補正に使う☆（＾～＾）
pub const MINUS_ZERO_LEN: usize = 1;
pub fn get_index_from_caret_numbers(caret_number: i16) -> usize {
    if -1 < caret_number {
        caret_number as usize
    } else {
        -caret_number as usize + MINUS_ZERO_LEN
    }
}

/// 余談。
/// キャレットは、クリアーするとかリセットすることはできない☆（＾～＾）
/// 常に現在位置を示す☆（＾～＾）
/// 初期位置は持たない☆（＾～＾）
/// できるか、できないかではない、これは　そうであるべき　という　思想　だぜ☆（*＾～＾*）
pub struct Caret {
    facing_left: bool,
    number: i16,
}
impl Caret {
    pub fn new_facing_right_caret() -> Self {
        Caret::new_facing_right_caret_with_number(0)
    }

    pub fn new_facing_right_caret_with_number(init_num: i16) -> Self {
        Caret {
            facing_left: false,
            number: init_num,
        }
    }

    /// 要素を返してから、向きの通りに移動します。境界チェックは行いません。
    /// 境界なんかないから、どんどん　進んでいくぜ☆（＾～＾）
    pub fn go_to_next(&mut self, _app: &Application) -> i16 {
        let old = self.number;

        if self.facing_left {
            self.number -= 1;
        } else {
            self.number += 1;
        }

        /*
        // ログ出力☆（＾～＾）
        {
            if self.is_facing_left() {
                app.comm
                    .print(&format!("[Caret: {}<--{}]", self.number, old).to_string());
            } else {
                app.comm
                    .print(&format!("[Caret: {}-->{}]", old, self.number).to_string());
            }
        }
         */

        old
    }

    /// 足踏みする☆（＾～＾）
    /// ミュータブルにしたくない場合だけ使う☆（＾～＾）なるべく go_to_next を使えだぜ☆（＾～＾）
    pub fn step_in(&self, _comm: &Communication) -> i16 {
        self.number
    }

    /// ちょっと戻りたいときに☆（＾～＾）
    pub fn go_back(&mut self, _app: &Application) -> i16 {
        let old = self.number;

        if self.facing_left {
            self.number += 1;
        } else {
            self.number -= 1;
        }

        /*
        // ログ出力☆（＾～＾）
        {
            if self.is_facing_left() {
                app.comm
                    .print(&format!("[CaretBK: {}<--{}]", self.number, old).to_string());
            } else {
                app.comm
                    .print(&format!("[CaretBK: {}-->{}]", old, self.number).to_string());
            }
        }
        */

        old
    }

    pub fn is_internal_of(&self, closed_interval: ClosedInterval) -> bool {
        closed_interval.get_minimum_caret_number() <= self.number
            && self.number <= closed_interval.get_maximum_caret_number()
    }

    /// その場で、向きだけ変えるぜ☆（＾～＾）
    pub fn look_back_to_negative(&mut self, app: &Application) {
        // app.comm.print("[LookBack N]");
        if !self.is_facing_left() {
            // 向きを変えるだけでは、回転テーブル・ターン☆（＾～＾）
            self.facing_left = true;

            // 振り返ってから、１歩前へ☆（＾～＾）
            self.go_to_next(&app);
        }
    }

    /// その場で、向きだけ変えるぜ☆（＾～＾）
    pub fn look_back_to_positive(&mut self, app: &Application) {
        // app.comm.print("[LookBack P]");
        if self.is_facing_left() {
            // 向きを変えるだけでは、回転テーブル・ターン☆（＾～＾）
            self.facing_left = false;

            // 振り返ってから、１歩前へ☆（＾～＾）
            self.go_to_next(&app);
        }
    }

    /// その場で、向きだけ変えるぜ☆（＾～＾）
    pub fn look_back_to_opponent(&mut self, app: &Application) {
        // app.comm.print("[LookBack O]");

        // 向きを変えるだけでは、回転テーブル・ターン☆（＾～＾）
        self.facing_left = !self.facing_left;

        // 振り返ってから、１歩前へ☆（＾～＾）
        self.go_to_next(&app);
    }

    pub fn is_facing_left(&self) -> bool {
        self.facing_left
    }

    /// 等しい。
    pub fn equals(&self, target: i16) -> bool {
        self.number == target
    }
    /// target 以上。
    pub fn is_greater_than_or_equal_to(&self, target: i16) -> bool {
        target <= self.number
    }
    pub fn while_to(&self, target: &ClosedInterval, _app: &Application) -> bool {
        if self.is_facing_left() {
            /*
            app.comm.print(&format!(
                "[min:{}, num:{}]",
                target.get_minimum_caret_number(),
                self.number
            ));
            */
            target.get_minimum_caret_number() < self.number
        } else {
            /*
            app.comm.print(&format!(
                "[num:{}, max:{}]",
                self.number,
                target.get_maximum_caret_number(),
            ));
            */
            self.number < target.get_maximum_caret_number()
        }
    }

    /// マイナスゼロが無いので、負の配列ではインデックスを１小さくします。
    pub const NEGATIVE_ZERO_LEN: i16 = 1;

    /// トランケート用に使う。
    ///
    /// 向いている方向に関わらず、正か負かを返します。
    /// インデックスを返します。負の配列では 数を 0 側に 1 つ寄せます。
    ///
    /// # Returns
    ///
    /// (is_positive, index)
    pub fn to_index_for_truncation(&self) -> (bool, usize) {
        // 正と負で、0 の扱い方が異なることに注意。
        if self.is_facing_left() {
            // 負の無限大の方を向いているとき。
            if self.number <= 0 {
                // 0以下の左隣は負
                (false, get_index_from_caret_numbers(self.number))
            } else {
                // 1以上の左隣は正。
                (true, get_index_from_caret_numbers(self.number))
            }
        } else {
            // 正の無限大の方を向いているとき。
            if self.number >= 0 {
                // 0以上の右隣は正。
                (true, get_index_from_caret_numbers(self.number))
            } else {
                // 0未満の右隣は負。
                (false, get_index_from_caret_numbers(self.number))
            }
        }
    }

    /// デバッグ表示用。
    pub fn to_human_presentable(&self, _app: &Application) -> String {
        if self.is_facing_left() {
            format!("[Caret: <--{}]", self.number).to_string()
        } else {
            format!("[Caret: {}-->]", self.number).to_string()
        }
    }
}
