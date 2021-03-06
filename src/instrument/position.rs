use instrument::half_player_phase::*;
use instrument::piece_etc::*;
use sound::shogi_note_operation::*;
use std::*;
use studio::address::*;
use studio::application::*;
use studio::board_size::*;
use studio::parser::*;

pub const BOARD_START: usize = 0;
pub const DEFAULT_BOARD_SIZE: usize = (DEFAULT_FILE_LEN * DEFAULT_RANK_LEN) as usize;
pub const HANDS_LEN: usize = 3 * 8;

/// 指先。
pub struct Fingertip {
    id_piece: IdentifiedPiece,
    previous_address: Address,
}
impl Fingertip {
    pub fn from_idp_prev(idp: IdentifiedPiece, prev: Address) -> Self {
        Fingertip {
            id_piece: idp,
            previous_address: prev,
        }
    }

    pub fn set_fingertip(&mut self, id_piece_opt: IdentifiedPiece, previous_address_opt: Address) {
        self.id_piece = id_piece_opt;
        self.previous_address = previous_address_opt;
    }

    pub fn turn_over(&mut self) {
        self.id_piece.turn_over();
    }

    pub fn rotate_point_symmetrically(&mut self, app: &Application) {
        self.id_piece.rotate_point_symmetrically(&app);
    }

    pub fn get_fingertip(&self) -> (IdentifiedPiece, Address) {
        (self.id_piece, self.previous_address)
    }

    pub fn get_idp(&self) -> IdentifiedPiece {
        self.id_piece
    }

    pub fn get_prev(&self) -> Address {
        self.previous_address
    }
}

pub struct Position {
    phase: HalfPlayerPhaseObject,
    board_size: BoardSize,
    pub board: [Option<IdentifiedPiece>; DEFAULT_BOARD_SIZE],
    pub hands: [Vec<IdentifiedPiece>; HANDS_LEN],
    pub fingertip: Option<Fingertip>,
}
impl Position {
    // ###############
    // # Constructor #
    // ###############

    /// 本将棋用に初期化します。駒を並べる前の局面です。
    pub fn new_honshogi_origin(app: &Application) -> Self {
        if app.is_debug() {
            app.comm.println("[#Position.new_honshogi_origin]")
        }

        // このあと すぐリセットする。
        let mut instance = Self {
            phase: HalfPlayerPhaseObject::new_empty(&app),
            board_size: BoardSize::create_hon_shogi(),
            board: [None; DEFAULT_BOARD_SIZE],
            hands: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
            fingertip: None,
        };

        instance.repeat_origin_position(&app);
        instance
    }

    // #####
    // # A #
    // #####

    pub fn add_hand(&mut self, id_piece_opt: Option<IdentifiedPiece>) {
        if let Some(id_piece) = id_piece_opt {
            let hand_index = hand_id_piece_to_hand_index(id_piece);
            self.hands[hand_index].push(id_piece)
        }
    }

    /// USI position 読込時に使う。使ってない駒を盤上に置く。
    pub fn activate_piece(&mut self, piece_opt: Option<Piece>, cell: Cell, app: &Application) {
        if let Some(piece) = piece_opt {
            let disactivate_piece = piece.to_disactivate();
            let hand_index_obj = HandIndex::from_piece(disactivate_piece);
            let id_piece = self.hands[hand_index_obj.get_index()]
                .pop()
                .unwrap_or_else(|| panic!(app.comm.panic("Fail. activate_piece.")));

            let destination = self.board_size.cell_to_address(cell);
            self.board[destination] = Some(id_piece);
        }
    }

    // #####
    // # G #
    // #####

    pub fn get_phase(&self) -> HalfPlayerPhaseObject {
        self.phase
    }

    pub fn get_board_size(&self) -> BoardSize {
        self.board_size
    }

    pub fn get_id_piece(&self, cell: Cell) -> Option<IdentifiedPiece> {
        let address = self.board_size.cell_to_address(cell);
        self.board[address]
    }

    pub fn get_id_piece_by_address(&self, address: usize) -> Option<IdentifiedPiece> {
        self.board[address]
    }

