use moves::*;
use std::*;

/// First turn player is 0.
/// Second turn player is 1.
#[derive(Clone, Copy, PartialEq)]
pub enum Piece{
    // King is 玉.
    K0 = 0,
    // Rook is 飛.
    R0,
    // Bishop is 角.
    B0,
    // Gold is 金.
    G0,
    // Silver is 銀.
    S0,
    // kNight is 桂.
    N0,
    // Lance is 香.
    L0,
    // Pawn is 歩.
    P0,
    // Promoted rook is 竜.
    PR0,
    // Promoted bishop is 馬.
    PB0,
    // Promoted silver is 成銀.
    PS0,
    // Promoted knight is 成桂.
    PN0,
    // Promoted lance is 成香.
    PL0,
    // Promoted pawn is と.
    PP0,
    K1,
    R1,
    B1,
    G1,
    S1,
    N1,
    L1,
    P1,
    PR1,
    PB1,
    PS1,
    PN1,
    PL1,
    PP1,
    // Empty square.
    Empty,
    // Num is size or error.
    Num
}
impl fmt::Display for Piece{
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        // 文字列リテラルでないとダメみたいなんで、他に似たようなコードがあるのに、また書くことに☆（＾～＾）
        use position::Piece::*;
        match *self {
            K0 => { write!(f," K")},
            R0 => { write!(f," R")},
            B0 => { write!(f," B")},
            G0 => { write!(f," G")},
            S0 => { write!(f," S")},
            N0 => { write!(f," N")},
            L0 => { write!(f," L")},
            P0 => { write!(f," P")},
            PR0 => { write!(f,"+R")},
            PB0 => { write!(f,"+B")},
            PS0 => { write!(f,"+S")},
            PN0 => { write!(f,"+N")},
            PL0 => { write!(f,"+L")},
            PP0 => { write!(f,"+P")},
            K1 => { write!(f," k")},
            R1 => { write!(f," r")},
            B1 => { write!(f," b")},
            G1 => { write!(f," g")},
            S1 => { write!(f," s")},
            N1 => { write!(f," n")},
            L1 => { write!(f," l")},
            P1 => { write!(f," p")},
            PR1 => { write!(f,"+r")},
            PB1 => { write!(f,"+b")},
            PS1 => { write!(f,"+s")},
            PN1 => { write!(f,"+n")},
            PL1 => { write!(f,"+l")},
            PP1 => { write!(f,"+p")},
            Empty => { write!(f,"  ")},
            Num => { write!(f,"??")},
        }
    }
}

pub fn parse_sign_to_piece(line:&str, start:&mut i8) -> Piece {
    use position::Piece::*;

    if line.len() <= *start as usize + 1 {
        return Empty;
    }

    let sign = line.to_string().chars().next().unwrap();
    let pieceType = match sign {
        'K' => K0,
        'R' => R0,
        'B' => B0,
        'G' => G0,
        'S' => S0,
        'N' => N0,
        'L' => L0,
        'P' => P0,
        'k' => K1,
        'r' => R1,
        'b' => B1,
        'g' => G1,
        's' => S1,
        'n' => N1,
        'l' => L1,
        'p' => P1,
        '+' => {
            let sign = line.to_string().chars().next().unwrap();
            match sign {
                'R' => PR0,
                'B' => PB0,
                'S' => PS0,
                'N' => PN0,
                'L' => PL0,
                'P' => PP0,
                'r' => PR1,
                'b' => PB1,
                's' => PS1,
                'n' => PN1,
                'l' => PL1,
                'p' => PP1,
                _ => panic!("Failed: Sfen unexpected piece."),
            }
        },
        _ => panic!("Failed: Sfen unexpected piece."),
    };

    let sign = line.to_string().chars().next().unwrap();
    if sign == '*' {
        *start += 2;
        pieceType
    } else {
        panic!("Failed: Sfen unexpected drop.");
    }
}

pub fn promotion_piece(piece:&Piece) -> Piece {
    use position::Piece::*;

    match piece {
        R0 => PR0,
        B0 => PB0,
        S0 => PS0,
        N0 => PN0,
        L0 => PL0,
        P0 => PP0,
        R1 => PR1,
        B1 => PB1,
        S1 => PS1,
        N1 => PN1,
        L1 => PL1,
        P1 => PP1,
        _ => panic!("Failed: Sfen unexpected promotion.")
    }
}

