use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

use data;
use nnet::*;
use game::*;
use display::{Message, display_thread};

fn load_evol(options: Options) -> (usize, Evolver) {
    data::load()
        .map_or_else(
            || {
                let evol = Evolver::new(options.clone());
                (0, evol)
            },
            |(id, genomes)| {
                let evol = Evolver::from_save(options.clone(), genomes);
                (id + 1, evol)
            }
        )
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Confidence(f32);

impl From<Vec<f32>> for Confidence {
    fn from(list: Vec<f32>) -> Self {
        Confidence(list[0])
    }
}

impl Eq for Confidence {}
impl Ord for Confidence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn agent_turn(game: &mut SquareGame, id: usize, agent: &Network) -> bool {
    let action = {
        game.remaining
            .par_iter()
            .max_by_key(|action| {
                let state: Vec<_> = {
                    game.edge_neighbors(**action)
                        .into_iter()
                        .flat_map(|cell| match *cell {
                            Some(cell) => {
                                game.cell_neighbors(cell)
                                    .into_iter()
                                    .filter_map(|&(x, y)| {
                                        if x == cell[0] && y == cell[1] {
                                            None
                                        } else if game.cell(x, y) == 0.0 {
                                            Some(0.0)
                                        } else {
                                            Some(1.0)
                                        }
                                    })
                                    .collect()
                            },
                            None => vec![
                                -1.0, -1.0, -1.0,
                            ],
                        })
                        .collect()
                };

                let res = agent.compute(&state);
                Confidence::from(res)
            })
            .unwrap()
            .clone()
    };

    game.make_move_id(action, id as f32 + 1.0)
}

fn play_systemic(game: &mut SquareGame, id: usize) -> bool {
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

fn run_game(game: &mut SquareGame, agents: &[Network]) -> [u32; 2] {
    let mut player = 0;
    while !game.remaining.is_empty() {
        let has_closed = if player == 0 {
            agent_turn(game, player, &agents[player])
        } else {
            play_systemic(game, player)
        };

        if !has_closed {
            player = (player + 1) & 1;
        }
    }

    [game.score(0), game.score(1)]
}

pub fn evolve(board_size: usize) {
    let game = SquareGame::new(board_size);
    let display = Arc::new(Mutex::new(display_thread(board_size)));

    let options = (
        6,
        vec![6],
        1
    );

    let (start, evol) = load_evol(options);

    for generation in start.. {
        println!("Starting generation {}", generation);

        let agents: Vec<(FlatNetwork, u32)> = {
            evol.next_generation()
                .par_chunks(2)
                .enumerate()
                .flat_map(|(id, agents)| {
                    let mut game = game.clone();
                    let scores = run_game(&mut game, agents);

                    display.try_lock().ok()
                        .and_then(|sender| {
                            sender.0
                                .send(Message::Evolution {
                                    generation,
                                    agent: id,
                                    game: game.clone()
                                })
                                .ok()
                        });

                    println!(" -> Game {} score: {} - {}", id, scores[0], scores[1]);
                    for (agent, score) in agents.iter().zip(&scores) {
                        evol.network_score(agent.clone(), *score);
                    }

                    agents.into_iter().cloned()
                        .map(|agent| agent.into_data())
                        .zip(
                            scores.into_iter().cloned()
                        )
                        .collect::<Vec<_>>()
                })
                .collect()
        };

        data::save(generation, agents);
    }
}