    pub fn get_cell_display_by_address(&self, address: Address) -> CellDisplay {
        if address.is_fingertip() {
            // Fingertip升に表示するもの。
            if let Some(prev) = self.get_fingertip_prev() {
                // 持っている駒を表示。
                CellDisplay::from_idp_prev(self.get_fingertip_idp(), prev)
            } else {
                CellDisplay::from_empty_fingertip()
            }
        } else {
            CellDisplay::from_idp(self.board[address.get_index()])
        }
    }

    pub fn get_hand_count(&self, piece: Piece) -> i8 {
        let hand_index_obj = HandIndex::from_piece(piece);
        self.hands[hand_index_obj.get_index()].len() as i8
    }

    /// 指先の何か。
    pub fn get_fingertip_idp(&self) -> Option<IdentifiedPiece> {
        if let Some(ref fingertip) = self.fingertip {
            Some(fingertip.get_idp())
        } else {
            None
        }
    }

    /// 指先の何かが元有った場所。
    pub fn get_fingertip_prev(&self) -> Option<Address> {
        if let Some(ref fingertip) = self.fingertip {
            Some(fingertip.get_prev())
        } else {
            None
        }
    }

    // #####
    // # P #
    // #####

    pub fn peek_hand(&self, piece: Piece) -> Option<IdentifiedPiece> {
        let hand_index_obj = HandIndex::from_piece(piece);
        let stack = &self.hands[hand_index_obj.get_index()];
        if stack.is_empty() {
            None
        } else {
            Some(stack[stack.len() - 1])
        }
    }

    // #####
    // # R #
    // #####

    /// 自分の駒を持ち駒として持っているところから始めます。
    pub fn repeat_origin_position(&mut self, app: &Application) {
        //println!("#Position: reset_origin_position.");
        self.phase.repeat_phase(&app);
        self.board_size = BoardSize::create_hon_shogi();
        self.board = [None; DEFAULT_BOARD_SIZE];
        self.hands = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];

