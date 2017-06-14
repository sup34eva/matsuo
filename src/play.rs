use std::sync::mpsc::Receiver;

use data;
use evol::*;
use game::*;
use nnet::*;
use display::*;

fn wait_for_turn(game: &mut SquareGame, receiver: &Receiver<(f32, f32)>, board_size: usize) -> bool {
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

pub fn play(board_size: usize) {
    let mut game = SquareGame::new(board_size);
    let (sender, receiver, handle) = display_thread(board_size);

    let (_, genomes) = data::load().unwrap();
    let (data, _) = {
        genomes.into_iter()
            .max_by_key(|&(_, score)| score)
            .unwrap()
    };

    let agent = Network::from_data(data);

    let mut player = 0;
    while !game.remaining.is_empty() {
        sender.send(Message::Play(game.clone())).expect("send");

        let has_closed = if player == 0 {
            wait_for_turn(&mut game, &receiver, board_size)
        } else {
            agent_turn(&mut game, player, &agent)
        };

        if !has_closed {
            player = (player + 1) & 1;
        }
    }

    handle.join().unwrap();
}
