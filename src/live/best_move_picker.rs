use human::human_interface::*;
use instrument::game_player::*;
use instrument::piece_etc::*;
use instrument::position::*;
use musician::best_thread::*;
use sheet_music_format::kifu_usi::usi_move::*;
use std::collections::HashMap;
use std::fs;
use studio::application::Application;
use video_recorder::cassette_deck::*;

pub struct BestMovePicker {
    best_thread_map: HashMap<i8, BestThread>,
}
impl BestMovePicker {
    pub fn default() -> Self {
        let mut instance = BestMovePicker {
            best_thread_map: HashMap::new(),
        };

        instance.clear();

        instance
    }

    /// 初期状態をセットします。
    pub fn clear(&mut self) {
        self.best_thread_map.clear();

        for id in PieceIdentify::iterator() {
            let number = id.get_number();
            let best_thread = BestThread::new();
            self.best_thread_map.insert(number, best_thread);
        }
    }

    pub fn get_max_note_len(&self) -> usize {
        let mut max = 0;

        for best_thread in self.best_thread_map.values() {
            if max < best_thread.len() {
                max = best_thread.len();
            }
        }

        max
    }

    /// 最善手を返す。
    pub fn get_mut_best_move(
        &mut self,
        position: &mut Position,
        ply: i16,
        deck: &mut CassetteDeck,
        app: &Application,
    ) -> UsiMove {
        // クリアー。
        self.clear();

        // RPMを検索。
        println!(
            "#get_mut_best_move start. Phase: {:?}",
            position.get_phase()
        );

        // TODO とりあえず rbox.json ファイルを１個読む。
        'path_loop: for tape_box_file in fs::read_dir(&app.kw29_conf.training).unwrap() {
            // JSONファイルを元にオブジェクト化☆（＾～＾）
            deck.change_with_tape_box_file(
                &tape_box_file.unwrap().path().display().to_string(),
                position.get_board_size(),
                &app,
            );

            // トレーニング・テープ・ボックスを１箱選択。
            app.comm.println(&format!(
                "#Tape-box: {}. Phase: {:?}.",
                deck.to_human_presentable_of_training_tape_box(),
                position.get_phase()
            ));

            /*
            // 確認表示。
            {
                use piece_etc::PieceIdentify::*;
                HumanInterface::show_position(&comm, -1, &position);
                // 先手玉の番地。
                {
                    if let Some((_idp,addr_obj)) = position.find_wild(Some(Phase::First), K00) {
                        comm.println(&format!("info First-K00: {}.", addr_obj.get_index()));
                    }
                }
                {
                    if let Some((_idp,addr_obj)) = position.find_wild(Some(Phase::First), K01) {
                        comm.println(&format!("info First-K01: {}.", addr_obj.get_index()));
                    }
                }
                // 後手玉の番地。
                {
                    if let Some((_idp,addr_obj)) = position.find_wild(Some(Phase::Second), K00) {
                        comm.println(&format!("info Second-K00: {}.", addr_obj.get_index()));
                    }
                }
                {
                    if let Some((_idp,addr_obj)) = position.find_wild(Some(Phase::Second), K01) {
                        comm.println(&format!("info Second-K01: {}.", addr_obj.get_index()));
                    }
                }
            }
            */

            // テープをセット☆（＾～＾）
            while deck.change_next_if_training_tape_exists(&app) {
                // テープを１本選択☆（＾～＾）
                app.comm.println(&format!(
                    "#Tape: {}",
                    deck.to_human_presentable_of_current_tape_of_training_box(
                        position.get_board_size(),
                        &app
                    )
                ));

                // 駒（0～40個）の番地を全部スキャン。（駒の先後は分からない）
                // 'piece_loop:
                // let mut debug_count = 0;
                for my_piece_id in PieceIdentify::iterator() {
                    /*
                    if 0 <= debug_count && debug_count <= 3 {
                        // ここだけテストするぜ☆（＾～＾）
                    } else {
                        // それ以外は無視。
                        app.comm.println("デバッグ中☆（＾～＾）ループを中断。");
                        continue;
                    }
                    */

                    // 駒を１つ選択☆（＾～＾）
                    app.comm
                        .println(&format!("#Piece: {}", my_piece_id.to_human_presentable()));

                    // 現局面の盤上の自駒の番地。
                    if let Some((my_idp, my_addr_obj)) =
                        position.find_wild(Some(position.get_phase()), *my_piece_id)
                    {
                        // Display.
                        app.comm.println(&format!(
                            "[{}] Find: {}'{}'{}.",
                            deck.get_ply(Slot::Training),
                            position.get_phase().to_log(),
                            my_idp.to_human_presentable(),
                            my_addr_obj.to_physical_sign(position.get_board_size())
                        ));
                        HumanInterface::bo(deck, Slot::Learning, &position, &app);

                        let mut best_thread = BestThread::new();

                        // ノートをスキャン。
                        // TODO 次方向と、前方向の両方へスキャンしたい。
                        let mut forwarding_note_count = 0;
                        'note_scan: loop {
                            app.comm.println(&format!(
                                "[Before pattern match: Caret: {}]",
                                deck.to_human_presentable_of_caret_of_current_tape_of_training_box(
                                    &app
                                ),
                            ));

                            // 以下の４択☆（＾～＾）
                            // （１）最後の１手分。局面もキャレットも進んでいる。
                            // （２）最後ではない１手分。局面もキャレットも進んでいる。
                            // （３）テープ終わっていた。キャレットを戻す。
                            // （４）実現しない操作だった。局面とキャレットを戻す。
                            match GamePlayer::try_read_tape_for_1move(
                                deck,
                                Slot::Training,
                                position,
                                ply,
                                &app,
                            ) {
                                (is_end_of_tape, Some(rmove)) => {
                                    // ヒットしたようだぜ☆（＾～＾）
                                    forwarding_note_count += rmove.len();
                                    app.comm.println(&format!(
                                        "[After pattern match: is_end_of_tape: {}, Hit {}th note! Caret: {}, Rmove: {}]",
                                        is_end_of_tape,
                                        forwarding_note_count,
                                        deck.to_human_presentable_of_caret_of_current_tape_of_training_box(
                                            &app
                                        ),
                                        rmove.to_human_presentable(
                                            deck,
                                            Slot::Training,
                                            position.get_board_size(),
                                            &app
                                    )));

                                    // ベストムーブを作って追加しようぜ☆（＾～＾）
                                    let best_move = rmove.to_best_move(
                                        deck,
                                        Slot::Training,
                                        position.get_board_size(),
                                        &app,
                                    );
                                    best_thread.push_move(best_move);
                                    // とりあえず抜ける☆（＾～＾）
                                    break 'note_scan;
                                }
                                (true, None) => {
                                    // テープの終わりなら仕方ない☆（＾～＾）終わりだぜ☆（＾～＾）
                                    app.comm.println(&format!(
                                        "[End of tape of Piece loop: Caret: {}]",
                                        deck.to_human_presentable_of_caret_of_current_tape_of_training_box(
                                            &app
                                        ),
                                    ));
                                    break 'note_scan;
                                }
                                (false, None) => {
                                    // 一致しなかった☆（＾～＾）
                                    // 見つかるか、テープの終わりまで、続行して探せだぜ☆（＾～＾）
                                    app.comm.println("[Continue tape]");

                                    // 今のままだと　現状回帰してしまったので無限ループしてしまう☆（＾～＾）
                                    // キャレットを１手分、ごそっと進めようぜ☆（*＾～＾*）
                                    app.comm.println("[Increase caret forcely]");
                                    deck.go_1move_forcely(Slot::Training, &app);
                                }
                            }
                        }

                        // let thread_len = best_thread.len() as i16;
                        // let thread_to_human_presentable =
                        //    best_thread.to_human_presentable(position.get_board_size());
                        if !best_thread.is_empty() {
                            self.best_thread_map
                                .insert(my_piece_id.get_number(), best_thread);
                        }

                        // 指した手数分、後ろ向きに読み進めながら記録しろだぜ☆（＾～＾）
                        // TODO それを逆順にすれば　指し手だぜ☆（＾～＾）
                        app.comm.println(&format!(
                            "Tried! Then go to opponent {}th note of move! Training deck box: {}. Deck: {}.",
                            forwarding_note_count,
                            deck.to_human_presentable_of_training_tape_box(),
                            deck.to_human_presentable()
                        ));
                        // TODO ここでテープボックスが無くなっているのは　なぜなのか☆（＾～＾）？
                        deck.turn_caret_to_opponent(Slot::Training);
                        {
                            let learning_slot = &mut deck.slots[Slot::Training as usize];
                            if let Some(ref mut tape_box) = learning_slot.tape_box {
                                GamePlayer::read_tape_for_n_moves_forcely(
                                    tape_box,
                                    forwarding_note_count,
                                    position,
                                    learning_slot.ply,
                                    &app,
                                );
                            } else {
                                panic!("Tape box none in backward.");
                            }
                        }
                        deck.turn_caret_to_opponent(Slot::Training);
                        app.comm.println("Backed.");
                    }

                    //debug_count += 1;
                }

                // いくつか読み取れれば打ち止め。
                if self.get_max_note_len() > 4 {
                    println!("#Break. Exit piece count = {}.", self.get_max_note_len());
                    break 'path_loop;
                }
            } // テープのループ。
        } // トレーニング・ディレクトリー内のループ。

        //println!("#match_thread loop end.");

        let mut best_move_opt = None;

        // １つチョイスしようぜ☆（*＾～＾*）
        'choice: for pid in PieceIdentify::iterator() {
            let pid_num = pid.get_number();
            let best_thread = &self.best_thread_map[&pid_num];

            // Header.
            // println!("Pid: {}.", pid_num);

            // とりあえず１つチョイス☆（＾～＾）
            if !best_thread.is_empty() {
                best_move_opt = Some(&best_thread.moves[0]);

                // 検索結果を見てみようぜ☆（＾～＾）
                // Operation.
                // println!("  Ope: {} End.", rmove.to_operation_string(position.get_board_size()));

                // Identify.
                // println!("  Num: {} End.", rmove.to_identify_string());
                break 'choice;
            }
        }

        // let best_thread = ThreadsOfPiece {
        //     max_ply: 0,
        //     record: RpmRecord::default(),
        // };

        // 自分の駒ごとの、現局面にマッチする最長の手筋を更新していく。

        if let Some(best_move) = best_move_opt {
            best_move.usi_move
        } else {
            UsiMove::create_resign()
        }
    }

    /*
    /// 指し手単位での、パターン・マッチ。
    ///
    /// # Returns
    ///
    /// (is_end_of_tape, move_opt)
    pub fn try_read_training_tape_for_1move(
        &mut self,
        deck: &mut CassetteDeck,
        position: &mut Position,
        ply: i16,
        my_piece_id: PieceIdentify,
        my_addr_obj: Address,
        app: &Application,
    ) -> (bool, Option<ShogiMove>) {
        /*
        comm.println(&format!(
            "#>{} note.",
            note_caret.to_human_presentable()
        ));
        */
        // とりあえず 1手分ごそっと動かそうぜ☆（＾～＾）
        /// 結果は次の４つだぜ☆（＾～＾）
        /// （１）最後の１手分。局面もキャレットも進んでいる。
        /// （２）最後ではない１手分。局面もキャレットも進んでいる。
        /// （３）テープ終わっていた。キャレットを戻す。
        /// （４）実現しない操作だった。局面とキャレットを戻す。
        match GamePlayer::try_read_tape_for_1move(deck, Slot::Training, position, ply, &app) {
            (is_end_of_tape, Some(rmove)) => {
                // テープの通りに、局面をタッチしてみて１手分は　なるほど進んだようだぜ☆（＾～＾）
                // USI に変換してみようぜ☆（＾～＾）
                let bmove =
                    rmove.to_best_move(deck, Slot::Training, position.get_board_size(), &app);

                app.comm.println(&format!(
                    "#{}Rmove:{}. subject('{}'{}){}",
                    deck.to_human_presentable_of_caret_of_current_tape_of_training_box(&app),
                    rmove.to_human_presentable(
                        deck,
                        Slot::Training,
                        position.get_board_size(),
                        &app
                    ),
                    bmove.subject_pid.to_human_presentable(),
                    bmove
                        .subject_addr
                        .to_human_presentable(position.get_board_size()),
                    if let Some(cap_pid) = bmove.capture_pid {
                        format!(
                            " object('{}'{})",
                            cap_pid.to_human_presentable(),
                            bmove
                                .capture_addr
                                .unwrap()
                                .to_human_presentable(position.get_board_size())
                        )
                        .to_string()
                    } else {
                        "".to_string()
                    }
                ));

                if self.position_match(
                    deck,
                    Slot::Training,
                    position,
                    my_piece_id,
                    my_addr_obj,
                    &rmove,
                    &bmove,
                    &app,
                ) {
                    // 局面と一致。
                    // TODO 現局面で この手を指せるか試してみる。
                    // 例えば 味方の駒の上に駒を動かすような動きは イリーガル・タッチ として弾く。

                    // 新規に テープを作る。ムーブ１つだけ。
                    //let mut recorder = CassetteTapeEditor::new_cassette_tape_editor();
                    //recorder.put_1note(&rmove, comm);
                    //recorder.reset_caret();
                    /*
                    let mut ply_2 = 1;
                    let mut cassette_tape_box_2 = CassetteTapeBox::new_empty(&app);
                    {
                        let mut cassette_tape_2 = CassetteTape::from_1_move(&rmove, &app);
                        cassette_tape_box_2.change_with_tape(cassette_tape_2);
                    }
                    */
                    /*
                    println!(
                        "BMP: This move rtape: {}.",
                        recorder.to_human_presentable(position.get_board_size())
                    );
                     */

                    // TODO 同じことを２回している？
                    /// 結果は次の４つだぜ☆（＾～＾）
                    /// （１）最後の１手分。局面もキャレットも進んでいる。
                    /// （２）最後ではない１手分。局面もキャレットも進んでいる。
                    /// （３）テープ終わっていた。キャレットを戻す。
                    /// （４）実現しない操作だった。局面とキャレットを戻す。
                    match GamePlayer::try_read_tape_for_1move(
                        deck,
                        Slot::Training,
                        position,
                        ply, //ply_2,
                        &app,
                    ) {
                        (true, _) => {
                            // テープの終わり
                            (true, None)
                        }
                        (false, Some(rmove)) => {
                            // 合法タッチ。戻さず抜けます。
                            app.comm.println(&format!(
                                "Hit and go! ({}) {}",
                                bmove.subject_pid.to_human_presentable(),
                                &rmove.to_human_presentable(
                                    deck,
                                    Slot::Training,
                                    position.get_board_size(),
                                    &app
                                )
                            ));
                            HumanInterface::bo_with_tape(
                                deck,
                                Slot::Training,
                                ply, //ply_2,
                                &position,
                                &app,
                            );
                            (false, Some(rmove))
                        }
                        (false, None) => {
                            // 非合法タッチ。（自動で戻されています）
                            app.comm.println(&format!(
                                "Canceled: {}.",
                                rmove.to_human_presentable(
                                    deck,
                                    Slot::Training,
                                    position.get_board_size(),
                                    &app
                                )
                            ));
                            HumanInterface::bo_with_tape(
                                deck,
                                Slot::Training,
                                ply, // ply_2,
                                &position,
                                &app,
                            );
                            (false, None)
                        }
                    }
                } else {
                    // パターン不一致。
                    app.comm.println("[No match.]");
                    (false, None)
                }
            }
            (is_end_of_tape, None) => {
                // パターン不一致。
                app.comm.println("[No match.]");
                (is_end_of_tape, None)
            }
        }
    }
                */

    /*
    /// 現局面が一致しているか判定。
    pub fn position_match(
        &mut self,
        deck: &mut CassetteDeck,
        slot: Slot,
        position: &mut Position,
        my_piece_id: PieceIdentify,
        my_addr_obj: Address,
        rmove: &ShogiMove,
        bmove: &BestMove,
        app: &Application,
    ) -> bool {
        // パターンマッチから外れたら抜けていく。
        if my_piece_id.get_number() != bmove.subject_pid.get_number()
            || bmove.subject_addr.get_index() != my_addr_obj.get_index() as usize
        {
            // No match. 背番号と、アドレスが不一致なら何もしない。
            app.comm.println(
                "#[No-match: 背番号と、アドレスが不一致なら、何もせずループを続行]",
            );
            return false;
        }

        // 番地を指定して、そこにある駒が　相手の駒か判定。合法手だけを残す。
        if let Some(addr) = bmove.capture_addr {
            if let Some(cell) = addr.to_cell(position.get_board_size()) {
                if let Some(idp) = position.get_id_piece(cell) {
                    if let Some(_is_opponent) = idp.is_opponent(position) {
                        // 相手の駒を取った合法手。
                    } else {
                        app.comm.println(&format!(
                            "#[No-match: 味方の駒を取ってしまう。{}]",
                            rmove.to_human_presentable(deck, slot, position.get_board_size(), &app)
                        ));
                        return false;
                    }
                } else {
                    // 現局面では、取ろうとした駒がなかった。
                    app.comm.println(&format!(
                        "#[No-match: 現局面では、取ろうとした駒がなかった。{}]",
                        rmove.to_human_presentable(deck, slot, position.get_board_size(), &app)
                    ));
                    return false;
                }
            } else {
                // プログラムの不具合。
                panic!(
                    "#[IL-盤上以外の駒を取った(1)。{}]",
                    rmove.to_human_presentable(deck, slot, position.get_board_size(), &app)
                );
            }
        } else {
            // 駒を取らなかった合法手。
        };

        // パターンがマッチした。
        app.comm
            .println(&format!("#[Matched: address={}]", my_addr_obj.get_index()));
        true
    }
    */
}