        use instrument::half_player_phase::HalfPlayerPhaseValue::*;
        use instrument::piece_etc::IdentifiedPiece;
        use instrument::piece_etc::Piece::*;
        use instrument::piece_etc::PieceIdentify::*;
        // きふわらべは 駒台の駒をスタック構造と捉えて後ろから取っていくので、
        // 大橋流の順に並べるために、逆順に駒台に追加してください。
        // 玉2枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(K2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, K00));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(K1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, K01));
        }
        // 飛2枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(R2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, R20));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(R1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, R21));
        }
        // 角2枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(B2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, B18));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(B1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, B19));
        }
        // 金4枚。（逆順）
        {
            let vec = &mut self.hands[HandIndex::from_piece(G2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, G04));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, G02));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(G1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, G05));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, G03));
        }
        // 銀4枚。（逆順）
        {
            let vec = &mut self.hands[HandIndex::from_piece(S2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, S08));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, S06));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(S1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, S09));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, S07));
        }
        // 桂4枚。（逆順）
        {
            let vec = &mut self.hands[HandIndex::from_piece(N2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, N12));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, N10));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(N1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, N11));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, N13));
        }
        // 香4枚。（逆順）
        {
            let vec = &mut self.hands[HandIndex::from_piece(L2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, L16));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, L14));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(L1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, L17));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, L15));
        }
        // 歩18枚。（逆順）
        {
            let vec = &mut self.hands[HandIndex::from_piece(P2).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P38));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P36));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P34));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P32));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P30));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P28));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P26));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P24));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P22));
        }
        {
            let vec = &mut self.hands[HandIndex::from_piece(P1).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P39));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P37));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P35));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P33));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P31));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P29));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P27));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P25));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P23));
        }
    }

    /// ゲームに使う駒がまだ決まっていないところから始めます。自由初期局面用。
    pub fn repeat_empty_position(&mut self, app: &Application) {
        //println!("#Position: reset_empty_position.");
        self.phase.repeat_phase(&app);
        self.board_size = BoardSize::create_hon_shogi();
        self.board = [None; DEFAULT_BOARD_SIZE];
        self.hands = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];

        use instrument::half_player_phase::HalfPlayerPhaseValue::*;
        use instrument::piece_etc::IdentifiedPiece;
        use instrument::piece_etc::Piece::*;
        use instrument::piece_etc::PieceIdentify::*;
        // 玉2枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(K3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, K00));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, K01));
        }
        // 飛2枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(R3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, R20));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, R21));
        }
        // 角2枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(B3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, B18));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, B19));
        }
        // 金4枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(G3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, G02));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, G04));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, G03));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, G05));
        }
        // 銀4枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(S3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, S06));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, S08));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, S07));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, S09));
        }
        // 桂4枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(N3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, N10));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, N12));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, N11));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, N13));
        }
        // 香4枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(L3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, L14));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, L16));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, L15));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, L17));
        }
        // 歩18枚。
        {
            let vec = &mut self.hands[HandIndex::from_piece(P3).get_index()];
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P22));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P24));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P26));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P28));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P30));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P32));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P34));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P36));
            vec.push(IdentifiedPiece::from_phase_pro_id(Second, false, P38));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P23));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P25));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P27));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P29));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P31));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P33));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P35));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P37));
            vec.push(IdentifiedPiece::from_phase_pro_id(First, false, P39));
        }
    }

    /// TODO 識別子を消していいのか？
    pub fn remove_id_piece(&mut self, cell: Cell) -> Option<IdentifiedPiece> {
        let address = self.get_board_size().cell_to_address(cell);
        let id_piece = self.board[address];
        self.set_id_piece(cell, None);
        id_piece
    }

    pub fn remove_hand(&mut self, piece: Piece, app: &Application) -> IdentifiedPiece {
        let hand_index_obj = HandIndex::from_piece(piece);
        self.hands[hand_index_obj.get_index()]
            .pop()
            .unwrap_or_else(|| panic!(app.comm.panic("Fail. remove_hand.")))
    }

    // #####
    // # S #
    // #####

    pub fn seek_a_player(&mut self, is_facing_left: bool, app: &Application) {
        self.phase.seek_a_player_for_position(is_facing_left, &app)
    }

    /// TODO 識別子を追加していいのか？
    /// Obsolute. new --> add().
    pub fn set_id_piece(&mut self, cell: Cell, id_piece: Option<IdentifiedPiece>) {
        let address = self.board_size.cell_to_address(cell);
        self.board[address] = id_piece;
    }

    pub fn search_hand(
        &self,
        phase_value: HalfPlayerPhaseValue,
        pid: PieceIdentify,
    ) -> Option<IdentifiedPiece> {
        let pt = pid.get_piece_type();
        let pi = Piece::from_ph_pt(phase_value, pt);

        let hand_index_obj = HandIndex::from_piece(pi);

        for idp in &self.hands[hand_index_obj.get_index()] {
            if idp.get_id() == pid {
                return Some(*idp);
            }
        }

        None
    }

    /// 駒の検索。
    ///
    /// # Returns
    ///
    /// 識別駒、番地。
    pub fn scan_pid(
        &self,
        phase_value: HalfPlayerPhaseValue,
        pid: PieceIdentify,
        //app: &Application,
    ) -> Option<(IdentifiedPiece, Address)> {
        /*
        if app.is_debug() {
            app.comm.println(&format!(
                "[#Scan pid: BOARD_START:{}, Len:{}]",
                BOARD_START,
                self.board_size.len()
            ));
        }
        */

        // 盤上のスキャン。
        for addr in BOARD_START..self.board_size.len() {
            // Id piece.
            if let Some(idp) = self.board[addr] {
                if idp.get_phase().get_state() == phase_value && idp.get_id() == pid {
                    // 駒の先後と、背番号が一致したら。
                    let addr_obj = Address::from_raw(addr);
                    return Some((idp, addr_obj));
                } /* else if app.is_debug() {
                      app.comm.println(&format!(
                          "[#Scan pid: NOT: PositionPhase:{:?}, PiecePhase:{:?}, Addr:{}, Idp:{}]",
                          phase_value,
                          idp.get_phase().get_state(),
                          addr,
                          idp.to_human_presentable()
                      ));
                  }*/
            }
        }

        // TODO 駒台のスタックの先頭かどうか分からない。あとで直すことにして　とりあえず。
        if let Some(idp) = self.search_hand(phase_value, pid) {
            let addr_obj = Address::from_hand_ph_pt(phase_value, idp.get_type());
            Some((idp, addr_obj))
        } else {
            None
        }
    }

    // #####
    // # T #
    // #####

    /// 駒台の駒を、指へ☆（＾～＾）
    pub fn try_move_hand_to_fingertip(
        &mut self,
        address: Address,
        board_size: BoardSize,
        app: &Application,
    ) -> bool {
        if let Some(ref _fingertip) = self.fingertip {
            if app.is_debug() {
                app.comm.println(&format!(
                    "[#既に何かをつかんでいた☆（＾～＾）！鳩ノ巣原理は使えない☆（＾～＾）！{}]",
                    address.to_human_presentable(board_size)
                ));
            }
            false
        } else {
            let hand_index_obj = HandIndex::from_piece(
                address
                    .get_hand_piece()
                    .unwrap_or_else(|| panic!(app.comm.panic("Fail. get_hand_piece."))),
            );

            if let Some(id_piece) = self.hands[hand_index_obj.get_index()].pop() {
                self.fingertip = Some(Fingertip::from_idp_prev(id_piece, address));
                return true;
            } else if app.is_debug() {
                app.comm.println(&format!(
                    "[#駒台に置いてない駒をつかもうとした☆（＾～＾）！{}]",
                    address.to_human_presentable(board_size)
                ));
            }
            false
        }
    }

    /// 指定のノートを実行（タッチ）するだけ。（非合法タッチでも行います）
    ///
    /// 盤、駒台（Ａ）と、スカイ升（Ｂ）の間で駒を移動する。
    /// ＡとＢは、両方空っぽか、片方だけ駒があるかの　どちらかとする。両方に駒があるケースはないものとする。
    ///
    /// トグルと考えてよい。もう一度実行すると、前の状態に戻ります。
    /// 操作が完遂できなかった場合、何もしなかった状態に戻し、偽を返す。完遂か、未着手の二者一択。
    ///
    /// # Arguments
    ///
    /// * `facing_left` - ラーニング・テープのキャレットの向き。
    ///
    /// # Returns
    ///
    /// (complete, Identified piece)
    pub fn touch_ope(
        &mut self,
        facing_left: bool,
        rnote_ope: &ShogiNoteOpe,
        app: &Application,
    ) -> (bool, Option<IdentifiedPiece>) {
        let board_size = self.get_board_size();

        match rnote_ope.address {
            Some(address) => {
                // どこかを指定した。
                if address.is_on_board(self.board_size) {
                    // 盤上。
                    match self.board[address.get_index()] {
                        Some(board_id_piece) => {
                            // 盤上の駒と、指先の何かを入れ替えます。何かには None も含まれます。（非合法でも行います）

                            let tuple = if let Some(ref fingertip) = self.fingertip {
                                if app.is_debug() {
                                    app.comm.println(&format!(
                                        "<IL-駒重なり:{}>",
                                        address.to_human_presentable(board_size)
                                    ));
                                }

                                // （未着手）指に既に何か持ってた。指に持っている駒を優先します。
                                (false, Some(fingertip.get_idp()))
                            } else {
                                // （完遂）指が空いてたので駒をつかむ。盤上の駒の方を優先します。
                                (true, Some(board_id_piece))
                            };

                            // スワップ。
                            // 盤上の何かを退避。
                            let tmp_board_idp_opt = self.board[address.get_index()];
                            // 盤上にスカイの何かを置く。
                            self.board[address.get_index()] =
                                if let Some(ref fingertip) = self.fingertip {
                                    Some(fingertip.id_piece)
                                } else {
                                    None
                                };
                            // スカイに盤上の何かを置く。
                            self.fingertip = if let Some(tmp_board_idp) = tmp_board_idp_opt {
                                Some(Fingertip::from_idp_prev(tmp_board_idp, address))
                            } else {
                                None
                            };

                            tuple
                        }
                        None => {
                            // 盤上の None と、指先の何かを入れ替えます。何かには None も含まれます。（非合法でも行います）
                            let tuple = if let Some(ref fingertip) = self.fingertip {
                                // （完遂）駒を指につまんでいた。指につまんでいる駒を置く。
                                (true, Some(fingertip.get_idp()))
                            } else {
                                if app.is_debug() {
                                    app.comm.println(&format!(
                                        "<IL-ほこり取り:{}>",
                                        address.to_human_presentable(board_size)
                                    ));
                                }
                                // （未着手）ほこりを取る。一応、違法。
                                (false, None)
                            };

                            // スワップ。
                            // 盤上の何かを退避。
                            let tmp_board_idp_opt = self.board[address.get_index()];
                            // 盤上にスカイの何かを置く。
                            self.board[address.get_index()] =
                                if let Some(ref fingertip) = self.fingertip {
                                    Some(fingertip.id_piece)
                                } else {
                                    None
                                };
                            // スカイに盤上の何かを置く。
                            self.fingertip = if let Some(tmp_board_idp) = tmp_board_idp_opt {
                                Some(Fingertip::from_idp_prev(tmp_board_idp, address))
                            } else {
                                None
                            };

                            tuple
                        }
                    }
                // 駒台。
                } else if let Some(fingertip_idp) = self.get_fingertip_idp() {
                    let id_piece_opt = Some(fingertip_idp);
                    // comm.println(&format!("hand_index = {}.", address.get_hand_index()));
                    self.add_hand(id_piece_opt);
                    self.fingertip = None;

                    // （完遂）指に何か持っていた。合法。駒台に置く。
                    (true, Some(fingertip_idp))
                } else {
                    // 盤上ではなく、指には何も持ってない。駒台の駒をつかむ。
                    if self.try_move_hand_to_fingertip(address, board_size, app) {
                        if let Some(ref fingertip) = self.fingertip {
                            // 合法。掴んだ駒を返す。
                            (true, Some(fingertip.get_idp()))
                        } else {
                            if app.is_debug() {
                                app.comm.println(&format!(
                                    "<IL-駒台ほこり取り:{}>",
                                    address.to_human_presentable(board_size)
                                ));
                            }
                            // （未着手）駒台のほこりを取った。
                            (false, None)
                        }
                    } else {
                        // （未着手）そんなことは、できなかった☆（＾～＾）
                        (false, None)
                    }
                }
            }
            None => {
                // 盤上や駒台の、どこも指していない。
                if rnote_ope.is_phase_change() {
                    self.seek_a_player(facing_left, &app);
                    // （完遂） phase change.
                    (true, None)
                } else if let Some(ref mut fingertip) = self.fingertip {
                    // 指に何か持っている。
                    if rnote_ope.fingertip_turn {
                        // （完遂）成りの操作。
                        fingertip.turn_over();
                    } else if rnote_ope.fingertip_rotate {
                        // （完遂）先後入れ替えの操作。
                        fingertip.rotate_point_symmetrically(&app);
                    };
                    (true, Some(fingertip.get_idp()))
                } else if rnote_ope.is_resign() {
                    // 投了☆
                    if app.is_debug() {
                        app.comm.println("<投了>");
                    }
                    (true, None)
                } else {
                    if app.is_debug() {
                        app.comm.println("<未定義-使っていない空間ほこり取り>");
                    }
                    // （未着手）TODO 未定義の操作。使っていない駒台でほこりを取ったり、テープの範囲外にアクセスしたり。一応、違法。
                    (false, None)
                }
            }
        }
    }

    /// 空行が多くなるものの、持ち駒を４行表示。
    /// １行に１０駒入れれば、４行で４０駒全部入る。
    /// 横幅は７０文字としておく。
    pub fn to_hand_4lines(
        &self,
        phase_value: HalfPlayerPhaseValue,
    ) -> (String, String, String, String) {
        let mut line0 = String::new();
        let mut line1 = String::new();
        let mut line2 = String::new();
        let mut line3 = String::new();

        use instrument::half_player_phase::HalfPlayerPhaseValue::*;
        use instrument::piece_etc::Piece::*;

        let array = match phase_value {
            First => [K1, R1, B1, G1, S1, N1, L1, P1],
            Second => [K2, R2, B2, G2, S2, N2, L2, P2],
            ZeroPointFive | OnePointFive => [K3, R3, B3, G3, S3, N3, L3, P3],
        };

        // まず駒を集める。
        let mut gather = Vec::new();
        for piece in &array {
            let hand_index_obj = HandIndex::from_piece(*piece);
            for id_piece in self.hands[hand_index_obj.get_index()].iter() {
                gather.push(id_piece);
            }
        }

        // ４０個の駒があるつもりで、文字列を作成する。
        for i in 0..40 {
            let piece_display = if i < gather.len() {
                gather[i].to_human_presentable()
            } else {
                "".to_string()
            };

            if i == 0 {
                line0 = if i < gather.len() {
                    piece_display
                } else {
                    "    ".to_string()
                };
            } else if i < 10 {
                line0 = format!(
                    "{} {}",
                    line0,
                    if i < gather.len() {
                        piece_display
                    } else {
                        "    ".to_string()
                    }
                );
            } else if i == 10 {
                line1 = if i < gather.len() {
                    piece_display
                } else {
                    "    ".to_string()
                };
            } else if i < 20 {
                line1 = format!(
                    "{} {}",
                    line1,
                    if i < gather.len() {
                        piece_display
                    } else {
                        "    ".to_string()
                    }
                );
            } else if i == 20 {
                line2 = if i < gather.len() {
                    piece_display
                } else {
                    "    ".to_string()
                };
            } else if i < 30 {
                line2 = format!(
                    "{} {}",
                    line2,
                    if i < gather.len() {
                        piece_display
                    } else {
                        "    ".to_string()
                    }
                );
            } else if i == 30 {
                line3 = if i < gather.len() {
                    piece_display
                } else {
                    "    ".to_string()
                };
            } else {
                line3 = format!(
                    "{} {}",
                    line3,
                    if i < gather.len() {
                        piece_display
                    } else {
                        "    ".to_string()
                    }
                );
            }
        }

        // 全角が混ざっているので、桁数では横幅調整できない。
        // 73列を使い、74列目に縦線を引きたい。
        //
        // 左端に装飾2列 ＋ 要素が最大１０個 × 1要素は横幅4列分 ＋ 隙間の半角スペースが9個 ＋ 右に22個マージン。
        //
        // 装飾を付けたりして、ちょっとだけマージンを調整する。
        (
            format!("> {}                      ", line0).to_string(),
            format!("> {}                      ", line1).to_string(),
            format!("> {}                      ", line2).to_string(),
            format!("> {}                      ", line3).to_string(),
        )
    }

    /// 余談。
    /// 将棋盤。きふわらべは、同時に１個の将棋盤しかもたない☆（＾～＾）２つ目とか無い☆（＾～＾）
    pub fn to_text(&self) -> String {
        use instrument::half_player_phase::HalfPlayerPhaseValue::*;
        let mut content = String::new();

        // 先手の持ち駒。４行表示。
        let (line0, line1, line2, line3) = self.to_hand_4lines(HalfPlayerPhaseValue::First);
        Parser::appendln(&mut content, &format!("{}|", line0));
        Parser::appendln(&mut content, &format!("{}|", line1));
        Parser::appendln(&mut content, &format!("{}|", line2));
        Parser::appendln(&mut content, &format!("{}|", line3));

        match self.phase.get_state() {
            First => {
                // hand-graphic.
                Parser::appendln(
                    &mut content,
                    "|         |   +----+----+----+----+----+----+----+----+----+             |",
                );
            }
            ZeroPointFive | OnePointFive | Second => {
                Parser::appendln(
                    &mut content,
                    "              +----+----+----+----+----+----+----+----+----+             |",
                );
            }
        }

        for row in 0..=16 {
            // let rank = row / 2 + 1;
            let rank = 9 - (row / 2);

            // 先手の手。
            match self.phase.get_state() {
                First => {
                    match row {
                        0 => Parser::append(&mut content, &"|         |  ".to_string()),
                        1 => Parser::append(&mut content, &"+---+ +-+ |  ".to_string()),
                        2 => Parser::append(&mut content, &"    | | | |  ".to_string()),
                        3 => Parser::append(&mut content, &"    | | | |  ".to_string()),
                        4 => Parser::append(&mut content, &"    +-+ +-+  ".to_string()),
                        // 全角数字がずれるので、桁数指定はしない。7桁固定。
                        5 => Parser::append(
                            &mut content,
                            &format!(
                                "     {} ",
                                self.get_cell_display_by_address(Address::from_fingertip())
                                    .to_fingertip_display(self.get_board_size())
                            ),
                        ),
                        6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => {
                            Parser::append(&mut content, &"             ".to_string())
                        }
                        _ => panic!("Unexpected row: {0}.", row),
                    };
                }
                OnePointFive => {
                    match row {
                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 => {
                            Parser::append(&mut content, &"             ".to_string())
                        }
                        // 全角数字がずれるので、桁数指定はしない。7桁固定。
                        11 => Parser::append(
                            &mut content,
                            &format!(
                                "     {} ",
                                self.get_cell_display_by_address(Address::from_fingertip())
                                    .to_fingertip_display(self.get_board_size())
                            ),
                        ),
                        12 => Parser::append(&mut content, &"----------+  ".to_string()),
                        13 => Parser::append(&mut content, &"      +---+  ".to_string()),
                        14 => Parser::append(&mut content, &"      +---+  ".to_string()),
                        15 => Parser::append(&mut content, &"      +---+  ".to_string()),
                        16 => Parser::append(&mut content, &"------+      ".to_string()),
                        _ => panic!("Unexpected row: {0}.", row),
                    };
                }
                ZeroPointFive | Second => {
                    Parser::append(&mut content, &"             ".to_string())
                }
            }

            if row % 2 == 0 {
                Parser::append(
                    &mut content,
                    &format!(
                        // 全角文字が混ざると、文字数では横幅調整できない。
                        "{0}|{1}|{2}|{3}|{4}|{5}|{6}|{7}|{8}|{9}|",
                        rank, // Parser::i8_to_rank_char(rank),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(1, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(2, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(3, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(4, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(5, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(6, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(7, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(8, rank),
                            self.board_size
                        ))
                        .to_display(),
                        self.get_cell_display_by_address(Address::from_cell(
                            Cell::from_file_rank(9, rank),
                            self.board_size
                        ))
                        .to_display()
                    ),
                );
            } else {
                Parser::append(
                    &mut content,
                    &" +----+----+----+----+----+----+----+----+----+".to_string(),
                );
            }

            // Second player finger.
            match self.phase.get_state() {
                OnePointFive | First => Parser::append(&mut content, "             |"),
                ZeroPointFive => {
                    match row {
                        0 => Parser::append(&mut content, "     +------ |"),
                        1 => Parser::append(&mut content, " +---+       |"),
                        2 => Parser::append(&mut content, " +---+       |"),
                        3 => Parser::append(&mut content, " +---+       |"),
                        4 => Parser::append(&mut content, " +---------- |"),
                        5 => Parser::append(
                            &mut content,
                            &format!(
                                "  {}    |",
                                self.get_cell_display_by_address(Address::from_fingertip())
                                    .to_fingertip_display(self.get_board_size())
                            ),
                        ),
                        6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => {
                            Parser::append(&mut content, "             |")
                        }
                        _ => panic!("Unexpected row: {0}.", row),
                    };
                }
                Second => {
                    match row {
                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 => {
                            Parser::append(&mut content, "             |")
                        }
                        11 => Parser::append(
                            &mut content,
                            &format!(
                                "  {}    |",
                                self.get_cell_display_by_address(Address::from_fingertip())
                                    .to_fingertip_display(self.get_board_size())
                            ),
                        ),
                        12 => Parser::append(&mut content, " +-+ +-+     |"),
                        13 => Parser::append(&mut content, " | | | |     |"),
                        14 => Parser::append(&mut content, " | | | |     |"),
                        15 => Parser::append(&mut content, " | +-+ +---+ |"),
                        16 => Parser::append(&mut content, " |         | |"),
                        _ => panic!("Unexpected row: {0}.", row),
                    };
                }
            }

            Parser::append_ln(&mut content);
        }

        match self.phase.get_state() {
            ZeroPointFive | OnePointFive | First => {
                Parser::appendln(
                    &mut content,
                    "              +----+----+----+----+----+----+----+----+----+             |",
                );
            }
            Second => {
                // hand.
                Parser::appendln(
                    &mut content,
                    "              +----+----+----+----+----+----+----+----+----+ |         | |",
                );
            }
        }

        Parser::appendln(
            &mut content,
            "               1    2    3    4    5    6    7    8    9                 |",
        );

        // 後手の持ち駒。４行表示。
        let (line0, line1, line2, line3) = self.to_hand_4lines(HalfPlayerPhaseValue::Second);
        Parser::appendln(&mut content, &format!("{}|", line0));
        Parser::appendln(&mut content, &format!("{}|", line1));
        Parser::appendln(&mut content, &format!("{}|", line2));
        Parser::appendln(&mut content, &format!("{}|", line3));

        content
    }
}
