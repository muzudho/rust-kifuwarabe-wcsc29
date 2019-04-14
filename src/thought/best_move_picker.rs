use communication::*;
use conf::kifuwarabe_wcsc29_config::*;
//use human::human_interface::*;
use piece_etc::*;
use position::*;
use rpm_conv::thread::rpm_move::*;
use rpm_conv::thread::rpm_thread::*;
use rpm_for_json::rpm_book_file::*;
use rpm_play::rpm_move_player::*;
use std::collections::HashMap;
use std::fs;
use usi_conv::usi_move::*;

pub struct BestMovePicker {
    thread_by_piece_id : HashMap<i8, RpmThread>,
}
impl BestMovePicker {
    pub fn default() -> Self {
        let mut instance = BestMovePicker {
            thread_by_piece_id: HashMap::new(),
        };

        instance.clear();

        instance
    }

    /// 初期状態をセットします。
    pub fn clear(&mut self) {
        self.thread_by_piece_id.clear();

        for id in PieceIdentify::iterator() {
            let number = id.get_number();
            let thread = RpmThread::new();
            self.thread_by_piece_id.insert(number, thread);
        }
    }

    pub fn get_max_note_len(&self) -> usize {
        let mut max = 0;

        for thread in self.thread_by_piece_id.values() {
            if max < thread.len() {
                max = thread.len();
            }
        }

        max
    }

    /// TODO 学習ファイルをもとに動く。
    pub fn get_best_move(&mut self, comm:&Communication, kw29config:&KifuwarabeWcsc29Config, position:&Position) -> UsiMove {

        // クリアー。
        self.clear();

        // RPMを検索。
        // println!("#get_best_move start. Phase: {:?}", position.get_phase());

        // TODO とりあえず -rpmrec.json ファイルを１個読む。
        'path_loop: for path in fs::read_dir(&kw29config.rpm_record).unwrap() {
            let file = path.unwrap().path().display().to_string();

            /*
            // 確認表示。
            {
                use piece_etc::PieceIdentify::*;
                comm.println(&format!("info file: {}, Phase: {:?}.", file, position.get_phase()));
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

            let book_file = RpmBookFile::load(&file);
            
            // ファイルの中身をすこし見てみる。
            //comm.println(&format!("file: {}, Book len: {}.", file, book_file.book.len() ));
            if !book_file.book.is_empty() {
                //comm.println(&format!("Ope len: {}, Num len: {}.", book_file.book[0].body.operation.len(), book_file.book[0].body.piece_number.len() ));

                let mut record_index = -1;

                // レコードがいっぱいある。
                for record_for_json in book_file.book {
                    record_index += 1;
                    comm.println(&format!("Record index: {}, Phase: {:?}.", record_index, position.get_phase()));

                    // 駒（0～40個）の番地を全部スキャン。（駒の先後は分からない）
                    'piece_loop: for my_piece_id in PieceIdentify::iterator() {

                        // 現局面の盤上の自駒の番地。
                        if let Some((my_idp, my_addr_obj)) = position.find_wild(Some(position.get_phase()), *my_piece_id) {
                            comm.println(&format!("My piece: {:?}, {:?}, {}.", position.get_phase(), my_idp.to_human_presentable(), my_addr_obj.to_physical_sign(position.get_board_size())));

                            // トラックをスキャン。
                            let mut note_idx = 0;
                            'track_scan: loop {

                                // とりあえず 1手分をパースします。
                                if let Some(rmove) = RpmMove::parse_1move(comm, &record_for_json, &mut note_idx, position.get_board_size()) {
                                    //comm.println("Scanning.");
                                    // どの駒が動いた１手なのか、またその番地。
                                    let (ftp_id_opt, ftp_address) = rmove.to_first_touch_piece_id(position.get_board_size());

                                    // 背番号と、アドレスの一致。
                                    if my_piece_id.get_number() == ftp_id_opt.get_number() &&
                                        ftp_address.get_index() == my_addr_obj.get_index() as usize {
                                        // 一致。

                                        //comm.println(&format!("matched address. address={}.", my_addr_obj.get_index()));
                                        comm.println(&format!("Rmove: {}.", rmove));
                                        

/*
                                        // TODO 現局面で この手を指せるか試してみる。
                                        // 例えば 味方の駒の上に駒を動かすような動きは イリーガル・タッチ として弾く。
                                        {
                                            if RpmMovePlayer::forward_1move_on_record(&comm, &mut position) {
                                                // 合法タッチ。
                                                comm.println(&format!("Rmove: {}.", rmove));

                                                // 局面を動かしてしまったので戻す。
                                                RpmMovePlayer::back_1move_on_record(&comm, &mut position);
                                            } else {
                                                // 非合法タッチ。（ムーブはキャンセルされる）
                                                comm.println(&format!("Illegal rmove: {}.", rmove));
                                                continue 'track_scan;
                                            }
                                        }
*/


                                        let mut thread = RpmThread::new();
                                        thread.push_move(rmove);

                                        //if self.thread_by_piece_id[&my_piece_id.get_number()].max_ply < thread.max_ply {
                                        // 最後に見つかったものに、差し替え。
                                        self.thread_by_piece_id.insert(my_piece_id.get_number(), thread);
                                        //comm.println("Change!");
                                        //}

                                        // TODO とりあえず抜ける。
                                        comm.println("Hit and break!");
                                        break 'piece_loop;
                                    } else {
                                        // 一致しなかったら何もしない。
                                        // No match.
                                        //comm.println(&format!("[{:>3}] --- {:>3} // -", note_idx, ftp_id.get_number()));
                                    }
                                } else {
                                    // トラックの終わり。
                                    // comm.println("Break: End of track.");
                                    break 'track_scan;
                                }
                            }
                        }
                    }

                    // いくつか読み取れれば打ち止め。
                    if self.get_max_note_len() > 0 {
                        //println!("#Break. Exit piece count = {}.", count);
                        break 'path_loop;
                    }

                } // record_loop
            } // book
        } // path_loop

        //println!("#match_thread loop end.");

        let mut best_rpm_move_opt = None;

        // １つチョイスしようぜ☆（*＾～＾*）
        'choice: for pid in PieceIdentify::iterator() {
            let pid_num = pid.get_number();
            let thread = &self.thread_by_piece_id[&pid_num];

            // Header.
            // println!("Pid: {}.", pid_num);

            // とりあえず１つチョイス☆（＾～＾）
            if !thread.is_empty() {
                best_rpm_move_opt = Some(&thread.moves[0]);

                // 検索結果を見てみようぜ☆（＾～＾）
                // Operation.
                // println!("  Ope: {} End.", rmove.to_operation_string(position.get_board_size()));

                // Identify.
                // println!("  Num: {} End.", rmove.to_identify_string());
                break 'choice;
            }
        }

        // let thread = ThreadsOfPiece {
        //     max_ply: 0,
        //     record: RpmRecord::default(),
        // };


        // 自分の駒ごとの、現局面にマッチする最長の手筋を更新していく。

        if let Some(best_rpm_move) = best_rpm_move_opt {
            let (umove, _id, _address) = best_rpm_move.to_usi_move(position.get_board_size());
            umove
        } else {
            UsiMove::create_resign()
        }
    }
}