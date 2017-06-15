use std::sync::mpsc::Receiver;

use game::*;
use tree::*;
use display::*;

fn wait_for_player(game: &mut SquareGame, receiver: &Receiver<(f32, f32)>, board_size: usize) -> bool {
    loop {
        if let Ok((x, y)) = receiver.recv() {
            let slices = (5.0 * board_size as f32) + 1.0;
            let x = (x * slices).floor() as usize;
            let y = (y * slices).floor() as usize;

            let edge_x = x / 5 * 2;
            let edge_y = y / 5 * 2;
            let action = match (x % 5 == 0, y % 5 == 0) {
                (false, true) => {
                    Some([edge_x + 1, edge_y])
                },
                (true, false) => {
                    Some([edge_x, edge_y + 1])
                },
                _ => None,
            };

            if let Some(action) = action {
                if game.remaining.contains(&action) {
                    println!("{:?}", action);
                    return game.make_move_id(action, 1.0);
                }
            }
        }
    }
}

pub enum GameMode {
    Autoplay,
    Game,
}

pub fn play(board_size: usize, mode: GameMode) {
    let mut game = SquareGame::new(board_size);
    let (sender, receiver, handle) = display_thread(board_size);

    let mut player = 0;
    while !game.remaining.is_empty() {
        sender.send(game.clone()).expect("send");

        let has_closed = if player == 0 {
            match mode {
                GameMode::Autoplay => play_ia(&mut game, player),
                GameMode::Game => wait_for_player(&mut game, &receiver, board_size),
            }
        } else {
            play_ia(&mut game, player)
        };

        if !has_closed {
            player = (player + 1) & 1;
        }
    }

    sender.send(game.clone()).expect("send");
    handle.join().unwrap();
}
