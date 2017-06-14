use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct SquareGame {
    pub size: usize,
    board: Vec<f32>,
    scores: [u32; 2],
    pub remaining: HashSet<[usize; 2]>,
}

impl SquareGame {
    pub fn new(board_size: usize) -> SquareGame {
        let size = 2 * board_size + 1;

        let capacity = size * size;
        let mut board = Vec::with_capacity(capacity);
        board.resize(capacity, 0.0);

        let mut remaining = HashSet::new();
        for x in 0..size {
            for y in 0..size {
                if (x + y) % 2 == 1 {
                    remaining.insert([x, y]);
                }
            }
        }

        println!("Board size: {}", board.len());
        println!("Edge count: {}", remaining.len());

        SquareGame {
            size,
            board,
            scores: [0, 0],
            remaining,
        }
    }

    #[inline(always)]
    pub fn cell(&self, x: usize, y: usize) -> f32 {
        self.board[x * self.size + y]
    }

    #[inline(always)]
    fn set_cell(&mut self, [x, y]: [usize; 2], val: f32) {
        self.board[x * self.size + y] = val;
    }

    #[inline(always)]
    pub fn score(&self, index: usize) -> u32 {
        self.scores[index]
    }

    pub fn make_move_id(&mut self, index: [usize; 2], player: f32) -> bool {
        self.set_cell(index, player);
        self.remaining.remove(&index);

        let mut has_closed = false;
        for cell in self.edge_neighbors(index).iter().filter_map(|c| *c) {
            has_closed = self.check_cell(cell, player) || has_closed;
        }

        has_closed
    }

    fn check_cell(&mut self, index: [usize; 2], player: f32) -> bool {
        for &(x, y) in &self.cell_neighbors(index) {
            if self.cell(x, y) == 0.0 {
                return false;
            }
        }

        self.set_cell(index, player);
        if player == 1.0 {
            self.scores[0] += 1;
        } else {
            self.scores[1] += 1;
        }

        true
    }

    pub fn edge_neighbors(&self, [x, y]: [usize; 2]) -> [Option<[usize; 2]>; 2] {
        let max_index = self.size - 1;
        if y % 2 == 1 {
            [
                x.checked_sub(1).map(|x| [x, y]),
                if x < max_index {
                    Some([x + 1, y])
                } else {
                    None
                },
            ]
        } else {
            [
                y.checked_sub(1).map(|y| [x, y]),
                if y < max_index {
                    Some([x, y + 1])
                } else {
                    None
                },
            ]
        }
    }

    pub fn cell_neighbors(&self, [x, y]: [usize; 2]) -> [(usize, usize); 4] {
        [
            (x + 1, y),
            (x - 1, y),
            (x, y + 1),
            (x, y - 1),
        ]
    }
}