pub fn file_rank_to_index(file:i8, rank:i8) -> usize {
    ((10 - rank)*11 + file) as usize
}

// TODO
// #[derive(Clone, Copy)]
pub struct Position {
    // With frame. 11x11.
    pub board : [Piece;121],
}
impl Position {
    pub fn new() -> Position {
        use position::Piece::*;
        Position {
            board : [
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
            ],
        }
    }

    pub fn parse(&mut self, line:&str) {
        use position::Piece::*;

        let mut start = 0;

        if line.starts_with("position startpos") {
            self.board  = [
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, L1, N1, S1, G1, K1, G1, S1, N1, L1, Empty,
                Empty, Empty, R1, Empty, Empty, Empty, Empty, Empty, B1, Empty, Empty, 
                Empty, P1, P1, P1, P1, P1, P1, P1, P1, P1, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
                Empty, P0, P0, P0, P0, P0, P0, P0, P0, P0, Empty, 
                Empty, Empty, B0, Empty, Empty, Empty, Empty, Empty, R0, Empty, Empty, 
                Empty, L0, N0, S0, G0, K0, G0, S0, N0, L0, Empty, 
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, 
            ];
            
            if line.len() > 17 {
                // `position startpos moves `. [0]p, [1]o, ...
                start = 24;

                // Examples.
                // position startpos moves 2g2f 8c8d
                let mut moves = Moves::new();
                moves.parse(line, &mut start);
                println!("Moves count: {}", moves.items.len());

                // TODO 指し手通り、進めたい。
                for mov in &moves.items {
                    self.make_move(mov);
                    self.show_board();
                }
            }
        } else if line.starts_with("position sfen ") {
            // TODO sfen under construction.

            // `position sfen `. [0]p, [1]o, ...
            start = 14;
            let mut rank=9;
            let mut file=1;

            let sign = line.to_string().chars().next().unwrap();
            let mut spaces = match sign {
                '1' => {1},
                '2' => {2},
                '3' => {3},
                '4' => {4},
                '5' => {5},
                '6' => {6},
                '7' => {7},
                '8' => {8},
                '9' => {9},
                '/' => {-1},
                _ => {0},
            };

            if spaces == 0 {
                self.set_piece(file, rank, parse_sign_to_piece(line, &mut start));
                file += 1;
            } else if spaces == -1 {
                file = 1;
                rank = 9;
            } else {
                while spaces > 0 {
                    self.set_piece(file, rank, Empty);
                    file += 1;
                    spaces -= 1;
                }
            }

            loop {
                let sign = line.to_string().chars().next().unwrap();
                if sign == ' ' {
                    break;
                }
            }
        }
    }

    pub fn show_board(&self) {
        println!("info show_board begin...");
        
        let rank_array = ['?', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];

        for rank in (1..10).rev() {
            println!(
                "info {} {}{}{}{}{}{}{}{}{}",
                rank_array[rank as usize],
                self.get_piece(1, rank),
                self.get_piece(2, rank),
                self.get_piece(3, rank),
                self.get_piece(4, rank),
                self.get_piece(5, rank),
                self.get_piece(6, rank),
                self.get_piece(7, rank),
                self.get_piece(8, rank),
                self.get_piece(9, rank));
        }
        println!("info    1 2 3 4 5 6 7 8 9");
        println!("info show_board end...");
    }

    pub fn get_piece(&self, file:i8, rank:i8) -> Piece {
        self.board[file_rank_to_index(file, rank)]
    }

    fn remove_piece(&mut self, file:i8, rank:i8) -> Piece {
        use position::Piece::*;
        let piece = self.board[file_rank_to_index(file, rank)];
        self.set_piece(file, rank, Empty);
        piece
    }

    pub fn set_piece(&mut self, file:i8, rank:i8, piece:Piece) {
        self.board[file_rank_to_index(file, rank)] = piece;
    }

    pub fn make_move(&mut self, mov:&Move){
        use moves::PieceType::*;
        
        if mov.drop != Empty {
            // TODO drop

        } else {
            let mut source_piece = self.remove_piece(mov.sourceFile, mov.sourceRank);
            if mov.promotion {
                source_piece = promotion_piece(&source_piece);
            }
            self.set_piece(mov.destinationFile, mov.destinationRank, source_piece);
        }
    }
}