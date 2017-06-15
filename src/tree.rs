use rayon::prelude::*;
use game::*;

pub fn play_ia(game: &mut SquareGame, id: usize) -> bool {
    let action = {
        game.remaining
            .par_iter()
            .max_by_key(|action| -> u16 {
                game.edge_neighbors(**action)
                    .into_iter()
                    .filter_map(|cell| {
                        cell.map(|cell| {
                            let count = {
                                game.cell_neighbors(cell)
                                    .into_iter()
                                    .map(|&(x, y)| {
                                        if game.cell(x, y) == 0.0 {
                                            0
                                        } else {
                                            1
                                        }
                                    })
                                    .sum()
                            };

                            match count {
                                0 => 2,
                                1 => 1,
                                2 => 0,
                                3 => 3,
                                _ => unreachable!(),
                            }
                        })
                    })
                    .sum()
            })
            .unwrap()
            .clone()
    };

    game.make_move_id(action, id as f32 + 1.0)
}
