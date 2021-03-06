use musician::best_move::*;
use studio::application::Application;
use studio::board_size::BoardSize;

/// 手筋１個分。読み筋。
#[derive(Default)]
pub struct BestThread {
    pub moves: Vec<BestMove>,
}
impl BestThread {
    pub fn new() -> Self {
        BestThread { moves: Vec::new() }
    }

    pub fn from_buffer(moves_buf: Vec<BestMove>) -> Self {
        BestThread { moves: moves_buf }
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn to_human_presentable(&self, board_size: BoardSize, app: &Application) -> String {
        let mut text = String::new();

        for bmove in &self.moves {
            text = format!("{} {}", text, bmove.to_human_presentable(board_size, &app))
        }

        text.to_string()
    }
}
