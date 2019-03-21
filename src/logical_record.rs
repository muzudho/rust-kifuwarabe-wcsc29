use board::*;
use fen::*;
use std::*;
use logical_move::*;

pub struct LogicalRecord {
    pub items : Vec<LogicalMove>,
}
impl LogicalRecord {
    pub fn new() -> LogicalRecord {
        LogicalRecord {
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, mov:LogicalMove) {
        self.items.push(mov);
    }

    /*
    pub fn clear(&mut self) {
        self.items.clear();
    }
     */

    pub fn parse2(&mut self, line:&str, start:&mut usize) {
        self.items.clear();

        loop {
            let lmove = Fen::parse3(&line, start);
            self.items.push(lmove);

            if *start + 1 < line.len() {
                *start += 1;
            } else {
                break;
            }
        }
    }

    pub fn get_current_phase(&self) -> Phase {
        match self.items.len() % 2 {
            0 => Phase::First,
            _ => Phase::Second,
        }
    }

    pub fn make_move(&mut self, mov:LogicalMove, board:&mut Board){
        if mov.drop != None {
            // TODO drop

        } else {
            let mut source_piece = board.remove_piece(mov.source_file, mov.source_rank);
            if mov.promotion {
                source_piece = promotion_piece(source_piece);
            }
            board.set_piece(mov.destination_file, mov.destination_rank, source_piece);
            self.push(mov);
        }
    }
}